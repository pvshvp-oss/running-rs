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

pub use crate::callable::*; // export the types and traits in callable.rs for public use
pub use crate::command::*; // export the types and traits in command.rs for public use
pub use crate::log::*; // export the types and traits in log.rs for public use
pub use crate::task::*; // export the types and traits in task.rs for public use

#[cfg(test)]
mod tests {

    // IMPORTS

    use std::sync::Once;
    use fern::colors::{Color, ColoredLevelConfig};

    // GLOBAL VARIABLES

    pub static INIT: Once = Once::new();

    // FUNCTIONS

    pub fn setup_logging(verbosity: log::LevelFilter) -> () {
            
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
