use std::task::{Context, Poll};

use crate::task::Task;

pub struct Executor<O> {
    num_tasks: usize,
    tasks: Vec<Task<O>>,
}

impl<O> Executor<O> {
    pub fn new() -> Self {
        let num_tasks = 1;
        let tasks = Vec::new();
        Executor { num_tasks, tasks }
    }

    pub fn add_task<T>(&mut self, task: T)
    where T: Future<Output = O> + Send + 'static
    {
        let task = Task::new(self.num_tasks, task);
        self.num_tasks += 1;
        self.tasks.push(task);
    }

    pub fn select(mut self) -> Option<O> {
        if self.tasks.is_empty() {
            return None;
        }

        loop {
            for task in self.tasks.iter_mut() {
                let Task {id: _, task, waker} = task;
                
                let mut cx = Context::from_waker(&waker);
                
                match task.as_mut().poll(&mut cx) {
                    Poll::Pending => { continue },
                    Poll::Ready(output) => { return Some(output) },
                };
            }
        }
    }
}