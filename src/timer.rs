use std::{
    cmp::{Ordering, Reverse}, 
    collections::BinaryHeap, 
    sync::{Arc, Condvar, LazyLock, Mutex}, 
    task::Waker, 
    time::Instant
};

pub static TIMER: LazyLock<Arc<Timer>> = LazyLock::new(|| Arc::new(Timer::new()));

#[derive(Debug)]
pub struct Timer {
    instants_and_wakers: Mutex<ShutdownAndHeap>,
    condvar: Condvar,
}

#[derive(Debug)]
struct ShutdownAndHeap {
    shutdown: bool,
    heap: BinaryHeap<Reverse<InstantWaker>>,
}

#[derive(Debug)]
struct InstantWaker {
    instant: Instant,
    waker: Waker

}

impl Timer {
    fn new() -> Self {
        Timer { 
            instants_and_wakers: Mutex::new(
                ShutdownAndHeap { 
                    shutdown: false, 
                    heap: BinaryHeap::new(),
                }
            ), 
            condvar: Condvar::new(), 
        }
    }

    pub fn start(self: Arc<Self>) {
        std::thread::spawn(move || 
            'task: loop {
                let mut shutdown_and_heap = self.instants_and_wakers.lock().unwrap();

                if shutdown_and_heap.shutdown {
                    shutdown_and_heap.shutdown = false;
                    break 'task;
                }

                while shutdown_and_heap.heap.peek().is_none() {
                    shutdown_and_heap = self.condvar.wait(shutdown_and_heap).unwrap();

                    if shutdown_and_heap.shutdown {
                        shutdown_and_heap.shutdown = false;
                        break 'task;
                    }
                }

                while let Some(Reverse(instant_waker)) = shutdown_and_heap.heap.pop() {
                    if instant_waker.instant <= Instant::now() {
                        instant_waker.waker.wake();
                    } else {
                        shutdown_and_heap.heap.push(Reverse(instant_waker));
                        break;
                    }
                }
            }
        );
    }

    pub fn register(self: Arc<Self>, instant: Instant, waker: Waker) {
        let mut shutdown_and_heap = self.instants_and_wakers.lock().unwrap();
        shutdown_and_heap.heap.push(Reverse(InstantWaker { instant, waker }));
        self.condvar.notify_one();
    }

    pub fn shutdown_and_empty(self: Arc<Self>) {
        let mut shutdown_and_heap = self.instants_and_wakers.lock().unwrap();
        shutdown_and_heap.heap.clear();
        shutdown_and_heap.shutdown = true;
        self.condvar.notify_all();
    }
}

impl Ord for InstantWaker {
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant
            .cmp(&other.instant)
    }
}

impl PartialOrd for InstantWaker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InstantWaker {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl Eq for InstantWaker {}