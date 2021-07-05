// IMPORTS

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "async")]
use async_trait::async_trait;

// GLOBAL VARIABLES

static TASK_ID_GENERATOR: AtomicUsize = AtomicUsize::new(0); // initialize the unique task ID generator

// GLOBAL FUNCTIONS

pub fn generate_task_id() -> usize {
    TASK_ID_GENERATOR.fetch_add(1, Ordering::Relaxed)
}

// STRUCTS

pub struct Job {
    tasks: VecDeque<Box<dyn Runnable>>,
}

// IMPORTS

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "async")]
use async_trait::async_trait;

// GLOBAL VARIABLES

static TASK_ID_GENERATOR: AtomicUsize = AtomicUsize::new(0); // initialize the unique task ID generator

// GLOBAL FUNCTIONS

pub fn generate_task_id() -> usize {
    TASK_ID_GENERATOR.fetch_add(1, Ordering::Relaxed)
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