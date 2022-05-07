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

pub struct Fps {
    fps: i32,
    fps_counter: i32,
    last_time: Instant,
}

impl Default for Fps {
    fn default() -> Self {
        Self::new()
    }
}

impl Fps {
    pub fn new() -> Self {
        Self {
            fps: 0,
            fps_counter: 0,
            last_time: Instant::now(),
        }
    }

    pub fn fps(&self) -> i32 {
        self.fps
    }

    pub fn update(&mut self) {
        self.fps_counter += 1;

        if self.last_time.elapsed().as_secs() >= 1 {
            self.last_time = Instant::now();

            self.fps = self.fps_counter;

            self.fps_counter = 0;
        }
    }
}
