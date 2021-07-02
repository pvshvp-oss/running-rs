// IMPORTS

use crate::Runnable;
use std::collections::VecDeque;

// STRUCTS

pub struct Job {
    tasks: VecDeque<Box<dyn Runnable>>,
}
