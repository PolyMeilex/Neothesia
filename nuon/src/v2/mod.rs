use std::hash::{Hash, Hasher};

pub use crate::Color;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

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

pub struct Ui {
    pub hovered: Option<Id>,
    pub active: Option<Id>,

    pointer_pos: Point,
    pointer_pos_delta: Point,
    pub mouse_pressed: bool,
    pub mouse_down: bool,

    translation_stack: TranslationStack,

    pub quads: Vec<(Rect, [f32; 4], Color)>,
    pub icons: Vec<(Point, String)>,
    pub text: Vec<(Rect, TextJustify, String)>,
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
            quads: Vec::new(),
            icons: Vec::new(),
            text: Vec::new(),
        }
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
        self.quads.clear();
        self.icons.clear();
        self.text.clear();
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

    pub fn build(&self, ui: &mut Ui, build: impl FnOnce(&mut Ui)) {
        let offset = self.origin;
        let prev = ui.translation_stack.offset();
        ui.translation_stack
            .0
            .push(Point::new(prev.x + offset.x, prev.y + offset.y));
        build(ui);
        ui.translation_stack.0.pop();
    }
}

pub fn translate() -> Translate {
    Translate::new()
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

        ui.quads.push((rect, self.border_radius, self.color));
    }
}

#[derive(Debug, Clone)]
pub struct ClickArea {
    id: Id,
    rect: Rect,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClickAreaEvent {
    Idle { hovered: bool },
    PressStart,
    PressEnd { clicked: bool },
}

impl ClickAreaEvent {
    pub fn is_clicked(&self) -> bool {
        *self == ClickAreaEvent::PressEnd { clicked: true }
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

        if ui.mouse_pressed && mouseover {
            ui.active = Some(id);
            return ClickAreaEvent::PressStart;
        }

        if !ui.mouse_down && ui.active == Some(id) {
            ui.active = None;
            ClickAreaEvent::PressEnd { clicked: mouseover }
        } else {
            ClickAreaEvent::Idle { hovered: mouseover }
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

        ui.quads.push((rect, self.border_radius, color));

        let (x, y) = match self.text_justify {
            TextJustify::Left => {
                let y = rect.origin.y + rect.size.height / 2.0 - 10.0;
                (rect.origin.x + 2.0, y)
            }
            TextJustify::Right => {
                let icon_size = 20.0;
                let x = rect.origin.x + rect.size.width - icon_size;
                let y = rect.origin.y + rect.size.height / 2.0 - 10.0;
                (x - 2.0, y)
            }
            TextJustify::Center => {
                let x = rect.origin.x + rect.size.width / 2.0 - 10.0;
                let y = rect.origin.y + rect.size.height / 2.0 - 10.0;
                (x, y)
            }
        };

        ui.icons.push((Point::new(x, y), self.icon.to_string()));

        clicked
    }
}

pub fn slider(ui: &mut Ui, x: &mut f32) -> bool {
    let pos = ui.translation_stack.translate(Point::new(*x, 100.0));
    let size = Size::new(50.0, 50.0);
    let rect = Rect::new(pos, size);

    let id = Id::hash(x as *const f32);

    let clicked = ClickArea::check(ui, id, rect).is_clicked();

    if ui.active == Some(id) {
        *x += ui.pointer_pos_delta.x;
    }

    let color = if ui.active == Some(id) {
        Color::new_u8(52, 52, 52, 1.0)
    } else if ui.hovered == Some(id) && ui.active.is_none() {
        Color::new_u8(42, 42, 42, 1.0)
    } else {
        Color::new_u8(32, 32, 32, 1.0)
    };

    ui.quads.push((rect, [5.0; 4], color));

    let x = rect.origin.x + rect.size.width / 2.0 - 10.0;
    let y = rect.origin.y + rect.size.height / 2.0 - 10.0;
    ui.icons.push((Point::new(x, y), "\u{F3E5}".to_string()));

    clicked
}

#[derive(Debug, Clone)]
pub struct Label {
    pos: Point,
    size: Size,
    text: String,
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
            text: String::new(),
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

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn build(&self, ui: &mut Ui) {
        let rect = Rect::new(ui.translation_stack.translate(self.pos), self.size);
        ui.text
            .push((rect, TextJustify::Center, self.text.to_string()));
    }
}
