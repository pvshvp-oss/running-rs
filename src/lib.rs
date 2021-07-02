// ENABLE LANGUAGE FEATURES

#![feature(unboxed_closures)] // to switch from parenthetical notation to generics for `Fn*` traits
#![feature(fn_traits)] // to use `call_once` and `call` methods on Fn* traits
#![feature(trait_alias)] // to give simple names for sets of traits
#![feature(min_specialization)] // for specialization of trait implementations
#![feature(stmt_expr_attributes)] // for selective evaluation of expressions based on attributes

// CRATE LEVEL DOCUMENTATION

//! `running` is a library for running *callables* (functions and closures), and
//! *external commands* (programs, scripts, and operating system commands), or a
//! set of them with optional live logging and optional asynchrony.

// DECLARE MODULES

mod callable; // for types and traits pertaining to the execution of functions and closures
mod command; // for types and traits pertaining to the execution of programs, scripts, and operating system commands
mod log; // for types and traits pertaining to logging
mod task; // for types and traits pertaining to the execution of a batch of callables and commands

// API FACADE

pub use crate::callable::*;
pub use crate::command::*;
pub use crate::log::*;
pub use crate::task::*;

// IMPORTS

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "async")]
use async_trait::async_trait;

// TRAIT ALIASES

pub trait GenericErrorTraits = Any + Send; // traits for a generic error type
pub trait GenericReturnTraits = Any + Send; // traits for a generic return type

// CUSTOM TYPES

pub type GenericErrorType = Box<(dyn GenericErrorTraits)>; // a generic error type
pub type GenericReturnType = Box<(dyn GenericReturnTraits)>; // a generic return type

// GLOBAL VARIABLES

pub static TASK_ID: AtomicUsize = AtomicUsize::new(0); // initialize the unique task ID generator

// FUNCTIONS

fn generate_task_id() -> usize {
    TASK_ID.fetch_add(1, Ordering::Relaxed)
}

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

// TRAITS

/** A trait that represents entities that can be executed (or run). This can include functions, closures, scripts, executable binaries, operating system commands (that can themselves be made up of pipes and redirections), or a set containing one or more of the above (referred to here as `Job`s)

The generic variable `R` refers to the return type whereas `E` refers to the error type.
*/
#[cfg(feature = "async")]
#[async_trait]
pub trait Runnable {
    async fn run(&mut self);
    // async fn output<T>(&mut self) -> T;
}

/** A trait that represents entities that can be executed (or run). This can include functions, closures, scripts, executable binaries, operating system commands (that can themselves be made up of pipes and redirections), or a set containing one or more of the above (referred to here as `Job`s)

The generic variable `R` refers to the return type whereas `E` refers to the error type.
*/
#[cfg(not(feature = "async"))]
pub trait Runnable {
    fn run(&mut self);
}

// STRUCTS

#[derive(Debug)]
pub struct RunError {}

#[cfg(test)]
mod tests {

    // IMPORTS

    #[cfg(feature = "logging")]
    use std::sync::Once;

    // GLOBAL VARIABLES

    #[cfg(feature = "logging")]
    pub static INIT: Once = Once::new();

    // FUNCTIONS

    #[cfg(feature = "logging")]
    pub fn setup_logging(verbosity: log::LevelFilter) -> () {
        // IMPORTS
        use fern::colors::{Color, ColoredLevelConfig};

        INIT.call_once(|| {
            let mut base_config = fern::Dispatch::new();
            base_config = match verbosity {
                log::LevelFilter::Off => base_config
                    .level(verbosity)
                    .level_for("console-target", log::LevelFilter::Off),
                log::LevelFilter::Trace => base_config
                    .level(verbosity)
                    .level_for("console-target", log::LevelFilter::Debug),
                log::LevelFilter::Debug => base_config
                    .level(verbosity)
                    .level_for("console-target", log::LevelFilter::Info),
                log::LevelFilter::Info => base_config
                    .level(verbosity)
                    .level_for("console-target", log::LevelFilter::Warn),
                log::LevelFilter::Warn => base_config
                    .level(verbosity)
                    .level_for("console-target", log::LevelFilter::Error),
                log::LevelFilter::Error => base_config
                    .level(verbosity)
                    .level_for("console-target", log::LevelFilter::Off),
            };

            let file_config = fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{}[{}][{}] {}",
                        chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        record.target(),
                        record.level(),
                        message
                    ))
                })
                .chain(
                    fern::log_file(format!(
                        "running-rs_test_{}.log",
                        chrono::Local::now().format("%Y-%m-%d_%H_%M_%S")
                    ))
                    .unwrap(),
                );

            let colors = ColoredLevelConfig::new()
                .trace(Color::Cyan)
                .info(Color::Blue)
                .debug(Color::Green)
                .warn(Color::Yellow)
                .error(Color::Red);
            let stdout_config = fern::Dispatch::new()
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        record.target(),
                        colors.color(record.level()),
                        message
                    ))
                })
                .chain(std::io::stdout());

            base_config
                .chain(file_config)
                .chain(stdout_config)
                .apply()
                .unwrap();
        })
    }

    // TESTS

    #[test]
    #[cfg(feature = "logging")]
    fn try_string_from() {
        let value: isize = 5;
        assert_eq!(String::from("5"), crate::try_string_from(&value).unwrap())
    }
}
