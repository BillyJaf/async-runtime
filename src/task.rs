use std::{
    pin::Pin, sync::Arc, task::Waker
};

use crate::waker::ThreadWaker;

pub struct Task<O> {
    pub id: usize,
    pub task: Pin<Box<dyn Future<Output = O> + Send>>,
    pub waker: Waker
}

impl<O> Task<O> {
    pub fn new<T>(id: usize, unpinned_task: T) -> Self 
    where T: Future<Output = O> + Send + 'static
    {
        let task = Box::pin(unpinned_task);
        let waker = Waker::from(Arc::new(ThreadWaker::new()));
        Task { id, task, waker }
    }
}