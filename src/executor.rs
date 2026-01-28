use std::{collections::{HashMap, HashSet}, task::{Context, Poll}};

use crate::task::Task;

pub struct Executor<O> {
    task_ids: HashSet<usize>,
    tasks: Vec<Task<O>>,
}

#[derive(Debug)]
pub enum ExecutorError {
    DuplicateId,
}

impl<O> Executor<O> {
    pub fn new() -> Self {
        let task_ids = HashSet::new();
        let tasks = Vec::new();
        Executor { task_ids, tasks }
    }

    pub fn add_task<T>(&mut self, task: T, id: usize) -> Result<(), ExecutorError>
    where T: Future<Output = O> + Send + 'static
    {
        if self.task_ids.contains(&id) {
            return Err(ExecutorError::DuplicateId)
        }

        let task = Task::new(id, task);
        self.tasks.push(task);
        self.task_ids.insert(id);

        Ok(())
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

    pub fn join(mut self) -> HashMap<usize, O> {
        let mut outputs = HashMap::new();

        if self.tasks.is_empty() {
            return outputs;
        }

        loop {
            if self.tasks.len() == 0 {
                return outputs;
            }

            let mut completed_tasks = Vec::new();

            for (index, task) in self.tasks.iter_mut().enumerate() {
                let Task {id, task, waker} = task;
                
                let mut cx = Context::from_waker(&waker);

                match task.as_mut().poll(&mut cx) {
                    Poll::Pending => { continue },
                    Poll::Ready(output) => { 
                        completed_tasks.push(index);
                        outputs.insert(*id, output);
                    },
                };
            }

            for index_of_task in completed_tasks.into_iter() {
                self.tasks.remove(index_of_task);
            }
        }
    }
}