use crate::utils;
use glium::Surface;

pub struct ButtonsRenderer<'a> {
  display: &'a glium::Display,
  program: glium::Program,
  vertex_buffer: glium::VertexBuffer<Vertex>,
  indices: glium::IndexBuffer<u16>,
}

#[derive(Copy, Clone)]
struct Vertex {
  pos: [f32; 2],
}
implement_vertex!(Vertex, pos);

impl<'a> ButtonsRenderer<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
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

    let vertex_shader_src = include_str!("../../shaders/ui/button.vert");
    let fragment_shader_src = include_str!("../../shaders/ui/button.frag");

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

    ButtonsRenderer {
      display,
      program,
      vertex_buffer,
      indices,
    }
  }
  pub fn draw(
    &self,
    target: &mut glium::Frame,
    public_state: &crate::render::PublicState,
    btn: Button,
  ) {
    target
      .draw(
        &self.vertex_buffer,
        &self.indices,
        &self.program,
        &uniform! {btnPos:btn.pos.to_array(), btnSize:btn.size.to_array(),btnHover:btn.hover as i8},
        &glium::DrawParameters {
          viewport: Some(public_state.viewport.to_owned()),
          ..Default::default()
        },
      )
      .unwrap();
  }
}

pub struct Button {
  pub pos: utils::Vec2, 
  pub size: utils::Vec2,
  pub hover: bool,
}

impl Button {
  pub fn hover_check(&mut self, m_pos: &utils::Vec2) {
    self.hover = m_pos.x > self.pos.x
      && m_pos.x < self.pos.x + self.size.x * 2.0
      && m_pos.y < self.pos.y
      && m_pos.y > self.pos.y - self.size.y * 2.0;
  }
}