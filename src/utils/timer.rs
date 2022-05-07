use std::time::{Duration, Instant};

pub struct Timer {
    time: Duration,
    last_time: Instant,
    paused: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            time: Duration::ZERO,
            last_time: Instant::now(),
            paused: false,
        }
    }

    pub fn start(&mut self) {
        self.last_time = Instant::now();
        self.time = Duration::ZERO;
    }

    #[cfg(not(feature = "record"))]
    pub fn update(&mut self) {
        if !self.paused {
            self.time += self.last_time.elapsed();
        }
        self.last_time = Instant::now();
    }

    /// Got to next frame (60FPS)
    #[cfg(feature = "record")]
    pub fn update(&mut self) {
        // 60FPS per 1s (in nanos)
        self.time += Duration::from_secs(1) / 60;
    }

    pub fn set_time(&mut self, time: Duration) {
        self.time = time;
    }

    pub fn time(&self) -> Duration {
        self.time
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn pause_resume(&mut self) {
        if self.paused {
            self.paused = false;
        } else {
            self.paused = true;
        }
    }
}
