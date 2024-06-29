pub use lilt::*;

#[derive(Debug)]
pub struct Animation {
    raw: f32,
    up_speed: f32,
    down_speed: f32,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            raw: 0.0,
            up_speed: 0.04,
            down_speed: 0.1,
        }
    }
}

impl Animation {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn set_up_speed(&mut self, speed: f32) {
        self.up_speed = speed;
    }

    #[allow(dead_code)]
    pub fn set_down_speed(&mut self, speed: f32) {
        self.down_speed = speed;
    }

    pub fn is_done(&self) -> bool {
        self.raw == 0.0
    }

    fn clamp(&mut self) {
        self.raw = self.raw.min(1.0);
        self.raw = self.raw.max(0.0);
    }

    pub fn update(&mut self, up: bool) {
        if up {
            self.raw += self.up_speed;
        } else {
            self.raw -= self.down_speed;
        }
        self.clamp();
    }

    pub fn expo_out(&self, up: bool) -> f32 {
        if up {
            expo_out(self.raw)
        } else {
            self.raw
        }
    }
}

/// exponential out curve
pub fn expo_out(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0f32.powf(-10.0 * t)
    }
}
