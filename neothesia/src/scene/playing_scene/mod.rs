use midi_file::{MidiNote, midly::MidiMessage};
use neothesia_core::{
    fingering::{Fingering, NoteKey},
    render::{
        FingeringLabels, GlowRenderer, GuidelineRenderer, NoteLabels, QuadInstance, QuadRenderer,
        TextRenderer,
    },
};
use std::time::Duration;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};

use self::top_bar::TopBar;

use super::{NuonRenderer, Scene};
use crate::{
    NeothesiaEvent, context::Context, locals, render::WaterfallRenderer,
    scene::MouseToMidiEventState, song::Song, utils::window::WinitEvent,
};

mod keyboard;
pub use keyboard::Keyboard;

pub(crate) mod midi_player;
use midi_player::MidiPlayer;

mod rewind_controller;
use rewind_controller::RewindController;

mod toast_manager;
use toast_manager::ToastManager;

mod animation;
mod top_bar;

pub struct PlayingScene {
    keyboard: Keyboard,
    waterfall: WaterfallRenderer,
    guidelines: GuidelineRenderer,
    text_renderer: TextRenderer,
    nuon_renderer: NuonRenderer,

    note_labels: Option<NoteLabels>,
    fingering_labels: Option<FingeringLabels>,
    /// Note picked for fingering edit
    selected_note: Option<MidiNote>,
    /// Time the waterfall was last rendered at
    waterfall_time: f32,

    player: MidiPlayer,
    rewind_controller: RewindController,
    quad_renderer_bg: QuadRenderer,
    quad_renderer_fg: QuadRenderer,
    glow: Option<GlowRenderer>,
    toast_manager: ToastManager,

    nuon: nuon::Ui,
    mouse_to_midi_state: MouseToMidiEventState,

    deduced_chord_name: String,

    top_bar: TopBar,
}

impl PlayingScene {
    pub fn new(ctx: &mut Context, song: Song) -> Self {
        let keyboard = Keyboard::new(ctx, song.config.clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines(),
            ctx.config.horizontal_guidelines(),
            song.file.measures.clone(),
        );

        let hidden_tracks: Vec<usize> = song
            .config
            .tracks
            .iter()
            .filter(|t| !t.visible)
            .map(|t| t.track_id)
            .collect();

        let mut waterfall = WaterfallRenderer::new(
            &ctx.gpu,
            &song.file.tracks,
            &hidden_tracks,
            &ctx.config,
            &ctx.transform,
            keyboard_layout.clone(),
        );

        let text_renderer = ctx.text_renderer_factory.new_renderer();

        let note_labels = ctx.config.note_labels().then_some(NoteLabels::new(
            *keyboard.pos(),
            waterfall.notes(),
            ctx.config.waterfall_note_naming(),
            ctx.text_renderer_factory.new_renderer(),
        ));

        let fingering_labels = ctx.config.fingering().then(|| {
            FingeringLabels::new(
                *keyboard.pos(),
                waterfall.notes(),
                Fingering::compute(&song.file.tracks, &song.fingering_parts(), &song.fingering),
                ctx.config.fingering_on_keys(),
                ctx.config.note_labels(),
                ctx.text_renderer_factory.new_renderer(),
                ctx.text_renderer_factory.new_renderer(),
            )
        });

        let player = MidiPlayer::new(
            ctx.output_manager.connection().clone(),
            song,
            keyboard_layout.range.clone(),
            ctx.config.separate_channels(),
        );
        waterfall.update(player.time_without_lead_in());

        let quad_renderer_bg = ctx.quad_renderer_factory.new_renderer();
        let quad_renderer_fg = ctx.quad_renderer_factory.new_renderer();

        let glow = ctx.config.glow().then_some(GlowRenderer::new(
            &ctx.gpu,
            &ctx.transform,
            keyboard.layout(),
        ));

        Self {
            keyboard,
            guidelines,
            note_labels,
            fingering_labels,
            selected_note: None,
            waterfall_time: 0.0,
            text_renderer,
            nuon_renderer: NuonRenderer::new(ctx),

            waterfall,
            player,
            rewind_controller: RewindController::new(),
            quad_renderer_bg,
            quad_renderer_fg,
            glow,
            toast_manager: ToastManager::default(),

            nuon: nuon::Ui::new(),
            mouse_to_midi_state: MouseToMidiEventState::default(),
            deduced_chord_name: String::new(),

            top_bar: TopBar::new(),
        }
    }

    fn update_glow(&mut self, delta: Duration) {
        let Some(glow) = &mut self.glow else {
            return;
        };

        glow.clear();

        let keys = &self.keyboard.layout().keys;
        let states = self.keyboard.key_states();

        for (key, state) in keys.iter().zip(states) {
            let Some(color) = state.pressed_by_file() else {
                continue;
            };

            glow.push(
                key.id(),
                *color,
                key.x(),
                self.keyboard.pos().y,
                key.width(),
                delta,
            );
        }
    }

    fn update_chord_identifier(&mut self, enabled: bool) {
        if !enabled {
            return;
        }

        let start = self.keyboard.layout().range.start();
        let notes = self
            .keyboard
            .key_states()
            .iter()
            .enumerate()
            .filter(|(_, state)| state.pressed_by_user().is_some())
            .map(|(id, _)| id as u8 + start)
            .collect::<Vec<_>>();

        self.deduced_chord_name = super::freeplay::chords::deduce_name(&notes).unwrap_or_default();
    }

    #[profiling::function]
    fn update_midi_player(&mut self, ctx: &Context, delta: Duration) -> f32 {
        if self.top_bar.is_looper_active() && self.player.time() > self.top_bar.loop_end_timestamp()
        {
            self.player.set_time(self.top_bar.loop_start_timestamp());
            self.keyboard.reset_notes();
        }

        if self.player.play_along().are_required_keys_pressed() {
            let delta = (delta / 10) * (ctx.config.speed_multiplier() * 10.0) as u32;
            let midi_events = self.player.update(delta);
            self.keyboard.file_midi_events(&ctx.config, &midi_events);
        }

        self.player.time_without_lead_in() + ctx.config.animation_offset()
    }

    #[profiling::function]
    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);

        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.set_pos(*self.keyboard.pos());
        }
        if let Some(fingering_labels) = self.fingering_labels.as_mut() {
            fingering_labels.set_pos(*self.keyboard.pos());
        }

        self.waterfall
            .resize(&ctx.config, self.keyboard.layout().clone());
    }
}

impl PlayingScene {
    /// Drops the selection once playback continues and draws the selection outline
    fn update_fingering_selection(&mut self, ctx: &Context) {
        if !self.player.is_paused() {
            self.selected_note = None;
        }

        let Some(note) = &self.selected_note else {
            return;
        };

        let Some(rect) = self.note_rect(ctx, note) else {
            return;
        };

        self.quad_renderer_fg.push(QuadInstance {
            position: [rect.origin.x - 2.0, rect.origin.y - 2.0],
            size: [rect.width() + 4.0, rect.height() + 4.0],
            color: [0.0; 4],
            border_radius: [5.0; 4],
            border_color: [1.0, 0.8, 0.24, 1.0],
            border_width: 2.0,
        });
    }

    /// Logical rect of a falling note, same math as the waterfall shader
    fn note_rect(&self, ctx: &Context, note: &MidiNote) -> Option<nuon::Rect> {
        let layout = self.keyboard.layout();
        if !layout.range.contains(note.note) {
            return None;
        }

        let key = &layout.keys[(note.note - layout.range.start()) as usize];
        let speed = ctx.config.animation_speed() / ctx.window_state.scale_factor as f32;

        let bottom =
            self.keyboard.pos().y - (note.start.as_secs_f32() - self.waterfall_time) * speed;
        let height = note.duration.as_secs_f32() * speed;

        Some(nuon::Rect::new(
            (key.x(), bottom - height).into(),
            (key.width(), height).into(),
        ))
    }

    fn note_at_cursor(&self, ctx: &Context) -> Option<MidiNote> {
        let fingering = self.fingering_labels.as_ref()?.fingering();
        let pos = nuon::Point::new(
            ctx.window_state.cursor_logical_position.x,
            ctx.window_state.cursor_logical_position.y,
        );

        if pos.y >= self.keyboard.pos().y {
            return None;
        }

        self.waterfall
            .notes()
            .iter()
            .filter(|note| fingering.get(note).is_some())
            .filter_map(|note| Some((note, self.note_rect(ctx, note)?)))
            .filter(|(_, rect)| rect.contains(pos))
            // Sharp notes are narrower and drawn over neutral ones
            .min_by(|(_, a), (_, b)| a.width().total_cmp(&b.width()))
            .map(|(note, _)| note.clone())
    }

    /// Returns true when the event was consumed
    fn handle_fingering_input(&mut self, ctx: &mut Context, event: &WindowEvent) -> bool {
        if self.fingering_labels.is_none() {
            return false;
        }

        // Don't steal clicks from the top bar
        if event.left_mouse_pressed()
            && self.player.is_paused()
            && ctx.window_state.cursor_logical_position.y > 75.0
        {
            self.selected_note = self.note_at_cursor(ctx);
            if self.selected_note.is_some() {
                self.toast_manager.toast(locals::playing::fingering_hint());
            }
            return false;
        }

        let Some(note) = self.selected_note.clone() else {
            return false;
        };

        let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(code),
                    ..
                },
            ..
        } = event
        else {
            return false;
        };

        let finger = match code {
            KeyCode::Digit1 | KeyCode::Numpad1 => Some(1),
            KeyCode::Digit2 | KeyCode::Numpad2 => Some(2),
            KeyCode::Digit3 | KeyCode::Numpad3 => Some(3),
            KeyCode::Digit4 | KeyCode::Numpad4 => Some(4),
            KeyCode::Digit5 | KeyCode::Numpad5 => Some(5),
            KeyCode::Digit0 | KeyCode::Numpad0 | KeyCode::Backspace | KeyCode::Delete => None,
            KeyCode::Escape => {
                if *state == ElementState::Released {
                    self.selected_note = None;
                }
                return true;
            }
            _ => return false,
        };

        // Swallow the release as well, so the key does not reach pc-keyboard-to-midi
        if *state == ElementState::Pressed {
            self.set_finger(&note, finger);
        }

        true
    }

    fn set_finger(&mut self, note: &MidiNote, finger: Option<u8>) {
        let song = self.player.song_mut();
        song.fingering.set_override(NoteKey::new(note), finger);
        song.fingering.save();

        let fingering =
            Fingering::compute(&song.file.tracks, &song.fingering_parts(), &song.fingering);

        if let Some(labels) = self.fingering_labels.as_mut() {
            labels.set_fingering(fingering);
        }

        self.toast_manager.toast(match finger {
            Some(finger) => locals::playing::finger_set(finger),
            None => locals::playing::finger_auto(),
        });
    }
}

impl Scene for PlayingScene {
    #[profiling::function]
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

        self.rewind_controller.update(&mut self.player, ctx, delta);
        self.toast_manager.update(&mut self.text_renderer);

        let time = self.update_midi_player(ctx, delta);
        self.waterfall_time = time;
        self.waterfall.update(time);
        self.guidelines.update(
            &mut self.quad_renderer_bg,
            ctx.config.animation_speed(),
            ctx.window_state.scale_factor as f32,
            time,
            ctx.window_state.logical_size,
        );
        self.keyboard
            .update(&mut self.quad_renderer_fg, &mut self.text_renderer);
        self.update_chord_identifier(ctx.config.chord_identifier());
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.update(
                ctx.window_state.physical_size,
                ctx.window_state.scale_factor as f32,
                self.keyboard.renderer(),
                ctx.config.animation_speed(),
                time,
            );
        }
        if let Some(fingering_labels) = self.fingering_labels.as_mut() {
            fingering_labels.update(
                ctx.window_state.physical_size,
                ctx.window_state.scale_factor as f32,
                self.keyboard.renderer(),
                ctx.config.animation_speed(),
                time,
            );
        }
        self.update_fingering_selection(ctx);

        self.update_glow(delta);

        TopBar::update(self, ctx);

        if ctx.config.chord_identifier() {
            nuon::label()
                .text(&self.deduced_chord_name)
                .font_size(25.0)
                .y(self.keyboard.pos().y - 35.0)
                .height(25.0)
                .width(ctx.window_state.logical_size.width)
                .build(&mut self.nuon);
        }

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);

        self.quad_renderer_bg.prepare();
        self.quad_renderer_fg.prepare();

        if let Some(glow) = &mut self.glow {
            glow.prepare();
        }

        #[cfg(debug_assertions)]
        self.text_renderer.queue_fps(
            ctx.fps_ticker.avg(),
            self.top_bar
                .topbar_expand_animation
                .animate_bool(5.0, 80.0, ctx.frame_timestamp),
        );
        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );

        if self.player.is_finished() && !self.player.is_paused() {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(Some(self.player.song().clone())))
                .ok();
        }
    }

    #[profiling::function]
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quad_renderer_bg.render(rpass);
        self.waterfall.render(rpass);
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.render(rpass);
        }
        if let Some(fingering_labels) = &self.fingering_labels {
            fingering_labels.render_notes(rpass);
        }
        self.quad_renderer_fg.render(rpass);
        if let Some(glow) = &self.glow {
            glow.render(rpass);
        }
        if let Some(fingering_labels) = &self.fingering_labels {
            fingering_labels.render_keys(rpass);
        }
        self.text_renderer.render(rpass);

        self.nuon_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        if self.handle_fingering_input(ctx, event) {
            return;
        }

        self.rewind_controller
            .handle_window_event(ctx, event, &mut self.player);

        if self.rewind_controller.is_rewinding() {
            self.keyboard.reset_notes();
        }

        if event.back_mouse_pressed() || event.key_released(Key::Named(NamedKey::Escape)) {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(Some(self.player.song().clone())))
                .ok();
        }

        if event.key_released(Key::Named(NamedKey::Space)) {
            self.player.pause_resume();
        }

        handle_settings_input(ctx, &mut self.toast_manager, &mut self.waterfall, event);
        super::handle_pc_keyboard_to_midi_event(ctx, event);
        super::handle_mouse_to_midi_event(
            &mut self.keyboard,
            &mut self.mouse_to_midi_state,
            ctx,
            event,
        );

        if event.window_resized() || event.scale_factor_changed() {
            self.resize(ctx)
        }

        super::handle_nuon_window_event(&mut self.nuon, event, ctx);
    }

    fn midi_event(&mut self, _ctx: &mut Context, channel: u8, message: &MidiMessage) {
        self.player.user_midi_event(channel, message);
        self.keyboard.user_midi_event(message);
    }
}

fn handle_settings_input(
    ctx: &mut Context,
    toast_manager: &mut ToastManager,
    waterfall: &mut WaterfallRenderer,
    event: &WindowEvent,
) {
    if event.key_released(Key::Named(NamedKey::ArrowUp))
        || event.key_released(Key::Named(NamedKey::ArrowDown))
    {
        let amount = if ctx.window_state.modifiers_state.shift_key() {
            0.5
        } else {
            0.1
        };

        if event.key_released(Key::Named(NamedKey::ArrowUp)) {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() + amount);
        } else {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() - amount);
        }

        toast_manager.speed_toast(ctx.config.speed_multiplier());
        return;
    }

    if event.key_released(Key::Named(NamedKey::PageUp))
        || event.key_released(Key::Named(NamedKey::PageDown))
    {
        let amount = if ctx.window_state.modifiers_state.shift_key() {
            500.0
        } else {
            100.0
        };

        if event.key_released(Key::Named(NamedKey::PageUp)) {
            ctx.config
                .set_animation_speed(ctx.config.animation_speed() + amount);
        } else {
            ctx.config
                .set_animation_speed(ctx.config.animation_speed() - amount);
        }

        waterfall
            .pipeline()
            .set_speed(&ctx.gpu.queue, ctx.config.animation_speed());
        toast_manager.animation_speed_toast(ctx.config.animation_speed());
        return;
    }

    if let Some(ch @ ("_" | "-" | "+" | "=")) = event.character_released() {
        let amount = if ctx.window_state.modifiers_state.shift_key() {
            0.1
        } else {
            0.01
        };

        if matches!(ch, "-" | "_") {
            ctx.config
                .set_animation_offset(ctx.config.animation_offset() - amount);
        } else {
            ctx.config
                .set_animation_offset(ctx.config.animation_offset() + amount);
        }

        toast_manager.offset_toast(ctx.config.animation_offset());
    }
}
