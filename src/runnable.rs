// IMPORTS

use std::collections::VecDeque;
use crate::Runnable; 

pub struct Job {
    tasks: VecDeque<Box<dyn Runnable>>,
}