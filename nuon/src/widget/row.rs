use crate::{Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget};

pub struct Row<'a, MSG> {
    children: Vec<Element<'a, MSG>>,
    gap: f32,
}

impl<'a, MSG> Default for Row<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG> Row<'a, MSG> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            gap: 0.0,
        }
    }

    pub fn push(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.children.push(widget.into());
        self
    }

    pub fn when(self, v: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if v {
            f(self)
        } else {
            self
        }
    }
}

impl<'a, MSG> Widget<MSG> for Row<'a, MSG> {
    fn layout(&self, ctx: &LayoutCtx) -> Node {
        let mut children = Vec::with_capacity(self.children.len());

        let mut item_layout_ctx = LayoutCtx {
            x: ctx.x,
            y: ctx.y,
            w: ctx.w,
            h: ctx.h,
        };

        let mut total_width = 0.0;

        for ch in self.children.iter() {
            let node = ch.as_widget().layout(&item_layout_ctx);

            item_layout_ctx.x += node.w;
            item_layout_ctx.w -= node.w;

            item_layout_ctx.x += self.gap;
            item_layout_ctx.w -= self.gap;

            total_width += node.w;
            total_width += self.gap;

            children.push(node);
        }

        total_width -= self.gap;
        total_width = total_width.max(0.0);

        Node {
            x: ctx.x,
            y: ctx.y,
            w: total_width,
            h: ctx.h,
            children,
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, ctx: &RenderCtx) {
        for (ch, layout) in self.children.iter().zip(layout.children.iter()) {
            ch.as_widget().render(renderer, layout, ctx);
        }
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        for (ch, layout) in self.children.iter_mut().zip(layout.children.iter()) {
            ch.as_widget_mut().update(event.clone(), layout, ctx);
        }
    }
}

impl<'a, MSG: 'static> From<Row<'a, MSG>> for Element<'a, MSG> {
    fn from(value: Row<'a, MSG>) -> Self {
        Element::new(value)
    }
}
