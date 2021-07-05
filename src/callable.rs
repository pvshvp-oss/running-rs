// IMPORTS

use crate::{generate_task_id, GenericErrorType, Runnable};
use std::{fmt::Debug, panic, panic::AssertUnwindSafe};

// CUSTOM TYPES

pub type GenericErrorType = Box<(dyn GenericErrorTraits)>; // a generic error type
pub type GenericReturnType = Box<(dyn GenericReturnTraits)>; // a generic return type

// TRAIT ALIASES

pub trait GenericErrorTraits = Any + Send; // traits for a generic error type
pub trait GenericReturnTraits = Any + Send; // traits for a generic return type

#[cfg(feature = "logging")]
fn try_string_from<R: GenericReturnTraits>(value: &R) -> Option<String> {
    // IMPORTS
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};

    let value_any = value as &dyn Any;
    if let Some(inner) = value_any.downcast_ref::<String>() {
        Some(inner.clone())
    } else if let Some(inner) = value_any.downcast_ref::<&str>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<OsString>() {
        Some(String::from(inner.to_string_lossy()))
    } else if let Some(inner) = value_any.downcast_ref::<&OsStr>() {
        Some(String::from(inner.to_string_lossy()))
    } else if let Some(inner) = value_any.downcast_ref::<PathBuf>() {
        Some(String::from(inner.as_path().to_string_lossy()))
    } else if let Some(inner) = value_any.downcast_ref::<&Path>() {
        Some(String::from(inner.to_string_lossy()))
    } else if let Some(_) = value_any.downcast_ref::<()>() {
        Some(String::from("()"))
    } else if let Some(inner) = value_any.downcast_ref::<usize>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<u8>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<u16>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<u32>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<u64>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<u128>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<isize>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<i8>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<i16>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<i32>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<i64>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<i128>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<f32>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<f64>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<char>() {
        Some(String::from(inner.to_string()))
    } else if let Some(inner) = value_any.downcast_ref::<bool>() {
        Some(String::from(inner.to_string()))
    } else {
        None
    }
}

#[cfg(feature = "async")]
use async_trait::async_trait;

#[cfg(feature = "logging")]
use {
    crate::{Loggable, LoggingData, LoggingPreferences},
    std::borrow::Cow,
};

// MACROS

#[cfg(not(feature = "logging"))]
#[allow(unused_macros)]
macro_rules! callable{
    ( $first_parent:ident $(:: $path_fragment_type_a:ident)* $(. $path_fragment_type_b:ident)* ( $($arguments:expr),* ) ) => {
        {
            let callback = || -> _ {
                $first_parent $(:: $path_fragment_type_a)* $(. $path_fragment_type_b)* ($($arguments),*)
            };
            $crate::Function::new(callback).args(())
        }
    };
}

#[cfg(feature = "logging")]
#[allow(unused_macros)]
macro_rules! callable{
    ( $first_parent:ident $(:: $path_fragment_type_a:ident)* $(. $path_fragment_type_b:ident)* ( $($arguments:expr),* ) ) => {
        {
            let callback = || -> _ {
                $first_parent $(:: $path_fragment_type_a)* $(. $path_fragment_type_b)* ($($arguments),*)
            };
            $crate::Function::new(callback, stringify!($first_parent$(::$path_fragment_type_a)*$(.$path_fragment_type_b)*)).args((), stringify!($($arguments),*))
        }
    };
}

// STRUCT DECLARATIONS

#[cfg(not(feature = "logging"))]
pub struct Function<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    id: usize,
    handle: Option<F>,
    arguments: Option<A>,
    output: Option<Result<R, GenericErrorType>>,
}

#[cfg(feature = "logging")]
pub struct Function<'a, 'b, 'c, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    id: usize,
    handle: Option<F>,
    arguments: Option<A>,
    output: Option<Result<R, GenericErrorType>>,
    logging_preferences: LoggingPreferences<'a>,
    logging_data: LoggingData<'b, 'c>,
}

// MODULE LEVEL FUNCTIONS

fn run_function<A, R, F>(handle: F, arguments: A) -> Option<Result<R, GenericErrorType>>
where
    F: FnOnce<A, Output = R>,
{
    return Some(panic::catch_unwind::<_, R>(AssertUnwindSafe(|| {
        handle.call_once(arguments)
    })));
}

// STRUCT IMPLEMENTATIONS

#[cfg(not(feature = "logging"))]
impl<A, R, F> Function<A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    // PUBLIC METHODS

    pub fn new(handle: F) -> Self {
        Function {
            id: generate_task_id(),
            handle: Some(handle),
            arguments: None,
            output: None,
        }
    }

    pub fn args(mut self, arguments: A) -> Self {
        self.arguments = Some(arguments);
        self
    }

    // PRIVATE METHODS

    fn run_function(&mut self) {
        let handle = self.handle.take().unwrap();
        let arguments = self.arguments.take().unwrap();
        self.output = run_function(handle, arguments);
    }
}

#[cfg(feature = "logging")]
impl<'a, 'b, 'c, A, R, F> Function<'a, 'b, 'c, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    // PUBLIC METHODS

    pub fn new<S: Into<Cow<'b, str>>>(handle: F, handle_string: S) -> Self {
        let mut callable = Function {
            id: generate_task_id(),
            handle: Some(handle),
            arguments: None,
            output: None,
            logging_data: LoggingData {
                input: handle_string.into(),
                output: "".into(),
            },
            logging_preferences: LoggingPreferences::default(),
        };
        callable
            .logging_preferences
            .set_label(format!("task_id: {}", callable.id));
        callable
    }

    pub fn args<S: Into<Cow<'b, str>>>(mut self, arguments: A, argument_string: S) -> Self {
        self.arguments = Some(arguments);
        let mut input_string: String =
            format!("{}({})", self.logging_data.input, argument_string.into());
        input_string.retain(|c| !c.is_whitespace());
        self.logging_data.input = input_string.into();
        self
    }

    pub fn set_logging_preferences(&mut self, logging_preferences: LoggingPreferences<'a>) {
        self.logging_preferences = logging_preferences;
    }

    // PRIVATE METHODS

    fn run_function(&mut self) {
        let handle = self.handle.take().unwrap();
        let arguments = self.arguments.take().unwrap();
        self.output = run_function(handle, arguments);
    }
}

// TRAIT IMPLEMENTATIONS

#[cfg(feature = "logging")]
impl<'a, 'b, 'c, A, R, F> Loggable<'a, 'b, 'c> for Function<'a, 'b, 'c, A, R, F>
where
    F: FnOnce<A, Output = R>,
{
    fn logging_preferences(&self) -> &LoggingPreferences<'a> {
        &self.logging_preferences
    }

    fn logging_data(&self) -> &LoggingData<'b, 'c> {
        &self.logging_data
    }
}

#[cfg(not(feature = "async"))]
#[cfg(not(feature = "logging"))]
impl<A, R, F> Runnable for Function<A, R, F>
where
    A: Send,
    R: Send + Debug,
    F: Send + FnOnce<A, Output = R>,
{
    fn run(&mut self) {
        self.run_function();
    }
}

#[cfg(feature = "async")]
#[cfg(not(feature = "logging"))]
#[async_trait]
impl<A, R, F> Runnable for Function<A, R, F>
where
    A: Send,
    R: Send + Debug,
    F: Send + FnOnce<A, Output = R>,
{
    #[cfg(feature = "async")]
    async fn run(&mut self) {
        self.run_function();
    }
}

#[cfg(feature = "async")]
#[cfg(feature = "logging")]
#[async_trait]
impl<'a, 'b, 'c, A, R, F> Runnable for Function<'a, 'b, 'c, A, R, F>
where
    A: Send,
    R: Send + Debug,
    F: Send + FnOnce<A, Output = R>,
{
    async fn run(&mut self) {
        self.run_function();
        self.log_input();
        if let Some(output) = &self.output {
            match output {
                Ok(output) => {
                    self.logging_data.set_output(format!("{:?}", output));
                }
                Err(_error) => self.logging_data.set_output("Function/Closure panicked..."),
            }
        }
        self.log_output();
    }
}

#[cfg(not(feature = "async"))]
#[cfg(feature = "logging")]
impl<'a, 'b, 'c, A, R, F> Runnable for Function<'a, 'b, 'c, A, R, F>
where
    A: Send,
    R: Send + Debug,
    F: Send + FnOnce<A, Output = R>,
{
    fn run(&mut self) {
        self.run_function();
        self.log_input();
        if let Some(output) = &self.output {
            match output {
                Ok(output) => {
                    self.logging_data.set_output(format!("{:?}", output));
                }
                Err(_error) => self.logging_data.set_output("Function/Closure panicked..."),
            }
        }
        self.log_output();
    }
}

// TESTS

#[cfg(test)]
mod tests {

    // IMPORTS

    use crate::Runnable;

    #[cfg(feature = "async")]
    use futures::executor::block_on;

    #[cfg(feature = "logging")]
    use crate::tests::setup_logging;

    // TESTS

    #[test]
    fn vector_pop() {
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
}
