use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex, OnceLock}, task::Waker, time::Instant};

pub static TIMER: OnceLock<Arc<Timer>> = OnceLock::new();

#[derive(Debug)]
pub struct Timer {
    instants_and_wakers: Mutex<VecDeque<InstantWaker>>,
    condvar: Condvar,
}

#[derive(Debug)]
struct InstantWaker {
    instant: Instant,
    waker: Waker
}

impl Timer {
    pub fn new() -> Self {
        Timer { instants_and_wakers: Mutex::new(VecDeque::new()), condvar: Condvar::new() }
    }

    pub fn register(self: Arc<Self>, instant: Instant, waker: Waker) {
        let mut pairs = self.instants_and_wakers.lock().unwrap();
        pairs.push_back(InstantWaker { instant, waker });
        self.condvar.notify_one();
    }

    pub fn start(self: Arc<Self>) {
        std::thread::spawn(move || self.run());
    }

    fn run(self: Arc<Self>) {
        loop {
            let mut instants_and_wakers = self.instants_and_wakers.lock().unwrap();

            let instant_waker = {
                while instants_and_wakers.is_empty() {
                    instants_and_wakers = self.condvar.wait(instants_and_wakers).unwrap();
                }
                
                instants_and_wakers.pop_front().unwrap()
            };

            if instant_waker.instant >= Instant::now() {
                instant_waker.waker.wake();
            } else {
                let mut instants_and_wakers = self.instants_and_wakers.lock().unwrap();
                instants_and_wakers.push_back(instant_waker);
            }
        }
    }
}