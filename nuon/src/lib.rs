use std::hash::{Hash, Hasher};

pub use euclid;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

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
    pub fn hash(v: impl Hash) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        v.hash(&mut hasher);
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

#[derive(Debug)]
pub struct LayerStack {
    layers: Vec<LayerData>,
    history_stack: Vec<usize>,
    curr: usize,
}

impl LayerStack {
    fn new() -> Self {
        LayerStack {
            layers: vec![LayerData::default()],
            history_stack: vec![],
            curr: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &LayerData> {
        self.layers.iter()
    }

    pub fn current_mut(&mut self) -> &mut LayerData {
        &mut self.layers[self.curr]
    }

    pub fn push(&mut self) {
        self.history_stack.push(self.curr);
        self.curr = self.layers.len();
        self.layers.push(LayerData::default());
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
        self.curr = 0;
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
}

#[derive(Debug, Clone)]
pub struct TextRenderElement {
    pub rect: Rect,
    pub text_justify: TextJustify,
    pub size: f32,
    pub bold: bool,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ImageRenderElement {
    pub rect: Rect,
    pub image: iced_core::image::Handle,
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
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scissor_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) {
        let rect = Rect::new(
            ui.translation_stack.translate(self.rect.origin),
            self.rect.size,
        );

        ui.layers.push();
        ui.layers.current_mut().scissor_rect = rect;

        build(ui);
        ui.layers.pop();
    }
}

pub fn layer() -> Layer {
    Layer::new()
}

#[derive(Debug, Clone, Default)]
pub struct Scroll {
    rect: Rect,
    scroll: f32,
}

impl Scroll {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scissor_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub fn scissor_size(mut self, w: f32, h: f32) -> Self {
        self.rect.size.width = w;
        self.rect.size.height = h;
        self
    }

    pub fn scroll(mut self, scroll: f32) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) {
        self::layer().scissor_rect(self.rect).build(ui, |ui| {
            self::quad()
                .size(self.rect.size.width, self.rect.size.height)
                .color([17; 3])
                .build(ui);

            let last = self::translate().y(-self.scroll).build(ui, build);
            let last_y = last.y - self.rect.size.height;

            let percentage = self.scroll / last_y;

            let w = 10.0;
            let h = self.rect.height() / (last_y / self.rect.height());

            self::quad()
                .y(0.0)
                .x(self.rect.size.width - w)
                .size(w, self.rect.height())
                .color([37, 35, 42])
                .border_radius([5.0; 4])
                .build(ui);
            self::quad()
                .y(percentage * (self.rect.size.height - h))
                .x(self.rect.size.width - w)
                .size(w, h)
                .color([74, 68, 88])
                .border_radius([5.0; 4])
                .build(ui);
        });
    }
}

pub fn scroll() -> Scroll {
    Scroll::new()
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
    handle: iced_core::image::Handle,
}

pub fn image(handle: iced_core::image::Handle) -> Image {
    Image::new(handle)
}

impl Image {
    pub fn new(handle: iced_core::image::Handle) -> Self {
        Self {
            rect: Rect::zero(),
            handle,
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

    pub fn build(&self, ui: &mut Ui) {
        let rect = Rect::new(
            ui.translation_stack.translate(self.rect.origin),
            self.rect.size,
        );
        ui.layers.current_mut().images.push(ImageRenderElement {
            rect,
            image: self.handle.clone(),
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
        let mouseover = rect.contains(ui.pointer_pos);

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
    id: Option<&'static str>,
    pos: Point,
    size: Size,
    color: Color,
    hover_color: Color,
    preseed_color: Color,
    border_radius: [f32; 4],
    icon: &'static str,
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
            text_justify: TextJustify::Center,
        }
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
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

    pub fn text_justify(mut self, text_justify: TextJustify) -> Self {
        self.text_justify = text_justify;
        self
    }

    fn gen_id(&self) -> Id {
        if let Some(id) = self.id {
            Id::hash(id)
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

        let icon_size = 20.0;
        let half_size = icon_size / 2.0;

        let (x, y) = match self.text_justify {
            TextJustify::Left => {
                let y = rect.origin.y + rect.size.height / 2.0 - half_size;
                (rect.origin.x + 2.0, y)
            }
            TextJustify::Right => {
                let x = rect.origin.x + rect.size.width - icon_size;
                let y = rect.origin.y + rect.size.height / 2.0 - half_size;
                (x - 2.0, y)
            }
            TextJustify::Center => {
                let x = rect.origin.x + rect.size.width / 2.0 - half_size;
                let y = rect.origin.y + rect.size.height / 2.0 - half_size;
                (x, y)
            }
        };

        layer.icons.push(IconRenderElement {
            origin: Point::new(x, y),
            size: icon_size,
            icon: self.icon.to_string(),
        });

        clicked
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pos: Point,
    size: Size,
    font_size: f32,
    text: String,
    icon: String,
    bold: bool,
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
            text: String::new(),
            icon: String::new(),
            bold: false,
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
                text_justify: TextJustify::Center,
                size: self.font_size,
                bold: self.bold,
                text: self.text.to_string(),
            });
        }

        if !self.icon.is_empty() {
            let (x, y) = {
                let half_size = self.font_size / 2.0;
                let x = rect.origin.x + rect.size.width / 2.0 - half_size;
                let y = rect.origin.y + rect.size.height / 2.0 - half_size;
                (x, y)
            };

            layer.icons.push(IconRenderElement {
                origin: Point::new(x, y),
                size: self.font_size,
                icon: self.icon.to_string(),
            });
        }
    }
}
