use crate::{
    scene::{Scene, SceneEvent, SceneType},
    ui::Ui,
    wgpu_jumpstart::gpu::Gpu,
    MainState,
};

use super::bg_pipeline::BgPipeline;
use std::sync::mpsc;
use std::thread;

pub enum Event {
    MidiOpen(lib_midi::Midi),
}

pub struct MenuScene {
    aysnc_job: async_job::Job<async_job::Event>,
    bg_pipeline: BgPipeline,
}

impl MenuScene {
    pub fn new(gpu: &mut Gpu, state: &mut MainState) -> Self {
        let (sender, receiver) = mpsc::channel();

        state.time_menager.start_timer();

        Self {
            aysnc_job: async_job::Job::new(receiver, sender),
            bg_pipeline: BgPipeline::new(&gpu.device),
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

            self.bg_pipeline
                .update_time(&mut gpu.encoder, &gpu.device, time);
        }
        // Listen To Async Job Finish Event
        if self.aysnc_job.working {
            if let Ok(event) = self.aysnc_job.receiver.try_recv() {
                self.aysnc_job.working = false;

                match event {
                    async_job::Event::MidiLoaded(midi) => {
                        return SceneEvent::MainMenu(Event::MidiOpen(midi));
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
                        tx.send(async_job::Event::Err(format!("{}", e)))
                            .expect("tx send failed in midi loader");
                    }
                } else {
                    tx.send(async_job::Event::Err("File dialog returned None".into()))
                        .expect("tx send failed in midi loader");
                }
            });
        }

        SceneEvent::None
    }
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
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
            [56.0 / 255.0, 145.0 / 255.0, 255.0 / 255.0]
        } else {
            // [111.0 / 255.0, 75.0 / 255.0, 185.0 / 255.0]
            [160.0 / 255.0, 81.0 / 255.0, 232558.0 / 255.0]
        };

        ui.queue_rectangle(ButtonInstance {
            position: [x, y],
            size: [w, h],
            color,
            radius: 15.0,
            is_hovered: if is_hover { 1 } else { 0 },
        });

        ui.queue_text(wgpu_glyph::Section {
            text: text,
            color: [1.0, 1.0, 1.0, 1.0],
            screen_position: (x, y),
            scale: wgpu_glyph::Scale::uniform(40.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Center,
                v_align: wgpu_glyph::VerticalAlign::Center,
            },
            ..Default::default()
        });

        if is_hover && state.mouse_clicked {
            true
        } else {
            false
        }
    }
}
