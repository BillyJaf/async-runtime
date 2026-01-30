use std::{
    collections::{HashMap, HashSet}, sync::{Arc, mpsc::sync_channel}, task::{Context, Poll, Waker}
};

use crate::{executor::Executor, spawner::Spawner, timer::{Timer, TIMER}};

pub struct Runtime<O> {
    last_used_task_id: usize,
    task_ids: HashSet<usize>,
    executor: Executor<O>,
    spawner: Spawner<O>
}

#[derive(Debug)]
pub enum TaskSpawnError {
    DuplicateId
}

impl<O: 'static> Runtime<O> {
    pub fn new() -> Self {
        const MAX_TASKS: usize = 1000;
        let (task_sender, task_queue) = sync_channel(MAX_TASKS);
        let timer = TIMER.get_or_init(|| Arc::new(Timer::new())).clone();
        timer.start();
        Runtime {
            last_used_task_id: 1,
            task_ids: HashSet::new(),
            executor: Executor::new(task_queue),
            spawner: Spawner::new(task_sender)
        }
    }

    pub fn spawn<T>(&mut self, task: T) -> usize
    where T: Future<Output = O> + Send + 'static
    {
        let mut task_id = self.last_used_task_id;
        while self.task_ids.contains(&task_id) {
            task_id += 1;
        }
        self.last_used_task_id = task_id;
        self.task_ids.insert(task_id);
        self.spawner.spawn(task_id, task);
        task_id
    }

    pub fn spawn_with_id<T>(&mut self, id: usize, task: T) -> Result<usize, TaskSpawnError>
    where T: Future<Output = O> + Send + 'static
    {   
        if self.task_ids.contains(&id) {
            return Err(TaskSpawnError::DuplicateId);
        }

        self.task_ids.insert(id);
        self.spawner.spawn(id, task);
        Ok(id)
    }

    pub fn select(self) -> Option<O> {
        drop(self.spawner);
        while let Ok(locked_task) = self.executor.task_queue.recv() {
            let mut task_slot = locked_task.task.lock().unwrap();
            if let Some(mut task) = task_slot.take() {
                let waker = Waker::from(locked_task.clone());
                let context = &mut Context::from_waker(&waker);
                match task.as_mut().poll(context) {
                    Poll::Pending => { *task_slot = Some(task); },
                    Poll::Ready(output) => { return Some(output); }
                }
            }
        }
        return None;
    }

     pub fn join(self) -> HashMap<usize, O> {
        drop(self.spawner);

        let mut results_by_id: HashMap<usize, O> = HashMap::new();

        while let Ok(locked_task) = self.executor.task_queue.recv() {
            let mut task_slot = locked_task.task.lock().unwrap();
            if let Some(mut task) = task_slot.take() {
                let waker = Waker::from(locked_task.clone());
                let context = &mut Context::from_waker(&waker);
                match task.as_mut().poll(context) {
                    Poll::Pending => { *task_slot = Some(task); },
                    Poll::Ready(output) => { results_by_id.insert(locked_task.id, output); }
                }
            }
        }

        return results_by_id;
    }
}