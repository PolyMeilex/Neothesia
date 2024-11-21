use std::time::Duration;

use nuon::{
    Color, Element, Event, LayoutCtx, MouseButton, Node, RenderCtx, Renderer, UpdateCtx, Widget,
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

impl LooperState {
    pub fn is_grabbed(&self) -> bool {
        self.start.is_pressed || self.end.is_pressed
    }
}

pub struct Looper<'a, MSG> {
    color: Color,
    state: &'a mut LooperState,
    player: &'a MidiPlayer,
    on_start_move: Option<fn(Duration) -> MSG>,
    on_end_move: Option<fn(Duration) -> MSG>,
    start: Duration,
    end: Duration,
}

impl<'a, MSG> Looper<'a, MSG> {
    pub fn new(state: &'a mut LooperState, player: &'a MidiPlayer) -> Self {
        Self {
            color: Color::new_u8(255, 56, 187, 1.0),
            state,
            player,
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

impl<'a, MSG> Widget<MSG> for Looper<'a, MSG> {
    fn layout(&self, ctx: &LayoutCtx) -> Node {
        let start = self.player.time_to_percentage(&self.start) * ctx.w;
        let start = Node {
            x: ctx.x + start,
            y: ctx.y,
            w: 5.0,
            h: ctx.h + 10.0,
            children: vec![],
        };

        let end = self.player.time_to_percentage(&self.end) * ctx.w;
        let end = Node {
            x: ctx.x + end,
            y: ctx.y,
            w: 5.0,
            h: ctx.h + 10.0,
            children: vec![],
        };

        let bg = Node {
            x: start.x,
            y: ctx.y,
            w: end.x - start.x,
            h: ctx.h + 10.0,
            children: vec![],
        };

        Node {
            x: ctx.x,
            y: ctx.y,
            w: ctx.w,
            h: ctx.h,
            children: vec![bg, start, end],
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, _ctx: &RenderCtx) {
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

        for (layout, state) in layout.children[1..]
            .iter()
            .zip([&self.state.start, &self.state.end])
        {
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

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        match event {
            Event::CursorMoved { position } => {
                let start = &layout.children[1];
                let end = &layout.children[2];

                self.state.start.is_hovered = start.contains(position.x, position.y);
                self.state.end.is_hovered = end.contains(position.x, position.y);

                if self.state.start.is_pressed && position.x < end.x - 10.0 {
                    if let Some(msg) = self.on_start_move.as_ref() {
                        let w = layout.w;
                        let x = position.x;
                        let p = x / w;
                        let timestamp = self.player.percentage_to_time(p);
                        ctx.messages.push(msg(timestamp));
                    }

                    return;
                }

                if self.state.end.is_pressed && position.x > start.x + 10.0 {
                    if let Some(msg) = self.on_end_move.as_ref() {
                        let w = layout.w;
                        let x = position.x;
                        let p = x / w;
                        let timestamp = self.player.percentage_to_time(p);
                        ctx.messages.push(msg(timestamp));
                    }
                }
            }
            Event::MousePress {
                button: MouseButton::Left,
            } => {
                for state in [&mut self.state.start, &mut self.state.end] {
                    if state.is_hovered {
                        state.is_pressed = true;
                        ctx.capture_event();
                    }
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
            } => {
                for state in [&mut self.state.start, &mut self.state.end] {
                    state.is_pressed = false;
                }
            }
            _ => {}
        }
    }
}

impl<'a, MSG: 'static> From<Looper<'a, MSG>> for Element<'a, MSG> {
    fn from(value: Looper<'a, MSG>) -> Self {
        Element::new(value)
    }
}
