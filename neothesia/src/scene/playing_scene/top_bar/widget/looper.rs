use std::time::Duration;

use nuon::{
    Color, Element, Event, LayoutCtx, MouseButton, Node, ParentLayout, RenderCtx, Renderer, Tree,
    UpdateCtx, Widget,
};

use crate::scene::playing_scene::midi_player::MidiPlayer;

#[derive(Default, Debug)]
pub struct TickState {
    is_hovered: bool,
    is_pressed: bool,
}

#[derive(Debug, Default)]
pub struct LooperState {
    start: TickState,
    end: TickState,
}

pub struct Looper<MSG> {
    color: Color,
    on_start_move: Option<fn(Duration) -> MSG>,
    on_end_move: Option<fn(Duration) -> MSG>,
    start: Duration,
    end: Duration,
}

impl<MSG> Default for Looper<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> Looper<MSG> {
    pub fn new() -> Self {
        Self {
            color: Color::new_u8(255, 56, 187, 1.0),
            on_start_move: None,
            on_end_move: None,
            start: Duration::ZERO,
            end: Duration::ZERO,
        }
    }

    pub fn start(mut self, time: Duration) -> Self {
        self.start = time;
        self
    }

    pub fn end(mut self, time: Duration) -> Self {
        self.end = time;
        self
    }

    pub fn on_start_move(mut self, msg: fn(Duration) -> MSG) -> Self {
        self.on_start_move = Some(msg);
        self
    }

    pub fn on_end_move(mut self, msg: fn(Duration) -> MSG) -> Self {
        self.on_end_move = Some(msg);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl<MSG> Widget<MSG> for Looper<MSG> {
    type State = LooperState;

    fn layout(
        &self,
        _tree: &mut Tree<Self::State>,
        parent: &ParentLayout,
        ctx: &LayoutCtx,
    ) -> Node {
        let player = ctx.globals.get::<MidiPlayer>();

        let start = player.time_to_percentage(&self.start) * parent.w;
        let start = Node {
            x: parent.x + start,
            y: parent.y,
            w: 5.0,
            h: parent.h + 10.0,
            children: vec![],
        };

        let end = player.time_to_percentage(&self.end) * parent.w;
        let end = Node {
            x: parent.x + end,
            y: parent.y,
            w: 5.0,
            h: parent.h + 10.0,
            children: vec![],
        };

        let bg = Node {
            x: start.x,
            y: parent.y,
            w: end.x - start.x,
            h: parent.h + 10.0,
            children: vec![],
        };

        Node {
            x: parent.x,
            y: parent.y,
            w: parent.w,
            h: parent.h,
            children: vec![bg, start, end],
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        _ctx: &RenderCtx,
    ) {
        let state = tree.state.get();

        let bg = &layout.children[0];

        renderer.quad(
            bg.x,
            bg.y,
            bg.w,
            bg.h,
            Color {
                a: 0.35,
                ..self.color
            },
        );

        for (layout, state) in layout.children[1..].iter().zip([&state.start, &state.end]) {
            renderer.quad(
                layout.x,
                layout.y,
                layout.w,
                layout.h,
                if state.is_hovered || state.is_pressed {
                    Color::WHITE
                } else {
                    self.color
                },
            );
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
        let player = ctx.globals.get::<MidiPlayer>();

        match event {
            Event::CursorMoved { position } => {
                let start = &layout.children[1];
                let end = &layout.children[2];

                state.start.is_hovered = start.contains(position.x, position.y);
                state.end.is_hovered = end.contains(position.x, position.y);

                if state.start.is_pressed && position.x < end.x - 10.0 {
                    if let Some(msg) = self.on_start_move.as_ref() {
                        let w = layout.w;
                        let x = position.x;
                        let p = x / w;
                        let timestamp = player.percentage_to_time(p);
                        ctx.messages.push(msg(timestamp));
                    }

                    return;
                }

                if state.end.is_pressed && position.x > start.x + 10.0 {
                    if let Some(msg) = self.on_end_move.as_ref() {
                        let w = layout.w;
                        let x = position.x;
                        let p = x / w;
                        let timestamp = player.percentage_to_time(p);
                        ctx.messages.push(msg(timestamp));
                    }
                }
            }
            Event::MousePress {
                button: MouseButton::Left,
            } => {
                for state in [&mut state.start, &mut state.end] {
                    if state.is_hovered {
                        state.is_pressed = true;
                        ctx.grab_mouse();
                        ctx.capture_event();
                    }
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
            } => {
                for state in [&mut state.start, &mut state.end] {
                    if state.is_pressed {
                        ctx.ungrab_mouse();
                    }
                    state.is_pressed = false;
                }
            }
            _ => {}
        }
    }
}

impl<MSG: 'static> From<Looper<MSG>> for Element<MSG> {
    fn from(value: Looper<MSG>) -> Self {
        Element::new(value)
    }
}
