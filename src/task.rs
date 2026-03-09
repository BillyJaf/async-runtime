use std::{
    pin::Pin, 
    sync::{Arc, Mutex, mpsc::SyncSender},
    task::Wake
};

pub struct Task<O> {
    pub id: usize,
    // We have to wrap the task in a Mutex to satisfy the compiler.
    // Even though the main thread's executor is the only thread that drives
    // progress, the compiler is not aware that only one reference to the task
    // will ever exist at one time. For there to be two references, a task must
    // have been sent twice without being tended to by the receiver - which is
    // not possible as the receiver is the only driver.
    pub task: Mutex<Pin<Box<dyn Future<Output = O> + Send>>>,
    pub task_sender: SyncSender<Arc<Task<O>>>,
}

impl<O> Task<O> {
    pub fn new<T>(id: usize, task: T, task_sender: SyncSender<Arc<Task<O>>>) -> Self
    where T: Future<Output = O> + Send + 'static
    {   
        Task { id, task: Mutex::new(Box::pin(task)), task_sender }
    }
}

impl<O> Wake for Task<O> {
    fn wake(self: Arc<Self>) {
        let cloned = self.clone();
        // It is safe to unwrap this method. This can only fail if the receiver has shutdown
        // and we attempt to send a task, however, for the receiver to shutdown, the method
        // Timer::shutdown_and_empty must have been called, thus emptying the heap of wakers.
        self.task_sender.try_send(cloned).unwrap();
    }
}