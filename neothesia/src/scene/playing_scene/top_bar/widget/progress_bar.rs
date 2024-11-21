use nuon::{
    Color, Element, Event, LayoutCtx, MouseButton, Node, RenderCtx, Renderer, UpdateCtx, Widget,
};

use crate::scene::playing_scene::midi_player::MidiPlayer;

#[derive(Default, Debug)]
pub struct ProgressBarState {
    is_hovered: bool,
    is_pressed: bool,
}

impl ProgressBarState {
    pub fn is_grabbed(&self) -> bool {
        self.is_pressed
    }
}

pub struct ProgressBar<'a, MSG> {
    color: Color,
    player: &'a MidiPlayer,
    state: &'a mut ProgressBarState,
    on_press: Option<MSG>,
    on_release: Option<MSG>,
}

impl<'a, MSG> ProgressBar<'a, MSG> {
    pub fn new(state: &'a mut ProgressBarState, player: &'a MidiPlayer) -> Self {
        Self {
            color: Color::new_u8(255, 255, 255, 1.0),
            player,
            state,
            on_press: None,
            on_release: None,
        }
    }

    pub fn on_press(mut self, msg: MSG) -> Self {
        self.on_press = Some(msg);
        self
    }

    pub fn on_release(mut self, msg: MSG) -> Self {
        self.on_release = Some(msg);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl<'a, MSG: Clone> Widget<MSG> for ProgressBar<'a, MSG> {
    fn layout(&self, ctx: &LayoutCtx) -> Node {
        Node {
            x: ctx.x,
            y: ctx.y,
            w: ctx.w,
            h: ctx.h,
            children: vec![],
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, _ctx: &RenderCtx) {
        let progress_w = layout.w * self.player.percentage();

        renderer.quad(layout.x, layout.y, progress_w, layout.h, self.color);

        for m in self.player.song().file.measures.iter() {
            let length = self.player.length().as_secs_f32();
            let start = self.player.leed_in().as_secs_f32() / length;
            let measure = m.as_secs_f32() / length;

            let x = (start + measure) * layout.w;

            let light_measure: Color = Color::new(1.0, 1.0, 1.0, 0.5);
            let dark_measure: Color = Color::new(0.4, 0.4, 0.4, 1.0);

            let color = if x < progress_w {
                light_measure
            } else {
                dark_measure
            };

            renderer.quad(x, layout.y, 1.0, layout.h, color);
        }
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        match event {
            Event::CursorMoved { position } => {
                self.state.is_hovered = layout.contains(position.x, position.y);
            }
            Event::MousePress {
                button: MouseButton::Left,
            } => {
                if self.state.is_hovered {
                    self.state.is_pressed = true;
                    if let Some(msg) = self.on_press.clone() {
                        ctx.messages.push(msg);
                    }
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
            } => {
                if self.state.is_pressed {
                    if let Some(msg) = self.on_release.clone() {
                        ctx.messages.push(msg);
                    }
                }
                self.state.is_pressed = false;
            }
            _ => {}
        }
    }
}

impl<'a, MSG: Clone + 'static> From<ProgressBar<'a, MSG>> for Element<'a, MSG> {
    fn from(value: ProgressBar<'a, MSG>) -> Self {
        Element::new(value)
    }
}
