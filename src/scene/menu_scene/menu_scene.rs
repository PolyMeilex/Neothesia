use crate::{
    scene::{Scene, SceneEvent, SceneType},
    ui::Ui,
    wgpu_jumpstart::gpu::Gpu,
    MainState,
};

use std::sync::mpsc;
use std::thread;

pub enum Event {
    MidiOpen(lib_midi::Midi),
}

pub struct MenuScene {
    aysnc_job: async_job::Job<async_job::Event>,
}

impl MenuScene {
    pub fn new(_gpu: &mut Gpu) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            aysnc_job: async_job::Job::new(receiver, sender),
        }
    }
}

impl Scene for MenuScene {
    fn state_type(&self) -> SceneType {
        SceneType::MainMenu
    }
    fn update(&mut self, state: &mut MainState, _gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent {
        // Listen To Async Job Finish Event
        if self.aysnc_job.working {
            if let Ok(event) = self.aysnc_job.receiver.try_recv() {
                self.aysnc_job.working = false;

                match event {
                    async_job::Event::MidiLoaded(midi) => {
                        return SceneEvent::MainMenu(Event::MidiOpen(midi));
                    }
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
                let midi = lib_midi::Midi::new("./test.mid").unwrap();
                tx.send(async_job::Event::MidiLoaded(midi))
                    .expect("tx send failed in midi loader");
            });
        }

        SceneEvent::None
    }
    fn render(&mut self, _state: &mut MainState, _gpu: &mut Gpu, _frame: &wgpu::SwapChainOutput) {}
}

mod async_job {
    use std::sync::mpsc::{Receiver, Sender};

    pub enum Event {
        MidiLoaded(lib_midi::Midi),
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
    use crate::ui::QuadInstance;
    pub fn queue(
        state: &super::MainState,
        ui: &mut super::Ui,
        text: &str,
        pos: (f32, f32),
        size: (f32, f32),
    ) -> bool {
        let (x, y) = pos;
        let (w, h) = size;

        let coll_x = x - 500.0 / 2.0;
        let coll_y = y - 100.0 / 2.0;

        let is_hover = state.mouse_pos.0 > coll_x
            && state.mouse_pos.0 < coll_x + w
            && state.mouse_pos.1 > coll_y
            && state.mouse_pos.1 < coll_y + h;

        let color = if is_hover {
            [121.0 / 255.0, 85.0 / 255.0, 195.0 / 255.0]
        } else {
            [111.0 / 255.0, 75.0 / 255.0, 185.0 / 255.0]
        };

        ui.queue_rectangle(QuadInstance {
            position: [x, y],
            size: [w, h],
            color,
            radius: 15.0,
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
