use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

pub use euclid;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

mod settings;
use neothesia_image::ImageIdentifier;
pub use settings::*;

pub fn center_y(container_h: f32, item_h: f32) -> f32 {
    container_h / 2.0 - item_h / 2.0
}

pub fn center_x(container_w: f32, item_w: f32) -> f32 {
    container_w / 2.0 - item_w / 2.0
}

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

impl From<[u8; 3]> for Color {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::new_u8(r, g, b, 1.0)
    }
}

impl From<[u8; 4]> for Color {
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::new_u8(r, g, b, a as f32 / 255.0)
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

    pub fn packet_u32(&self) -> u32 {
        let r = self.r * 255.0;
        let g = self.g * 255.0;
        let b = self.b * 255.0;
        let a = self.a * 255.0;
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextJustify {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Id(u64);

impl<H: Hash> From<H> for Id {
    fn from(value: H) -> Self {
        Self::hash(value)
    }
}

impl Id {
    const NULL: Self = Id(0);

    pub fn as_raw(&self) -> u64 {
        self.0
    }

    pub fn hash(v: impl Hash) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        v.hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn hash_with(v: impl FnOnce(&mut std::hash::DefaultHasher)) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        v(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Default, Debug)]
pub struct TranslationStack(Vec<Point>);

impl TranslationStack {
    pub fn offset(&self) -> Point {
        self.0.last().copied().unwrap_or(Point::zero())
    }

    pub fn translate(&self, point: Point) -> Point {
        let offset = self.offset();
        Point::new(point.x + offset.x, point.y + offset.y)
    }
}

#[derive(Default, Debug)]
pub struct LayerData {
    pub scissor_rect: Rect,
    pub quads: Vec<QuadRenderElement>,
    pub icons: Vec<IconRenderElement>,
    pub text: Vec<TextRenderElement>,
    pub images: Vec<ImageRenderElement>,
}

impl LayerData {
    fn clear(&mut self) {
        self.quads.clear();
        self.icons.clear();
        self.text.clear();
        self.images.clear();
    }
}

#[derive(Debug, Clone, Copy)]
enum LayerId {
    Regular(usize),
    Overlay(usize),
}

#[derive(Debug)]
pub struct LayerStack {
    layers: Vec<LayerData>,
    overlay_layers: Vec<LayerData>,
    history_stack: Vec<LayerId>,
    curr: LayerId,
}

impl LayerStack {
    fn new() -> Self {
        LayerStack {
            layers: vec![LayerData::default()],
            overlay_layers: vec![],
            history_stack: vec![],
            curr: LayerId::Regular(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.layers.len() + self.overlay_layers.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &LayerData> {
        self.layers.iter().chain(self.overlay_layers.iter())
    }

    pub fn current_scissor_rect(&self) -> Option<Rect> {
        let scissor_rect = self.current().scissor_rect;
        (!scissor_rect.is_empty()).then_some(scissor_rect)
    }

    pub fn current(&self) -> &LayerData {
        match self.curr {
            LayerId::Regular(id) => &self.layers[id],
            LayerId::Overlay(id) => &self.overlay_layers[id],
        }
    }

    pub fn current_mut(&mut self) -> &mut LayerData {
        match self.curr {
            LayerId::Regular(id) => &mut self.layers[id],
            LayerId::Overlay(id) => &mut self.overlay_layers[id],
        }
    }

    pub fn push(&mut self) {
        self.history_stack.push(self.curr);

        match self.curr {
            LayerId::Regular(_) => {
                self.curr = LayerId::Regular(self.layers.len());
                self.layers.push(LayerData::default());
            }
            LayerId::Overlay(_) => {
                self.curr = LayerId::Overlay(self.overlay_layers.len());
                self.overlay_layers.push(LayerData::default());
            }
        }
    }

    pub fn push_overlay(&mut self) {
        self.history_stack.push(self.curr);
        self.curr = LayerId::Overlay(self.overlay_layers.len());
        self.overlay_layers.push(LayerData::default());
    }

    pub fn pop(&mut self) {
        if let Some(prev) = self.history_stack.pop() {
            self.curr = prev;
        }
    }

    pub fn clear(&mut self) {
        self.layers[0].clear();
        // TODO: Reuse the memory from all dropped layers
        self.layers.drain(1..);
        self.overlay_layers.clear();

        self.curr = LayerId::Regular(0);
    }
}

#[derive(Debug, Clone)]
pub struct QuadRenderElement {
    pub rect: Rect,
    pub border_radius: [f32; 4],
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct IconRenderElement {
    pub origin: Point,
    pub size: f32,
    pub icon: String,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct TextRenderElement {
    pub rect: Rect,
    pub text_justify: TextJustify,
    pub size: f32,
    pub bold: bool,
    pub text: String,
    pub color: Color,
    pub font_family: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct ImageRenderElement {
    pub rect: Rect,
    pub image: ImageIdentifier,
    pub border_radius: [f32; 4],
}

pub struct Ui {
    pub hovered: Option<Id>,
    pub active: Option<Id>,

    pointer_pos: Point,
    pointer_pos_delta: Point,
    pub mouse_pressed: bool,
    pub mouse_down: bool,

    pub translation_stack: TranslationStack,

    pub layers: LayerStack,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub fn new() -> Self {
        Self {
            hovered: None,
            active: None,
            pointer_pos: Point::new(-1.0, -1.0),
            pointer_pos_delta: Point::new(0.0, 0.0),
            mouse_pressed: false,
            mouse_down: false,
            translation_stack: TranslationStack::default(),
            layers: LayerStack::new(),
        }
    }

    pub fn set_scissor_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let layer = self.layers.current_mut();
        layer.scissor_rect.origin.x = x;
        layer.scissor_rect.origin.y = y;
        layer.scissor_rect.size.width = w;
        layer.scissor_rect.size.height = h;
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        let pointer_pos = Point::new(x, y);

        let delta = pointer_pos - self.pointer_pos;
        self.pointer_pos_delta.x += delta.x;
        self.pointer_pos_delta.y += delta.y;

        self.pointer_pos = pointer_pos;
    }

    pub fn mouse_down(&mut self) {
        self.mouse_pressed = true;
        self.mouse_down = true;
    }

    pub fn mouse_up(&mut self) {
        self.mouse_pressed = false;
        self.mouse_down = false;
    }

    pub fn done(&mut self) {
        self.layers.clear();
        self.mouse_pressed = false;
        self.pointer_pos_delta = Point::zero();
    }
}

#[derive(Debug, Clone, Default)]
pub struct Translate {
    origin: Point,
}

impl Translate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pos(self, x: f32, y: f32) -> Self {
        self.x(x).y(y)
    }

    pub fn x(mut self, x: f32) -> Self {
        self.origin.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.origin.y = y;
        self
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> Point {
        let offset = self.origin;
        let prev = ui.translation_stack.offset();
        ui.translation_stack
            .0
            .push(Point::new(prev.x + offset.x, prev.y + offset.y));
        build(ui);

        let pop = ui.translation_stack.0.pop().unwrap();
        Point::new(pop.x - prev.x - offset.x, pop.y - prev.y - offset.y)
    }

    pub fn add_to_current(&self, ui: &mut Ui) {
        if let Some(current) = ui.translation_stack.0.last_mut() {
            *current = Point::new(current.x + self.origin.x, current.y + self.origin.y);
        }
    }
}

pub fn translate() -> Translate {
    Translate::new()
}

#[derive(Debug, Clone, Default)]
pub struct Layer {
    rect: Rect,
    overlay: bool,
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scissor_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub fn overlay(mut self, overlay: bool) -> Self {
        self.overlay = overlay;
        self
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) {
        let rect = if self.rect == Rect::zero() {
            ui.layers.current_mut().scissor_rect
        } else {
            Rect::new(
                ui.translation_stack.translate(self.rect.origin),
                self.rect.size,
            )
        };

        if self.overlay {
            ui.layers.push_overlay();
        } else {
            ui.layers.push();
        }

        ui.layers.current_mut().scissor_rect = rect;

        build(ui);
        ui.layers.pop();
    }
}

pub fn layer() -> Layer {
    Layer::new()
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ScrollState {
    #[default]
    Uninitialized,
    Ready {
        value: f32,
        max: f32,
        mouse_drag_offset: f32,
    },
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, line_delta: f32) {
        let delta = -line_delta;

        match self {
            ScrollState::Uninitialized => {}
            ScrollState::Ready { value, max, .. } => {
                *value = (*value + delta).clamp(0.0, *max);
            }
        }
    }

    fn value(&self) -> f32 {
        match self {
            ScrollState::Uninitialized => 0.0,
            ScrollState::Ready { value, max, .. } => (*value).clamp(0.0, *max),
        }
    }

    fn set_value(&mut self, v: f32) {
        match self {
            ScrollState::Uninitialized => {}
            ScrollState::Ready { value, max, .. } => *value = v.clamp(0.0, *max),
        };
    }

    fn mouse_drag_offset(&self) -> f32 {
        match self {
            ScrollState::Uninitialized => 0.0,
            ScrollState::Ready {
                mouse_drag_offset, ..
            } => *mouse_drag_offset,
        }
    }

    fn set_mouse_drag_offset(&mut self, y: f32) {
        match self {
            ScrollState::Uninitialized => {}
            ScrollState::Ready {
                mouse_drag_offset, ..
            } => *mouse_drag_offset = y,
        };
    }

    fn set_max(&mut self, max: f32) {
        *self = match self {
            ScrollState::Uninitialized => Self::Ready {
                value: 0.0,
                max,
                mouse_drag_offset: 0.0,
            },
            ScrollState::Ready {
                value,
                mouse_drag_offset,
                ..
            } => Self::Ready {
                value: *value,
                max,
                mouse_drag_offset: *mouse_drag_offset,
            },
        };
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scroll {
    rect: Rect,
    scroll: ScrollState,
}

impl Scroll {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scissor_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub fn scroll(mut self, scroll: ScrollState) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn scissor_size(mut self, w: f32, h: f32) -> Self {
        self.rect.size.width = w;
        self.rect.size.height = h;
        self
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> ScrollState {
        //      ┌► ┌─────┐ ◄┐
        //      │  │~~~~~│  │ visible_h
        //      │  │~~~  │  │
        // full │  ├─────┤ ◄┘
        //      │  │~~~  │
        //      │  │~~~~~│
        //      └► └─────┘

        let mut state = self.scroll;
        let scroll = state.value();

        self::layer().scissor_rect(self.rect).build(ui, |ui| {
            let last = self::translate().y(-scroll).build(ui, build);

            let visible_h = self.rect.size.height;
            let full_h = last.y;

            let max_scroll = (full_h - visible_h).max(0.0);
            state.set_max(max_scroll);

            if max_scroll == 0.0 {
                return;
            };

            let mult = visible_h / full_h;
            let h = visible_h * mult;

            let w = 10.0;

            self::quad()
                .y(0.0)
                .x(self.rect.size.width - w)
                .size(w, self.rect.height())
                .color([37, 35, 42])
                .border_radius([5.0; 4])
                .build(ui);

            let y = scroll * mult;
            let x = self.rect.size.width - w;

            // TODO: Don't assume single scroll per view
            let res = self::click_area("scroll").size(w, h).pos(x, y).build(ui);

            let color = if res.is_hovered() || res.is_pressed() {
                [87, 81, 101]
            } else {
                [74, 68, 88]
            };

            if res.is_press_start() {
                state.set_mouse_drag_offset(ui.pointer_pos.y - y);
            } else if res.is_pressed() {
                let y = ui.pointer_pos.y - state.mouse_drag_offset();
                state.set_value(y / mult);
            }

            self::quad()
                .pos(x, y)
                .size(w, h)
                .color(color)
                .border_radius([5.0; 4])
                .build(ui);
        });

        state
    }
}

pub fn scroll() -> Scroll {
    Scroll::new()
}

pub struct Card {}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> Point {
        let last = self::translate().build(ui, |ui| {
            self::layer().build(ui, build);
        });

        self::quad()
            .size(last.x, last.y)
            .color([37, 35, 42])
            .border_radius([5.0; 4])
            .build(ui);

        last
    }
}

pub fn card() -> Card {
    Card::new()
}

pub struct RowGroup {}

impl Default for RowGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGroup {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) -> Point {
        let last = self::translate().build(ui, |ui| {
            self::layer().build(ui, build);
        });

        self::quad()
            .size(last.x, last.y)
            .color([37, 35, 42])
            .border_radius([10.0; 4])
            .build(ui);

        last
    }
}

pub fn row_group() -> RowGroup {
    RowGroup::new()
}

#[derive(Debug, Clone)]
pub struct Quad {
    rect: Rect,
    color: Color,
    border_radius: [f32; 4],
}

pub fn quad() -> Quad {
    Quad::new()
}

impl Default for Quad {
    fn default() -> Self {
        Self::new()
    }
}

impl Quad {
    pub fn new() -> Self {
        Self {
            rect: Rect::zero(),
            color: Color::new_u8(0, 0, 0, 0.0),
            border_radius: [0.0; 4],
        }
    }

    pub fn pos(self, x: f32, y: f32) -> Self {
        self.x(x).y(y)
    }

    pub fn x(mut self, x: f32) -> Self {
        self.rect.origin.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.rect.origin.y = y;
        self
    }

    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.rect.size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.rect.size.height = height;
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn border_radius(mut self, border_radius: [f32; 4]) -> Self {
        self.border_radius = border_radius;
        self
    }

    pub fn build(&self, ui: &mut Ui) {
        let rect = Rect::new(
            ui.translation_stack.translate(self.rect.origin),
            self.rect.size,
        );

        ui.layers.current_mut().quads.push(QuadRenderElement {
            rect,
            border_radius: self.border_radius,
            color: self.color,
        });
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    rect: Rect,
    image: ImageIdentifier,
    border_radius: [f32; 4],
}

pub fn image(image: ImageIdentifier) -> Image {
    Image::new(image)
}

impl Image {
    pub fn new(image: ImageIdentifier) -> Self {
        Self {
            rect: Rect::zero(),
            image,
            border_radius: [0.0; 4],
        }
    }

    pub fn pos(self, x: f32, y: f32) -> Self {
        self.x(x).y(y)
    }

    pub fn x(mut self, x: f32) -> Self {
        self.rect.origin.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.rect.origin.y = y;
        self
    }

    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.rect.size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.rect.size.height = height;
        self
    }

    pub fn border_radius(mut self, border_radius: [f32; 4]) -> Self {
        self.border_radius = border_radius;
        self
    }

    pub fn build(&self, ui: &mut Ui) {
        let rect = Rect::new(
            ui.translation_stack.translate(self.rect.origin),
            self.rect.size,
        );
        ui.layers.current_mut().images.push(ImageRenderElement {
            rect,
            image: self.image,
            border_radius: self.border_radius,
        });
    }
}

#[derive(Debug, Clone)]
pub struct ClickArea {
    id: Id,
    rect: Rect,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClickAreaEvent {
    Idle { hovered: bool, pressed: bool },
    PressStart,
    PressEnd { clicked: bool },
}

impl ClickAreaEvent {
    pub fn null() -> Self {
        Self::Idle {
            hovered: false,
            pressed: false,
        }
    }

    pub fn is_clicked(&self) -> bool {
        *self == ClickAreaEvent::PressEnd { clicked: true }
    }

    pub fn is_pressed(&self) -> bool {
        matches!(
            self,
            ClickAreaEvent::PressStart | ClickAreaEvent::Idle { pressed: true, .. }
        )
    }

    pub fn is_press_start(&self) -> bool {
        matches!(self, ClickAreaEvent::PressStart)
    }

    pub fn is_hovered(&self) -> bool {
        matches!(self, ClickAreaEvent::Idle { hovered: true, .. })
    }
}

pub fn click_area(id: impl Into<Id>) -> ClickArea {
    ClickArea::new(id)
}

impl ClickArea {
    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            rect: Rect::zero(),
        }
    }

    pub fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub fn pos(self, x: f32, y: f32) -> Self {
        self.x(x).y(y)
    }

    pub fn x(mut self, x: f32) -> Self {
        self.rect.origin.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.rect.origin.y = y;
        self
    }

    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.rect.size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.rect.size.height = height;
        self
    }

    pub fn build(&self, ui: &mut Ui) -> ClickAreaEvent {
        Self::check(
            ui,
            self.id,
            Rect::new(
                ui.translation_stack.translate(self.rect.origin),
                self.rect.size,
            ),
        )
    }

    fn check(ui: &mut Ui, id: Id, rect: Rect) -> ClickAreaEvent {
        let in_scissor_rect = ui
            .layers
            .current_scissor_rect()
            .map(|scissor_rect| scissor_rect.contains(ui.pointer_pos))
            .unwrap_or(true);

        let mouseover = in_scissor_rect && rect.contains(ui.pointer_pos);

        if mouseover {
            ui.hovered = Some(id);
        } else if ui.hovered == Some(id) {
            ui.hovered = None;
        }

        if ui.mouse_pressed && mouseover && ui.active.is_none() {
            ui.active = Some(id);
            return ClickAreaEvent::PressStart;
        }

        let pressed = ui.active == Some(id);

        if !ui.mouse_down && pressed {
            ui.active = None;
            ClickAreaEvent::PressEnd { clicked: mouseover }
        } else {
            ClickAreaEvent::Idle {
                hovered: mouseover && ui.active.is_none(),
                pressed,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    id: Option<Id>,
    pos: Point,
    size: Size,
    color: Color,
    hover_color: Color,
    preseed_color: Color,
    border_radius: [f32; 4],
    icon: &'static str,
    label: Cow<'static, str>,
    text_justify: TextJustify,
}

pub fn button() -> Button {
    Button::new()
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    pub fn new() -> Self {
        Self {
            id: None,
            pos: Point::zero(),
            size: Size::new(50.0, 50.0),
            color: Color::new_u8(0, 0, 0, 0.0),
            hover_color: Color::new_u8(57, 55, 62, 1.0),
            preseed_color: Color::new_u8(67, 65, 72, 1.0),
            border_radius: [0.0; 4],
            icon: "X",
            label: Cow::Borrowed(""),
            text_justify: TextJustify::Center,
        }
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn pos(self, x: f32, y: f32) -> Self {
        self.x(x).y(y)
    }

    pub fn x(mut self, x: f32) -> Self {
        self.pos.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.pos.y = y;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size.width = width;
        self.size.height = height;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.size.height = height;
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn hover_color(mut self, color: impl Into<Color>) -> Self {
        self.hover_color = color.into();
        self
    }

    pub fn preseed_color(mut self, color: impl Into<Color>) -> Self {
        self.preseed_color = color.into();
        self
    }

    pub fn border_radius(mut self, border_radius: [f32; 4]) -> Self {
        self.border_radius = border_radius;
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = icon;
        self
    }

    pub fn label(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn text_justify(mut self, text_justify: TextJustify) -> Self {
        self.text_justify = text_justify;
        self
    }

    fn gen_id(&self) -> Id {
        if let Some(id) = self.id {
            id
        } else if !self.label.is_empty() {
            Id::hash(&self.label)
        } else {
            Id::hash(self.icon)
        }
    }

    fn calc_bg_color(&self, ui: &mut Ui, id: Id) -> Color {
        if ui.active == Some(id) {
            self.preseed_color
        } else if ui.hovered == Some(id) && ui.active.is_none() {
            self.hover_color
        } else {
            self.color
        }
    }

    pub fn build(&self, ui: &mut Ui) -> bool {
        let rect = Rect::new(ui.translation_stack.translate(self.pos), self.size);

        let id = self.gen_id();
        let clicked = ClickArea::check(ui, id, rect).is_clicked();

        let color = self.calc_bg_color(ui, id);

        let layer = ui.layers.current_mut();
        layer.quads.push(QuadRenderElement {
            rect,
            border_radius: self.border_radius,
            color,
        });

        let pad_x = match self.text_justify {
            TextJustify::Left => 1.0,
            TextJustify::Right => -1.0,
            TextJustify::Center => 0.0,
        };

        if self.label.is_empty() {
            let icon_size = 20.0;
            let pad_x = pad_x * 2.0;

            let y = rect.origin.y + self::center_y(rect.size.height, icon_size);
            let x = match self.text_justify {
                TextJustify::Left => rect.origin.x + pad_x,
                TextJustify::Right => {
                    let x = rect.origin.x + rect.size.width - icon_size;
                    x + pad_x
                }
                TextJustify::Center => rect.origin.x + center_x(rect.size.width, icon_size),
            };

            layer.icons.push(IconRenderElement {
                origin: Point::new(x, y),
                size: icon_size,
                icon: self.icon.to_string(),
                color: Color::WHITE,
            });
        } else {
            let pad_x = pad_x * 10.0;

            layer.text.push(TextRenderElement {
                rect: Rect::new(
                    Point::new(rect.origin.x + pad_x, rect.origin.y),
                    Size::new(rect.size.width - pad_x * 2.0, rect.size.height),
                ),
                text_justify: self.text_justify,
                size: 16.0,
                bold: false,
                text: self.label.to_string(),
                color: Color::new_u8(255, 255, 255, 1.0),
                font_family: Cow::Borrowed("Roboto"),
            });
        }

        clicked
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pos: Point,
    size: Size,
    font_size: f32,
    text_justify: TextJustify,
    color: Color,
    text: String,
    icon: String,
    bold: bool,
    font_family: Cow<'static, str>,
}

pub fn label() -> Label {
    Label::new()
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

impl Label {
    pub fn new() -> Self {
        Self {
            pos: Point::zero(),
            size: Size::new(50.0, 50.0),
            font_size: 13.0,
            text_justify: TextJustify::Center,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            text: String::new(),
            icon: String::new(),
            bold: false,
            font_family: Cow::Borrowed("Roboto"),
        }
    }

    pub fn pos(self, x: f32, y: f32) -> Self {
        self.x(x).y(y)
    }

    pub fn x(mut self, x: f32) -> Self {
        self.pos.x = x;
        self
    }

    pub fn y(mut self, y: f32) -> Self {
        self.pos.y = y;
        self
    }

    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.size.height = height;
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn text_justify(mut self, text_justify: TextJustify) -> Self {
        self.text_justify = text_justify;
        self
    }

    pub fn font_family(mut self, font_family: &'static str) -> Self {
        self.font_family = Cow::Borrowed(font_family);
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn build(&self, ui: &mut Ui) {
        let layer = ui.layers.current_mut();
        let rect = Rect::new(ui.translation_stack.translate(self.pos), self.size);

        if !self.text.is_empty() {
            layer.text.push(TextRenderElement {
                rect,
                text_justify: self.text_justify,
                size: self.font_size,
                bold: self.bold,
                text: self.text.to_string(),
                color: self.color,
                font_family: self.font_family.clone(),
            });
        }

        if !self.icon.is_empty() {
            let pad_x = match self.text_justify {
                TextJustify::Left => 1.0,
                TextJustify::Right => -1.0,
                TextJustify::Center => 0.0,
            };

            let icon_size = self.font_size;
            let pad_x = pad_x * 10.0;

            let y = rect.origin.y + self::center_y(rect.size.height, icon_size);
            let x = match self.text_justify {
                TextJustify::Left => rect.origin.x + pad_x,
                TextJustify::Right => {
                    let x = rect.origin.x + rect.size.width - icon_size;
                    x + pad_x
                }
                TextJustify::Center => rect.origin.x + center_x(rect.size.width, icon_size),
            };

            layer.icons.push(IconRenderElement {
                origin: Point::new(x, y),
                size: icon_size,
                icon: self.icon.to_string(),
                color: self.color,
            });
        }
    }
}

pub fn combo_list<'a, ITEM: ToString>(
    ui: &mut Ui,
    id: impl Into<Id>,
    (item_w, item_h): (f32, f32),
    list: &'a [ITEM],
) -> Option<&'a ITEM> {
    self::quad()
        .width(item_w)
        .height(item_h * list.len() as f32)
        .color([27, 25, 32])
        .build(ui);

    let id = id.into();

    let mut res = None;
    for (nth, item) in list.iter().enumerate() {
        let id = Id::hash_with(|h| {
            id.as_raw().hash(h);
            nth.hash(h);
        });

        if self::button()
            .id(id)
            .y(item_h * nth as f32)
            .size(item_w, item_h)
            .label(item.to_string())
            .text_justify(TextJustify::Left)
            .border_radius([5.0; 4])
            .hover_color([160, 81, 255])
            .preseed_color([180, 90, 255])
            .build(ui)
        {
            res = Some(item);
        }
    }

    res
}
