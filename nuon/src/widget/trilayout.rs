use crate::{Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget};

pub struct TriLayout<'a, MSG> {
    start: Option<Element<'a, MSG>>,
    center: Option<Element<'a, MSG>>,
    end: Option<Element<'a, MSG>>,
}

impl<'a, MSG> Default for TriLayout<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG> TriLayout<'a, MSG> {
    pub fn new() -> Self {
        Self {
            start: None,
            center: None,
            end: None,
        }
    }

    pub fn start(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.start = Some(widget.into());
        self
    }

    pub fn center(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.center = Some(widget.into());
        self
    }

    pub fn end(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.end = Some(widget.into());
        self
    }

    fn iter(&self) -> impl Iterator<Item = &Element<MSG>> {
        self.start
            .as_ref()
            .into_iter()
            .chain(self.center.as_ref())
            .chain(self.end.as_ref())
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Element<'a, MSG>> {
        self.start
            .as_mut()
            .into_iter()
            .chain(self.center.as_mut())
            .chain(self.end.as_mut())
    }
}

impl<'a, MSG> Widget<MSG> for TriLayout<'a, MSG> {
    fn layout(&self, ctx: &LayoutCtx) -> Node {
        let mut children = vec![];

        if let Some(start) = self.start.as_ref() {
            let node = start.as_widget().layout(&LayoutCtx {
                x: ctx.x,
                y: ctx.y,
                w: ctx.w,
                h: ctx.h,
            });

            children.push(node);
        }

        if let Some(center) = self.center.as_ref() {
            let mut node = center.as_widget().layout(&LayoutCtx {
                x: ctx.x,
                y: ctx.y,
                w: ctx.w,
                h: ctx.h,
            });

            let x_offset = ctx.w / 2.0 - node.w / 2.0;
            node.for_each_descend_mut(&|node| {
                node.x += x_offset;
            });

            children.push(node);
        }

        if let Some(end) = self.end.as_ref() {
            let mut node = end.as_widget().layout(&LayoutCtx {
                x: ctx.x,
                y: ctx.y,
                w: ctx.w,
                h: ctx.h,
            });

            let x_offset = ctx.w - node.w;
            node.for_each_descend_mut(&|node| {
                node.x += x_offset;
            });

            children.push(node);
        }

        Node {
            x: ctx.x,
            y: ctx.y,
            w: ctx.w,
            h: ctx.h,
            children,
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, ctx: &RenderCtx) {
        for (ch, layout) in self.iter().zip(layout.children.iter()) {
            ch.as_widget().render(renderer, layout, ctx);
        }
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        for (ch, layout) in self.iter_mut().zip(layout.children.iter()) {
            ch.as_widget_mut().update(event.clone(), layout, ctx);
        }
    }
}

impl<'a, MSG: 'static> From<TriLayout<'a, MSG>> for Element<'a, MSG> {
    fn from(value: TriLayout<'a, MSG>) -> Self {
        Element::new(value)
    }
}
