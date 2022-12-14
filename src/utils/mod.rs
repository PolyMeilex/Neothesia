pub mod resources;
pub mod window;

#[derive(Debug, Default, Clone, Copy)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Size<T> {
    pub w: T,
    pub h: T,
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
