use std::{
    pin::Pin, 
    task::{Context, Poll}, 
    time::{Duration, Instant}
};

use crate::timer::TIMER;

pub struct Sleep {
    instant_finish: Instant,
    registered: bool,
}

impl Sleep {
    pub fn new(seconds: u64) -> Self {
        let instant_finish = Instant::now() + Duration::from_secs(seconds);
        Sleep { instant_finish, registered: false }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.instant_finish {
            return Poll::Ready(());
        }

        if !self.registered {
            self.registered = true;
            TIMER.clone().register(self.instant_finish, cx.waker().clone());
        }

        Poll::Pending
    }
}