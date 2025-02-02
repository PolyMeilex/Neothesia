use nuon::{
    button, Color, Element, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, Tree, Widget,
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

    children: [Element<MSG>; 2],

    speed: f32,
}

impl<MSG: 'static + Clone> Default for SpeedPill<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG: 'static + Clone> SpeedPill<MSG> {
    pub fn new() -> Self {
        Self {
            w: 90.0,
            h: 30.0,

            children: [Element::null(), Element::null()],
            speed: 0.0,
        }
    }

    pub fn on_minus(mut self, msg: MSG) -> Self {
        self.children[0] = button::Button::new()
            .color(Color::new_u8(67, 67, 67, 1.0))
            .hover_color(Color::new_u8(87, 87, 87, 1.0))
            .preseed_color(Color::new_u8(97, 97, 97, 1.0))
            .width(45.0)
            .height(20.0)
            .border_radius([10.0, 0.0, 10.0, 0.0])
            .on_click(msg)
            .into();
        self
    }

    pub fn on_plus(mut self, msg: MSG) -> Self {
        self.children[1] = button::Button::new()
            .color(Color::new_u8(67, 67, 67, 1.0))
            .hover_color(Color::new_u8(87, 87, 87, 1.0))
            .preseed_color(Color::new_u8(97, 97, 97, 1.0))
            .width(45.0)
            .height(20.0)
            .border_radius([0.0, 10.0, 0.0, 10.0])
            .on_click(msg)
            .into();
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl<MSG: Clone> Widget<MSG> for SpeedPill<MSG> {
    type State = ();

    fn children(&self) -> &[Element<MSG>] {
        &self.children
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        let minus = self.children[0].as_widget().layout(
            &mut tree.children[0],
            &ParentLayout {
                x: parent.x,
                y: parent.y + 5.0,
                w: parent.w,
                h: parent.h,
            },
            ctx,
        );
        let plus = self.children[1].as_widget().layout(
            &mut tree.children[1],
            &ParentLayout {
                x: parent.x + minus.w,
                y: parent.y + 5.0,
                w: parent.w,
                h: parent.h,
            },
            ctx,
        );

        Node {
            x: parent.x,
            y: parent.y,
            w: self.w,
            h: self.h,
            children: vec![minus, plus],
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        nuon::default_render(self, renderer, layout, tree, ctx);

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
}

impl<MSG: Clone + 'static> From<SpeedPill<MSG>> for Element<MSG> {
    fn from(value: SpeedPill<MSG>) -> Self {
        Element::new(value)
    }
}
