use std::{collections::HashMap, sync::{
    Arc, 
    mpsc::Receiver
}, task::{Context, Poll, Waker}};

use crate::{task::Task, timer::TIMER};

pub struct Executor<O> {
    pub task_queue: Receiver<Arc<Task<O>>>
}

impl<O: 'static> Executor<O> {
    pub fn new(task_queue: Receiver<Arc<Task<O>>>) -> Self {
        Executor { task_queue }
    }

    pub fn select(self) -> Option<O> {
        while let Ok(task_struct) = self.task_queue.recv() {
            let mut task = task_struct.task.lock().unwrap();
            let waker = Waker::from(task_struct.clone());
            let context = &mut Context::from_waker(&waker);

            if let Poll::Ready(output) = task.as_mut().poll(context) { 
                TIMER.clone().shutdown_and_empty();
                return Some(output); 
            }
        }
        TIMER.clone().shutdown_and_empty();
        return None;
    }

    pub fn join(self) -> HashMap<usize, O> {
        let mut results_by_id: HashMap<usize, O> = HashMap::new();

        while let Ok(task_struct) = self.task_queue.recv() {
            let mut task = task_struct.task.lock().unwrap();
            let waker = Waker::from(task_struct.clone());
            let context = &mut Context::from_waker(&waker);
            
            if let Poll::Ready(output) = task.as_mut().poll(context) {
                results_by_id.insert(task_struct.id, output);
            }
        }
        TIMER.clone().shutdown_and_empty();
        return results_by_id;
    }
}