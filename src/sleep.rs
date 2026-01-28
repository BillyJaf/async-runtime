use std::{ops::Add, pin::Pin, task::{Context, Poll}, time::{Duration, Instant}};

pub struct Sleep {
    finish_time: Instant,
}

impl Sleep {
    pub fn new(seconds: u64) -> Self {
        let finish_time = Instant::now().add(Duration::from_secs(seconds));
        Sleep { finish_time }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let time_now = Instant::now();
        if time_now > self.finish_time {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}