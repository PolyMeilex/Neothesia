pub mod freeplay;
pub mod menu_scene;
pub mod playing_scene;

use crate::{
    NeothesiaEvent, context::Context, scene::playing_scene::Keyboard, utils::window::WinitEvent,
};
use midi_file::midly::MidiMessage;
use neothesia_core::render::{Image, ImageIdentifier, ImageRenderer, QuadRenderer, TextRenderer};
use std::{collections::HashMap, time::Duration};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, delta: Duration);
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>);
    fn window_event(&mut self, _ctx: &mut Context, _event: &WindowEvent) {}
    fn midi_event(&mut self, _ctx: &mut Context, _channel: u8, _message: &MidiMessage) {}
}

pub fn handle_pc_keyboard_to_midi_event(ctx: &mut Context, event: &WindowEvent) {
    let WindowEvent::KeyboardInput {
        event:
            KeyEvent {
                state,
                physical_key: PhysicalKey::Code(key_code),
                repeat: false,
                ..
            },
        ..
    } = event
    else {
        return;
    };

    if *state == ElementState::Pressed {
        match key_code {
            KeyCode::BracketLeft => ctx.config.pc_keyboard_shift_octave_down(),
            KeyCode::BracketRight => ctx.config.pc_keyboard_shift_octave_up(),
            _ => {}
        }
    }

    let mut note = match key_code {
        KeyCode::KeyZ => 0,
        KeyCode::KeyS => 1,
        KeyCode::KeyX => 2,
        KeyCode::KeyD => 3,
        KeyCode::KeyC => 4,
        KeyCode::KeyV => 5,
        KeyCode::KeyG => 6,
        KeyCode::KeyB => 7,
        KeyCode::KeyH => 8,
        KeyCode::KeyN => 9,
        KeyCode::KeyJ => 10,
        KeyCode::KeyM => 11,
        KeyCode::KeyQ => 12,
        KeyCode::Digit2 => 13,
        KeyCode::KeyW => 14,
        KeyCode::Digit3 => 15,
        KeyCode::KeyE => 16,
        KeyCode::KeyR => 17,
        KeyCode::Digit5 => 18,
        KeyCode::KeyT => 19,
        KeyCode::Digit6 => 20,
        KeyCode::KeyY => 21,
        KeyCode::Digit7 => 22,
        KeyCode::KeyU => 23,
        KeyCode::KeyI => 24,
        KeyCode::Digit9 => 25,
        KeyCode::KeyO => 26,
        KeyCode::Digit0 => 27,
        KeyCode::KeyP => 28,
        _ => return,
    };

    note += 12 * ctx.config.pc_keyboard_octave();
    if note > 127 {
        return;
    }

    let message = match state {
        ElementState::Pressed => MidiMessage::NoteOn {
            key: note.into(),
            vel: 100.into(),
        },
        ElementState::Released => MidiMessage::NoteOff {
            key: note.into(),
            vel: 0.into(),
        },
    };
    ctx.proxy
        .send_event(NeothesiaEvent::MidiInput {
            channel: 0,
            message,
        })
        .ok();
}

#[derive(Default, Debug)]
struct MouseToMidiEventState {
    mouse_key_press: Option<u8>,
}

fn handle_mouse_to_midi_event(
    keyboard: &mut Keyboard,
    state: &mut MouseToMidiEventState,
    ctx: &Context,
    event: &WindowEvent,
) {
    if !(event.left_mouse_pressed() || event.left_mouse_released() || event.cursor_moved()) {
        return;
    }

    fn cancel_mouse_key_press(state: &mut MouseToMidiEventState, ctx: &Context) {
        let Some(key) = state.mouse_key_press else {
            return;
        };

        state.mouse_key_press = None;

        let message = MidiMessage::NoteOff {
            key: key.into(),
            vel: 0.into(),
        };
        ctx.proxy
            .send_event(NeothesiaEvent::MidiInput {
                channel: 0,
                message,
            })
            .ok();
    }

    let bbox = nuon::Rect::new(
        (keyboard.pos().x, keyboard.pos().y).into(),
        (keyboard.layout().width, keyboard.layout().height).into(),
    );
    let mouse_pos = nuon::Point::new(
        ctx.window_state.cursor_logical_position.x,
        ctx.window_state.cursor_logical_position.y,
    );

    if !bbox.contains(mouse_pos) || !ctx.window_state.left_mouse_btn {
        cancel_mouse_key_press(state, ctx);
        return;
    }

    let sharp = keyboard
        .layout()
        .keys
        .iter()
        .filter(|key| key.kind().is_sharp());
    let neutral = keyboard
        .layout()
        .keys
        .iter()
        .filter(|key| key.kind().is_neutral());

    for key in sharp.chain(neutral) {
        let pos = nuon::Point::new(key.x(), keyboard.pos().y);
        let size = nuon::Size::from(key.size());
        let rect = nuon::Rect::new(pos, size);
        if !rect.contains(mouse_pos) {
            continue;
        }

        let key = keyboard.layout().range.start() + key.id() as u8;

        if Some(key) == state.mouse_key_press {
            return;
        }

        cancel_mouse_key_press(state, ctx);
        state.mouse_key_press = Some(key);

        let message = MidiMessage::NoteOn {
            key: key.into(),
            vel: 100.into(),
        };
        ctx.proxy
            .send_event(NeothesiaEvent::MidiInput {
                channel: 0,
                message,
            })
            .ok();
        return;
    }
}

struct NuonLayer {
    quad_renderer: QuadRenderer,
    text_renderer: TextRenderer,
    images: Vec<Image>,
}

pub struct NuonRenderer {
    layers: Vec<NuonLayer>,
    image_map: HashMap<ImageIdentifier, Image>,
    image_renderer: ImageRenderer,
}

impl NuonRenderer {
    pub fn new(ctx: &Context) -> Self {
        Self {
            layers: Vec::new(),
            image_map: HashMap::new(),
            image_renderer: ImageRenderer::new(
                &ctx.gpu.device,
                ctx.gpu.texture_format,
                &ctx.transform,
            ),
        }
    }

    fn ensure_layers(&mut self, ctx: &mut Context, len: usize) {
        self.layers.resize_with(len, || NuonLayer {
            quad_renderer: ctx.quad_renderer_factory.new_renderer(),
            text_renderer: ctx.text_renderer_factory.new_renderer(),
            images: Vec::new(),
        });
    }

    pub fn add_image(&mut self, image: Image) -> ImageIdentifier {
        let ident = image.identifier();
        self.image_map.insert(ident, image);
        ident
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu_jumpstart::RenderPass<'rpass>) {
        for layer in self.layers.iter() {
            layer.quad_renderer.render(rpass);
            layer.text_renderer.render(rpass);
            for image in layer.images.iter() {
                self.image_renderer.render(rpass, image);
            }
        }
    }
}

fn handle_nuon_window_event(nuon: &mut nuon::Ui, event: &WindowEvent, ctx: &Context) {
    if event.cursor_moved() {
        nuon.mouse_move(
            ctx.window_state.cursor_logical_position.x,
            ctx.window_state.cursor_logical_position.y,
        );
    } else if event.left_mouse_pressed() {
        nuon.mouse_down();
    } else if event.left_mouse_released() {
        nuon.mouse_up();
    }
}

fn render_nuon(ui: &mut nuon::Ui, nuon_renderer: &mut NuonRenderer, ctx: &mut Context) {
    nuon_renderer.ensure_layers(ctx, ui.layers.len());

    for (layer, out) in ui.layers.iter().zip(nuon_renderer.layers.iter_mut()) {
        out.quad_renderer.clear();
        out.images.clear();

        let scissor_rect = layer.scissor_rect;
        let pos = LogicalPosition::new(scissor_rect.origin.x, scissor_rect.origin.y)
            .to_physical::<u32>(ctx.window_state.scale_factor);
        let size = LogicalSize::new(scissor_rect.width(), scissor_rect.height())
            .to_physical::<u32>(ctx.window_state.scale_factor);
        let scissor_rect =
            neothesia_core::Rect::new((pos.x, pos.y).into(), (size.width, size.height).into());

        out.quad_renderer.set_scissor_rect(scissor_rect);
        out.text_renderer.set_scissor_rect(scissor_rect);

        for quad in layer.quads.iter() {
            out.quad_renderer
                .push(neothesia_core::render::QuadInstance {
                    position: quad.rect.origin.into(),
                    size: quad.rect.size.into(),
                    color: wgpu_jumpstart::Color::new(
                        quad.color.r,
                        quad.color.g,
                        quad.color.b,
                        quad.color.a,
                    )
                    .into_linear_rgba(),
                    border_radius: quad.border_radius,
                });
        }

        for img in layer.images.iter() {
            if let Some(image) = nuon_renderer.image_map.get_mut(&img.image) {
                image.set_rect(img.rect, img.border_radius);
                out.images.push(image.clone());
            }
        }

        for icon in layer.icons.iter() {
            out.text_renderer.queue_icon(
                icon.origin.x,
                icon.origin.y,
                icon.size,
                &icon.icon,
                cosmic_text::Color(icon.color.packet_u32()),
            );
        }

        for text in layer.text.iter() {
            let buffer = if text.bold {
                TextRenderer::gen_buffer_with_attr(
                    text.size,
                    &text.text,
                    cosmic_text::Attrs::new()
                        .family(cosmic_text::Family::Name(&text.font_family))
                        .weight(cosmic_text::Weight::BOLD)
                        .color(cosmic_text::Color(text.color.packet_u32())),
                )
            } else {
                TextRenderer::gen_buffer_with_attr(
                    text.size,
                    &text.text,
                    cosmic_text::Attrs::new()
                        .family(cosmic_text::Family::Name(&text.font_family))
                        .color(cosmic_text::Color(text.color.packet_u32())),
                )
            };

            match text.text_justify {
                nuon::TextJustify::Left => {
                    out.text_renderer.queue_buffer_left(text.rect, buffer);
                }
                nuon::TextJustify::Right => {
                    out.text_renderer.queue_buffer_right(text.rect, buffer);
                }
                nuon::TextJustify::Center => {
                    out.text_renderer.queue_buffer_centered(text.rect, buffer);
                }
            }
        }

        out.quad_renderer.prepare();
        out.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );
    }

    ui.done();
}
