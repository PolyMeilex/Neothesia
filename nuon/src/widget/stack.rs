use crate::{Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget};

pub struct Stack<'a, MSG> {
    children: Vec<Element<'a, MSG>>,
}

impl<'a, MSG> Default for Stack<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG> Stack<'a, MSG> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn push(mut self, widget: impl Into<Element<'a, MSG>>) -> Self {
        self.children.push(widget.into());
        self
    }

    pub fn push_if(mut self, condition: bool, widget: impl Into<Element<'a, MSG>>) -> Self {
        if condition {
            self.children.push(widget.into());
        }
        self
    }
}

impl<'a, MSG> Widget<MSG> for Stack<'a, MSG> {
    fn layout(&self, ctx: &LayoutCtx) -> Node {
        let mut children = Vec::with_capacity(self.children.len());

        for ch in self.children.iter() {
            children.push(ch.as_widget().layout(ctx));
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
        for (ch, layout) in self.children.iter().zip(layout.children.iter()) {
            ch.as_widget().render(renderer, layout, ctx);
        }
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        for (ch, layout) in self.children.iter_mut().zip(layout.children.iter()).rev() {
            ch.as_widget_mut().update(event.clone(), layout, ctx);

            if ctx.is_event_captured() {
                return;
            }
        }
    }
}

impl<'a, MSG: 'static> From<Stack<'a, MSG>> for Element<'a, MSG> {
    fn from(value: Stack<'a, MSG>) -> Self {
        Element::new(value)
    }
}
