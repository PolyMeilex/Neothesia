use crate::{
    button, Color, Element, Event, LayoutCtx, Node, RenderCtx, Renderer, Tree, UpdateCtx, Widget,
};

fn minus_icon() -> &'static str {
    "\u{F2EA}"
}

fn plus_icon() -> &'static str {
    "\u{F4FE}"
}

pub struct SpeedPill<MSG> {
    w: f32,
    h: f32,

    minus: button::Button<MSG>,
    plus: button::Button<MSG>,

    speed: f32,
}

impl<MSG: Clone> Default for SpeedPill<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG: Clone> SpeedPill<MSG> {
    pub fn new() -> Self {
        let minus = button::Button::new()
            .color(Color::new_u8(67, 67, 67, 1.0))
            .hover_color(Color::new_u8(87, 87, 87, 1.0))
            .preseed_color(Color::new_u8(97, 97, 97, 1.0))
            .width(45.0)
            .height(20.0)
            .border_radius([10.0, 0.0, 10.0, 0.0]);
        let plus = button::Button::new()
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

impl<MSG: Clone> Widget<MSG> for SpeedPill<MSG> {
    type State = ();

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.minus), Tree::new(&self.plus)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children3(&[&self.minus, &self.plus]);
    }

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

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, tree: &Tree, ctx: &RenderCtx) {
        self.minus
            .render(renderer, &layout.children[0], &tree.children[0], ctx);
        self.plus
            .render(renderer, &layout.children[1], &tree.children[1], ctx);

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

    fn update(&mut self, event: Event, layout: &Node, tree: &mut Tree, ctx: &mut UpdateCtx<MSG>) {
        self.minus.update(
            event.clone(),
            &layout.children[0],
            &mut tree.children[0],
            ctx,
        );
        self.plus
            .update(event, &layout.children[1], &mut tree.children[1], ctx);
    }
}

impl<'a, MSG: Clone + 'static> From<SpeedPill<MSG>> for Element<'a, MSG> {
    fn from(value: SpeedPill<MSG>) -> Self {
        Element::new(value)
    }
}
