use crate::GeneralErrorType;
use crate::generate_task_id;
use crate::Runnable;
use std::ops::{Deref, DerefMut};
use std::{fmt::Debug, panic, panic::AssertUnwindSafe};

/// The logging data for a callable. Contains the string form of the callable's
/// handle and the string form of its arguments
#[derive(Debug, Clone)]
struct CallableLoggingData {
    handle: String,
    arguments: String,
}

/// Represents one token within the format specification of a callable. The
/// format specification may have the callable handle, its arguments, and
/// arbitrary strings
#[derive(Debug, Clone)]
pub enum CallableLoggingFormatToken {
    Handle,
    Arguments,
    ArbitraryString(String),
}

/// The logging format for a callable, in the format of an ordered list. Each
/// item in the list is a `CallableLoggingFormatToken`
#[derive(Debug, Clone)]
pub struct CallableLoggingFormat{
    logging_format: Vec<CallableLoggingFormatToken>,
}

pub type CallableLoggingFormatBuilder = CallableLoggingFormat;

impl Deref for CallableLoggingFormat {
    type Target = Vec<CallableLoggingFormatToken>;

    fn deref(&self) -> &Self::Target {
        return &self.logging_format;
    }
}

impl DerefMut for CallableLoggingFormat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.logging_format;
    }
}

impl CallableLoggingFormat {
    /// Create a new callable logging format with an empty list.
    pub fn new() -> Self {
        CallableLoggingFormat{
            logging_format: Vec::new()
        }
    }

    /// Append the callable's handle to end of the format specification
    pub fn append_handle(mut self) -> Self {
        self.push(CallableLoggingFormatToken::Handle);
        return self;
    }

    /// Append the callable's arguments to the end of the format specification
    pub fn append_arguments(mut self) -> Self {
        self.push(CallableLoggingFormatToken::Arguments);
        return self;
    }

    /// Append an arbitrary string to the end of the format specification
    pub fn append_string<S: Into<String>>(mut self, given_string: S) -> Self {
        self.logging_format.push(CallableLoggingFormatToken::ArbitraryString(given_string.into()));
        return self;
    }
}

/// Stores the minimum information needed define a callable
#[derive(Debug, Clone, Copy)]
pub struct AtomicCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    handle: Option<F>,    // the callable's handle
    arguments: Option<A>, // a tuple representing the arguments
}

/// AtomicCallable with the output stored
// #[derive(Debug, Clone)]
pub struct StoredCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    atomic_callable: AtomicCallable<A, R, F>,
    output: Option<Result<R,GeneralErrorType>>, 
}

/// Make StoredCallable ergonomic by allowing access to the fields and methods of the inner AtomicCallable
impl<A, R, F> Deref for StoredCallable<A, R, F>
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    type Target = AtomicCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.atomic_callable;
    }
}

impl<A, R, F> DerefMut for StoredCallable<A, R, F>
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.atomic_callable;
    }
}

/// A struct denoting a callable object, like a function, method, or a closure 
/// that implements one of Fn, FnOnce or FnMut. 
// #[derive(Debug, Clone)]
pub struct Callable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    stored_callable: StoredCallable<A, R, F>,
}

pub type Function<A, R, F> = Callable<A, R, F>;
pub type Method<A, R, F> = Callable<A, R, F>;
pub type Closure<A, R, F> = Callable<A, R, F>;

impl<A, R, F> Callable<A, R, F> 
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    pub fn new(self, handle: F) -> Self {
        return Callable {
            stored_callable: StoredCallable {
                atomic_callable: AtomicCallable {
                    handle: Some(handle),
                    arguments: None,
                },
                output: None
            }
        }
    }

    pub fn args(mut self, arguments: A) -> Self {
        self.arguments = Some(arguments);
        return self;
    }
}

impl<A, R, F> Deref for Callable<A, R, F> 
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    type Target = StoredCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.stored_callable;
    }
}

impl<A, R, F> DerefMut for Callable<A, R, F> 
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.stored_callable;
    }
}

/// A struct denoting a logged callable object, like a function, method, or a closure 
/// that implements one of Fn, FnOnce or FnMut. 
// #[derive(Debug, Clone)]
pub struct LoggedCallable<
    'a, // the lifetime specifier of the logging format
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    A: Clone,
F: FnOnce<A, Output = R>,
{
    stored_callable: StoredCallable<A, R, F>,
    task_id: usize,
    logging_data: Option<CallableLoggingData>,
    logging_format: Option<&'a CallableLoggingFormat>,
}

pub type LoggedFunction<'a, A, R, F> = LoggedCallable<'a, A, R, F>;
pub type LoggedMethod<'a, A, R, F> = LoggedCallable<'a, A, R, F>;
pub type LoggedClosure<'a, A, R, F> = LoggedCallable<'a, A, R, F>;

impl<'a, A, R, F> LoggedCallable<'a, A, R, F> 
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    pub fn new<S: Into<String>>(self, handle: F, handle_string: S) -> Self {
        return LoggedCallable {
            stored_callable: StoredCallable {
                atomic_callable: AtomicCallable {
                    handle: Some(handle),
                    arguments: None,
                },
                output: None
            },
            task_id: generate_task_id(),
            logging_data: Some(
                CallableLoggingData {
                    handle: handle_string.into(),
                    arguments: String::new(),                  
                }
            ),
            logging_format: None,
        }
    }

    pub fn args<S: Into<String>>(mut self, arguments: A, arguments_string: S) -> Self {
        self.arguments = Some(arguments);
        if let Some(mut logging_data_inner) = self.logging_data.as_mut(){
            logging_data_inner.arguments = arguments_string.into();
        }
        return self;
    }
}

impl<'a, A, R, F> Deref for LoggedCallable<'a, A, R, F> 
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    type Target = StoredCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.stored_callable;
    }
}

impl<'a, A, R, F> DerefMut for LoggedCallable<'a, A, R, F> 
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.stored_callable;
    }
}

impl<A, R, F> Runnable for Callable<A, R, F>
where
    A: Clone,
    F: FnOnce<A, Output = R>,
{
    default fn run(&mut self) {
        self.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let handle = self.handle.take().expect("Handle not provided...");
            let arguments = self.arguments.take().expect("Arguments not provided or are not in the valid format...");
            handle.call_once(arguments)
        })));
    }
}

impl<A, R, F> Runnable for Callable<A, R, F>
where
A: Clone,
    F: Fn<A, Output = R>,
{
    fn run(&mut self) {
        self.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let handle = self.handle.as_ref().expect("Handle not provided...");
            let arguments = self.arguments.as_ref().expect("Arguments not provided or are not in the valid format...");
            handle.call(arguments.clone())
        })));
    }
}

// impl<A, R, F> Runnable for Callable<A, R, F>
// where
//     A: Clone,
//     F: FnMut<A, Output = R>,
// {
//     fn run(&mut self) {
//         self.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
//             let mut handle = self.handle.as_mut().expect("Handle not provided...");
//             let mut arguments = self.arguments.as_mut().expect("Arguments not provided or are not in the valid format...");
//             handle.call_mut(*arguments)
//         })));
//     }
// }

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
