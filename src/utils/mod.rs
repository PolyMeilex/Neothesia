pub struct Vec2{
  pub x:f32,
  pub y:f32,
}

impl Vec2{
  pub fn to_array(&self) -> [f32;2]{
    [self.x,self.y]
  }
}