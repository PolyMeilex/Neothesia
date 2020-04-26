#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use crate::web_wrappers::time::Instant;

pub struct TimeMenager {
    fps: Fps,
    timer: Option<Timer>,
    pub last_time: Instant,
}

impl Default for TimeMenager {
    fn default() -> Self {
        Self {
            fps: Fps::new(),
            timer: None,
            last_time: Instant::now(),
        }
    }
}

impl TimeMenager {
    pub fn new() -> Self {
        TimeMenager::default()
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
    pub fn pause_resume_timer(&mut self) {
        if let Some(timer) = &mut self.timer {
            if timer.paused {
                timer.paused = false;
            } else {
                timer.paused = true;
            }
        }
    }
    pub fn timer_get_elapsed(&mut self) -> Option<f32> {
        if let Some(timer) = &mut self.timer {
            Some(timer.time_elapsed as f32 / 1_000_000.0)
        } else {
            None
        }
    }
    pub fn clear_timer(&mut self) {
        self.timer = None;
    }
    pub fn update(&mut self) {
        self.last_time = Instant::now();
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
    last_time: Instant,
    pub paused: bool,
}
impl Timer {
    fn new() -> Self {
        Self {
            time_elapsed: 0,
            last_time: Instant::now(),
            paused: false,
        }
    }
    fn update(&mut self) {
        if !self.paused {
            // We use nanos only because when using secs timing error quickly piles up
            // It is not visible when running 60FPS
            // but on higher refresh rate it is important
            self.time_elapsed += self.last_time.elapsed().as_nanos();
        }
        self.last_time = Instant::now();
    }
}

struct Fps {
    fps: i32,
    fps_counter: i32,
    last_time: Instant,
}
impl Fps {
    fn new() -> Self {
        Self {
            fps: 0,
            fps_counter: 0,
            last_time: Instant::now(),
        }
    }
    fn update(&mut self) {
        self.fps_counter += 1;

        if self.last_time.elapsed().as_secs() >= 1 {
            self.last_time = Instant::now();

            self.fps = self.fps_counter;

            self.fps_counter = 0;
        }
    }
}
