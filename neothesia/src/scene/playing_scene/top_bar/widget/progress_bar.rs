use nuon::{
    Color, Element, Event, LayoutCtx, MouseButton, Node, ParentLayout, RenderCtx, Renderer, Tree,
    UpdateCtx, Widget,
};

use crate::scene::playing_scene::midi_player::MidiPlayer;

#[derive(Default, Debug)]
pub struct ProgressBarState {
    is_hovered: bool,
    is_pressed: bool,
}

pub struct ProgressBar<MSG> {
    color: Color,
    on_press: Option<MSG>,
    on_release: Option<MSG>,
}

impl<MSG> Default for ProgressBar<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> ProgressBar<MSG> {
    pub fn new() -> Self {
        Self {
            color: Color::new_u8(255, 255, 255, 1.0),
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

impl<MSG: Clone> Widget<MSG> for ProgressBar<MSG> {
    type State = ProgressBarState;

    fn layout(
        &self,
        _tree: &mut Tree<Self::State>,
        parent: &ParentLayout,
        _ctx: &LayoutCtx,
    ) -> Node {
        Node {
            x: parent.x,
            y: parent.y,
            w: parent.w,
            h: parent.h,
            children: vec![],
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        let _state = tree.state.get();
        let player = ctx.globals.get::<MidiPlayer>();

        let progress_w = layout.w * player.percentage();

        renderer.quad(layout.x, layout.y, progress_w, layout.h, self.color);

        for m in player.song().file.measures.iter() {
            let length = player.length().as_secs_f32();
            let start = player.leed_in().as_secs_f32() / length;
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

    fn update(
        &self,
        event: Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        let state = tree.state.get_mut();

        match event {
            Event::CursorMoved { position } => {
                state.is_hovered = layout.contains(position.x, position.y);
            }
            Event::MousePress {
                button: MouseButton::Left,
            } => {
                if state.is_hovered {
                    state.is_pressed = true;
                    ctx.grab_mouse();
                    if let Some(msg) = self.on_press.clone() {
                        ctx.messages.push(msg);
                    }
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
            } => {
                if state.is_pressed {
                    ctx.ungrab_mouse();
                    if let Some(msg) = self.on_release.clone() {
                        ctx.messages.push(msg);
                    }
                }
                state.is_pressed = false;
            }
            _ => {}
        }
    }
}

impl<MSG: Clone + 'static> From<ProgressBar<MSG>> for Element<MSG> {
    fn from(value: ProgressBar<MSG>) -> Self {
        Element::new(value)
    }
}
