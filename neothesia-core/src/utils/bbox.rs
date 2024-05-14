use std::ops::Add;

use super::{Point, Size};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Bbox<T = f32> {
    pub pos: Point<T>,
    pub size: Size<T>,
}

impl<T> Bbox<T> {
    pub fn new(pos: impl Into<Point<T>>, size: impl Into<Size<T>>) -> Self {
        Self {
            pos: pos.into(),
            size: size.into(),
        }
    }
}

impl<T: Clone> Bbox<T> {
    pub fn x(&self) -> T {
        self.pos.x.clone()
    }

    pub fn y(&self) -> T {
        self.pos.y.clone()
    }

    pub fn w(&self) -> T {
        self.size.w.clone()
    }

    pub fn h(&self) -> T {
        self.size.h.clone()
    }
}

impl<T> Bbox<T>
where
    T: PartialEq + PartialOrd + Copy + Add<Output = T>,
{
    pub fn contains(&self, px: T, py: T) -> bool {
        let Point { x, y } = self.pos;
        let Size { w, h } = self.size;

        (x..(x + w)).contains(&px) && (y..(y + h)).contains(&py)
    }
}
