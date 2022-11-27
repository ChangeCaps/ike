use std::time::Instant;

use ike_ecs::system::ResMut;

#[derive(Clone, Copy, Debug)]
pub struct Time {
    start: Instant,
    last_frame: Instant,
    current_frame: Instant,
}

impl Default for Time {
    #[inline]
    fn default() -> Self {
        Self {
            start: Instant::now(),
            last_frame: Instant::now(),
            current_frame: Instant::now(),
        }
    }
}

impl Time {
    #[inline]
    pub fn update(&mut self) {
        self.last_frame = self.current_frame;
        self.current_frame = Instant::now();
    }

    #[inline]
    pub fn delta(&self) -> f32 {
        self.current_frame
            .duration_since(self.last_frame)
            .as_secs_f32()
    }

    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.current_frame.duration_since(self.start).as_secs_f32()
    }

    #[inline]
    pub fn update_system(mut time: ResMut<Self>) {
        time.update();
    }
}
