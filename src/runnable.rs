// IMPORTS

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use crate::{AsyncKind, BlockingKind, SynchronyType};

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

// TRAITS

/** A trait that represents entities that can be executed (or run). This can include functions, closures, scripts, executable binaries, operating system commands (that can themselves be made up of pipes and redirections), or a set containing one or more of the above (referred to here as `Job`s)

The generic variable `R` refers to the return type whereas `E` refers to the error type.
*/
#[async_trait]
pub trait Runnable<S = AsyncKind> 
where
    S:SynchronyType  
{
    async fn run(&mut self);
    // async fn output<T>(&mut self) -> T;
}

/** A trait that represents entities that can be executed (or run). This can include functions, closures, scripts, executable binaries, operating system commands (that can themselves be made up of pipes and redirections), or a set containing one or more of the above (referred to here as `Job`s)

The generic variable `R` refers to the return type whereas `E` refers to the error type.
*/
pub trait Runnabl<S = BlockingKind> {
    fn run(&mut self);
}

// STRUCTS

#[derive(Debug)]
pub struct RunError {}