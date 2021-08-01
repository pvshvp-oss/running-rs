use crate::generate_task_id;
use crate::Runnable;
use std::any::Any;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::{panic, panic::AssertUnwindSafe};

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
    Output,
    ArbitraryString(String),
}

/// The logging format for a callable, in the format of an ordered list. Each
/// item in the list is a `CallableLoggingFormatToken`
#[derive(Debug, Clone)]
pub struct CallableLoggingFormat {
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
        CallableLoggingFormat { logging_format: Vec::new() }
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

    /// Append the callable's output to the end of the format specification
    pub fn append_output(mut self) -> Self {
        self.push(CallableLoggingFormatToken::Output);
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
    F: FnOnce<A, Output = R>,
{
    handle: Option<F>,    // the callable's handle
    arguments: Option<A>, // a tuple representing the arguments
}

pub type CallableError = Box<(dyn Any + Send)>;

/// AtomicCallable with the output stored
// #[derive(Debug, Clone)]
pub struct StoredCallable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    atomic_callable: AtomicCallable<A, R, F>,
    output: Option<Result<R, CallableError>>,
}

/// Make StoredCallable ergonomic by allowing access to the fields and methods
/// of the inner AtomicCallable
impl<A, R, F> Deref for StoredCallable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type Target = AtomicCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.atomic_callable;
    }
}

impl<A, R, F> DerefMut for StoredCallable<A, R, F>
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
    fn new(self: Self, handle: F) -> Self;
    fn args(self: Self, arguments: A) -> Self;
}

/// A struct denoting a callable object, like a function, method, or a closure
/// that implements one of Fn, FnOnce or FnMut.
// #[derive(Debug, Clone)]
pub struct Callable<
    A, // arguments as a tuple
    R, // return type
    F, // Fn trait (like Fn, FnOnce, and FnMut)
> where
    F: FnOnce<A, Output = R>,
{
    stored_callable: StoredCallable<A, R, F>,
}

pub type Function<A, R, F> = Callable<A, R, F>;
pub type Method<A, R, F> = Callable<A, R, F>;
pub type Closure<A, R, F> = Callable<A, R, F>;

impl<A, R, F> CallableCreate<A, R, F> for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn new(self, handle: F) -> Self {
        return Callable {
            stored_callable: StoredCallable {
                atomic_callable: AtomicCallable { handle: Some(handle), arguments: None },
                output: None,
            },
        };
    }

    default fn args(mut self, arguments: A) -> Self {
        self.arguments = Some(arguments);
        return self;
    }
}

impl<R, F> CallableCreate<(), R, F> for Callable<(), R, F>
where
    F: FnOnce<(), Output = R>,
{
    fn new(self, handle: F) -> Self {
        return Callable {
            stored_callable: StoredCallable {
                atomic_callable: AtomicCallable { handle: Some(handle), arguments: Some(()) },
                output: None,
            },
        };
    }

    fn args(mut self, arguments: ()) -> Self {
        self.arguments = Some(arguments);
        return self;
    }
}

impl<A, R, F> Deref for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type Target = StoredCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.stored_callable;
    }
}

impl<A, R, F> DerefMut for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.stored_callable;
    }
}

impl<A, R, F> Runnable for Callable<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn run(&mut self) {
        self.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let arguments = self
                .arguments
                .take()
                .expect("Arguments not provided or are not in the valid format...");
            let handle = self.handle.take().expect("Handle not provided or is moved...");
            handle.call_once(arguments)
        })));
    }
}

impl<A, R, F> Runnable for Callable<A, R, F>
where
    F: FnMut<A, Output = R>,
{
    default fn run(&mut self) {
        self.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let arguments = self
                .arguments
                .take()
                .expect("Arguments not provided or are not in the valid format...");
            let handle = self.handle.as_mut().expect("Handle not provided or is moved...");
            handle.call_mut(arguments)
        })));
    }
}

impl<A, R, F> Runnable for Callable<A, R, F>
where
    F: Fn<A, Output = R>,
{
    fn run(&mut self) {
        self.output = Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let arguments = self
                .arguments
                .take()
                .expect("Arguments not provided or are not in the valid format...");
            let handle = self.handle.as_mut().expect("Handle not provided or is moved...");
            handle.call(arguments)
        })));
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
    fn new<S: Into<String>>(self: Self, handle: F, handle_string: S) -> Self;
    fn args<S: Into<String>>(self: Self, arguments: A, arguments_string: S) -> Self;
}

/// A trait that exists solely to specialize the implementation of the
/// `generate_log` method in `LoggedCallable` over the return type
pub trait LoggedCallableLog
{
    fn generate_log(&self) -> String;
}

/// A struct denoting a logged callable object, like a function, method, or a
/// closure that implements one of Fn, FnOnce or FnMut.
// #[derive(Debug, Clone)]
pub struct LoggedCallable<
    'a, // the lifetime specifier of the logging format,
    A,  // arguments as a tuple
    R,  // return type
    F,  // Fn trait (like Fn, FnOnce, and FnMut)
> where
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

impl<'a, A, R, F> LoggedCallableLog for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn generate_log(&self) -> String {
        if let Some(logging_format) = self.logging_format {
            // let return_string: String;
            logging_format.iter().fold(String::new(), |mut accumulator_string, token| {
            
                accumulator_string.push_str(match token {
                    CallableLoggingFormatToken::Handle => {
                        if let Some(logging_data) = self.logging_data.as_ref() {
                            &logging_data.handle
                        } else {
                            "N.A."
                        }
                    }
                    CallableLoggingFormatToken::Arguments => {
                        if let Some(logging_data) = self.logging_data.as_ref() {
                            &logging_data.arguments
                        } else {
                            "N.A."
                        }
                    }
                    CallableLoggingFormatToken::ArbitraryString(arbitrary_string) => {
                        arbitrary_string
                    }
                    CallableLoggingFormatToken::Output => {
                        if let Some(output) = self.output.as_ref() {
                            match output {
                                Ok(_return_value) => {
                                    // return_string = format!("{:?}", return_value);
                                    // &return_string
                                    "N.A."
                                }
                                Err(_error) => "N.A.",
                            }
                        } else {
                            "N.A."
                        }
                    }
                });
                return accumulator_string;
            })
        } else {
            String::from("N.A.")
        }
    }
}

impl<'a, A, R, F> LoggedCallableLog for LoggedCallable<'a, A, R, F>
where
    R: Debug,
    F: FnOnce<A, Output = R>,
{
    default fn generate_log(&self) -> String {
        if let Some(logging_format) = self.logging_format {
            logging_format.iter().fold(String::new(), |mut accumulator_string, token| {
                let return_string: String;
                accumulator_string.push_str(match token {
                    CallableLoggingFormatToken::Handle => {
                        if let Some(logging_data) = self.logging_data.as_ref() {
                            &logging_data.handle
                        } else {
                            "N.A."
                        }
                    }
                    CallableLoggingFormatToken::Arguments => {
                        if let Some(logging_data) = self.logging_data.as_ref() {
                            &logging_data.arguments
                        } else {
                            "N.A."
                        }
                    }
                    CallableLoggingFormatToken::ArbitraryString(arbitrary_string) => {
                        arbitrary_string
                    }
                    CallableLoggingFormatToken::Output => {
                        if let Some(output) = self.output.as_ref() {
                            match output {
                                Ok(return_value) => {
                                    return_string = format!("{:?}", return_value);
                                    &return_string
                                }
                                Err(_error) => "N.A.",
                            }
                        } else {
                            "N.A."
                        }
                    }
                });
                return accumulator_string;
            })
        } else {
            String::from("N.A.")
        }
    }
}

impl<'a, A, R, F> LoggedCallableLog for LoggedCallable<'a, A, R, F>
where
    R: Display + Debug,
    F: FnOnce<A, Output = R>,
{
    fn generate_log(&self) -> String {
        if let Some(logging_format) = self.logging_format {
            logging_format.iter().fold(String::new(), |mut accumulator_string, token| {
                let return_string: String;
                accumulator_string.push_str(match token {
                    CallableLoggingFormatToken::Handle => {
                        if let Some(logging_data) = self.logging_data.as_ref() {
                            &logging_data.handle
                        } else {
                            "N.A."
                        }
                    }
                    CallableLoggingFormatToken::Arguments => {
                        if let Some(logging_data) = self.logging_data.as_ref() {
                            &logging_data.arguments
                        } else {
                            "N.A."
                        }
                    }
                    CallableLoggingFormatToken::ArbitraryString(arbitrary_string) => {
                        arbitrary_string
                    }
                    CallableLoggingFormatToken::Output => {
                        if let Some(output) = self.output.as_ref() {
                            match output {
                                Ok(return_value) => {
                                    return_string = format!("{}", return_value);
                                    &return_string
                                }
                                Err(_error) => "N.A.",
                            }
                        } else {
                            "N.A."
                        }
                    }
                });
                return accumulator_string;
            })
        } else {
            String::from("N.A.")
        }
    }
}

impl<'a, A, R, F> LoggedCallableCreate<A, R, F> for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn new<S: Into<String>>(self, handle: F, handle_string: S) -> Self {
        return LoggedCallable {
            stored_callable: StoredCallable {
                atomic_callable: AtomicCallable { handle: Some(handle), arguments: None },
                output: None,
            },
            task_id: generate_task_id(),
            logging_data: Some(CallableLoggingData {
                handle: handle_string.into(),
                arguments: String::new(),
            }),
            logging_format: None,
        };
    }

    default fn args<S: Into<String>>(mut self, arguments: A, arguments_string: S) -> Self {
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
    fn new<S: Into<String>>(self, handle: F, handle_string: S) -> Self {
        return LoggedCallable {
            stored_callable: StoredCallable {
                atomic_callable: AtomicCallable { handle: Some(handle), arguments: Some(()) },
                output: None,
            },
            task_id: generate_task_id(),
            logging_data: Some(CallableLoggingData {
                handle: handle_string.into(),
                arguments: String::from("()"),
            }),
            logging_format: None,
        };
    }

    fn args<S: Into<String>>(mut self, arguments: (), arguments_string: S) -> Self {
        self.arguments = Some(arguments);
        if let Some(mut logging_data_inner) = self.logging_data.as_mut() {
            logging_data_inner.arguments = arguments_string.into();
        }
        return self;
    }
}

impl<'a, A, R, F> Deref for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    type Target = StoredCallable<A, R, F>;

    fn deref(&self) -> &Self::Target {
        return &self.stored_callable;
    }
}

impl<'a, A, R, F> DerefMut for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.stored_callable;
    }
}

impl<'a, A, R, F> Runnable for LoggedCallable<'a, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    default fn run(&mut self) {
        let output = panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let arguments = self
                .arguments
                .take()
                .expect("Arguments not provided or are not in the valid format...");
            let handle = self.handle.take().expect("Handle not provided or is moved...");
            handle.call_once(arguments)
        }));
        self.generate_log();
        self.output = Some(output);
    }
}

impl<'a, A, R, F> Runnable for LoggedCallable<'a, A, R, F>
where
    F: FnMut<A, Output = R>,
{
    default fn run(&mut self) {
        let output = panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let arguments = self
                .arguments
                .take()
                .expect("Arguments not provided or are not in the valid format...");
            let handle = self.handle.as_mut().expect("Handle not provided or is moved...");
            handle.call_mut(arguments)
        }));
        self.generate_log();
        self.output = Some(output);
    }
}

impl<'a, A, R, F> Runnable for LoggedCallable<'a, A, R, F>
where
    F: Fn<A, Output = R>,
{
    fn run(&mut self) {
        let output = panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
            let arguments = self
                .arguments
                .take()
                .expect("Arguments not provided or are not in the valid format...");
            let handle = self.handle.as_mut().expect("Handle not provided or is moved...");
            handle.call(arguments)
        }));
        self.generate_log();
        self.output = Some(output);
    }
}

// #[cfg(test)]
// mod tests {
//     use futures::executor::block_on;

//     use crate::tests::setup_logging;

//     // TESTS

//     #[test]
//     fn vector_pop() {
//         setup_logging(log::LevelFilter::Debug);

//         let mut vector: Vec<isize> = vec![1, 2, 3, 4, 5, 6];
//         let mut callable = callable!(vector.pop());
//         let output: Option<isize>;

//         #[cfg(feature = "async")]
//         {
//             block_on(callable.run());
//             output = callable.output.unwrap().unwrap();
//         }

//         #[cfg(not(feature = "async"))]
//         {
//             callable.run();
//             output = callable.output.unwrap().unwrap();
//         }

//         println!("vector_pop() output: {:?}", output);
//         assert_eq!(output, Some(6));
//         assert_eq!(vector, [1, 2, 3, 4, 5]);
//     }

//     #[test]
//     fn vector_push() {
//         #[cfg(feature = "logging")]
//         setup_logging(log::LevelFilter::Debug);

//         let mut vector: Vec<isize> = vec![1, 2, 3, 4, 5];
//         let mut callable = callable!(vector.push(7));
//         let output: ();

//         #[cfg(feature = "async")]
//         {
//             block_on(callable.run());
//             output = callable.output.unwrap().unwrap();
//         }

//         #[cfg(not(feature = "async"))]
//         {
//             callable.run();
//             output = callable.output.unwrap().unwrap();
//         }

//         println!("vector_push() output: {:?}", output);
//         assert_eq!(output, ());
//         assert_eq!(vector, [1, 2, 3, 4, 5, 7]);
//     }

//     #[test]
//     fn vector_pop_and_push() {
//         #[cfg(feature = "logging")]
//         setup_logging(log::LevelFilter::Debug);

//         let mut vector: Vec<isize> = vec![1, 2, 3, 4, 5, 6];
//         let mut callable = callable!(vector.pop());
//         let output: Option<isize>;

//         #[cfg(feature = "async")]
//         {
//             block_on(callable.run());
//             output = callable.output.unwrap().unwrap();
//         }

//         #[cfg(not(feature = "async"))]
//         {
//             callable.run();
//             output = callable.output.unwrap().unwrap();
//         }

//         println!("vector_pop() output: {:?}", output);
//         assert_eq!(output, Some(6));
//         assert_eq!(vector, [1, 2, 3, 4, 5]);

//         let mut callable = callable!(vector.push(7));
//         let output: ();

//         #[cfg(feature = "async")]
//         {
//             block_on(callable.run());
//             output = callable.output.unwrap().unwrap();
//         }

//         #[cfg(not(feature = "async"))]
//         {
//             callable.run();
//             output = callable.output.unwrap().unwrap();
//         }

//         println!("vector_push() output: {:?}", output);
//         assert_eq!(output, ());
//         assert_eq!(vector, [1, 2, 3, 4, 5, 7]);
//     }

//     #[test]
//     #[should_panic]
//     fn panic() {
//         let panicking_closure = || {
//             panic!("Panicking test...");
//         };
//         let mut callable = callable!(panicking_closure());

//         #[cfg(feature = "async")]
//         block_on(callable.run());

//         #[cfg(not(feature = "async"))]
//         callable.run();

//         callable.output.unwrap().unwrap();
//     }

//     #[test]
//     #[cfg(feature = "logging")]
//     fn try_string_from() {
//         let value: isize = 5;
//         assert_eq!(String::from("5"),
// crate::try_string_from(&value).unwrap())     }
// }
