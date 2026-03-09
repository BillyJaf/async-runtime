use std::sync::{
    Arc, mpsc::SyncSender
};

use crate::task::Task;

pub struct Spawner<O> {
    pub task_sender: SyncSender<Arc<Task<O>>>
}

impl<O> Spawner<O> {
    pub fn new(task_sender: SyncSender<Arc<Task<O>>>) -> Self {
        Spawner { task_sender }
    }

    pub fn spawn<T>(&self, id: usize, task: T)
    where T: Future<Output = O> + Send + 'static
    {
        let task = Arc::new(Task::new(id, task, self.task_sender.clone()));
        // Can safely call unwrap as the receiver will still be active while spawning.
        // Both the join and select methods drop the spawner immediately - the receiver
        // trivially still exists at this point of execution.
        self.task_sender.try_send(task).unwrap();
    }
}