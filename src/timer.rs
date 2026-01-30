use std::{
    cmp::{Ordering, Reverse}, 
    collections::BinaryHeap, 
    sync::{Arc, Condvar, Mutex, OnceLock}, 
    task::Waker, 
    time::Instant
};

pub static TIMER: OnceLock<Arc<Timer>> = OnceLock::new();

#[derive(Debug)]
pub struct Timer {
    instants_and_wakers: Mutex<BinaryHeap<Reverse<InstantWaker>>>,
    condvar: Condvar,
}

#[derive(Debug)]
struct InstantWaker {
    instant: Instant,
    waker: Waker
}

impl Timer {
    pub fn new() -> Self {
        Timer { instants_and_wakers: Mutex::new(BinaryHeap::new()), condvar: Condvar::new() }
    }

    pub fn start(self: Arc<Self>) {
        std::thread::spawn(move || 
            loop {
                let mut instants_and_wakers = self.instants_and_wakers.lock().unwrap();
                while instants_and_wakers.peek().is_none() {
                    instants_and_wakers = self.condvar.wait(instants_and_wakers).unwrap();
                }

                while let Some(Reverse(instant_waker)) = instants_and_wakers.pop() {
                    if instant_waker.instant <= Instant::now() {
                        instant_waker.waker.wake();
                    } else {
                        instants_and_wakers.push(Reverse(instant_waker));
                        break;
                    }
                }
            }
        );
    }

    pub fn register(self: Arc<Self>, instant: Instant, waker: Waker) {
        let mut instants_and_wakers = self.instants_and_wakers.lock().unwrap();
        instants_and_wakers.push(Reverse(InstantWaker { instant, waker }));
        self.condvar.notify_one();
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