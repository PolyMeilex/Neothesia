use std::time::Instant;

pub struct Timer {
    pub time_elapsed: u128,
    last_time: Instant,
    pub paused: bool,
}
impl Timer {
    pub fn new() -> Self {
        Self {
            time_elapsed: 0,
            last_time: Instant::now(),
            paused: false,
        }
    }
    pub fn start(&mut self) {
        self.last_time = Instant::now();
        self.time_elapsed = 0;
    }

    #[cfg(not(feature = "record"))]
    pub fn update(&mut self) {
        if !self.paused {
            // We use nanos only because when using secs timing error quickly piles up
            // It is not visible when running 60FPS
            // but on higher refresh rate it is important
            self.time_elapsed += self.last_time.elapsed().as_nanos();
        }
        self.last_time = Instant::now();
    }

    /// Got to next frame (60FPS)
    #[cfg(feature = "record")]
    pub fn update(&mut self) {
        // 60FPS per 1s (in nanos)
        self.time_elapsed += 1000000000 / 60;
    }

    pub fn set_time(&mut self, time: f32) {
        if time > 0.0 {
            self.time_elapsed = (time * 1_000_000.0).round() as u128;
        }
    }
    pub fn get_elapsed(&self) -> f32 {
        self.time_elapsed as f32 / 1_000_000.0
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
