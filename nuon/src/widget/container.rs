use crate::{
    null::Null, Color, Element, Event, LayoutCtx, Node, RenderCtx, Renderer, UpdateCtx, Widget,
};

pub struct Container<'a, MSG> {
    child: Element<'a, MSG>,
    background: Option<Color>,
    width: Option<f32>,
    height: Option<f32>,
}

impl<'a, MSG: 'static> Default for Container<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'static> Container<'a, MSG> {
    pub fn new() -> Self {
        Self {
            child: Null.into(),
            background: None,
            width: None,
            height: None,
        }
    }

    pub fn child(mut self, child: impl Into<Element<'a, MSG>>) -> Self {
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

impl<'a, MSG> Widget<MSG> for Container<'a, MSG> {
    type State = ();

    fn layout(&self, ctx: &LayoutCtx) -> Node {
        self.child.as_widget().layout(&LayoutCtx {
            x: ctx.x,
            y: ctx.y,
            w: self.width.unwrap_or(ctx.w),
            h: self.height.unwrap_or(ctx.h),
        })
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, ctx: &RenderCtx) {
        if let Some(bg) = self.background {
            renderer.quad(layout.x, layout.y, layout.w, layout.h, bg);
        }

        self.child.as_widget().render(renderer, layout, ctx)
    }

    fn update(&mut self, event: Event, layout: &Node, ctx: &mut UpdateCtx<MSG>) {
        self.child.as_widget_mut().update(event, layout, ctx)
    }
}

impl<'a, MSG: 'static> From<Container<'a, MSG>> for Element<'a, MSG> {
    fn from(value: Container<'a, MSG>) -> Self {
        Element::new(value)
    }
}
