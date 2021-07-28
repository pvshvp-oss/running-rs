#![feature(unboxed_closures)] // to switch from parenthetical notation to generics for `Fn*` traits
#![feature(fn_traits)] // to use `call_once` and `call` methods on Fn* traits
#![feature(trait_alias)] // to give simple names for sets of traits
#![feature(specialization)] // for specialization of trait implementations
#![feature(stmt_expr_attributes)] // for selective evaluation of expressions based on attributes
#![feature(exhaustive_patterns)]

// CRATE LEVEL DOCUMENTATION

//! `running` is a library for running *callables* (functions and closures), and
//! *external commands* (programs, scripts, and operating system commands), or a
//! set of them with optional live logging and optional asynchrony.

// IMPORTS

use std::any::Any;

// DECLARE MODULES

mod callable; // for types and traits pertaining to the execution of functions and closures
mod instruction; // for types and traits pertaining to the execution of programs, scripts, and operating system commands
mod runnable; // for types and traits pertaining to the execution of a batch of callables and commands

// API FACADE

pub use crate::callable::*; // export the types and traits in callable.rs for public use
pub use crate::instruction::*; // export the types and traits in command.rs for public use
pub use crate::runnable::*; // export the types and traits in task.rs for public use

// CUSTOM TYPES

/// A trait for a general error type
pub trait GeneralErrorTrait = Any + Send;
/// A trait for a genera return type
pub trait GeneralReturnTrait = Any + Send;

/// A type representing a general error type
pub type GeneralErrorType = Box<(dyn GeneralErrorTrait)>;
/// A type representing a general return type
pub type GeneralReturnType = Box<(dyn GeneralReturnTrait)>; // a generic return type

/// A trait that is implemented by the types of logging supported (LoggedKind and UnLoggedKind)
pub trait LoggingType {}
/// An empty type to indicate that something is logged
pub struct LoggedKind {}
/// An empty type to indicate that something is not logged
pub struct UnLoggedKind {}

/// A trait that is implemented by the types of synchrony supported (BlockingKind and AsyncKind)
pub trait SynchronyType {}
/// An empty type to indicate that something is blocking
pub struct BlockingKind {}
/// An empty type to indicate that something is asynchronous
pub struct AsyncKind {}

// TRAIT IMPLEMENTATIONS

impl LoggingType for LoggedKind {}
impl LoggingType for UnLoggedKind {}
impl SynchronyType for BlockingKind {}
impl SynchronyType for AsyncKind {}

#[cfg(test)]
mod tests {

    // IMPORTS

    use fern::colors::{Color, ColoredLevelConfig}; // for setting up logging colors on the console
    use std::sync::Once; // for calling the log initialization once

    // GLOBAL VARIABLES

    pub static LOGGING_INITIALIZER: Once = Once::new();

    // FUNCTIONS

    /// Initializes logging once, only the first time it is called
    pub fn setup_logging(verbosity: log::LevelFilter) -> () {
        LOGGING_INITIALIZER.call_once(|| {
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
}
