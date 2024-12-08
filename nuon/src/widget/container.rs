use crate::{Color, Element, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, Tree, Widget};

pub struct Container<MSG> {
    child: [Element<MSG>; 1],
    background: Option<Color>,
    border_radius: [f32; 4],
    x: f32,
    y: f32,
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
            child: [Element::null()],
            background: None,
            border_radius: [0.0; 4],
            x: 0.0,
            y: 0.0,
            width: None,
            height: None,
        }
    }

    pub fn child(mut self, child: impl Into<Element<MSG>>) -> Self {
        self.child[0] = child.into();
        self
    }

    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border_radius(mut self, radius: [f32; 4]) -> Self {
        self.border_radius = radius;
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

    fn children(&self) -> &[Element<MSG>] {
        &self.child
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        let parent = &ParentLayout {
            x: parent.x + self.x,
            y: parent.y + self.y,
            w: self.width.unwrap_or(parent.w),
            h: self.height.unwrap_or(parent.h),
        };

        let mut children = Vec::with_capacity(self.child.len());

        let mut w = 0.0;
        let mut h = 0.0;

        for (ch, tree) in self.child.iter().zip(tree.children.iter_mut()) {
            let node = ch.as_widget().layout(tree, parent, ctx);

            w = node.w.max(node.w);
            h = node.h.max(node.h);

            children.push(node);
        }

        Node {
            x: parent.x,
            y: parent.y,
            w: self.width.unwrap_or(w),
            h: self.height.unwrap_or(h),
            children,
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        if let Some(bg) = self.background {
            renderer.rounded_quad(
                layout.x,
                layout.y,
                layout.w,
                layout.h,
                bg,
                self.border_radius,
            );
        }
        crate::default_render(self, renderer, layout, tree, ctx);
    }
}

impl<MSG: 'static> From<Container<MSG>> for Element<MSG> {
    fn from(value: Container<MSG>) -> Self {
        Element::new(value)
    }
}
