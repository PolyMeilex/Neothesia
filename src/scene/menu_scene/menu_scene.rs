use crate::{
    midi_device::{MidiDevicesMenager, MidiPortInfo},
    scene::{Scene, SceneEvent, SceneType},
    ui::Ui,
    wgpu_jumpstart::Gpu,
    MainState,
};

use super::bg_pipeline::BgPipeline;
use std::sync::mpsc;
use std::thread;

pub enum Event {
    MidiOpen(lib_midi::Midi, MidiPortInfo),
}

pub struct MenuScene {
    aysnc_job: async_job::Job<async_job::Event>,
    bg_pipeline: BgPipeline,
    midi_device_select: MidiDeviceSelect,
    file: Option<lib_midi::Midi>,
}

impl MenuScene {
    pub fn new(gpu: &mut Gpu, state: &mut MainState) -> Self {
        let (sender, receiver) = mpsc::channel();

        let midi_device_select = MidiDeviceSelect::new();
        state.time_menager.start_timer();

        Self {
            aysnc_job: async_job::Job::new(receiver, sender),
            bg_pipeline: BgPipeline::new(&gpu),
            midi_device_select,
            file: None,
        }
    }
}

impl Scene for MenuScene {
    fn state_type(&self) -> SceneType {
        SceneType::MainMenu
    }
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent {
        if let Some(time) = state.time_menager.timer_get_elapsed() {
            let time = time as f32 / 1000.0;

            self.bg_pipeline.update_time(gpu, time);
        }
        // Listen To Async Job Finish Event
        if self.aysnc_job.working {
            if let Ok(event) = self.aysnc_job.receiver.try_recv() {
                self.aysnc_job.working = false;

                match event {
                    async_job::Event::MidiLoaded(mut midi) => {
                        midi.merged_track.notes = midi
                            .merged_track
                            .notes
                            .into_iter()
                            .filter(|n| n.ch != 9)
                            .collect();
                        self.file = Some(midi);
                    }
                    async_job::Event::Err(e) => log::error!("Midi Load: {}", e),
                }
            }
        }

        // Select File Button
        if button::queue(
            state,
            ui,
            "Select File",
            (state.window_size.0 / 2.0, state.window_size.1 / 2.0 - 100.0),
            (500.0, 100.0),
            false,
        ) {
            let tx = self.aysnc_job.sender.clone();

            self.aysnc_job.working = true;
            thread::spawn(move || {
                let path = tinyfiledialogs::open_file_dialog("Select Midi", "", None);

                if let Some(path) = path {
                    let midi = lib_midi::Midi::new(&path);

                    if let Ok(midi) = midi {
                        tx.send(async_job::Event::MidiLoaded(midi))
                            .expect("tx send failed in midi loader");
                    } else if let Err(e) = midi {
                        tx.send(async_job::Event::Err(e))
                            .expect("tx send failed in midi loader");
                    }
                } else {
                    tx.send(async_job::Event::Err("File dialog returned None".into()))
                        .expect("tx send failed in midi loader");
                }
            });
        }

        self.midi_device_select.queue(ui, &state);

        if self.file.is_some()
            && button::queue(
                state,
                ui,
                "Play",
                (
                    state.window_size.0 - 250.0 / 2.0 - 10.0,
                    state.window_size.1 - 80.0 / 2.0 - 10.0,
                ),
                (250.0, 80.0),
                false,
            )
        {
            let file = std::mem::replace(&mut self.file, None);
            let select = std::mem::replace(&mut self.midi_device_select, MidiDeviceSelect::new());
            return SceneEvent::MainMenu(Event::MidiOpen(file.unwrap(), select.get_selected()));
        }

        SceneEvent::None
    }
    fn render(&mut self, _state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
        let encoder = &mut gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.bg_pipeline.render(&mut render_pass);
        }
    }
}

struct MidiDeviceSelect {
    midi_device_menager: MidiDevicesMenager,
    midi_outs: Vec<MidiPortInfo>,
    pub selected_id: usize,
}
impl MidiDeviceSelect {
    fn new() -> Self {
        let midi_device_menager = MidiDevicesMenager::new();
        Self {
            midi_outs: midi_device_menager.get_outs(),
            midi_device_menager,
            selected_id: 0,
        }
    }
    fn get_selected(mut self) -> MidiPortInfo {
        self.midi_outs.remove(self.selected_id)
    }
    fn next(&mut self) {
        self.selected_id += 1;
    }
    fn prev(&mut self) {
        self.selected_id -= 1;
    }
    fn update_outs_list(&mut self) {
        self.midi_outs = self.midi_device_menager.get_outs();
        log::info!("{:?}", self.midi_outs);
    }
    fn queue(&mut self, ui: &mut Ui, state: &MainState) {
        self.update_outs_list();

        let text = if self.midi_outs.len() > self.selected_id {
            &self.midi_outs[self.selected_id].name
        } else {
            "No Midi Devices"
        };

        ui.queue_text(wgpu_glyph::Section {
            text,
            color: [1.0, 1.0, 1.0, 1.0],
            screen_position: (state.window_size.0 / 2.0, state.window_size.1 / 2.0 + 25.0),
            scale: wgpu_glyph::Scale::uniform(40.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Center,
                v_align: wgpu_glyph::VerticalAlign::Center,
            },
            ..Default::default()
        });

        if button::queue(
            state,
            ui,
            "<",
            (
                state.window_size.0 / 2.0 - 250.0 / 2.0,
                state.window_size.1 / 2.0 + 100.0,
            ),
            (250.0, 50.0),
            self.selected_id == 0,
        ) {
            self.prev();
        }

        if button::queue(
            state,
            ui,
            ">",
            (
                state.window_size.0 / 2.0 + 250.0 / 2.01,
                state.window_size.1 / 2.0 + 100.0,
            ),
            (250.0, 50.0),
            self.selected_id + 1 >= self.midi_outs.len(),
        ) {
            self.next();
        }
    }
}

mod async_job {
    use std::sync::mpsc::{Receiver, Sender};

    pub enum Event {
        MidiLoaded(lib_midi::Midi),
        Err(String),
    }

    pub struct Job<Event> {
        pub receiver: Receiver<Event>,
        pub sender: Sender<Event>,

        pub working: bool,
    }
    impl Job<Event> {
        pub fn new(receiver: Receiver<Event>, sender: Sender<Event>) -> Self {
            Self {
                receiver,
                sender,
                working: false,
            }
        }
    }
}

mod button {
    use crate::ui::ButtonInstance;
    pub fn queue(
        state: &super::MainState,
        ui: &mut super::Ui,
        text: &str,
        pos: (f32, f32),
        size: (f32, f32),
        disabled: bool,
    ) -> bool {
        let (x, y) = pos;
        let (w, h) = size;

        let coll_x = x - w / 2.0;
        let coll_y = y - h / 2.0;

        let is_hover = state.mouse_pos.0 > coll_x
            && state.mouse_pos.0 < coll_x + w
            && state.mouse_pos.1 > coll_y
            && state.mouse_pos.1 < coll_y + h;

        let color = if is_hover {
            // [121.0 / 255.0, 85.0 / 255.0, 195.0 / 255.0]
            [56.0 / 255.0, 145.0 / 255.0, 1.0]
        } else {
            // [111.0 / 255.0, 75.0 / 255.0, 185.0 / 255.0]
            [160.0 / 255.0, 81.0 / 255.0, 232_558.0 / 255.0]
        };

        ui.queue_rectangle(ButtonInstance {
            position: [x, y],
            size: [w, h],
            color,
            radius: 15.0,
            is_hovered: if is_hover { 1 } else { 0 },
        });

        ui.queue_text(wgpu_glyph::Section {
            text,
            color: if !disabled {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.3, 0.3, 0.3, 1.0]
            },
            screen_position: (x, y),
            scale: wgpu_glyph::Scale::uniform(40.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Center,
                v_align: wgpu_glyph::VerticalAlign::Center,
            },
            ..Default::default()
        });

        is_hover && state.mouse_clicked && !disabled
    }
}
