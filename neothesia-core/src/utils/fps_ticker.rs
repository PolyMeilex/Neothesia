// The MIT License (MIT)
// Copyright (c) 2020 mitchmindtree

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Simple type for tracking frames-per-second.
#[derive(Clone, Debug)]
pub struct Fps {
    window_len: usize,

    window: VecDeque<Duration>,
    last: Instant,
    avg: f64,
    min: f64,
    max: f64,
}

impl Fps {
    /// The window length used by the default constructor.
    pub const DEFAULT_WINDOW_LEN: usize = 60;

    /// Create a new `Fps` with the given window length as a number of frames.
    ///
    /// The larger the window, the "smoother" the FPS.
    pub fn new(window_len: usize) -> Self {
        let window = VecDeque::with_capacity(window_len);
        let last = Instant::now();
        let (avg, min, max) = (0.0, 0.0, 0.0);
        Fps {
            window_len,
            window,
            last,
            avg,
            min,
            max,
        }
    }

    /// Call this once per frame to allow the `Fps` instance to sample the rate internally.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        while self.window.len() + 1 > self.window_len {
            self.window.pop_front();
        }
        self.window.push_back(delta);
        self.avg = self.calc_avg();
        self.min = self.calc_min();
        self.max = self.calc_max();
    }

    /// Retrieve the average frames-per-second at the moment of the last call to `tick`.
    pub fn avg(&self) -> f64 {
        self.avg
    }

    /// Retrieve the minimum frames-per-second that was reached within the window at the moment
    /// `tick` was last called.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Retrieve the maximum frames-per-second that was reached within the window at the moment
    /// `tick` was last called.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Calculate the frames per second from the current state of the window.
    fn calc_avg(&self) -> f64 {
        let sum_secs = self.window.iter().map(|d| d.as_secs_f64()).sum::<f64>();
        1.0 / (sum_secs / self.window.len() as f64)
    }

    /// Find the minimum frames per second that occurs over the window.
    fn calc_min(&self) -> f64 {
        1.0 / self
            .window
            .iter()
            .max()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }

    /// Find the minimum frames per second that occurs over the window.
    fn calc_max(&self) -> f64 {
        1.0 / self
            .window
            .iter()
            .min()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }
}

impl Default for Fps {
    fn default() -> Self {
        Fps::new(Self::DEFAULT_WINDOW_LEN)
    }
}
