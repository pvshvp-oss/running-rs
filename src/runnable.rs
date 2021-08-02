// IMPORTS
use crate::Run;
use std::collections::VecDeque;

pub struct Job {
    tasks: VecDeque<Box<dyn Run>>,
}
