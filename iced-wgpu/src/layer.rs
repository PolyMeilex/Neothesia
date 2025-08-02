use crate::{image, Rectangle};

#[derive(Debug)]
pub(crate) struct Layer {
    pub bounds: Rectangle,
    pub images: image::Batch,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            bounds: Rectangle::new((0.0, 0.0).into(), (f32::INFINITY, f32::INFINITY).into()),
            images: image::Batch::default(),
        }
    }

    pub fn reset(&mut self) {
        self.bounds = Rectangle::new((0.0, 0.0).into(), (f32::INFINITY, f32::INFINITY).into());
        self.images.clear();
    }
}
