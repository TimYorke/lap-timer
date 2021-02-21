use circular_queue::CircularQueue;
use fine_grained::Stopwatch;

pub struct FpsMonitor {
    frame_timestamps: CircularQueue<u64>,
    stopwatch: Stopwatch,
}

impl FpsMonitor {
    pub fn start_new(moving_window_size: usize) -> Self {
        let stopwatch = Stopwatch::start_new();
        FpsMonitor { frame_timestamps: CircularQueue::with_capacity(moving_window_size), stopwatch }
    }

    pub fn on_frame(&mut self) {
        self.frame_timestamps.push(self.stopwatch.total_time());
    }

    pub fn get_fps(&self) -> Option<u32> {
        if self.frame_timestamps.is_full() {
            let oldest_time = self.frame_timestamps.asc_iter().next().unwrap();
            let newest_time = self.frame_timestamps.iter().next().unwrap();
            let av_frame_period_ns = (newest_time - oldest_time) / (self.frame_timestamps.len() - 1) as u64;
            let fps = 10u64.pow(9) / av_frame_period_ns;
            Some(fps as u32)
        } else {
            None
        }
    }
}