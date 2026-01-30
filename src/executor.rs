use std::sync::{
    Arc, 
    mpsc::Receiver
};

use crate::task::Task;

pub struct Executor<O> {
    pub task_queue: Receiver<Arc<Task<O>>>
}

impl<O> Executor<O> {
    pub fn new(task_queue: Receiver<Arc<Task<O>>>) -> Self {
        Executor { task_queue }
    }
}