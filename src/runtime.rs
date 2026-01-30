use std::{sync::{Arc, mpsc::sync_channel}, task::{Context, Poll, Waker}};

use crate::{executor::Executor, spawner::Spawner, timer::{Timer, TIMER}};

pub struct Runtime<O> {
    executor: Executor<O>,
    spawner: Spawner<O>
}

impl<O: 'static> Runtime<O> {
    pub fn new() -> Self {
        const MAX_TASKS: usize = 1000;
        let (task_sender, task_queue) = sync_channel(MAX_TASKS);
        let timer = TIMER.get_or_init(|| Arc::new(Timer::new())).clone();
        timer.start();
        Runtime {
            executor: Executor::new(task_queue),
            spawner: Spawner::new(task_sender)
        }
    }

    pub fn spawn<T>(&self, task: T)
    where T: Future<Output = O> + Send + 'static
    {
        self.spawner.spawn(task);
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
}