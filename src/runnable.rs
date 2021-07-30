// IMPORTS

use crate::Runnable;
use std::collections::VecDeque;

pub struct Job {
    tasks: VecDeque<Box<dyn Runnable>>,
}
