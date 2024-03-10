use super::{Point, Size};

#[derive(Default, Clone, Copy)]
pub struct Bbox {
    pub pos: Point<f32>,
    pub size: Size<f32>,
}

impl Bbox {
    pub fn new(pos: Point<f32>, size: Size<f32>) -> Self {
        Self { pos, size }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        let Point { x, y } = self.pos;
        let Size { w, h } = self.size;

        (x..(x + w)).contains(&px) && (y..(y + h)).contains(&py)
    }

    pub fn x(&self) -> f32 {
        self.pos.x
    }

    pub fn y(&self) -> f32 {
        self.pos.y
    }

    pub fn w(&self) -> f32 {
        self.size.w
    }

    pub fn h(&self) -> f32 {
        self.size.h
    }
}
