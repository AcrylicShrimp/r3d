use std::{num::NonZeroU32, time::Duration};
use winit::window::Window;

pub struct TargetFrameInterval {
    target_frame_millihertz: Option<NonZeroU32>,
    interval: Duration,
}

impl TargetFrameInterval {
    pub fn new(target_frame_millihertz: Option<NonZeroU32>, window: &Window) -> Self {
        Self {
            target_frame_millihertz,
            interval: compute_target_frame_interval(
                target_frame_millihertz
                    .map(|n| n.get())
                    .unwrap_or_else(|| get_window_refresh_rate_millihertz(window)),
            ),
        }
    }

    pub fn target_frame_millihertz(&self) -> Option<NonZeroU32> {
        self.target_frame_millihertz
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn update_window(&mut self, window: &Window) {
        if self.target_frame_millihertz.is_some() {
            return;
        }

        self.interval = compute_target_frame_interval(get_window_refresh_rate_millihertz(window));
    }
}

fn get_window_refresh_rate_millihertz(window: &Window) -> u32 {
    window
        .current_monitor()
        .and_then(|monitor| monitor.refresh_rate_millihertz())
        .unwrap_or(60_000)
}

fn compute_target_frame_interval(target_frame_millihertz: impl Into<u64>) -> Duration {
    Duration::from_millis(1000_000 / target_frame_millihertz.into())
}
