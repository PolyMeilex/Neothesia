use glium::Surface;

pub struct NoteRenderer<'a> {
  display: &'a glium::Display,
  program: glium::Program,
  vertex_buffer: glium::VertexBuffer<Vertex>,
  per_instance: glium::VertexBuffer<InstanceAttr>,
  indices: glium::IndexBuffer<u16>,
}

#[derive(Copy, Clone)]
struct Vertex {
  pos: [f32; 2],
}
implement_vertex!(Vertex, pos);

#[derive(Copy, Clone)]
struct InstanceAttr {
  noteIn: (f32, f32, f32),
}
implement_vertex!(InstanceAttr, noteIn);


impl<'a> NoteRenderer<'a> {
  pub fn new(display: &'a glium::Display, notes: &Vec<crate::lib_midi::track::MidiNote>) -> Self {
    let vertex1 = Vertex { pos: [-0.5, -0.5] };
    let vertex2 = Vertex { pos: [0.5, -0.5] };
    let vertex3 = Vertex { pos: [0.5, 0.5] };
    let vertex4 = Vertex { pos: [-0.5, 0.5] };

    let shape: [Vertex; 4] = [vertex1, vertex2, vertex3, vertex4];
    let indices_vec: [u16; 6] = [0, 1, 3, 3, 1, 2];

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::IndexBuffer::new(
      display,
      glium::index::PrimitiveType::TrianglesList,
      &indices_vec,
    )
    .unwrap();

    let per_instance = {
      let data: Vec<InstanceAttr> = notes
        .iter()
        .map(|n| InstanceAttr {
          noteIn: (n.note as f32, n.start as f32, n.duration as f32),
        })
        .collect();

      glium::vertex::VertexBuffer::dynamic(display, &data).unwrap()
    };

    let vertex_shader_src = include_str!("../../shaders/note.vert");
    let fragment_shader_src = include_str!("../../shaders/note.frag");

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

    NoteRenderer {
      display,
      program,
      vertex_buffer,
      per_instance,
      indices,
    }
  }
  pub fn draw(
    &self,
    target: &mut glium::Frame,
    viewport: &glium::Rect,
    time: f32,
  ) {
    target
      .draw(
        (
          &self.vertex_buffer,
          self.per_instance.per_instance().unwrap(),
        ),
        &self.indices,
        &self.program,
        &uniform! {time:time},
        &glium::DrawParameters {
          viewport: Some(viewport.to_owned()),
          ..Default::default()
        },
      )
      .unwrap();
  }
}