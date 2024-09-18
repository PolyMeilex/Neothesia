use std::any::Any;

use smallvec::SmallVec;

#[derive(Debug)]
pub struct RowItem {
    pub width: f32,
    pub x: f32,
}

#[derive(Debug)]
pub struct RowLayout {
    items: SmallVec<[RowItem; 20]>,
    width: f32,
    gap: f32,
    dirty: bool,
    initialized: Option<Box<dyn Any>>,
}

impl Default for RowLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl RowLayout {
    pub fn new() -> Self {
        Self {
            items: SmallVec::new(),
            width: 0.0,
            gap: 0.0,
            dirty: true,
            initialized: None,
        }
    }

    pub fn set_gap(&mut self, gap: f32) {
        self.gap = gap;
    }

    pub fn once<T: Copy + 'static>(&mut self, cb: impl FnOnce(&mut Self) -> T) -> T {
        if self.initialized.is_none() {
            self.initialized = Some(Box::new(cb(self)));
        }

        let data = self.initialized.as_ref().unwrap();
        *data.downcast_ref().unwrap()
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    pub fn push(&mut self, width: f32) -> usize {
        let id = self.items.len();
        self.items.push(RowItem { width, x: 0.0 });
        id
    }

    fn width(&self) -> f32 {
        self.width
    }

    pub fn items(&self) -> &[RowItem] {
        &self.items
    }

    pub fn resolve_left(&mut self, origin_x: f32) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        let mut x = origin_x;

        for item in self.items.iter_mut() {
            item.x = x;
            x += item.width + self.gap;
        }

        self.width = self
            .items
            .last()
            .map(|i| (i.x + i.width) - origin_x)
            .unwrap_or(0.0);
    }

    pub fn resolve_center(&mut self, x: f32, width: f32) {
        if !self.dirty {
            return;
        }
        self.resolve_left(x);

        let center_x = width / 2.0 - self.width() / 2.0;

        for item in self.items.iter_mut() {
            item.x += center_x;
        }
    }

    pub fn resolve_right(&mut self, width: f32) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        let mut x = width;

        for item in self.items.iter_mut().rev() {
            x -= item.width;
            item.x = x;
            x -= self.gap;
        }

        self.width = self
            .items
            .last()
            .map(|i| (i.x - width) - i.width)
            .unwrap_or(0.0)
            .abs();
    }
}
