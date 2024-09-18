use crate::RowLayout;

#[derive(Debug)]
pub struct TriRowLayout {
    pub start: RowLayout,
    pub center: RowLayout,
    pub end: RowLayout,
}

impl Default for TriRowLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl TriRowLayout {
    pub fn new() -> Self {
        Self {
            start: RowLayout::new(),
            center: RowLayout::new(),
            end: RowLayout::new(),
        }
    }

    pub fn invalidate(&mut self) {
        self.start.invalidate();
        self.center.invalidate();
        self.end.invalidate();
    }

    pub fn resolve(&mut self, x: f32, width: f32) {
        self.start.resolve_left(x);
        self.center.resolve_center(x, width);
        self.end.resolve_right(width);
    }
}
