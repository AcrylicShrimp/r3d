use std::time::{Duration, Instant};

pub struct TimeManager {
    time_scale: f64,
    time: Duration,
    base_time: Duration,
    delta_time: Duration,
    unscaled_delta_time: Duration,
    initial_time: Instant,
    last_frame_time: Instant,
    last_scale_updated_time: Instant,
}

impl TimeManager {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            time_scale: 1.0,
            time: Duration::from_secs(0),
            base_time: Duration::from_secs(0),
            delta_time: Duration::from_secs(0),
            unscaled_delta_time: Duration::from_secs(0),
            initial_time: now,
            last_frame_time: now,
            last_scale_updated_time: now,
        }
    }

    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    pub fn time(&self) -> Duration {
        self.time + self.base_time
    }

    pub fn unscaled_time(&self) -> Duration {
        self.last_frame_time.duration_since(self.initial_time)
    }

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn unscaled_delta_time(&self) -> Duration {
        self.unscaled_delta_time
    }

    pub fn set_time_scale(&mut self, time_scale: f64) {
        self.time_scale = time_scale;
        self.base_time += self.time;
        self.time = Duration::from_secs(0);
        self.last_scale_updated_time = self.last_frame_time;
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.time = now
            .duration_since(self.last_scale_updated_time)
            .mul_f64(self.time_scale);
        self.delta_time = now
            .duration_since(self.last_frame_time)
            .mul_f64(self.time_scale);
        self.unscaled_delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
    }
}
