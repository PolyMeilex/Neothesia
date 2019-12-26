use crate::game_states::GameState;
use crate::game_states::GameStateType;
use crate::render::ui::Button;
use crate::utils;

mod menu_bg;
mod menu_logo;

use lib_midi::Midi;

pub struct MenuState<'a> {
  state_type: GameStateType,
  display: &'a glium::Display,
  menu_bg: menu_bg::MenuBg,
  menu_logo: menu_logo::MenuLogo,
  out_index: usize,
}

impl<'a> MenuState<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    MenuState {
      state_type: GameStateType::MenuState,
      display,
      menu_bg: menu_bg::MenuBg::new(display),
      menu_logo: menu_logo::MenuLogo::new(display),
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
      Err(_e) => return None, // Dialog (Probably) Got Canceled
    };

    // We Put Midi Load Before Calculating Time Offset Becouse Black Midis Cand Take Long Time To Load
    let midi = match Midi::new(&path) {
      Ok(midi) => midi,
      Err(e) => {
        println!("MIDI Reading Error: {}", e);
        return None;
      }
    };

    if midi.merged_track.notes.is_empty() {
      println!("No Notes In MIDI");
      return None;
    }

    // We Calculate How Long It Took For "Dialog" And "Midi Load" To compleate
    // And Add It To Start Time Of Plaing State
    // Becouse Those Functions Are Thread Blocking And Time Is Not Calculated While They Run
    let offset_time = offset_time.elapsed().as_millis() as f64 / 1000.0;
    let time = time + offset_time;

    let notes = midi.merged_track.notes.clone();
    Some(Box::new(crate::game_states::PlayingState::new(
      self.display,
      notes,
      time,
    )))
  }
}

impl<'a> GameState<'a> for MenuState<'a> {
  fn get_type(&self) -> GameStateType {
    self.state_type
  }
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &mut crate::render::PublicState,
  ) -> Option<Box<dyn GameState<'a> + 'a>> {
    self
      .menu_bg
      .draw(target, &public_state.viewport, public_state.time as f32);

    self.menu_logo.draw(target, &public_state.viewport);

    // File Select
    {
      let size = utils::Vec2 { x: 0.2, y: 0.1 };
      let mut btn = Button {
        pos: utils::Vec2 {
          x: 0.0 - size.x,
          y: 0.2 + size.y,
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

      let text_pos = utils::opengl_to_pixel(0.0, 0.2, public_state.viewport);

      public_state
        .ui_renderer
        .text_writer
        .add("Select File", text_pos.x, text_pos.y, true, None);
    }
    // Out Select
    let outs_info = public_state.midi_device.get_outs();
    let max_outs = outs_info.len();
    if max_outs > 0 {
      let mut out_text = &String::from("No Outputs");

      // Midi Device Range Check
      if self.out_index < max_outs {
        out_text = &outs_info[self.out_index].name;
      } else {
        // Reset Device Index, to make sure that disconected device is not selected
        self.out_index = 0;
      }

      //
      // Out Device Text
      //
      let text_pos = utils::opengl_to_pixel(0.0, 0.0, public_state.viewport);

      public_state
        .ui_renderer
        .text_writer
        .add(out_text, text_pos.x, text_pos.y, true, None);

      //
      // LEFT BTN
      //
      {
        let left_btn_size = utils::Vec2 { x: 0.1, y: 0.05 };
        let mut left_btn = Button {
          pos: utils::Vec2 {
            x: 0.0 - (left_btn_size.x) * 2.0,
            y: -0.2 + left_btn_size.y * 2.0,
          },
          size: left_btn_size,
          hover: false,
        };

        let mut btn_disable = false;
        if self.out_index > 0 {
          if left_btn.hover_check(public_state.m_pos) {
            if public_state.m_was_pressed {
              self.out_index -= 1;
            }
          }
        } else {
          btn_disable = true;
        }

        public_state
          .ui_renderer
          .buttons_renderer
          .draw(target, public_state, left_btn);

        // Text Arrow "<"
        let text_pos = utils::Vec2 {
          x: 0.0 - left_btn_size.x,
          y: -0.2 + left_btn_size.y,
        };
        let text_pos =
          utils::opengl_to_pixel(text_pos.x as f64, text_pos.y as f64, public_state.viewport);

        let text_color: Option<[f32; 4]> = if btn_disable {
          Some([0.3, 0.3, 0.3, 0.3])
        } else {
          None
        };

        public_state
          .ui_renderer
          .text_writer
          .add("<", text_pos.x, text_pos.y, true, text_color);
      }

      //
      // RIGHT BTN
      //
      {
        let right_btn_size = utils::Vec2 { x: 0.1, y: 0.05 };
        let mut right_btn = Button {
          pos: utils::Vec2 {
            x: 0.0,
            y: -0.2 + right_btn_size.y * 2.0,
          },
          size: right_btn_size,
          hover: false,
        };

        let mut btn_disable = false;
        if self.out_index < max_outs - 1 {
          if right_btn.hover_check(public_state.m_pos) {
            if public_state.m_was_pressed {
              self.out_index += 1;
            }
          }
        } else {
          btn_disable = true;
        }

        public_state
          .ui_renderer
          .buttons_renderer
          .draw(target, public_state, right_btn);

        // Text Arrow ">"
        let text_pos = utils::Vec2 {
          x: 0.0 + right_btn_size.x,
          y: -0.2 + right_btn_size.y,
        };
        let text_pos =
          utils::opengl_to_pixel(text_pos.x as f64, text_pos.y as f64, public_state.viewport);

        let text_color: Option<[f32; 4]> = if btn_disable {
          Some([0.3, 0.3, 0.3, 0.3])
        } else {
          None
        };

        public_state
          .ui_renderer
          .text_writer
          .add(">", text_pos.x, text_pos.y, true, text_color);
      }
    }
    None
  }

  fn prepare_drop(&mut self, public_state: &mut crate::render::PublicState) {}
}
