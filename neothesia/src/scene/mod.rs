pub mod menu_scene;
pub mod playing_scene;

use crate::context::Context;
use midi_file::midly::MidiMessage;
use neothesia_core::render::{QuadPipeline, TextRenderer};
use std::time::Duration;
use wgpu_jumpstart::{TransformUniform, Uniform};
use winit::event::WindowEvent;

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, delta: Duration);
    fn render<'pass>(
        &'pass mut self,
        transform: &'pass Uniform<TransformUniform>,
        rpass: &mut wgpu::RenderPass<'pass>,
    );
    fn window_event(&mut self, _ctx: &mut Context, _event: &WindowEvent) {}
    fn midi_event(&mut self, _ctx: &mut Context, _channel: u8, _message: &MidiMessage) {}
}

fn render_nuon(
    ui: &mut nuon::Ui,
    quad_pipeline: &mut QuadPipeline,
    layer: usize,
    text_renderer: &mut TextRenderer,
    renderer: &mut impl iced_core::image::Renderer<Handle = iced_core::image::Handle>,
) {
    for (rect, border_radius, color) in ui.quads.iter() {
        quad_pipeline.push(
            layer,
            neothesia_core::render::QuadInstance {
                position: rect.origin.into(),
                size: rect.size.into(),
                color: wgpu_jumpstart::Color::new(color.r, color.g, color.b, color.a)
                    .into_linear_rgba(),
                border_radius: *border_radius,
            },
        );
    }

    for (rect, image) in ui.images.iter() {
        renderer.draw_image(
            iced_core::Image {
                handle: image.clone(),
                filter_method: iced_core::image::FilterMethod::default(),
                rotation: iced_core::Radians(0.0),
                opacity: 1.0,
                snap: false,
            },
            iced_core::Rectangle {
                x: rect.origin.x,
                y: rect.origin.y,
                width: rect.size.width,
                height: rect.size.height,
            },
        );
    }

    for (pos, size, icon) in ui.icons.iter() {
        text_renderer.queue_icon(pos.x, pos.y, *size, icon);
    }

    for (rect, justify, size, bold, text) in ui.text.iter() {
        let buffer = if *bold {
            text_renderer.gen_buffer_bold(*size, text)
        } else {
            text_renderer.gen_buffer(*size, text)
        };

        match justify {
            nuon::TextJustify::Left => {
                text_renderer.queue_buffer_left(*rect, buffer);
            }
            nuon::TextJustify::Right => {
                text_renderer.queue_buffer_right(*rect, buffer);
            }
            nuon::TextJustify::Center => {
                text_renderer.queue_buffer_centered(*rect, buffer);
            }
        }
    }

    ui.done();
}
