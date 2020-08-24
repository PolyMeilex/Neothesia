use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort,
};

pub struct MidiDevicesMenager {
    midi_out: Option<MidiOutput>,
    // midi_in: Option<MidiInput>,
    midi_out_c: Option<MidiOutputConnection>,
    // midi_in_c: Option<MidiInputConnection>,
}

impl MidiDevicesMenager {
    pub fn new() -> Self {
        let midi_out = MidiOutput::new("midi_out").ok();
        // let midi_in = MidiInput::new("midi_in").ok();

        Self {
            midi_out,
            // midi_in,
            midi_out_c: None,
            // midi_in_c: None,
        }
    }
    pub fn get_outs(&self) -> Vec<MidiPortInfo> {
        match &self.midi_out {
            Some(midi_out) => {
                let mut outs = Vec::new();
                let ports = midi_out.ports();
                for p in ports {
                    let name = match midi_out.port_name(&p).ok() {
                        Some(name) => name,
                        None => String::from("Unknown"),
                    };
                    outs.push(MidiPortInfo { port: p, name })
                }
                outs
            }
            None => Vec::new(),
        }
    }
    // pub fn get_ins(&self) -> Vec<MidiPortInfo> {
    //     match &self.midi_in {
    //         Some(midi_in) => {
    //             let mut ins = Vec::new();
    //             let ports = midi_in.ports();

    //             for p in ports {
    //                 // for i in 0..midi_in.port_count() {
    //                 let name = match midi_in.port_name(&p).ok() {
    //                     Some(name) => name,
    //                     None => String::from("Unknown"),
    //                 };
    //                 ins.push(MidiPortInfo {
    //                     port: MidiPort::Input(p),
    //                     name,
    //                 })
    //             }
    //             ins
    //         }
    //         None => Vec::new(),
    //     }
    // }
    pub fn connect_out(&mut self, port: MidiPortInfo) {
        let midi_out = MidiOutput::new("midi_out").ok();

        if let Some(midi_out) = midi_out {
            self.midi_out_c = midi_out.connect(&port.port, "out").ok();
        }
    }

    pub fn send(&mut self, message: &[u8]) {
        if let Some(out) = &mut self.midi_out_c {
            out.send(message).ok();
        };
    }
}

// pub enum MidiPort {
//     Output(MidiOutputPort),
//     Input(MidiInputPort),
// }
// impl std::fmt::Debug for MidiPort {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             Self::Output(_) => write!(f, "Output"),
//             Self::Input(_) => write!(f, "Input"),
//         }
//     }
// }

pub struct MidiPortInfo {
    pub port: MidiOutputPort,
    pub name: String,
}

impl std::fmt::Debug for MidiPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
