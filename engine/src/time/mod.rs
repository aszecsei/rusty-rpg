use std::time::{Duration, Instant};

pub struct TimeManager {
    first_start: Instant,
    last_time: Instant,
}

impl TimeManager {
    pub fn new() -> Self {
        let now = Instant::now();
        TimeManager {
            first_start: now,
            last_time: now,
        }
    }

    pub fn time_since_start(&self) -> Duration {
        Instant::now() - self.first_start
    }

    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let delta_time = now - self.last_time;
        self.last_time = now;
        delta_time
    }
}