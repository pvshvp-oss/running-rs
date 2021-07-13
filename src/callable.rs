// IMPORTS

use crate::generate_task_id;
use crate::GenericErrorType;
use crate::Runnable;
use crate::{AsynchronousKind, BlockingKind, SynchronyType};
use crate::{LoggedKind, LoggingType, UnLoggedKind};
use async_trait::async_trait;
use std::{fmt::Debug, panic, panic::AssertUnwindSafe};
use std::marker::PhantomData;

// MACROS

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

#[allow(unused_macros)]
macro_rules! logged_callable{
    ( $first_parent:ident $(:: $path_fragment_type_a:ident)* $(. $path_fragment_type_b:ident)* ( $($arguments:expr),* ) ) => {
        {
            let callback = || -> _ {
                $first_parent $(:: $path_fragment_type_a)* $(. $path_fragment_type_b)* ($($arguments),*)
            };
            $crate::Callable::new(callback, stringify!($first_parent$(::$path_fragment_type_a)*$(.$path_fragment_type_b)*)).args((), stringify!($($arguments),*))
        }
    };
}

// STRUCT DECLARATIONS

pub enum Callable<A, R, F, L= UnLoggedKind, S= BlockingKind>
where
    F: FnOnce<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    UnLoggedBlockingCallable(
        Option<A>, // a tuple struct representing the arguments
        Option<R>, // the return value
        F,         // the callable's handle
        PhantomData<(L,S)>,
    ),
    UnLoggedAsyncCallable(
        Option<A>, // a tuple struct representing the arguments
        Option<R>, // the return value
        F,         // the callable's handle
        PhantomData<(L,S)>,
    ),
    LoggedBlockingCallable(
        usize,     // a unique identifier for the task
        Option<A>, // a tuple struct representing the arguments
        Option<R>, // the return value
        F,         // the callable's handle
        PhantomData<(L,S)>,
    ),
    LoggedAsyncCallable(
        usize,     // a unique identifier for the task
        Option<A>, // a tuple struct representing the arguments
        Option<R>, // the return value
        F,         // the callable's handle
        PhantomData<(L,S)>,
    ),
}

impl<A, R, F> Callable<A, R, F, UnLoggedKind, BlockingKind>
where
    F: FnOnce<A, Output = R>,
{
    pub fn new(handle: F) -> Callable<A, R, F, UnLoggedKind, BlockingKind> {
        return Callable::UnLoggedBlockingCallable(
            None,   // a tuple struct representing the arguments
            None,   // the return value
            handle, // the callable's handle
            PhantomData,
        );
    }

    pub fn args(&mut self, arguments: A) -> &Self {
        self.arguments = Some(arguments);
        &self
    }
}

impl<A, R, F> Callable<A, R, F, LoggedKind, BlockingKind>
where
    F: FnOnce<A, Output = R>,
{
    pub fn new(handle: F) -> Callable<A, R, F, UnLoggedKind, BlockingKind> {
        return Callable::UnLoggedBlockingCallable(
            None,   // a tuple struct representing the arguments
            None,   // the return value
            handle, // the callable's handle
            PhantomData,
        );
    }

    pub fn args(&mut self, arguments: A) -> &Self {
        self.arguments = Some(arguments);
        &self
    }
}

pub struct Callable<A, R, F, L = UnLoggedKind, S = BlockingKind>
where
    F: FnOnce<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    handle: Option<F>,
    arguments: Option<A>,
    output: Option<Result<R, GenericErrorType>>,
}

pub struct Callable<'a, 'b, 'c, A, R, F, L = LoggedKind, S = BlockingKind>
where
    F: FnOnce<A, Output = R>,
    L: LoggingType,
    S: SynchronyType,
{
    id: usize,
    handle: Option<F>,
    arguments: Option<A>,
    output: Option<Result<R, GenericErrorType>>,
    logging_preferences: LoggingPreferences<'a>,
    logging_data: LoggingData<'b, 'c>,
}

// Type aliases for variations

// pub type LoggedCallable<A, R, F, B> = Callable<A, R, F, LoggedKind, B>;
// pub type UnLoggedCallable<A, R, F, B> = Callable<A, R, F, UnLoggedKind, B>;
// pub type LoggedBlockingCallable<A, R, F> = Callable<A, R, F, LoggedKind, BlockingKind>;
// pub type LoggedAsyncCallable<A, R, F> = Callable<A, R, F, LoggedKind, AsynchronousKind>;
// pub type UnLoggedBlockingCallable<A, R, F> = Callable<A, R, F, UnLoggedKind, BlockingKind>;
// pub type UnLoggedAsyncCallable<A, R, F> = Callable<A, R, F, UnLoggedKind, AsynchronousKind>;
// pub type BlockingCallable<A, R, F, L> = Callable<A, R, F, L, BlockingKind>;
// pub type AsyncCallable<A, R, F, L> = Callable<A, R, F, L, AsynchronousKind>;

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
