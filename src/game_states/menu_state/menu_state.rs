use crate::game_states::GameState;
use crate::render::ui::Button;
use crate::utils;


pub struct MenuState<'a> {
  display: &'a glium::Display,
}

impl<'a> MenuState<'a> {
  pub fn new(display: &'a glium::Display) -> Self {
    MenuState { display }
  }
}

impl<'a> MenuState<'a> {
  fn play_song(&self, time: f64) -> Option<Box<GameState<'a> + 'a>> {
    let offset_time = std::time::Instant::now();

    let path = file_dialog::FileDialog::new()
      .path("./")
      .filters(vec!["mid", "midi"])
      .open();

    let path = match path {
      Ok(path) => path,
      Err(e) => return None,
    };

    // We Put Midi Load Before Calculating Time Offset Becouse Black Midis Cand Take Long Time To Load
    let midi = lib_midi::read_file(&path);

    let offset_time = offset_time.elapsed().as_millis() as f64 / 1000.0;
    let time = time + offset_time;

    if midi.merged_track.notes.len() == 0 {
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
  fn draw(
    &mut self,
    target: &mut glium::Frame,
    public_state: &crate::render::PublicState,
  ) -> Option<Box<GameState<'a> + 'a>> {
    let size = utils::Vec2 { x: 0.2, y: 0.1 };
    let mut btn = Button {
      pos: utils::Vec2 {
        x: 0.0 - size.x,
        y: 0.4 + size.y,
      },
      size: size,
      hover: false,
    };

    btn.hover_check(&public_state.m_pos);

    if btn.hover {
      if public_state.m_was_pressed {
        return self.play_song(public_state.time);
      }
    }

    public_state
      .ui_renderer
      .buttons_renderer
      .draw(target, public_state, btn);

    // 2
    let size = utils::Vec2 { x: 0.2, y: 0.1 };
    let mut btn = Button {
      pos: utils::Vec2 {
        x: 0.0 - size.x,
        y: 0.2 + size.y - 0.01,
      },
      size: size,
      hover: false,
    };

    btn.hover_check(&public_state.m_pos);


    public_state
      .ui_renderer
      .buttons_renderer
      .draw(target, public_state, btn);
    None
  }
}