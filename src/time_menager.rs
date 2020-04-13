pub struct TimeMenager {
    fps: Fps,
    timer: Option<Timer>,
}
impl TimeMenager {
    pub fn new() -> Self {
        Self {
            fps: Fps::new(),
            timer: None,
        }
    }
    pub fn start_timer(&mut self) {
        self.timer = Some(Timer::new());
    }
    pub fn pause_timer(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.paused = true;
        }
    }
    pub fn resume_timer(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.paused = false;
        }
    }
    pub fn timer_get_elapsed(&mut self) -> Option<u128> {
        if let Some(timer) = &mut self.timer {
            Some(timer.time_elapsed)
        } else {
            None
        }
    }
    pub fn clear_timer(&mut self) {
        self.timer = None;
    }
    pub fn update(&mut self) {
        self.fps.update();
        if let Some(timer) = &mut self.timer {
            timer.update();
        }
    }
    pub fn fps(&self) -> i32 {
        self.fps.fps
    }
}

struct Timer {
    pub time_elapsed: u128,
    last_time: std::time::Instant,
    pub paused: bool,
}
impl Timer {
    fn new() -> Self {
        Self {
            time_elapsed: 0,
            last_time: std::time::Instant::now(),
            paused: false,
        }
    }
    fn update(&mut self) {
        if !self.paused {
            self.time_elapsed = self.last_time.elapsed().as_millis();
        }
        // self.last_time = std::time::Instant::now();
    }
}

struct Fps {
    fps: i32,
    fps_counter: i32,
    last_time: std::time::Instant,
}
impl Fps {
    fn new() -> Self {
        Self {
            fps: 0,
            fps_counter: 0,
            last_time: std::time::Instant::now(),
        }
    }
    fn update(&mut self) {
        self.fps_counter += 1;

        if self.last_time.elapsed().as_secs() >= 1 {
            self.last_time = std::time::Instant::now();

            self.fps = self.fps_counter;

            self.fps_counter = 0;
        }
    }
}
