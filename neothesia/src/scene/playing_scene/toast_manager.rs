use neothesia_core::render::TextRenderer;

use crate::locals;

#[derive(Default)]
pub struct ToastManager {
    toast: Option<Toast>,
}

impl ToastManager {
    pub fn update(&mut self, text_renderer: &mut TextRenderer) {
        let Some(toast) = self.toast.as_ref() else {
            return;
        };

        let alive = toast.draw(text_renderer);
        if !alive {
            self.toast = None;
        }
    }

    pub fn toast(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast::new(text.into()));
    }

    pub fn speed_toast(&mut self, speed: f32) {
        self.toast(locals::playing::speed((speed * 100.0).round() / 100.0));
    }

    pub fn animation_speed_toast(&mut self, speed: f32) {
        self.toast(locals::playing::animation_speed(speed));
    }

    pub fn offset_toast(&mut self, offset: f32) {
        self.toast(locals::playing::offset((offset * 100.0).round() / 100.0));
    }
}

struct Toast {
    start_time: std::time::Instant,
    text: String,
}

impl Toast {
    fn new(text: String) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            text,
        }
    }

    fn draw(&self, text_renderer: &mut TextRenderer) -> bool {
        let time = self.start_time.elapsed().as_secs();

        if time < 1 {
            text_renderer.queue_text(&self.text);
            true
        } else {
            false
        }
    }
}
