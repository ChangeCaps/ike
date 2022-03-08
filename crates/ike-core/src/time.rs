use std::time::{Duration, Instant};

use ike_ecs::ResMut;

pub struct Time {
    delta: Duration,
    since_startup: Duration,
    startup: Instant,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: Default::default(),
            since_startup: Default::default(),
            startup: Instant::now(),
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        let now = Instant::now();

        let since_startup = now - self.startup;
        self.delta = since_startup - self.since_startup;
        self.since_startup = since_startup;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    pub fn since_startup(&self) -> Duration {
        self.since_startup
    }

    pub fn seconds_since_startup(&self) -> f32 {
        self.since_startup.as_secs_f32()
    }

    pub(crate) fn system(mut time: ResMut<Self>) {
        time.update();
    }
}
