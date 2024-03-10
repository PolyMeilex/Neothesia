pub mod bbox;
pub mod resources;

pub use bbox::Bbox;

#[derive(Debug, Default, Clone, Copy)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T> From<Point<T>> for [T; 2] {
    fn from(p: Point<T>) -> Self {
        [p.x, p.y]
    }
}

impl<T: Copy> From<&Point<T>> for [T; 2] {
    fn from(p: &Point<T>) -> Self {
        (*p).into()
    }
}

impl<T> std::ops::Add for Point<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> std::ops::AddAssign for Point<T>
where
    T: std::ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Size<T> {
    pub w: T,
    pub h: T,
}

impl<T> Size<T> {
    pub fn new(w: T, h: T) -> Self {
        Self { w, h }
    }
}

impl<T> From<(T, T)> for Size<T> {
    fn from((w, h): (T, T)) -> Self {
        Self { w, h }
    }
}

impl<T> From<Size<T>> for [T; 2] {
    fn from(p: Size<T>) -> Self {
        [p.w, p.h]
    }
}

impl<T: Copy> From<&Size<T>> for [T; 2] {
    fn from(p: &Size<T>) -> Self {
        (*p).into()
    }
}
