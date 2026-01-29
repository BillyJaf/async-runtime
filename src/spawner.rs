use std::sync::{Arc, mpsc::SyncSender};

use crate::task::Task;

pub struct Spawner<O> {
    pub task_sender: SyncSender<Arc<Task<O>>>
}

impl<O> Spawner<O> {
    pub fn new(task_sender: SyncSender<Arc<Task<O>>>) -> Self {
        Spawner { task_sender }
    }

    pub fn spawn<T>(&self, task: T)
    where T: Future<Output = O> + Send + 'static
    {
        let task = Arc::new(Task::new(task, self.task_sender.clone()));
        self.task_sender.try_send(task).expect("Too many tasks are queued.");
    }
}