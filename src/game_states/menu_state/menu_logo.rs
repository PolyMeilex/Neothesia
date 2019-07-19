use std::io::Cursor;


use glium::index::PrimitiveType;
use glium::Surface;
pub struct MenuLogo {
  program: glium::Program,
  vertex_buffer: glium::VertexBuffer<Vertex>,
  index_buffer: glium::IndexBuffer<u16>,
  texture: glium::texture::CompressedTexture2d,
}

#[derive(Copy, Clone)]
struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

impl MenuLogo {
  pub fn new(display: &glium::Display) -> Self {
    let image = image::load(
      Cursor::new(&include_bytes!("../../../res/logo.png")[..]),
      image::PNG,
    )
    .unwrap()
    .to_rgba();
    let image_dimensions = image.dimensions();
    let image =
      glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::CompressedTexture2d::new(display, image).unwrap();

    let vertex_buffer = {
      glium::VertexBuffer::new(
        display,
        &[
          Vertex {
            position: [-1.0, -1.0],
            tex_coords: [0.0, 0.0],
          },
          Vertex {
            position: [-1.0, 1.0],
            tex_coords: [0.0, 1.0],
          },
          Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 1.0],
          },
          Vertex {
            position: [1.0, -1.0],
            tex_coords: [1.0, 0.0],
          },
        ],
      )
      .unwrap()
    };

    let index_buffer =
      glium::IndexBuffer::new(display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap();

    let vertex_shader_src = include_str!("../../shaders/menu/logo.vert");
    let fragment_shader_src = include_str!("../../shaders/menu/logo.frag");

    let program = glium::Program::new(
      display,
      glium::program::ProgramCreationInput::SourceCode {
        vertex_shader: vertex_shader_src,
        fragment_shader: fragment_shader_src,
        geometry_shader: None,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        transform_feedback_varyings: None,
        outputs_srgb: true,
        uses_point_size: false,
      },
    )
    .unwrap();

    MenuLogo {
      program,
      vertex_buffer,
      index_buffer,
      texture,
    }
  }
  pub fn draw(&self, target: &mut glium::Frame, viewport: &glium::Rect) {
    target
      .draw(
        &self.vertex_buffer,
        &self.index_buffer,
        &self.program,
        &uniform! {
          matrix: [
            [0.3 * 1.1, 0.0, 0.0, 0.0],
            [0.0, 0.1 * 1.1, 0.0, 0.0],
            [0.0, 0.0, 0.3 * 1.1, 0.0],
            [0.0, 0.5, 0.0, 1.0f32]
          ],
          tex: &self.texture
        },
        &glium::DrawParameters {
          viewport: Some(viewport.to_owned()),
          blend: glium::Blend::alpha_blending(),
          ..Default::default()
        },
      )
      .unwrap();
  }
}