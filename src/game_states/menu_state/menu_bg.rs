use glium::Surface;

pub struct MenuBg {
  program: glium::Program,
  vertex_buffer: glium::VertexBuffer<Vertex>,
  indices: glium::IndexBuffer<u16>,
}

#[derive(Copy, Clone)]
struct Vertex {
  pos: [f32; 2],
}
implement_vertex!(Vertex, pos);

impl MenuBg {
  pub fn new(display: &glium::Display) -> Self {
    let vertex1 = Vertex { pos: [-1.0, -1.0] };
    let vertex2 = Vertex { pos: [1.0, -1.0] };
    let vertex3 = Vertex { pos: [1.0, 1.0] };
    let vertex4 = Vertex { pos: [-1.0, 1.0] };

    let shape: [Vertex; 4] = [vertex1, vertex2, vertex3, vertex4];
    let indices_vec: [u16; 6] = [0, 1, 3, 3, 1, 2];

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::IndexBuffer::new(
      display,
      glium::index::PrimitiveType::TrianglesList,
      &indices_vec,
    )
    .unwrap();

    let vertex_shader_src = include_str!("../../shaders/menu/bg.vert");
    let fragment_shader_src = include_str!("../../shaders/menu/bg.frag");

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

    MenuBg {
      program,
      vertex_buffer,
      indices,
    }
  }
  pub fn draw(&self, target: &mut glium::Frame, viewport: &glium::Rect, time: f32) {
    target
      .draw(
        &self.vertex_buffer,
        &self.indices,
        &self.program,
        &uniform! {u_time: time},
        &glium::DrawParameters {
          viewport: Some(viewport.to_owned()),
          ..Default::default()
        },
      )
      .unwrap();
  }
}