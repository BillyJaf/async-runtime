use std::{sync::Arc, task::Wake, thread::{self, Thread}};

pub struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

impl ThreadWaker {
    pub fn new() -> Self {
        ThreadWaker(thread::current())
    }
}