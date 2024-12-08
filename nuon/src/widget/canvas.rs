use crate::{Element, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, Tree, Widget};

pub struct Canvas<F> {
    draw: F,
}

impl<F: Fn(&mut dyn Renderer, &Node)> Canvas<F> {
    pub fn new(draw: F) -> Self {
        Self { draw }
    }
}

impl<MSG: Clone, F: Fn(&mut dyn Renderer, &Node)> Widget<MSG> for Canvas<F> {
    type State = ();

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
        _tree: &Tree<Self::State>,
        _ctx: &RenderCtx,
    ) {
        (self.draw)(renderer, layout)
    }
}

impl<MSG: Clone + 'static, F: Fn(&mut dyn Renderer, &Node) + 'static> From<Canvas<F>>
    for Element<MSG>
{
    fn from(value: Canvas<F>) -> Self {
        Element::new(value)
    }
}
