pub mod widget;
use std::any::{Any, TypeId};

pub use tree::Tree;
pub use widget::*;

pub use euclid;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

pub mod input;
mod renderer;
mod tree;

pub use input::{Event, MouseButton};
pub use renderer::Renderer;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn new_u8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }
}

pub struct RenderCtx {}

pub struct UpdateCtx<'a, MSG> {
    pub messages: &'a mut Vec<MSG>,
    pub event_captured: bool,
    pub mouse_grab: bool,
}

impl<MSG> UpdateCtx<'_, MSG> {
    pub fn grab_mouse(&mut self) {
        self.mouse_grab = true;
    }

    pub fn ungrab_mouse(&mut self) {
        self.mouse_grab = false;
    }

    pub fn capture_event(&mut self) {
        self.event_captured = true;
    }

    pub fn is_event_captured(&self) -> bool {
        self.event_captured
    }
}

#[derive(Default, Clone)]
pub struct LayoutCtx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub trait WidgetAny<MSG> {
    fn state_type_id(&self) -> TypeId;
    fn state(&self) -> Box<dyn Any>;
    fn children(&self) -> Vec<Tree>;
    fn diff(&self, tree: &mut Tree);

    fn layout(&self, tree: &mut Tree, ctx: &LayoutCtx) -> Node;
    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, tree: &Tree, ctx: &RenderCtx);
    fn update(
        &mut self,
        event: input::Event,
        layout: &Node,
        tree: &mut Tree,
        ctx: &mut UpdateCtx<MSG>,
    );
}

impl<MSG, W: Widget<MSG>> WidgetAny<MSG> for W {
    fn state_type_id(&self) -> TypeId {
        TypeId::of::<W::State>()
    }

    fn state(&self) -> Box<dyn Any> {
        Widget::state(self)
    }

    fn children(&self) -> Vec<Tree> {
        Widget::children(self)
    }

    fn diff(&self, tree: &mut Tree) {
        Widget::diff(self, tree)
    }

    fn layout(&self, tree: &mut Tree, ctx: &LayoutCtx) -> Node {
        Widget::layout(self, tree.remap_mut(), ctx)
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, tree: &Tree, ctx: &RenderCtx) {
        Widget::render(self, renderer, layout, tree.remap_ref(), ctx)
    }

    fn update(
        &mut self,
        event: input::Event,
        layout: &Node,
        tree: &mut Tree,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        Widget::update(self, event, layout, tree.remap_mut(), ctx)
    }
}

pub trait Widget<MSG> {
    type State: Any + Default;

    fn state(&self) -> Box<dyn Any> {
        Box::new(Self::State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![]
    }

    #[allow(unused)]
    fn diff(&self, tree: &mut Tree) {}

    fn layout(&self, tree: &mut Tree<Self::State>, ctx: &LayoutCtx) -> Node;
    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    );
    fn update(
        &mut self,
        event: input::Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    );
}

#[derive(Default, Clone)]
pub struct Node {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub children: Vec<Node>,
}

impl Node {
    pub fn as_rect(&self) -> crate::Rect {
        crate::Rect::new((self.x, self.y).into(), (self.w, self.h).into())
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.as_rect().contains((x, y).into())
    }

    pub fn for_each_descend_mut(&mut self, cb: &impl Fn(&mut Self)) {
        cb(self);
        for ch in self.children.iter_mut() {
            ch.for_each_descend_mut(cb);
        }
    }
}

pub struct Element<'a, MSG>(Box<dyn WidgetAny<MSG> + 'a>);

impl<'a, MSG> Element<'a, MSG> {
    pub fn new(widget: impl Widget<MSG> + 'a) -> Self {
        Self(Box::new(widget))
    }

    pub fn as_widget(&self) -> &dyn WidgetAny<MSG> {
        self.0.as_ref()
    }

    pub fn as_widget_mut(&mut self) -> &mut dyn WidgetAny<MSG> {
        self.0.as_mut()
    }
}
