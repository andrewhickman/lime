use std::time::{Duration, Instant};
use std::mem;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Ticker {
    last: Instant,
    count: u64,
}

impl Ticker {
    pub(crate) fn new() -> Self {
        Ticker {
            last: Instant::now(),
            count: 0,
        }
    }

    pub(crate) fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now - self.last;
        self.last = now;

        self.count += 1;

        elapsed
    }

    pub(crate) fn split(&mut self) -> u64 {
        mem::replace(&mut self.count, 0)
    }
}