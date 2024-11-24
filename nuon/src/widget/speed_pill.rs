use crate::{
    button, Color, Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget,
};

fn minus_icon() -> &'static str {
    "\u{F2EA}"
}

fn plus_icon() -> &'static str {
    "\u{F4FE}"
}

#[derive(Default)]
pub struct SpeedPillState {
    minus: button::ButtonState,
    plus: button::ButtonState,
}

pub struct SpeedPill<'a, MSG> {
    w: f32,
    h: f32,

    minus: button::Button<'a, MSG>,
    plus: button::Button<'a, MSG>,

    speed: f32,
}

impl<'a, MSG: Clone> SpeedPill<'a, MSG> {
    pub fn new(state: &'a mut SpeedPillState) -> Self {
        let minus = button::Button::new(&mut state.minus)
            .color(Color::new_u8(67, 67, 67, 1.0))
            .hover_color(Color::new_u8(87, 87, 87, 1.0))
            .preseed_color(Color::new_u8(97, 97, 97, 1.0))
            .width(45.0)
            .height(20.0)
            .border_radius([10.0, 0.0, 10.0, 0.0]);
        let plus = button::Button::new(&mut state.plus)
            .color(Color::new_u8(67, 67, 67, 1.0))
            .hover_color(Color::new_u8(87, 87, 87, 1.0))
            .preseed_color(Color::new_u8(97, 97, 97, 1.0))
            .width(45.0)
            .height(20.0)
            .border_radius([0.0, 10.0, 0.0, 10.0]);

        Self {
            w: 90.0,
            h: 30.0,

            minus,
            plus,
            speed: 0.0,
        }
    }

    pub fn on_minus(mut self, msg: MSG) -> Self {
        self.minus = self.minus.on_click(msg);
        self
    }

    pub fn on_plus(mut self, msg: MSG) -> Self {
        self.plus = self.plus.on_click(msg);
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl<'a, MSG: Clone> Widget<MSG> for SpeedPill<'a, MSG> {
    type State = ();

    fn layout(&self, ctx: &LayoutCtx) -> Node {
        let minus = self.minus.layout(&LayoutCtx {
            x: ctx.x,
            y: ctx.y + 5.0,
            w: ctx.w,
            h: ctx.h,
        });
        let plus = self.plus.layout(&LayoutCtx {
            x: ctx.x + minus.w,
            y: ctx.y + 5.0,
            w: ctx.w,
            h: ctx.h,
        });

        Node {
            x: ctx.x,
            y: ctx.y,
            w: self.w,
            h: self.h,
            children: vec![minus, plus],
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, ctx: &RenderCtx) {
        self.minus.render(renderer, &layout.children[0], ctx);
        self.plus.render(renderer, &layout.children[1], ctx);

        let pad = 2.0;

        {
            let minus = &layout.children[0];
            renderer.icon(minus.x + pad, minus.y, minus.h, minus_icon());
        }
        {
            let plus = &layout.children[1];
            let icon_size = plus.h;
            renderer.icon(
                plus.x + plus.w - icon_size - pad,
                plus.y,
                icon_size,
                plus_icon(),
            );
        }

        let label = format!("{}%", (self.speed * 100.0).round());
        renderer.centered_text(layout.x, layout.y, layout.w, layout.h, 13.0, &label);
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        self.minus.update(event.clone(), &layout.children[0], ctx);
        self.plus.update(event, &layout.children[1], ctx);
    }
}

impl<'a, MSG: Clone + 'static> From<SpeedPill<'a, MSG>> for Element<'a, MSG> {
    fn from(value: SpeedPill<'a, MSG>) -> Self {
        Element::new(value)
    }
}
