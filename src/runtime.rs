use std::{
    collections::{HashMap, HashSet}, 
    sync::mpsc::sync_channel,
};

use crate::{executor::Executor, spawner::Spawner, timer::TIMER};

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
        TIMER.clone().start();
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
        self.executor.select()
    }

     pub fn join(self) -> HashMap<usize, O> {
        drop(self.spawner);
        self.executor.join()
    }
}