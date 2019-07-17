mod downcast;
pub use downcast::SuperUnsafeDowncaster;

#[derive(Clone, Copy)]
pub struct Vec2 {
  pub x: f32,
  pub y: f32,
}

impl Vec2 {
  pub fn to_array(self) -> [f32; 2] {
    [self.x, self.y]
  }
}

pub fn pixel_to_opengl(pos_x: f64, pos_y: f64, viewport: glium::Rect) -> Vec2 {
  let vh = f64::from(viewport.height);
  let vw = f64::from(viewport.width);

  let cord_x = pos_x / (vw / 2.0) - 1.0;
  let cord_y = -((pos_y - f64::from(viewport.bottom)) / (vh / 2.0) - 1.0);

  Vec2 {
    x: cord_x as f32,
    y: cord_y as f32,
  }
}

pub fn opengl_to_pixel(cord_x: f64, cord_y: f64, viewport: glium::Rect) -> Vec2 {
  let vh = f64::from(viewport.height);
  let vw = f64::from(viewport.width);

  let pos_x = (cord_x + 1.0) * (vw / 2.0);
  let pos_y = (-cord_y + 1.0) * (vh / 2.0);
  Vec2 {
    x: pos_x as f32,
    y: pos_y as f32 + viewport.bottom as f32,
  }
}
