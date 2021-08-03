// IMPORTS
use crate::Run;
use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
};

// #[cfg(feature = "async")]
// use {async_trait::async_trait, tokio::process::Child};

// #[cfg(not(feature = "async"))]
// use std::process::Child;

// // TODO
// /*
// - Logging
// - Async
// - Pipe method and operator
// */

// // STRUCT DECLARATIONS

// #[cfg(not(feature = "async"))]
// pub struct Command {
//     inner_command: std::process::Command,
//     result: Option<std::io::Result<Child>>,
// }

// #[cfg(feature = "async")]
// pub struct Command {
//     inner_command: tokio::process::Command,
//     result: Option<std::io::Result<Child>>,
// }

// // STRUCT IMPLEMENTATIONS

// impl Command {
//     pub fn new<S: AsRef<OsStr>>(program: S) -> Command {
//         Command {
//             result: None,

//             #[cfg(feature = "async")]
//             inner_command: tokio::process::Command::new(program),

//             #[cfg(not(feature = "async"))]
//             inner_command: std::process::Command::new(program),
//         }
//     }

//     pub fn arg<S: AsRef<OsStr>>(&mut self, argument: S) -> &mut Command {
//         self.inner_command.arg(argument);
//         self
//     }

//     pub fn args<I, S>(&mut self, arguments: I) -> &mut Command
//     where
//         I: IntoIterator<Item = S>,
//         S: AsRef<OsStr>,
//     {
//         self.inner_command.args(arguments);
//         self
//     }
// }

// // TRAIT IMPLEMENTATIONS

// impl Deref for Command {
//     #[cfg(feature = "async")]
//     type Target = tokio::process::Command;

//     #[cfg(not(feature = "async"))]
//     type Target = std::process::Command;

//     fn deref(&self) -> &Self::Target {
//         &self.inner_command
//     }
// }

// impl DerefMut for Command {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner_command
//     }
// }

// #[cfg(feature = "async")]
// #[async_trait]
// impl Runnable for Command {
//     async fn run(&mut self) {
//         self.result = Some(self.inner_command.spawn().await);
//     }
// }

// #[cfg(not(feature = "async"))]
// impl Run for Command {
//     fn run(&mut self) {
//         self.result = Some(self.inner_command.spawn());
//     }
// }

// TESTS

// #[cfg(test)]
// mod tests {

//     // IMPORTS

//     #[cfg(feature = "async")]
//     use tokio::runtime::Runtime;

//     #[cfg(feature = "logging")]
//     use crate::tests::setup_logging;

//     // TESTS

//     #[test]
//     fn echo() {
//         #[cfg(feature = "logging")]
//         setup_logging(log::LevelFilter::Debug);

//         #[cfg(feature = "async")]
//         {
//             let runtime = Runtime::new().unwrap();
//             let output: String = String::from_utf8(
//                 runtime
//                     .block_on(async {
//                         super::Command::new("echo")
//                             .arg("Hello")
//                             .arg("World")
//                             .output()
//                             .await
//                     })
//                     .expect("Unable to run...")
//                     .stdout,
//             )
//             .expect("Unable to convert from utf-8 to String");
//             assert_eq!(output, String::from("Hello World\n"));
//         };

//         #[cfg(not(feature = "async"))]
//         {
//             let output: String = String::from_utf8(
//                 super::Command::new("echo")
//                     .arg("Hello")
//                     .arg("World")
//                     .output()
//                     .expect("Unable to run...")
//                     .stdout,
//             )
//             .expect("Unable to convert from utf-8 to String");
//             assert_eq!(output, String::from("Hello World\n"));
//         };
//     }
// }
