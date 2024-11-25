use crate::{Element, Event, LayoutCtx, Node, RenderCtx, Renderer, Tree, UpdateCtx, Widget};

pub struct TriLayout<'a, MSG> {
    start: Element<'a, MSG>,
    center: Element<'a, MSG>,
    end: Element<'a, MSG>,
}

impl<'a, MSG> Default for TriLayout<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG> TriLayout<'a, MSG> {
    pub fn new() -> Self {
        Self {
            start: Element::null(),
            center: Element::null(),
            end: Element::null(),
        }
    }

    pub fn start(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.start = widget.into();
        self
    }

    pub fn center(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.center = widget.into();
        self
    }

    pub fn end(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.end = widget.into();
        self
    }
}

impl<'a, MSG> Widget<MSG> for TriLayout<'a, MSG> {
    type State = ();

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(self.start.as_widget()),
            Tree::new(self.center.as_widget()),
            Tree::new(self.end.as_widget()),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(self.start.as_widget());
        tree.children[1].diff(self.center.as_widget());
        tree.children[2].diff(self.end.as_widget());
    }

    fn layout(&self, tree: &mut Tree<Self::State>, ctx: &LayoutCtx) -> Node {
        let start = self.start.as_widget().layout(
            &mut tree.children[0],
            &LayoutCtx {
                x: ctx.x,
                y: ctx.y,
                w: ctx.w,
                h: ctx.h,
            },
        );

        let center = {
            let mut node = self.center.as_widget().layout(
                &mut tree.children[1],
                &LayoutCtx {
                    x: ctx.x,
                    y: ctx.y,
                    w: ctx.w,
                    h: ctx.h,
                },
            );

            let x_offset = ctx.w / 2.0 - node.w / 2.0;
            node.for_each_descend_mut(&|node| {
                node.x += x_offset;
            });

            node
        };

        let end = {
            let mut node = self.end.as_widget().layout(
                &mut tree.children[2],
                &LayoutCtx {
                    x: ctx.x,
                    y: ctx.y,
                    w: ctx.w,
                    h: ctx.h,
                },
            );

            let x_offset = ctx.w - node.w;
            node.for_each_descend_mut(&|node| {
                node.x += x_offset;
            });

            node
        };

        Node {
            x: ctx.x,
            y: ctx.y,
            w: ctx.w,
            h: ctx.h,
            children: vec![start, center, end],
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        self.start
            .as_widget()
            .render(renderer, &layout.children[0], &tree.children[0], ctx);
        self.center
            .as_widget()
            .render(renderer, &layout.children[1], &tree.children[1], ctx);
        self.end
            .as_widget()
            .render(renderer, &layout.children[2], &tree.children[2], ctx);
    }

    fn update(
        &mut self,
        event: Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        self.start.as_widget_mut().update(
            event.clone(),
            &layout.children[0],
            &mut tree.children[0],
            ctx,
        );
        self.center.as_widget_mut().update(
            event.clone(),
            &layout.children[1],
            &mut tree.children[1],
            ctx,
        );
        self.end.as_widget_mut().update(
            event.clone(),
            &layout.children[2],
            &mut tree.children[2],
            ctx,
        );
    }
}

impl<'a, MSG: 'static> From<TriLayout<'a, MSG>> for Element<'a, MSG> {
    fn from(value: TriLayout<'a, MSG>) -> Self {
        Element::new(value)
    }
}
