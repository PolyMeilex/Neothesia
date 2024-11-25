use crate::{
    Color, Element, Event, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, Tree, UpdateCtx,
    Widget,
};

pub struct Container<MSG> {
    child: Element<MSG>,
    background: Option<Color>,
    width: Option<f32>,
    height: Option<f32>,
}

impl<MSG: 'static> Default for Container<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG: 'static> Container<MSG> {
    pub fn new() -> Self {
        Self {
            child: Element::null(),
            background: None,
            width: None,
            height: None,
        }
    }

    pub fn child(mut self, child: impl Into<Element<MSG>>) -> Self {
        self.child = child.into();
        self
    }

    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

impl<MSG> Widget<MSG> for Container<MSG> {
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
                x: parent.x,
                y: parent.y,
                w: self.width.unwrap_or(parent.w),
                h: self.height.unwrap_or(parent.h),
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
        if let Some(bg) = self.background {
            renderer.quad(layout.x, layout.y, layout.w, layout.h, bg);
        }

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

impl<MSG: 'static> From<Container<MSG>> for Element<MSG> {
    fn from(value: Container<MSG>) -> Self {
        Element::new(value)
    }
}
