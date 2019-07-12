pub struct MidiDevice {
  midi_out: Option<midir::MidiOutput>,
  midi_in: Option<midir::MidiOutput>,
  midi_out_c: Option<midir::MidiOutputConnection>,
  midi_in_c: Option<midir::MidiOutputConnection>,
}

impl MidiDevice {
  pub fn new() -> Self {
    let midi_out = midir::MidiOutput::new("midi_out").ok();
    let midi_in = midir::MidiOutput::new("midi_in").ok();

    MidiDevice {
      midi_out,
      midi_in,
      midi_out_c: None,
      midi_in_c: None,
    }
  }
  pub fn get_outs(&self) -> Vec<MidiCInfo> {
    let outs = match &self.midi_out {
      Some(midi_out) => {
        let mut outs = Vec::new();
        for i in 0..midi_out.port_count() {
          let name = match midi_out.port_name(i).ok() {
            Some(name) => name,
            None => String::from("Unknown"),
          };
          outs.push(MidiCInfo {
            id: i as usize,
            name: name,
          })
        }
        outs
      }
      None => Vec::new(),
    };
    outs
  }
  pub fn get_ins(&self) -> Vec<MidiCInfo> {
    let ins = match &self.midi_in {
      Some(midi_in) => {
        let mut ins = Vec::new();
        for i in 0..midi_in.port_count() {
          let name = match midi_in.port_name(i).ok() {
            Some(name) => name,
            None => String::from("Unknown"),
          };
          ins.push(MidiCInfo {
            id: i as usize,
            name: name,
          })
        }
        ins
      }
      None => Vec::new(),
    };
    ins
  }
  pub fn connect_out(&mut self, id: usize) {
    let midi_out = midir::MidiOutput::new("midi_out").ok();
    match midi_out {
      Some(midi_out) => {
        self.midi_out_c = midi_out.connect(id, "out").ok();
      }
      None => {}
    }

  }
  pub fn send(&mut self, message: &[u8]) {
    let _res = match &mut self.midi_out_c {
      Some(out) => out.send(message),
      None => Ok(()),
    };
  }
}

#[derive(Debug)]
pub struct MidiCInfo {
  pub id: usize,
  pub name: String,
}
