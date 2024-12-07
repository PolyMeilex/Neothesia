pub mod widget;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub use tree::{Tree, TreeState};
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

pub struct RenderCtx<'a> {
    pub globals: &'a GlobalStore<'a>,
}

pub struct UpdateCtx<'a, MSG> {
    pub messages: &'a mut Vec<MSG>,
    pub event_captured: bool,
    pub mouse_grab: bool,
    pub globals: &'a GlobalStore<'a>,
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

pub struct LayoutCtx<'a> {
    pub globals: &'a GlobalStore<'a>,
}

#[derive(Default, Clone)]
pub struct ParentLayout {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub trait WidgetAny<MSG> {
    fn state_type_id(&self) -> TypeId;
    fn state(&self) -> TreeState;

    fn children(&self) -> &[Element<MSG>];

    fn layout(&self, tree: &mut Tree, avalilable: &ParentLayout, ctx: &LayoutCtx) -> Node;
    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, tree: &Tree, ctx: &RenderCtx);
    fn update(&self, event: input::Event, layout: &Node, tree: &mut Tree, ctx: &mut UpdateCtx<MSG>);
}

impl<MSG, W: Widget<MSG>> WidgetAny<MSG> for W {
    fn state_type_id(&self) -> TypeId {
        TypeId::of::<W::State>()
    }

    fn state(&self) -> TreeState {
        TreeState::new(Widget::state(self))
    }

    fn children(&self) -> &[Element<MSG>] {
        Widget::children(self)
    }

    fn layout(&self, tree: &mut Tree, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        Widget::layout(self, tree.cast_mut(), parent, ctx)
    }

    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, tree: &Tree, ctx: &RenderCtx) {
        Widget::render(self, renderer, layout, tree.cast_ref(), ctx)
    }

    fn update(
        &self,
        event: input::Event,
        layout: &Node,
        tree: &mut Tree,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        Widget::update(self, event, layout, tree.cast_mut(), ctx)
    }
}

pub trait Widget<MSG> {
    type State: Any + Default;

    fn state(&self) -> Self::State {
        Self::State::default()
    }

    fn children(&self) -> &[Element<MSG>] {
        &[]
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        widget::stack::stack_layout(self, tree, parent, ctx)
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        default_render(self, renderer, layout, tree, ctx)
    }

    fn update(
        &self,
        event: input::Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        default_update(self, event, layout, tree, ctx)
    }
}

pub use widget::column::column_layout;
pub use widget::row::row_layout;
pub use widget::stack::stack_layout;

pub fn default_render<MSG, W: Widget<MSG> + ?Sized>(
    this: &W,
    renderer: &mut dyn Renderer,
    layout: &Node,
    tree: &Tree<W::State>,
    ctx: &RenderCtx,
) {
    for ((ch, layout), tree) in this
        .children()
        .iter()
        .zip(layout.children.iter())
        .zip(tree.children.iter())
    {
        ch.as_widget().render(renderer, layout, tree, ctx);
    }
}

pub fn default_update<MSG, W: Widget<MSG> + ?Sized>(
    this: &W,
    event: input::Event,
    layout: &Node,
    tree: &mut Tree<W::State>,
    ctx: &mut UpdateCtx<MSG>,
) {
    for ((ch, layout), tree) in this
        .children()
        .iter()
        .zip(layout.children.iter())
        .zip(tree.children.iter_mut())
        .rev()
    {
        ch.as_widget().update(event.clone(), layout, tree, ctx);

        if ctx.is_event_captured() {
            return;
        }
    }
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

pub struct Element<MSG>(Box<dyn WidgetAny<MSG>>);

impl<MSG> Element<MSG> {
    pub fn new(widget: impl Widget<MSG> + 'static) -> Self {
        Self(Box::new(widget))
    }

    pub fn null() -> Self {
        crate::null::Null::new().into()
    }

    pub fn as_widget(&self) -> &dyn WidgetAny<MSG> {
        self.0.as_ref()
    }

    pub fn as_widget_mut(&mut self) -> &mut dyn WidgetAny<MSG> {
        self.0.as_mut()
    }
}

#[derive(Default)]
pub struct GlobalStore<'a> {
    map: HashMap<TypeId, &'a dyn Any>,
}

impl<'a> GlobalStore<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(f: impl FnOnce(&mut Self)) -> Self {
        let mut this = Self::new();
        f(&mut this);
        this
    }

    pub fn insert(&mut self, v: &'a dyn Any) {
        self.map.insert(v.type_id(), v);
    }

    #[track_caller]
    pub fn get<T: Any>(&self) -> &T {
        self.map
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .unwrap()
    }
}
