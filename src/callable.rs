// region: IMPORTS

use crate::generate_task_id;
use crate::Error;
use crate::Represent;
use crate::{Run, RunAndCallback, RunAndDebug, RunAndDisplay, RunAndReturn};
use snafu::{Backtrace, ErrorCompat, OptionExt, ResultExt, Snafu};
use std::any::Any;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::{panic, panic::AssertUnwindSafe};

// endregion: IMPORTS

// region: ERRORS

#[derive(Debug, Snafu)]
pub enum CallableError {
    // TODO: https://docs.rs/snafu/0.6.10/snafu/index.html
    #[snafu(display("Callable handle missing. Either it was not provided, or was moved during a previous method call"))]
    CallableHandleMissing { backtrace: Backtrace },
    #[snafu(display("Callable arguments missing. Either they were not provided, or were moved during a previous method call"))]
    CallableArgumentsMissing { backtrace: Backtrace },
    #[snafu(display("Callable panicked"))]
    CallablePanicked { backtrace: Backtrace },
    #[snafu(display("Callable handle string missing. It is necessary for logging"))]
    CallableHandleStringMissing { backtrace: Backtrace },
    #[snafu(display("Callable argument string missing. It is necessary for logging"))]
    CallableArgumentStringMissing { backtrace: Backtrace },
    #[snafu(display("Callable logging format missing. It is necessary for logging"))]
    CallableLoggingFormatMissing { backtrace: Backtrace },
}

impl From<CallableError> for Error {
    fn from(callable_error: CallableError) -> Self {
        Box::new(callable_error)
    }
}

// endregion: ERRORS

// region: LOGGING INFO

/// The logging data for a callable. Contains the string form of the callable's
/// handle and the string form of its arguments
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
struct LoggingData {
    handle: String,
    arguments: String,
}

/// Represents one token within the format specification of a callable. The
/// format specification may have the callable handle, its arguments, and
/// arbitrary strings. Use the `new` and `append` methods to build up the format
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum LoggingFormatToken {
    Handle,
    Args,
    Output,
    ArbitraryString(String),
}

/// The logging format for a callable, in the format of an ordered list. Each
/// item in the list is a [LoggingFormatToken]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct LoggingFormat {
    logging_format: Vec<LoggingFormatToken>,
}

pub type LoggingFormatBuilder = LoggingFormat;

impl Deref for LoggingFormat {
    type Target = Vec<LoggingFormatToken>;

    fn deref(&self) -> &Self::Target {
        return &self.logging_format;
    }
}

impl DerefMut for LoggingFormat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.logging_format;
    }
}

impl LoggingFormat {
    /// Create a new callable logging format with an empty list.
    pub fn new() -> Self {
        LoggingFormat { logging_format: Vec::new() }
    }

    /// Append the callable's handle to end of the format specification
    pub fn append_handle(mut self) -> Self {
        self.push(LoggingFormatToken::Handle);
        return self;
    }

    /// Append the callable's arguments to the end of the format specification
    pub fn append_args(mut self) -> Self {
        self.push(LoggingFormatToken::Args);
        return self;
    }

    /// Append the callable's output to the end of the format specification
    pub fn append_output(mut self) -> Self {
        self.push(LoggingFormatToken::Output);
        return self;
    }

    /// Append an arbitrary string to the end of the format specification
    pub fn append_string<S: Into<String>>(mut self, given_string: S) -> Self {
        self.push(LoggingFormatToken::ArbitraryString(given_string.into()));
        return self;
    }
}

// endregion: LOGGING INFO

// region: CALLABLE

/// Stores the minimum information needed define a callable
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct AtomicCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    handle: Option<F>,    // the callable's handle
    arguments: Option<A>, // a tuple representing the arguments
}

/// A struct denoting a callable object, like a function, method, or a closure
/// that implements one of Fn, FnOnce or FnMut.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Callable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: AtomicCallable<A, R, F>,
}

pub type Function<A, R, F> = Callable<A, R, F>;
pub type Method<A, R, F> = Callable<A, R, F>;
pub type Closure<A, R, F> = Callable<A, R, F>;

impl<A, R, F> Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn compose_run_result(
        call_result: Result<Result<R, CallableError>, Box<dyn Any + Send>>,
    ) -> Result<R, Error> {
        let result = match call_result {
            Ok(inner) => inner,
            Err(_inner) => CallablePanicked.fail().into(),
        };
        let result = result.map_err(|error: CallableError| -> Error { error.into() });
        return result;
    }
}

impl<A, R, F> Deref for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type Target = AtomicCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.atomic_callable;
    }
}

impl<A, R, F> DerefMut for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.atomic_callable;
    }
}

/// A trait that exists solely to specialize the implementation of `new` and
/// `args` methods in `Callable` for the case of no arguments
pub trait CallableCreate<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    fn new(handle: F) -> Self;
    fn args(self: Self, arguments: A) -> Self;
}

/// Implementation for a general callable
impl<A, R, F> CallableCreate<A, R, F> for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    /// Creates a new callable with the given handle and no arguments
    default fn new(handle: F) -> Self {
        return Callable {
            atomic_callable: AtomicCallable { handle: Some(handle), arguments: None },
        };
    }

    /// Stores arguments in the callable
    default fn args(mut self, arguments: A) -> Self {
        self.arguments = Some(arguments);
        return self;
    }
}

/// Implementation for a callable with a handle that indicates that it takes no
/// arguments
impl<R, F> CallableCreate<(), R, F> for Callable<(), R, F>
where
    F: FnOnce<(), Output = R>,
{
    fn new(handle: F) -> Self {
        return Callable {
            atomic_callable: AtomicCallable { handle: Some(handle), arguments: Some(()) },
        };
    }

    fn args(mut self, arguments: ()) -> Self {
        self.arguments = Some(arguments);
        return self;
    }
}

trait InnerRunOnce<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn inner_run_once(&mut self) -> Result<Result<R, CallableError>, Box<dyn Any + Send>>;
}

impl<A, R, F> InnerRunOnce<A, R, F> for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn inner_run_once(&mut self) -> Result<Result<R, CallableError>, Box<dyn Any + Send>> {
        return panic::catch_unwind(AssertUnwindSafe(|| -> Result<R, CallableError> {
            let arguments: A = self.arguments.take().context(CallableArgumentsMissing)?;
            let handle: F = self.handle.take().context(CallableHandleMissing)?;
            Ok(handle.call_once(arguments))
        }));
    }
}

trait InnerRunMut<A, R, F>
where
    F: FnMut<A, Output = R>,
{
    fn inner_run_mut(&mut self) -> Result<Result<R, CallableError>, Box<dyn Any + Send>>;
}

impl<A, R, F> InnerRunMut<A, R, F> for Callable<A, R, F>
where
    F: FnMut<A, Output = R>,
{
    fn inner_run_mut(&mut self) -> Result<Result<R, CallableError>, Box<dyn Any + Send>> {
        return panic::catch_unwind(AssertUnwindSafe(|| -> Result<R, CallableError> {
            let arguments: A = self.arguments.take().context(CallableArgumentsMissing)?;
            let handle: &mut F = self.handle.as_mut().context(CallableHandleMissing)?;
            Ok(handle.call_mut(arguments))
        }));
    }
}

trait InnerRun<A, R, F>
where
    F: Fn<A, Output = R>,
{
    fn inner_run(&mut self) -> Result<Result<R, CallableError>, Box<dyn Any + Send>>;
}

impl<A, R, F> InnerRun<A, R, F> for Callable<A, R, F>
where
    F: Fn<A, Output = R>,
{
    fn inner_run(&mut self) -> Result<Result<R, CallableError>, Box<dyn Any + Send>> {
        return panic::catch_unwind(AssertUnwindSafe(|| -> Result<R, CallableError> {
            let arguments: A = self.arguments.take().context(CallableArgumentsMissing)?;
            let handle: &mut F = self.handle.as_mut().context(CallableHandleMissing)?;
            Ok(handle.call(arguments))
        }));
    }
}

impl<A, R, F> RunAndReturn for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type ReturnType = R;

    default fn run_and_return(&mut self) -> Result<Self::ReturnType, Error> {
        return Callable::<A, R, F>::compose_run_result(self.inner_run_once());
    }
}

impl<A, R, F> RunAndReturn for Callable<A, R, F>
where
    F: FnMut<A, Output = R>,
{
    default fn run_and_return(&mut self) -> Result<Self::ReturnType, Error> {
        return Callable::<A, R, F>::compose_run_result(self.inner_run_mut());
    }
}

impl<A, R, F> RunAndReturn for Callable<A, R, F>
where
    F: Fn<A, Output = R>,
{
    fn run_and_return(&mut self) -> Result<Self::ReturnType, Error> {
        return Callable::<A, R, F>::compose_run_result(self.inner_run());
    }
}

impl<A, R, F> Run for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn run(&mut self) -> Result<(), Error> {
        return self.run_and_return().map(|_inner| ());
    }
}

impl<A, R, F> RunAndCallback for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn run_and_then<C: FnOnce(Self::ReturnType) -> ()>(
        &mut self,
        callback: C,
    ) -> Result<(), Error> {
        match self.run_and_return() {
            Ok(inner) => Ok(callback(inner)),
            Err(inner) => Err(inner),
        }
    }
}

impl<A, R, F> RunAndDebug for Callable<A, R, F>
where
    R: Debug,
    F: FnOnce<A, Output = R>,
{
    fn run_and_debug(&mut self) -> Result<String, Error> {
        match self.run_and_return() {
            Ok(inner) => Ok(format!("{:?}", inner)),
            Err(inner) => Err(inner),
        }
    }
}

impl<A, R, F> RunAndDisplay for Callable<A, R, F>
where
    R: Display,
    F: FnOnce<A, Output = R>,
{
    fn run_and_display(&mut self) -> Result<String, Error> {
        match self.run_and_return() {
            Ok(inner) => Ok(format!("{}", inner)),
            Err(inner) => Err(inner),
        }
    }
}

// endregion: CALLABLE

// region: LOGGED CALLABLE

/// A struct denoting a logged callable object, like a function, method, or a
/// closure that implements one of Fn, FnOnce or FnMut.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct LoggedCallable<
    'a, // the lifetime specifier of the logging format,
    A,  // arguments as a tuple
    R,  // return type
    F,  // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    callable: Callable<A, R, F>,
    logging_data: Option<LoggingData>,
    logging_format: Option<&'a LoggingFormat>,
}

pub type LoggedFunction<'a, A, R, F> = LoggedCallable<'a, A, R, F>;
pub type LoggedMethod<'a, A, R, F> = LoggedCallable<'a, A, R, F>;
pub type LoggedClosure<'a, A, R, F> = LoggedCallable<'a, A, R, F>;

impl<'a, A, R, F> Deref for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type Target = Callable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.callable;
    }
}

impl<'a, A, R, F> DerefMut for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.callable;
    }
}

/// A trait that exists solely to specialize the implementation of `new` and
/// `args` methods in `LoggedCallable` for the case of no arguments
pub trait LoggedCallableCreate<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    fn new<S: Into<String>>(handle: F, handle_string: S) -> Self;
    fn args<S: Into<String>>(self: Self, arguments: A, arguments_string: S) -> Self;
}

impl<'a, A, R, F> LoggedCallableCreate<A, R, F> for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn new<S: Into<String>>(handle: F, handle_string: S) -> Self {
        return LoggedCallable {
            callable: Callable::new(handle),
            logging_data: Some(LoggingData {
                handle: handle_string.into(),
                arguments: String::new(),
            }),
            logging_format: None,
        };
    }

    fn args<S: Into<String>>(mut self, arguments: A, arguments_string: S) -> Self {
        self.arguments = Some(arguments);
        if let Some(mut logging_data_inner) = self.logging_data.as_mut() {
            logging_data_inner.arguments = arguments_string.into();
        }
        return self;
    }
}

impl<'a, R, F> LoggedCallableCreate<(), R, F> for LoggedCallable<'a, (), R, F>
where
    F: FnOnce<(), Output = R>,
{
    fn new<S: Into<String>>(handle: F, handle_string: S) -> Self {
        return LoggedCallable {
            callable: Callable::new(handle).args(()),
            logging_data: Some(LoggingData {
                handle: handle_string.into(),
                arguments: String::from("()"),
            }),
            logging_format: None,
        };
    }
}

impl<'a, A, R, F> LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    pub fn generate_log(&self, result: &Result<R, Error>) -> Result<String, Error> {
        let handle_string =
            &self.logging_data.as_ref().context(CallableHandleStringMissing)?.handle;
        let arguments_string =
            &self.logging_data.as_ref().context(CallableHandleStringMissing)?.arguments;
        let output_string = match result.as_ref() {
            Ok(inner) => inner.represent(),
            Err(inner) => inner.represent(),
        };

        self.logging_format.context(CallableLoggingFormatMissing)?.iter().fold(
            Ok(String::new()),
            |accumulator_string, token| {
                let intermediate_string = match token {
                    LoggingFormatToken::Handle => handle_string,
                    LoggingFormatToken::Args => arguments_string,
                    LoggingFormatToken::ArbitraryString(arbitrary_string) => arbitrary_string,
                    LoggingFormatToken::Output => &output_string,
                };
                return accumulator_string.map(|mut inner| {
                    inner.push_str(intermediate_string);
                    inner
                });
            },
        )
    }
}

impl<'a, A, R, F> RunAndReturn for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type ReturnType = R;

    default fn run_and_return(&mut self) -> Result<Self::ReturnType, Error> {
        let result = self.callable.run_and_return();
        self.generate_log(&result)?; // TODO: Use this to log
        return result;
    }
}

impl<'a, A, R, F> Run for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn run(&mut self) -> Result<(), Error> {
        let result = self.callable.run_and_return();
        self.generate_log(&result)?; // TODO: Use this to log
        return result.map(|_inner| ());
    }
}

impl<'a, A, R, F> RunAndCallback for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn run_and_then<C: FnOnce(Self::ReturnType) -> ()>(
        &mut self,
        callback: C,
    ) -> Result<(), Error> {
        let result = self.callable.run_and_return();
        self.generate_log(&result)?; // TODO: Use this to log
        match result {
            Ok(inner) => Ok(callback(inner)),
            Err(inner) => Err(inner),
        }
    }
}

impl<'a, A, R, F> RunAndDebug for LoggedCallable<'a, A, R, F>
where
    R: Debug,
    F: FnOnce<A, Output = R>,
{
    fn run_and_debug(&mut self) -> Result<String, Error> {
        let result = self.callable.run_and_return();
        self.generate_log(&result)?; // TODO: Use this to log
        match result {
            Ok(inner) => Ok(format!("{:?}", inner)),
            Err(inner) => Err(inner),
        }
    }
}

impl<'a, A, R, F> RunAndDisplay for LoggedCallable<'a, A, R, F>
where
    R: Display,
    F: FnOnce<A, Output = R>,
{
    fn run_and_display(&mut self) -> Result<String, Error> {
        let result = self.callable.run_and_return();
        self.generate_log(&result)?; // TODO: Use this to log
        match result {
            Ok(inner) => Ok(format!("{}", inner)),
            Err(inner) => Err(inner),
        }
    }
}

// endregion: LOGGED CALLABLE

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use crate::tests::setup_logging;

    // TESTS

    #[test]
    fn vector_pop() {
        setup_logging(log::LevelFilter::Debug);

        let mut vector: Vec<isize> = vec![1, 2, 3, 4, 5, 6];
        let mut callable = callable!(vector.pop());
        let output: Option<isize>;

        #[cfg(feature = "async")]
        {
            block_on(callable.run());
            output = callable.output.unwrap().unwrap();
        }

        #[cfg(not(feature = "async"))]
        {
            callable.run();
            output = callable.output.unwrap().unwrap();
        }

        println!("vector_pop() output: {:?}", output);
        assert_eq!(output, Some(6));
        assert_eq!(vector, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn vector_push() {
        #[cfg(feature = "logging")]
        setup_logging(log::LevelFilter::Debug);

        let mut vector: Vec<isize> = vec![1, 2, 3, 4, 5];
        let mut callable = callable!(vector.push(7));
        let output: ();

        #[cfg(feature = "async")]
        {
            block_on(callable.run());
            output = callable.output.unwrap().unwrap();
        }

        #[cfg(not(feature = "async"))]
        {
            callable.run();
            output = callable.output.unwrap().unwrap();
        }

        println!("vector_push() output: {:?}", output);
        assert_eq!(output, ());
        assert_eq!(vector, [1, 2, 3, 4, 5, 7]);
    }

    #[test]
    fn vector_pop_and_push() {
        #[cfg(feature = "logging")]
        setup_logging(log::LevelFilter::Debug);

        let mut vector: Vec<isize> = vec![1, 2, 3, 4, 5, 6];
        let mut callable = callable!(vector.pop());
        let output: Option<isize>;

        #[cfg(feature = "async")]
        {
            block_on(callable.run());
            output = callable.output.unwrap().unwrap();
        }

        #[cfg(not(feature = "async"))]
        {
            callable.run();
            output = callable.output.unwrap().unwrap();
        }

        println!("vector_pop() output: {:?}", output);
        assert_eq!(output, Some(6));
        assert_eq!(vector, [1, 2, 3, 4, 5]);

        let mut callable = callable!(vector.push(7));
        let output: ();

        #[cfg(feature = "async")]
        {
            block_on(callable.run());
            output = callable.output.unwrap().unwrap();
        }

        #[cfg(not(feature = "async"))]
        {
            callable.run();
            output = callable.output.unwrap().unwrap();
        }

        println!("vector_push() output: {:?}", output);
        assert_eq!(output, ());
        assert_eq!(vector, [1, 2, 3, 4, 5, 7]);
    }

    #[test]
    #[should_panic]
    fn panic() {
        let panicking_closure = || {
            panic!("Panicking test...");
        };
        let mut callable = callable!(panicking_closure());

        #[cfg(feature = "async")]
        block_on(callable.run());

        #[cfg(not(feature = "async"))]
        callable.run();

        callable.output.unwrap().unwrap();
    }

    #[test]
    #[cfg(feature = "logging")]
    fn try_string_from() {
        let value: isize = 5;
        assert_eq!(String::from("5"), crate::try_string_from(&value).unwrap())
    }
}
