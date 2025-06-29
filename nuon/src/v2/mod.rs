use std::hash::{Hash, Hasher};

use crate::Color;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

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

    pub pointer_pos: Point,
    mouse_pressed: bool,
    mouse_down: bool,

    pub quads: Vec<(Rect, Color)>,
    pub text: Vec<(Point, String)>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            hovered: None,
            active: None,
            pointer_pos: Point::new(0.0, 0.0),
            mouse_pressed: false,
            mouse_down: false,
            quads: Vec::new(),
            text: Vec::new(),
        }
    }

    pub fn mouse_down(&mut self) {
        self.mouse_pressed = true;
        self.mouse_down = true;
    }

    pub fn mouse_up(&mut self) {
        self.mouse_pressed = false;
        self.mouse_down = false;
    }

    pub fn button(&mut self, y: f32, name: &str) -> bool {
        let pos = Point::new(0.0, y);
        let size = Size::new(50.0, 50.0);
        let rect = Rect::new(pos, size);

        let id = Id::hash(name);

        let mouseover = rect.contains(self.pointer_pos);

        if mouseover {
            self.hovered = Some(id);
        } else if self.hovered == Some(id) {
            self.hovered = None;
        }

        if self.mouse_pressed && self.hovered == Some(id) {
            self.active = Some(id);
        }

        let clicked = if !self.mouse_down && self.active == Some(id) {
            self.active = None;
            mouseover
        } else {
            false
        };

        let color = if self.active == Some(id) {
            Color::new_u8(52, 52, 52, 1.0)
        } else if self.hovered == Some(id) && self.active.is_none() {
            Color::new_u8(42, 42, 42, 1.0)
        } else {
            Color::new_u8(32, 32, 32, 1.0)
        };

        self.quads.push((rect, color));

        let x = rect.origin.x + rect.size.width / 2.0 - 10.0;
        let y = rect.origin.y + rect.size.height / 2.0 - 10.0;
        self.text.push((Point::new(x, y), "\u{F3E5}".to_string()));

        clicked
    }

    pub fn done(&mut self) {
        self.quads.clear();
        self.text.clear();
        self.mouse_pressed = false;
    }
}
