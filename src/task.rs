use std::{
    pin::Pin, 
    sync::{Arc, Mutex, mpsc::SyncSender},
    task::Wake
};

pub struct Task<O> {
    pub id: usize,
    pub task: Mutex<Option<Pin<Box<dyn Future<Output = O> + Send>>>>,
    pub task_sender: SyncSender<Arc<Task<O>>>,
}

impl<O> Task<O> {
    pub fn new<T>(id: usize, task: T, task_sender: SyncSender<Arc<Task<O>>>) -> Self
    where T: Future<Output = O> + Send + 'static
    {   
        Task { id, task: Mutex::new(Some(Box::pin(task))), task_sender }
    }
}

impl<O> Wake for Task<O> {
    fn wake(self: Arc<Self>) {
        let cloned = self.clone();
        self.task_sender.try_send(cloned).expect("Too many tasks are queued.");
    }
}