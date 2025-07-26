pub mod menu_scene;
pub mod playing_scene;

use crate::context::Context;
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

fn render_nuon(
    ui: &mut nuon::Ui,
    quad_pipeline: &mut QuadRenderer,
    layer: usize,
    text_renderer: &mut TextRenderer,
    renderer: &mut impl iced_core::image::Renderer<Handle = iced_core::image::Handle>,
) {
    for quad in ui.quads.iter() {
        quad_pipeline.push(
            layer,
            neothesia_core::render::QuadInstance {
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
            },
        );
    }

    for img in ui.images.iter() {
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

    for icon in ui.icons.iter() {
        text_renderer.queue_icon(icon.origin.x, icon.origin.y, icon.size, &icon.icon);
    }

    for text in ui.text.iter() {
        let buffer = if text.bold {
            TextRenderer::gen_buffer_bold(text.size, &text.text)
        } else {
            TextRenderer::gen_buffer(text.size, &text.text)
        };

        match text.text_justify {
            nuon::TextJustify::Left => {
                text_renderer.queue_buffer_left(text.rect, buffer);
            }
            nuon::TextJustify::Right => {
                text_renderer.queue_buffer_right(text.rect, buffer);
            }
            nuon::TextJustify::Center => {
                text_renderer.queue_buffer_centered(text.rect, buffer);
            }
        }
    }

    ui.done();
}
