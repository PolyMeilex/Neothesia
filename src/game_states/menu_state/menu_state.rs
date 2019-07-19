use crate::game_states::GameState;
use crate::game_states::GameStateType;
use crate::render::ui::Button;
use crate::utils;

pub struct MenuState<'a> {
  state_type: GameStateType,
  display: &'a glium::Display,
  out_index: usize,
}

impl<'a> MenuState<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    MenuState {
      state_type: GameStateType::MenuState,
      display,
      out_index: 0,
    }
  }
  fn play_song(&self, time: f64) -> Option<Box<dyn GameState<'a> + 'a>> {
    let offset_time = std::time::Instant::now();

    let path = file_dialog::FileDialog::new()
      .path("./")
      .filters(vec!["mid", "midi"])
      .open();

    let path = match path {
      Ok(path) => path,
      Err(_e) => return None,
    };

    // We Put Midi Load Before Calculating Time Offset Becouse Black Midis Cand Take Long Time To Load
    let midi = match lib_midi::read_file(&path){
      Ok(midi) => midi,
      Err(e) => panic!(e),
    };

    let offset_time = offset_time.elapsed().as_millis() as f64 / 1000.0;
    let time = time + offset_time;

    if midi.merged_track.notes.is_empty() {
      // ? Probably no reason to panic here
      panic!(
        "No Notes In Track For Some Reason \n {:?}",
        midi.merged_track
      )
    }


    let notes = midi.merged_track.notes.clone();
    Some(Box::new(crate::game_states::PlayingState::new(
      self.display,
      notes,
      time,
    )))
  }
}

impl<'a> GameState<'a> for MenuState<'a> {
  fn get_type(&self) -> GameStateType{
    self.state_type
  }
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &mut crate::render::PublicState,
  ) -> Option<Box<dyn GameState<'a> + 'a>> {
    // File Select
    {
      let size = utils::Vec2 { x: 0.2, y: 0.1 };
      let mut btn = Button {
        pos: utils::Vec2 {
          x: 0.0 - size.x,
          y: 0.4 + size.y,
        },
        size,
        hover: false,
      };

      let hover = btn.hover_check(public_state.m_pos);

      if hover {
        if public_state.m_was_pressed {
          public_state.midi_device.connect_out(self.out_index);
          return self.play_song(public_state.time);
        }
      }

      public_state
        .ui_renderer
        .buttons_renderer
        .draw(target, public_state, btn);

      let text_pos = utils::opengl_to_pixel(0.0, 0.4, public_state.viewport);

      public_state.ui_renderer.text_writer.add(
        "Select File",
        text_pos.x,
        text_pos.y,
        true,
        Some([0.0, 0.0, 0.0, 1.0]),
      );
    }
    // Out Select
    {
      let outs_info = public_state.midi_device.get_outs();
      let max_outs = outs_info.len();
      let mut out_text = &String::from("No Outputs");
      if self.out_index < max_outs {
        out_text = &outs_info[self.out_index].name;
      } else {
        self.out_index = 0; // If index is device that was disconnected
      }

      {
        let left_btn_size = utils::Vec2 { x: 0.1, y: 0.05 };
        let mut left_btn = Button {
          pos: utils::Vec2 {
            x: 0.0 - (left_btn_size.x) * 2.0,
            y: 0.0 + left_btn_size.y * 2.0,
          },
          size: left_btn_size,
          hover: false,
        };

        if self.out_index > 0 {
          if left_btn.hover_check(public_state.m_pos) {
            if public_state.m_was_pressed {
              self.out_index -= 1;
            }
          }
        } else {
          left_btn.hover = true; // Button is not hovered, but should be greyed out
        }

        public_state
          .ui_renderer
          .buttons_renderer
          .draw(target, public_state, left_btn);
      }

      {
        let right_btn_size = utils::Vec2 { x: 0.1, y: 0.05 };
        let mut right_btn = Button {
          pos: utils::Vec2 {
            x: 0.0,
            y: 0.0 + right_btn_size.y * 2.0,
          },
          size: right_btn_size,
          hover: false,
        };

        if self.out_index < max_outs - 1 {
          if right_btn.hover_check(public_state.m_pos) {
            if public_state.m_was_pressed {
              self.out_index += 1;
            }
          }
        } else {
          right_btn.hover = true; // Button is not hovered, but should be greyed out
        }
        public_state
          .ui_renderer
          .buttons_renderer
          .draw(target, public_state, right_btn);
      }

      let text_pos = utils::opengl_to_pixel(0.0, 0.2, public_state.viewport);

      public_state
        .ui_renderer
        .text_writer
        .add(out_text, text_pos.x, text_pos.y, true, None);
    }
    None
  }

  fn prepare_drop(&mut self,public_state: &mut crate::render::PublicState){}
}
