pub mod menu_scene;
pub mod playing_scene;

use crate::context::Context;
use iced_core::image::Renderer as _;
use iced_graphics::text::cosmic_text;
use midi_file::midly::MidiMessage;
use neothesia_core::render::{QuadRenderer, TextRenderer};
use std::time::Duration;
use winit::event::WindowEvent;

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, delta: Duration);
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>);
    fn window_event(&mut self, _ctx: &mut Context, _event: &WindowEvent) {}
    fn midi_event(&mut self, _ctx: &mut Context, _channel: u8, _message: &MidiMessage) {}
}

struct NuonLayer {
    quad_renderer: QuadRenderer,
    text_renderer: TextRenderer,
}

#[derive(Default)]
pub struct NuonRenderer {
    layers: Vec<NuonLayer>,
}

impl NuonRenderer {
    fn ensure_layers(&mut self, ctx: &mut Context, len: usize) {
        self.layers.resize_with(len, || NuonLayer {
            quad_renderer: ctx.quad_renderer_facotry.new_renderer(),
            text_renderer: ctx.text_renderer_factory.new_renderer(),
        });
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu_jumpstart::RenderPass<'rpass>) {
        for layer in self.layers.iter() {
            layer.quad_renderer.render(rpass);
            layer.text_renderer.render(rpass);
        }
    }
}

fn render_nuon(ui: &mut nuon::Ui, nuon_renderer: &mut NuonRenderer, ctx: &mut Context) {
    nuon_renderer.ensure_layers(ctx, ui.layers.len());

    let renderer = &mut ctx.iced_manager.renderer;

    for (layer, out) in ui.layers.iter().zip(nuon_renderer.layers.iter_mut()) {
        out.quad_renderer.clear();
        out.quad_renderer.set_scissor_rect(layer.scissor_rect);
        out.text_renderer.set_scissor_rect(layer.scissor_rect);

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
            renderer.draw_image(
                iced_core::Image {
                    handle: img.image.clone(),
                    filter_method: iced_core::image::FilterMethod::default(),
                    rotation: iced_core::Radians(0.0),
                    opacity: 1.0,
                    snap: false,
                },
                iced_core::Rectangle {
                    x: img.rect.origin.x,
                    y: img.rect.origin.y,
                    width: img.rect.size.width,
                    height: img.rect.size.height,
                },
            );
        }

        for icon in layer.icons.iter() {
            out.text_renderer
                .queue_icon(icon.origin.x, icon.origin.y, icon.size, &icon.icon);
        }

        for text in layer.text.iter() {
            let buffer = if text.bold {
                TextRenderer::gen_buffer_with_attr(
                    text.size,
                    &text.text,
                    cosmic_text::Attrs::new()
                        .family(cosmic_text::Family::Name("Roboto"))
                        .weight(cosmic_text::Weight::BOLD)
                        .color(cosmic_text::Color(text.color.packet_u32())),
                )
            } else {
                TextRenderer::gen_buffer_with_attr(
                    text.size,
                    &text.text,
                    cosmic_text::Attrs::new()
                        .family(cosmic_text::Family::Name("Roboto"))
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
