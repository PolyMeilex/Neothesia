use crate::target::Target;

#[derive(Default)]
pub struct ToastManager {
    toast: Option<Toast>,
}

impl ToastManager {
    pub fn update(&mut self, target: &mut Target) {
        if let Some(mut toast) = self.toast.take() {
            self.toast = if toast.draw(target) {
                Some(toast)
            } else {
                None
            };
        }
    }

    pub fn toast(&mut self, text: String) {
        self.toast = Some(Toast::new(move |target| {
            let text = vec![wgpu_glyph::Text::new(&text)
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(20.0)];

            target.text_renderer.queue_text(wgpu_glyph::Section {
                text,
                screen_position: (0.0, 20.0),
                layout: wgpu_glyph::Layout::Wrap {
                    line_breaker: Default::default(),
                    h_align: wgpu_glyph::HorizontalAlign::Left,
                    v_align: wgpu_glyph::VerticalAlign::Top,
                },
                ..Default::default()
            });
        }));
    }

    pub fn speed_toast(&mut self, speed: f32) {
        self.toast(format!("Speed: {}", (speed * 100.0).round() / 100.0));
    }

    pub fn offset_toast(&mut self, offset: f32) {
        self.toast(format!("Offset: {}", (offset * 100.0).round() / 100.0));
    }
}

struct Toast {
    start_time: std::time::Instant,
    inner_draw: Box<dyn Fn(&mut Target)>,
}

impl Toast {
    fn new(draw: impl Fn(&mut Target) + 'static) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            inner_draw: Box::new(draw),
        }
    }

    fn draw(&mut self, target: &mut Target) -> bool {
        let time = self.start_time.elapsed().as_secs();

        if time < 1 {
            (*self.inner_draw)(target);

            true
        } else {
            false
        }
    }
}
