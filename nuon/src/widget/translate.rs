use crate::{null::Null, Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget};

pub struct Translate<'a, MSG> {
    x: f32,
    y: f32,
    child: Element<'a, MSG>,
}

impl<'a, MSG: 'static> Default for Translate<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'static> Translate<'a, MSG> {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            child: Null.into(),
        }
    }

    pub fn child(mut self, child: impl Into<Element<'a, MSG>>) -> Self {
        self.child = child.into();
        self
    }

    pub fn x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }
}

impl<'a, MSG> Widget<MSG> for Translate<'a, MSG> {
    type State = ();

    fn layout(&self, ctx: &LayoutCtx) -> Node {
        self.child.as_widget().layout(&LayoutCtx {
            x: ctx.x + self.x,
            y: ctx.y + self.y,
            w: ctx.w,
            h: ctx.h,
        })
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, ctx: &RenderCtx) {
        self.child.as_widget().render(renderer, layout, ctx)
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        self.child.as_widget_mut().update(event, layout, ctx)
    }
}

impl<'a, MSG: 'static> From<Translate<'a, MSG>> for Element<'a, MSG> {
    fn from(value: Translate<'a, MSG>) -> Self {
        Element::new(value)
    }
}
