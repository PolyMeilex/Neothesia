use crate::{Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget};

#[derive(Default, Debug)]
pub struct Null;

impl Null {
    pub fn new() -> Self {
        Self {}
    }
}

impl<MSG> Widget<MSG> for Null {
    fn layout(&self, ctx: &LayoutCtx) -> Node {
        Node {
            x: ctx.x,
            y: ctx.y,
            w: 0.0,
            h: 0.0,
            children: vec![],
        }
    }
    fn render(&self, _renderer: &mut dyn Renderer, _layout: &Node, _ctx: &RenderCtx) {}
    fn update(&mut self, _event: Event, _layout: &Node, _ctx: &mut UpdateCtx<MSG>) {}
}

impl<MSG> From<Null> for Element<'_, MSG> {
    fn from(value: Null) -> Self {
        Element::new(value)
    }
}
