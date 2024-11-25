use crate::{
    Element, Event, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, Tree, UpdateCtx, Widget,
};

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
            child: Element::null(),
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

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.child.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children2(&[&self.child]);
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        self.child.as_widget().layout(
            &mut tree.children[0],
            &ParentLayout {
                x: parent.x + self.x,
                y: parent.y + self.y,
                w: parent.w,
                h: parent.h,
            },
            ctx,
        )
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        self.child
            .as_widget()
            .render(renderer, layout, &tree.children[0], ctx)
    }

    fn update(
        &mut self,
        event: Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        self.child
            .as_widget_mut()
            .update(event, layout, &mut tree.children[0], ctx)
    }
}

impl<'a, MSG: 'static> From<Translate<'a, MSG>> for Element<'a, MSG> {
    fn from(value: Translate<'a, MSG>) -> Self {
        Element::new(value)
    }
}
