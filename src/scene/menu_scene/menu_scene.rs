use crate::{
    midi_device::{MidiDevicesMenager, MidiPortInfo},
    scene::{InputEvent, Scene, SceneEvent, SceneType},
    time_menager::Timer,
    ui::{ButtonInstance, Ui},
    wgpu_jumpstart::{Color, Gpu},
    MainState,
};

use super::bg_pipeline::BgPipeline;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use winit::event::{MouseButton, VirtualKeyCode};

#[derive(Debug)]
pub enum Event {
    MidiOpen(MidiPortInfo),
}

pub struct MenuScene<'a> {
    aysnc_job: async_job::Job<async_job::Event>,
    bg_pipeline: BgPipeline,
    midi_device_select: MidiDeviceSelect<'a>,
    timer: Timer,

    select_file_btn: Button<'a>,
    play_btn: Button<'a>,
}

impl<'a> MenuScene<'a> {
    pub fn new(state: &mut MainState, gpu: &mut Gpu) -> Self {
        let (sender, receiver) = mpsc::channel();

        let midi_device_select = MidiDeviceSelect::new(state);

        let timer = Timer::new();

        let (select_file_btn, play_btn) = Self::build_ui();

        let mut scene = Self {
            aysnc_job: async_job::Job::new(receiver, sender),
            bg_pipeline: BgPipeline::new(&gpu),
            midi_device_select,
            timer,

            select_file_btn,
            play_btn,
        };

        scene.resize(state, gpu);
        scene
    }
    fn build_ui() -> (Button<'a>, Button<'a>) {
        let select_file_btn = Button::new("Select File", |state| {
            let pos = (state.window_size.0 / 2.0, state.window_size.1 / 2.0 - 100.0);
            let size = (500.0, 100.0);
            (pos, size)
        });
        let play_btn = Button::new("Play", |state| {
            let pos = (
                state.window_size.0 - 250.0 / 2.0 - 10.0,
                state.window_size.1 - 80.0 / 2.0 - 10.0,
            );
            let size = (250.0, 80.0);
            (pos, size)
        });
        (select_file_btn, play_btn)
    }
    fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.select_file_btn.update_mouse_pos(x, y);
        self.play_btn.update_mouse_pos(x, y);

        self.midi_device_select.update_mouse_pos(x, y);
    }
    fn mouse_clicked(&mut self, state: &MainState) -> SceneEvent {
        if self.select_file_btn.check_clicked() {
            self.select_file_clicked();
        } else if self.play_btn.check_clicked() {
            let select =
                std::mem::replace(&mut self.midi_device_select, MidiDeviceSelect::new(state));
            return SceneEvent::MainMenu(Event::MidiOpen(select.get_selected()));
        } else {
            self.midi_device_select.mouse_clicked(state);
            return SceneEvent::None;
        }

        SceneEvent::None
    }
    fn select_file_clicked(&mut self) {
        let tx = self.aysnc_job.sender.clone();

        self.aysnc_job.working = true;
        thread::spawn(move || {
            use nfd2::Response;

            match nfd2::DialogBuilder::single()
                .filter("mid,midi")
                .open()
                .expect("File Dialog Error")
            {
                Response::Okay(path) => {
                    log::info!("File path = {:?}", path);
                    let midi = lib_midi::Midi::new(path.to_str().unwrap());

                    if let Ok(midi) = midi {
                        tx.send(async_job::Event::MidiLoaded(midi))
                            .expect("tx send failed in midi loader");
                    } else if let Err(e) = midi {
                        tx.send(async_job::Event::Err(e))
                            .expect("tx send failed in midi loader");
                    }
                }
                _ => {
                    log::error!("User canceled dialog");
                    tx.send(async_job::Event::Err("File dialog returned None".into()))
                        .expect("tx send failed in midi loader");
                }
            }
        });
    }
}

impl<'a> Scene for MenuScene<'a> {
    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }
    fn resize(&mut self, state: &mut MainState, _gpu: &mut Gpu) {
        self.select_file_btn.update(state);
        self.play_btn.update(state);

        self.midi_device_select.resize(state);
    }
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent {
        self.timer.update();
        let time = self.timer.get_elapsed() / 1000.0;

        self.bg_pipeline.update_time(gpu, time);
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
                        state.midi_file = Some(Rc::new(midi));
                    }
                    async_job::Event::Err(e) => log::warn!("Midi Load: {}", e),
                }
            }
        }

        // self.select_file_btn.queue(ui);

        // self.play_btn.hidden(!state.midi_file.is_some());
        // self.play_btn
        //     .disabled(self.midi_device_select.midi_outs.is_empty());
        // self.play_btn.queue(ui);

        // self.midi_device_select.queue(ui, &state);

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
    fn input_event(&mut self, state: &mut MainState, event: InputEvent) -> SceneEvent {
        match event {
            InputEvent::KeyReleased(key) => match key {
                VirtualKeyCode::Return => {
                    if state.midi_file.is_some() && !self.midi_device_select.midi_outs.is_empty() {
                        let select = std::mem::replace(
                            &mut self.midi_device_select,
                            MidiDeviceSelect::new(state),
                        );
                        SceneEvent::MainMenu(Event::MidiOpen(select.get_selected()))
                    } else {
                        SceneEvent::None
                    }
                }
                _ => SceneEvent::None,
            },
            InputEvent::MouseInput(s, button) => match button {
                MouseButton::Left => {
                    if let winit::event::ElementState::Pressed = s {
                        self.mouse_clicked(state)
                    } else {
                        SceneEvent::None
                    }
                }
                _ => SceneEvent::None,
            },
            InputEvent::CursorMoved(x, y) => {
                self.update_mouse_pos(x, y);
                SceneEvent::None
            } // _ => SceneEvent::None,
        }
    }
}

struct MidiDeviceSelect<'a> {
    midi_device_menager: MidiDevicesMenager,
    midi_outs: Vec<MidiPortInfo>,
    pub selected_id: usize,

    prev_btn: Button<'a>,
    next_btn: Button<'a>,
}
impl<'a> MidiDeviceSelect<'a> {
    fn new(state: &MainState) -> Self {
        let midi_device_menager = MidiDevicesMenager::new();

        let (prev_btn, next_btn) = Self::build_ui();

        let mut select = Self {
            midi_outs: midi_device_menager.get_outs(),
            midi_device_menager,
            selected_id: 0,

            prev_btn,
            next_btn,
        };
        select.resize(state);

        select.prev_btn.disabled(select.selected_id == 0);
        select
            .next_btn
            .disabled(select.selected_id + 1 >= select.midi_outs.len());

        select
    }
    fn build_ui() -> (Button<'a>, Button<'a>) {
        let prev_btn = Button::new("<", |state| {
            let pos = (
                state.window_size.0 / 2.0 - 250.0 / 2.0,
                state.window_size.1 / 2.0 + 100.0,
            );
            let size = (250.0, 50.0);
            (pos, size)
        });
        let next_btn = Button::new(">", |state| {
            let pos = (
                state.window_size.0 / 2.0 + 250.0 / 2.01,
                state.window_size.1 / 2.0 + 100.0,
            );
            let size = (250.0, 50.0);
            (pos, size)
        });
        (prev_btn, next_btn)
    }
    fn resize(&mut self, state: &MainState) {
        self.prev_btn.update(state);
        self.next_btn.update(state);
    }
    fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.prev_btn.update_mouse_pos(x, y);
        self.next_btn.update_mouse_pos(x, y);
    }
    fn mouse_clicked(&mut self, _state: &MainState) -> SceneEvent {
        self.prev_btn.disabled(self.selected_id == 0);
        self.next_btn
            .disabled(self.selected_id + 1 >= self.midi_outs.len());
        if self.prev_btn.check_clicked() {
            self.prev();
        } else if self.next_btn.check_clicked() {
            self.next();
        }

        SceneEvent::None
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
        log::trace!("{:?}", self.midi_outs);
    }
    fn queue(&mut self, ui: &mut Ui, state: &MainState) {
        self.update_outs_list();

        let text = if self.midi_outs.len() > self.selected_id {
            &self.midi_outs[self.selected_id].name
        } else {
            "No Midi Devices"
        };

        let text = vec![wgpu_glyph::Text::new(text)
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(30.0)];

        ui.queue_text(wgpu_glyph::Section {
            text,
            screen_position: (state.window_size.0 / 2.0, state.window_size.1 / 2.0 + 25.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Center,
                v_align: wgpu_glyph::VerticalAlign::Center,
            },
            ..Default::default()
        });

        self.prev_btn.queue(ui);
        self.next_btn.queue(ui);
    }
}

struct Button<'a> {
    text: &'a str,
    pos: [f32; 2],
    size: [f32; 2],
    disabled: bool,
    is_hovered: bool,
    is_hidden: bool,

    update_fn: Box<dyn Fn(&MainState) -> ((f32, f32), (f32, f32))>,
}
impl<'a> Button<'a> {
    fn new<F: Fn(&MainState) -> ((f32, f32), (f32, f32)) + 'static>(
        text: &'a str,
        update_fn: F,
    ) -> Self {
        Self {
            text,
            pos: [0.0, 0.0],
            size: [0.0, 0.0],
            disabled: false,
            is_hovered: false,
            is_hidden: false,

            update_fn: Box::new(update_fn),
        }
    }
    fn check_clicked(&self) -> bool {
        self.is_hovered && !self.is_hidden && !self.disabled
    }
    fn hidden(&mut self, is: bool) {
        self.is_hidden = is;
    }
    fn disabled(&mut self, is: bool) {
        self.disabled = is;
    }
    fn update(&mut self, state: &MainState) {
        let (pos, size) = (*self.update_fn)(state);
        self.pos = [pos.0, pos.1];
        self.size = [size.0, size.1];
    }
    fn update_mouse_pos(&mut self, mx: f32, my: f32) {
        let [x, y] = self.pos;
        let [w, h] = self.size;

        let coll_x = x - w / 2.0;
        let coll_y = y - h / 2.0;

        let is_hover = mx > coll_x && mx < coll_x + w && my > coll_y && my < coll_y + h;

        self.is_hovered = is_hover;
    }
    fn queue(&self, ui: &mut Ui) {
        if !self.is_hidden {
            ui.queue_button(ButtonInstance {
                position: self.pos,
                size: self.size,
                color: if self.is_hovered {
                    Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgb()
                } else {
                    Color::from_rgba8(160, 81, 255, 1.0).into_linear_rgb()
                },
                radius: 15.0,
                is_hovered: if self.is_hovered { 1 } else { 0 },
            });

            let color = if !self.disabled {
                Color::new(1.0, 1.0, 1.0, 1.0).into_linear_rgba()
            } else {
                Color::new(0.3, 0.3, 0.3, 1.0).into_linear_rgba()
            };
            let text = vec![wgpu_glyph::Text::new(self.text)
                .with_color(color)
                .with_scale(40.0)];
            ui.queue_text(wgpu_glyph::Section {
                text,
                screen_position: (self.pos[0], self.pos[1]),
                layout: wgpu_glyph::Layout::Wrap {
                    line_breaker: Default::default(),
                    h_align: wgpu_glyph::HorizontalAlign::Center,
                    v_align: wgpu_glyph::VerticalAlign::Center,
                },
                ..Default::default()
            });
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
