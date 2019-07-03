use glium::Surface;

pub struct KeyboardRenderer<'a> {
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

impl<'a> KeyboardRenderer<'a> {
  pub fn new(display: &'a glium::Display) -> KeyboardRenderer<'a> {
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

    let vertex_shader_src = r#"
        #version 330

        in vec2 pos;

        #define notesCount 52.0

        out INTERFACE {
            vec2 uv;
        } Out;

        void main() {
            gl_Position = vec4(pos,0.0,1.0);
            
            Out.uv = pos.xy * 0.5 + 0.5;
            Out.uv.x*=4.3; // Display 88 Keys 
            Out.uv.x+=0.04; // Move Keys To start from A
        }
    "#;

    let fragment_shader_src = r#"
        #version 330        

        out vec4 fragColor;

        uniform ActiveNotes{
            uint activeNotesArray[128];
        };

        in INTERFACE {
            vec2 uv;
        } In;

        #define border 0.002

        vec3 key_color;

        void black_key_render(bool drawBase,vec2 uv, float pos, float pitch_offset, float curr_pitch, inout vec3 color){
          float mod_x=mod(uv.x+4.*59./725.,413./725.);
          float div_x=floor((uv.x+4.*59./725.)/(413./725.));
            
          vec3 col = vec3(0.0);
          bool active = false;

          if(!drawBase){
            col = key_color;
            active = !(curr_pitch != pitch_offset+div_x*12. - 12.);
          }

          if(active || drawBase){
            color=mix(color,col,smoothstep(-border,0.,5./29.-abs(uv.y-.25+5./29.))*smoothstep(-border,0.,127./5800.-abs(mod_x-pos/5800.)));
          }
        }

        void main() {
          vec3 color= vec3(1.0);
          key_color=vec3(131.0/255.0,23.0/255.0,181.0/255.0);

          if(In.uv.y < .24){
            float key_loc = floor( 725./59.* (In.uv.x+531./1450.) )-7.0;

            float pitch=float((2*int(key_loc) - int(floor((float(key_loc)+.5)/3.5))));

            uint notes[128] = activeNotesArray;
            
            color=mix(vec3(0.0),color,smoothstep(0.,border,abs(mod(In.uv.x,59./725.)-59./1450.)));

            // Draw Black Keys
            if(In.uv.y > .08){
                black_key_render(true,In.uv, 183., 1., 0.0, color);
                black_key_render(true,In.uv, 743., 3., 0.0, color);
                black_key_render(true,In.uv, 1577., 6., 0.0, color);
                black_key_render(true,In.uv, 2115., 8., 0.0, color);
                black_key_render(true,In.uv, 2653., 10., 0.0, color);
            }

            // Draw Highlights
            for(int i=0;i<88;++i)
            {
              float mouse_pitch = float(notes[i]);
            

              if(mouse_pitch==pitch){
                color=key_color;
              }    

              if(In.uv.y > .08){
                black_key_render(false,In.uv, 183., 1., mouse_pitch, color);
                black_key_render(false,In.uv, 743., 3., mouse_pitch, color);
                black_key_render(false,In.uv, 1577., 6., mouse_pitch, color);
                black_key_render(false,In.uv, 2115., 8., mouse_pitch, color);
                black_key_render(false,In.uv, 2653., 10., mouse_pitch, color);
              }

            }

            
          }   
          else{
            color=vec3(0.0);
            // discard;
          } 
              
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

    KeyboardRenderer {
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
    let notes: glium::uniforms::UniformBuffer<[u32; 128]> =
      glium::uniforms::UniformBuffer::empty_dynamic(self.display).unwrap();

    let mut notes_data = [0; 128];

    for n in 0..notes_data.len() {
      notes_data[n] = 128;
    }

    notes.write(&notes_data);

    target
      .draw(
        &self.vertex_buffer,
        &self.indices,
        &self.program,
        &uniform! {ActiveNotes: &notes},
        &glium::DrawParameters {
          viewport: Some(rendered.viewport.to_owned()),
          ..Default::default()
        },
      )
      .unwrap();
  }
}