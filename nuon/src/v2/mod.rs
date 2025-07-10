use std::hash::{Hash, Hasher};

use crate::Color;

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

impl Id {
    pub fn hash(v: impl Hash) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        v.hash(&mut hasher);
        Self(hasher.finish())
    }
}

pub struct Ui {
    hovered: Option<Id>,
    active: Option<Id>,

    pointer_pos: Point,
    pointer_pos_delta: Point,
    mouse_pressed: bool,
    mouse_down: bool,

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

    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.pos.x = x;
        self.pos.y = y;
        self
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

    fn handle_input(&self, ui: &mut Ui) -> (Id, bool) {
        let rect = Rect::new(self.pos, self.size);

        let id = self.gen_id();

        let mouseover = rect.contains(ui.pointer_pos);

        if mouseover {
            ui.hovered = Some(id);
        } else if ui.hovered == Some(id) {
            ui.hovered = None;
        }

        if ui.mouse_pressed && ui.hovered == Some(id) {
            ui.active = Some(id);
        }

        let clicked = if !ui.mouse_down && ui.active == Some(id) {
            ui.active = None;
            mouseover
        } else {
            false
        };

        (id, clicked)
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
        let (id, clicked) = self.handle_input(ui);
        let color = self.calc_bg_color(ui, id);

        let rect = Rect::new(self.pos, self.size);

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
    let pos = Point::new(*x, 100.0);
    let size = Size::new(50.0, 50.0);
    let rect = Rect::new(pos, size);

    let id = Id::hash(x as *const f32);

    let mouseover = rect.contains(ui.pointer_pos);

    if mouseover {
        ui.hovered = Some(id);
    } else if ui.hovered == Some(id) {
        ui.hovered = None;
    }

    if ui.mouse_pressed && ui.hovered == Some(id) {
        ui.active = Some(id);
    }

    let clicked = if !ui.mouse_down && ui.active == Some(id) {
        ui.active = None;
        mouseover
    } else {
        false
    };

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

    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.pos.x = x;
        self.pos.y = y;
        self
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

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn build(&self, ui: &mut Ui) {
        let rect = Rect::new(self.pos, self.size);
        ui.text
            .push((rect, TextJustify::Center, self.text.to_string()));
    }
}
