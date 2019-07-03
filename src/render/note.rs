use glium::Surface;

pub struct NoteRenderer<'a> {
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

impl<'a> NoteRenderer<'a> {
  pub fn new(display: &'a glium::Display) -> NoteRenderer<'a> {
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

    let vertex_shader_src = r#"
        #version 330

        in vec2 pos;

        #define notesCount 52.0

        uniform vec2 m;

        out INTERFACE {
            vec2 uv;
            vec2 size;
        } Out;

        void main() {
            Out.size = vec2(0.9*2.0/notesCount, 1.0); 
            Out.uv = Out.size * pos;

            gl_Position = vec4(pos*Out.size+m, 0.0, 1.0);
            //gl_Position = vec4(pos*2, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 330        

        out vec4 fragColor;

        in INTERFACE {
            vec2 uv;
            vec2 size;
        } In;

        void main() {
            float radiusPosition = 
              pow(abs(In.uv.x/(0.5*In.size.x)), In.size.x/0.01) + 
              pow(abs(In.uv.y/(0.5*In.size.y)), In.size.y/0.01);

            if(	radiusPosition > 1.0){
                discard;
            }

            vec2 st = (In.uv + 1.0) / 2.0;
            vec3 color = vec3(st.x,st.y,0.7);

            fragColor = vec4(color,1.0);
        }
    "#;

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
      indices,
    }
  }
  pub fn draw(
    &self,
    target: &mut glium::Frame,
    rendered: &crate::render::GameRenderer,
    x: f64,
    y: f64,
  ) {
    target
      .draw(
        &self.vertex_buffer,
        &self.indices,
        &self.program,
        &uniform! {m: [x as f32,y as f32]},
        &glium::DrawParameters {
          viewport: Some(rendered.viewport.to_owned()),
          ..Default::default()
        },
      )
      .unwrap();
  }
}