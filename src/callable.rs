use crate::generate_task_id;
use crate::GeneralErrorType;
use crate::GeneralReturnType;
use crate::Runnable;
use crate::{AsyncKind, BlockingKind, SynchronyType};
use crate::{LoggedKind, LoggingType, UnLoggedKind};
use std::convert::Infallible;
use std::fmt::Display;
use std::marker::PhantomData;
use std::{fmt::Debug, panic, panic::AssertUnwindSafe};

/// Represents one token within the format specification of a callable. The
/// format specification may have the callable handle, its arguments, and
/// arbitrary strings
#[derive(Debug, Clone)]
enum CallableLoggingFormatToken {
    Handle,
    Arguments,
    ArbitraryString(String),
}

/// The logging format for a callable, in the format of an ordered list. Each
/// item in the list is a `CallableLoggingFormatToken`
#[derive(Debug, Clone)]
pub struct CallableLoggingFormat {
    logging_format: Vec<CallableLoggingFormatToken>,
}

type CallableLoggingFormatBuilder = CallableLoggingFormat;

impl CallableLoggingFormat {
    /// Create a new callable logging format with an empty list.
    pub fn new() -> Self {
        CallableLoggingFormat { logging_format: Vec::new() }
    }

    /// Append the callable's handle to end of the format specification
    pub fn append_handle(self) -> Self {
        self.format.append(CallableLoggingFormatToken::Handle);
        return self;
    }

    /// Append the callable's arguments to the end of the format specification
    pub fn append_arguments(self) -> Self {
        self.format.append(CallableLoggingFormatToken::Arguments);
        return self;
    }

    /// Append an arbitrary string to the end of the format specification
    pub fn append_string<S: Into<String>>(self, given_string: S) -> Self {
        self.format.append(CallableLoggingFormatToken::ArbitraryString(given_string.into()));
        return self;
    }
}

/// The logging data for a callable. Contains the string form of the callable's
/// handle and the string form of its arguments
#[derive(Debug, Clone)]
struct CallableLoggingData {
    handle: String,
    arguments: String,
}

/// Stores the minimum information needed define a callable
#[derive(Debug, Clone, Copy)]
struct AtomicCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    handle: F,            // the callable's handle
    arguments: Option<A>, // a tuple representing the arguments
}

/// AtomicCallable with the output stored
#[derive(Debug, Clone)]
struct StoredCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: AtomicCallable,
    
}

/// A struct denoting a callable object, like a function, method, or a closure 
/// that implements one of Fn, FnOnce or FnMut. 
#[derive(Debug, Clone)]
pub struct Callable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: AtomicCallable<A, R, F>,
    output: Option<R>
}

/// A struct denoting a callable object, like a function or a closure,
/// that usually implements one of Fn, FnOnce or FnMut. 
#[derive(Debug, Clone)]
pub struct LoggedCallable<
    'a,
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: AtomicCallable<A, R, F>
}

pub type Function<'a, A, R, F, L, S> = Callable<'a, A, R, F, L, S>;
pub type Method<'a, A, R, F, L, S> = Callable<'a, A, R, F, L, S>;
pub type Closure<'a, A, R, F, L, S> = Callable<'a, A, R, F, L, S>;

/// An enumeration denoting a callable object, like a function or a closure,
/// that usually implements one of Fn, FnOnce or FnMut. It accounts for logging
/// and asynchronous variants. It is declared as an enumeration because
/// specializing a generic struct to have different variables is not yet
/// possible
#[derive(Debug, Clone)]
enum CallableInner<
    'a, // lifetime of a logging format specifier, if it exists
    A,  // arguments as a tuple
    R,  // return type
    F,  // Fn trait (like Fn, FnOnce, and FnMut)
    L,  // Logging type: either LoggedKind or UnLoggedKind
    S,  // Synchrony type: either BlockingKind or AsyncKind
> where
    F: FnOnce<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    UnLoggedBlockingCallableOuter(UnLoggedBlockingCallable<A, R, F>),
    UnLoggedAsyncCallableOuter(UnLoggedAsyncCallable<A, R, F>),
    LoggedBlockingCallableOuter(LoggedBlockingCallable<'a, A, R, F>),
    LoggedAsyncCallableOuter(LoggedAsyncCallable<'a, A, R, F>),
    _PhantomVariant(Infallible, PhantomData<(L, S)>),
}

/// Represents a callable object that is blocking (synchronous) and is not
/// logged
#[derive(Debug, Clone)]
struct UnLoggedBlockingCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: Option<AtomicCallable<A, R, F>>, /* a callable that only contains the minimum information needed to store it */
    output: Option<GeneralReturnType>,
}

/// Represents a callable object that is asynchronous and is not logged
#[derive(Debug, Clone)]
struct UnLoggedAsyncCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: Option<AtomicCallable<A, R, F>>, /* a callable that only contains the minimum information needed to store it */
    output: Option<GeneralReturnType>,
}

/// Represents a callable object that is blocking (synchronous) and is logged
#[derive(Debug, Clone)]
struct LoggedBlockingCallable<
    'a, // lifetime of a logging format specifier, if it exists
    A,  // arguments as a tuple
    R,  // return type
    F,  // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    task_id: usize, // a task ID used to match input with the output
    atomic_callable: Option<AtomicCallable<A, R, F>>, /* a callable that only contains the minimum information needed to store it */
    logging_format: Option<&'a CallableLoggingFormat>, // logging format for callables
    logging_data: Option<CallableLoggingData>,        // logging data for callables
    output: Option<GeneralReturnType>,
}

/// Represents a callable object that is asynchronous and is logged
#[derive(Debug, Clone)]
struct LoggedAsyncCallable<
    'a, // lifetime of a logging format specifier, if it exists
    A,  // arguments as a tuple
    R,  // return type
    F,  // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    task_id: usize, // a task ID used to match input with the output
    atomic_callable: Option<AtomicCallable<A, R, F>>, /* a callable that only contains the minimum information needed to store it */
    logging_format: Option<&'a CallableLoggingFormat>, // logging format for callables
    logging_data: Option<CallableLoggingData>,        // logging data for callables
    output: Option<GeneralReturnType>,
}

/// Implementation of the callable type which is blocking, but unlogged
impl<'a, A, R, F>
    Callable<
        'a,           // lifetime of a logging format specifier, if it exists
        A,            // arguments as a tuple
        R,            // return type
        F,            // Fn trait (like Fn, FnOnce, and FnMut)
        UnLoggedKind, // Logging type: either LoggedKind or UnLoggedKind
        BlockingKind, // Synchrony type: either BlockingKind or AsyncKind
    >
where
    F: FnOnce<A, Output = R>,
{
    /// Create a new callable with the handle's identifier. The handle must
    /// either be a function or a closure that implements FnOnce, Fn, or FnMut
    /// as a consequence
    pub fn new(handle: F) -> Self {
        return Callable {
            inner_callable: CallableInner::UnLoggedBlockingCallableOuter(
                UnLoggedBlockingCallable {
                    atomic_callable: Some(AtomicCallable::new(handle)),
                    output: None,
                },
            ),
            phantom_data: PhantomData,
        };
    }

    /// Provide the arguments for a callable's handle
    pub fn args(mut self, arguments: A) -> Self {
        if let CallableInner::UnLoggedBlockingCallableOuter(unlogged_blocking_callable) =
            self.inner_callable
        {
            unlogged_blocking_callable.arguments = Some(arguments);
        }
        return self;
    }
}

/// Implementation of the callable type which is blocking, but logged
impl<'a, A, R, F>
    Callable<
        'a,           // lifetime of a logging format specifier, if it exists
        A,            // arguments as a tuple
        R,            // return type
        F,            // Fn trait (like Fn, FnOnce, and FnMut)
        LoggedKind,   // Logging type: either LoggedKind or UnLoggedKind
        BlockingKind, // Synchrony type: either BlockingKind or AsyncKind
    >
where
    F: FnOnce<A, Output = R>,
{
    /// Create a new callable with the handle's identifier. The handle must
    /// either be a function or a closure that implements FnOnce, Fn, or FnMut
    /// as a consequence
    pub fn new<S: Into<String>>(handle: F, handle_string: S) -> Self {
        return Callable {
            inner_callable: CallableInner::LoggedBlockingCallableOuter(LoggedBlockingCallable {
                task_id: generate_task_id(),
                logging_format: None,
                logging_data: Some(CallableLoggingData {
                    handle: handle_string.into(),
                    arguments: String::new(),
                }),
                atomic_callable: Some(AtomicCallable::new(handle)),
                output: None,
            }),
            phantom_data: PhantomData,
        };
    }

    /// Provide the arguments for a callable's handle
    pub fn args<S: Into<String>>(mut self, arguments: A, arguments_string: S) -> Self {
        if let CallableInner::LoggedBlockingCallableOuter(logged_blocking_callable) =
            self.inner_callable
        {
            logged_blocking_callable.arguments = Some(arguments);
            logged_blocking_callable.logging_data.arguments = arguments_string.into();
        }
        return self;
    }
}

impl<'a, A, R, F, L, S> Runnable
    for Callable<
        'a, // lifetime of a logging format specifier, if it exists
        A,  // arguments as a tuple
        R,  // return type
        F,  // Fn trait (like Fn, FnOnce, and FnMut)
        L,  // Logging type: either LoggedKind or UnLoggedKind
        S,  // Synchrony type: either BlockingKind or AsyncKind
    >
where
    F: FnOnce<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    default fn run(&mut self) -> Result<R, GeneralErrorType> {
        self.inner_callable.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            self.atomic_callable.handle.call_once(self.atomic_callable.arguments);
            self.atomic_callable = None;
        })));
    }
}

impl<'a, A, R, F, L, S> Runnable
    for Callable<
        'a, // lifetime of a logging format specifier, if it exists
        A,  // arguments as a tuple
        R,  // return type
        F,  // Fn trait (like Fn, FnOnce, and FnMut)
        L,  // Logging type: either LoggedKind or UnLoggedKind
        S,  // Synchrony type: either BlockingKind or AsyncKind
    >
where
    F: Fn<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    fn run(&mut self) -> Result<R, GeneralErrorType> {
        return panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            self.atomic_callable.handle.call(self.atomic_callable.arguments);
            self.atomic_callable = None;
        }));
    }
}

impl<'a, A, R, F, L, S> Runnable
    for Callable<
        'a, // lifetime of a logging format specifier, if it exists
        A,  // arguments as a tuple
        R,  // return type
        F,  // Fn trait (like Fn, FnOnce, and FnMut)
        L,  // Logging type: either LoggedKind or UnLoggedKind
        S,  // Synchrony type: either BlockingKind or AsyncKind
    >
where
    F: FnMut<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    fn run(&mut self) -> Result<R, GeneralErrorType> {
        return panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            self.atomic_callable.handle.call_mut(self.atomic_callable.arguments);
        }));
    }
}

// MODULE LEVEL FUNCTIONS

fn run_function<A, R, F>(handle: F, arguments: A) -> Option<Result<R, GeneralErrorType>>
where
    F: FnOnce<A, Output = R>,
{
    return Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| handle.call_once(arguments))));
}

// TESTS

#[cfg(test)]
mod tests {

    // IMPORTS

    use crate::Runnable;

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
