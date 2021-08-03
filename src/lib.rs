#![feature(unboxed_closures)] // to switch from parenthetical notation to generics for `Fn*` traits
#![feature(fn_traits)] // to use `call_once` and `call` methods on Fn* traits
#![feature(trait_alias)] // to give simple names for sets of traits
#![feature(specialization)] // for specialization of trait implementations
#![feature(stmt_expr_attributes)] // for selective evaluation of expressions based on attributes

//! `running` is a library for running *callables* (functions and closures), and
//! *external commands* (programs, scripts, and operating system commands), or a
//! set of them with optional live logging and optional asynchrony.

use async_trait::async_trait;
use snafu::ResultExt;
use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicUsize, Ordering};

mod callable; // for types and traits pertaining to the execution of functions and closures
mod instruction; /* for types and traits pertaining to the execution of programs, scripts, and
                  * operating system commands */
mod runnable; // for types and traits pertaining to the execution of a batch of callables and
              // commands

pub trait ErrorTrait: std::error::Error + snafu::ErrorCompat {}
impl<T> ErrorTrait for T where T: std::error::Error + snafu::ErrorCompat {}
pub type Error = Box<dyn ErrorTrait>;

static TASK_ID_GENERATOR: AtomicUsize = AtomicUsize::new(0); // initialize the unique task ID generator
pub fn generate_task_id() -> usize {
    TASK_ID_GENERATOR.fetch_add(1, Ordering::Relaxed)
}

pub trait Represent {
    fn represent(&self) -> String;
}

impl<T> Represent for T {
    default fn represent(&self) -> String {
        return String::new();
    }
}

impl<T> Represent for T
where
    T: Debug,
{
    default fn represent(&self) -> String {
        return format!("{:?}", self);
    }
}

impl<T> Represent for T
where
    T: Display + Debug,
{
    fn represent(&self) -> String {
        return format!("{}", self);
    }
}

/// A trait that represents entities that can be executed (or run). This can
/// include functions, closures, scripts, executable binaries, operating system
/// commands, or a set containing one or more of the above
pub trait Run {
    fn run(&mut self) -> Result<(), Error>;
}

/// Does what the [Run] trait does, but calls the callback function with the
/// return value when complete
pub trait RunAndCallback: RunAndReturn {
    fn run_and_then<C: FnOnce(Self::ReturnType) -> ()>(&mut self, callback: C)
        -> Result<(), Error>;
}

/// Does what the [Run] trait does, but returns the
/// return value when complete
pub trait RunAndReturn {
    type ReturnType;

    fn run_and_return(&mut self) -> Result<Self::ReturnType, Error>;
}

/// Does what the [Run] trait does, but returns the
/// debug string of the retrn value when complete
pub trait RunAndDebug: RunAndReturn {
    fn run_and_debug(&mut self) -> Result<String, Error>;
}

/// Does what the [Run] trait does, but returns the
/// display string of the retrn value when complete
pub trait RunAndDisplay: RunAndReturn {
    fn run_and_display(&mut self) -> Result<String, Error>;
}

/// A trait that represents entities that can be executed (or run)
/// asynchronously. This can include functions, closures, scripts, executable
/// binaries, operating system commands, or a set containing one or more of the
/// above
#[async_trait]
pub trait AsyncRun {
    async fn async_run(&mut self) -> Result<(), Error>;
}

/// Does what the [AsyncRun] trait does, but calls the callback function with
/// the return value when complete
#[async_trait]
pub trait AsyncRunAndCallback {
    type ReturnType;

    fn async_run_and_then<C: FnOnce(Self::ReturnType) -> ()>(
        &mut self,
        callback: C,
    ) -> Result<(), Error>;
}

/// Does what the [AsyncRun] trait does, but returns the
/// return value when complete
pub trait AsyncRunAndReturn {
    type ReturnType;

    fn async_run_and_return(&mut self) -> Result<Self::ReturnType, Error>;
}

/// Does what the [AsyncRun] trait does, but returns the
/// debug string of the retrn value when complete
pub trait AsyncRunAndDebug {
    fn async_run_and_debug(&mut self) -> Result<String, Error>;
}

/// Does what the [AsyncRun] trait does, but returns the
/// display string of the retrn value when complete
pub trait AsyncRunAndDisplay {
    fn async_run_and_display(&mut self) -> Result<String, Error>;
}

#[cfg(test)]
mod tests {

    // IMPORTS

    use fern::colors::{Color, ColoredLevelConfig}; /* for setting up logging colors on the
                                                    * console */
    use std::sync::Once; // for calling the log initialization once

    // GLOBAL VARIABLES

    pub static LOGGING_INITIALIZER: Once = Once::new();

    // FUNCTIONS

    /// Initializes logging once, only the first time it is called
    pub fn setup_logging(verbosity: log::LevelFilter) -> () {
        LOGGING_INITIALIZER.call_once(|| {
            let mut base_config = fern::Dispatch::new();
            base_config = match verbosity {
                log::LevelFilter::Off => {
                    base_config.level(verbosity).level_for("console-target", log::LevelFilter::Off)
                }
                log::LevelFilter::Trace => {
                    base_config
                        .level(verbosity)
                        .level_for("console-target", log::LevelFilter::Debug)
                }
                log::LevelFilter::Debug => {
                    base_config.level(verbosity).level_for("console-target", log::LevelFilter::Info)
                }
                log::LevelFilter::Info => {
                    base_config.level(verbosity).level_for("console-target", log::LevelFilter::Warn)
                }
                log::LevelFilter::Warn => {
                    base_config
                        .level(verbosity)
                        .level_for("console-target", log::LevelFilter::Error)
                }
                log::LevelFilter::Error => {
                    base_config.level(verbosity).level_for("console-target", log::LevelFilter::Off)
                }
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

            base_config.chain(file_config).chain(stdout_config).apply().unwrap();
        })
    }
}
