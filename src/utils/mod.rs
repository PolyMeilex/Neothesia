#[derive(Clone,Copy)]
pub struct Vec2 {
  pub x: f32,
  pub y: f32,
}

impl Vec2 {
  pub fn to_array(&self) -> [f32; 2] {
    [self.x, self.y]
  }
}

pub fn pixel_to_opengl(pos_x: f64, pos_y: f64, viewport: glium::Rect) -> Vec2 {
  let vh = viewport.height as f64;
  let vw = viewport.width as f64;

  let cord_x = pos_x / (vw / 2.0) - 1.0;
  let cord_y = -( (pos_y - viewport.bottom as f64) / (vh / 2.0) - 1.0);

  Vec2 {
    x: cord_x as f32,
    y: cord_y as f32,
  }
}

pub fn opengl_to_pixel(cord_x: f64, cord_y: f64, viewport: glium::Rect) -> Vec2 {
  let vh = viewport.height as f64;
  let vw = viewport.width as f64;

  let pos_x = (cord_x + 1.0) * (vw / 2.0);
  let pos_y = (-cord_y + 1.0) * (vh / 2.0);
  Vec2 {
    x: pos_x as f32,
    y: pos_y as f32 + viewport.bottom as f32,
  }
}
