use std::{
    pin::Pin, 
    sync::{Arc, Mutex}, 
    task::{Context, Poll, Waker}, 
    thread, 
    time::Duration
};

pub struct Sleep {
    sleep_state: Arc<Mutex<SleepState>>
}

struct SleepState {
    completed: bool,
    waker: Option<Waker>
}

impl Sleep {
    pub fn new(seconds: u64) -> Self {
        let sleep_state = Arc::new(Mutex::new(SleepState {
            completed: false,
            waker: None,
        }));

        let sleep_state_clone = sleep_state.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(seconds));
            let mut sleep_state = sleep_state_clone.lock().unwrap();
            sleep_state.completed = true;
            if let Some(waker) = sleep_state.waker.take() {
                waker.wake();
            }
        });

        Sleep { sleep_state }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut sleep_state = self.sleep_state.lock().unwrap();
        if sleep_state.completed {
            Poll::Ready(())
        } else {
            sleep_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}