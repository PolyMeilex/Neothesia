#![allow(clippy::too_many_arguments)]

mod layer;

mod buffer;

mod image;

use buffer::Buffer;
use neothesia_core::Color;

type Point<T = f32> = neothesia_core::euclid::default::Point2D<T>;
type Size<T = f32> = neothesia_core::euclid::default::Size2D<T>;
type Rectangle<T = f32> = neothesia_core::euclid::default::Rect<T>;

pub use image::{FilterMethod, Handle as ImageHandle, Image};

trait Snap {
    fn snap(&self) -> Option<Rectangle<u32>>;
}

impl Snap for Rectangle<f32> {
    fn snap(&self) -> Option<Rectangle<u32>> {
        let top_left = self.origin.round();
        let top_left = Point::new(top_left.x as u32, top_left.y as u32);

        let bottom_right = self.origin + self.size;
        let bottom_right = bottom_right.round();

        let width = (bottom_right.x as u32).checked_sub(top_left.x)?;
        let height = (bottom_right.y as u32).checked_sub(top_left.y)?;

        if width < 1 || height < 1 {
            return None;
        }

        Some(Rectangle::new(
            (top_left.x, top_left.y).into(),
            (width, height).into(),
        ))
    }
}

/// A [`wgpu`] graphics renderer for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
#[allow(missing_debug_implementations)]
pub struct Renderer {
    layer: layer::Layer,
    image: image::State,

    // TODO: Centralize all the image feature handling
    image_cache: std::cell::RefCell<image::Cache>,

    staging_belt: wgpu::util::StagingBelt,

    physical_size: Size<u32>,
    scale_factor: f64,
    projection: [f32; 16],

    device: wgpu::Device,
    queue: wgpu::Queue,
    image_pipeline: crate::image::Pipeline,
}

impl Renderer {
    pub fn new(
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        physical_size: Size<u32>,
        scale_factor: f64,
    ) -> Self {
        let image_pipeline =
            crate::image::Pipeline::new(&device, format, adapter.get_info().backend);

        Self {
            layer: layer::Layer::new(),
            image: image::State::new(),

            image_cache: std::cell::RefCell::new(image_pipeline.create_cache(&device)),

            // TODO: Resize belt smartly (?)
            // It would be great if the `StagingBelt` API exposed methods
            // for introspection to detect when a resize may be worth it.
            staging_belt: wgpu::util::StagingBelt::new(buffer::MAX_WRITE_SIZE as u64),

            physical_size,
            scale_factor,
            projection: Self::projection(physical_size),

            device,
            queue,
            image_pipeline,
        }
    }

    #[rustfmt::skip]
    fn projection(size: Size<u32>) -> [f32; 16] {
        [
            2.0 / size.width as f32, 0.0, 0.0, 0.0,
            0.0, -2.0 / size.height as f32, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -1.0, 1.0, 0.0, 1.0,
        ]
    }

    pub fn resize(&mut self, physical_size: Size<u32>, scale_factor: f64) {
        self.physical_size = physical_size;
        self.scale_factor = scale_factor;
        self.projection = Self::projection(self.physical_size);
    }

    fn draw(
        &mut self,
        clear_color: Option<Color>,
        target: &wgpu::TextureView,
    ) -> wgpu::CommandEncoder {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("iced_wgpu encoder"),
            });

        self.prepare(&mut encoder);
        self.render(&mut encoder, target, clear_color);

        {
            self.image.trim();
            self.image_cache.borrow_mut().trim();
        }

        encoder
    }

    pub fn present(
        &mut self,
        clear_color: Option<Color>,
        _format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
    ) -> wgpu::SubmissionIndex {
        let encoder = self.draw(clear_color, frame);

        self.staging_belt.finish();
        let submission = self.queue.submit([encoder.finish()]);
        self.staging_belt.recall();
        submission
    }

    fn prepare(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let scale_factor = self.scale_factor as f32;

        let physical_bounds = Rectangle::new(
            (0.0, 0.0).into(),
            (
                self.physical_size.width as f32,
                self.physical_size.height as f32,
            )
                .into(),
        );

        let layer = &mut self.layer;

        if physical_bounds
            .intersection(&(layer.bounds * scale_factor))
            .and_then(|r| r.snap())
            .is_none()
        {
            return;
        }

        if !layer.images.is_empty() {
            self.image.prepare(
                &self.image_pipeline,
                &self.device,
                &mut self.staging_belt,
                encoder,
                &mut self.image_cache.borrow_mut(),
                &layer.images,
                self.projection,
                scale_factor,
            );
        }
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        clear_color: Option<Color>,
    ) {
        use std::mem::ManuallyDrop;

        let mut render_pass =
            ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(background_color) => wgpu::LoadOp::Clear({
                                let [r, g, b, a] = background_color.into_linear_rgba();

                                wgpu::Color {
                                    r: f64::from(r),
                                    g: f64::from(g),
                                    b: f64::from(b),
                                    a: f64::from(a),
                                }
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            }));

        let image_cache = self.image_cache.borrow();

        let scale_factor = self.scale_factor as f32;
        let physical_bounds = Rectangle::new(
            (0.0, 0.0).into(),
            (
                self.physical_size.width as f32,
                self.physical_size.height as f32,
            )
                .into(),
        );

        let layer = &self.layer;
        if let Some(scissor_rect) = physical_bounds
            .intersection(&(layer.bounds * scale_factor))
            .and_then(|b| b.snap())
        {
            if !layer.images.is_empty() {
                self.image.render(
                    &self.image_pipeline,
                    &image_cache,
                    0,
                    scissor_rect,
                    &mut render_pass,
                );
            }
        }

        let _ = ManuallyDrop::into_inner(render_pass);
    }
}

impl Renderer {
    pub fn measure_image(&self, handle: &image::Handle) -> Size<u32> {
        self.image_cache.borrow_mut().measure_image(handle)
    }

    pub fn draw_image(&mut self, image: image::Image, bounds: Rectangle) {
        self.layer.images.push((image, bounds));
    }

    pub fn clear(&mut self) {
        self.layer.reset();
    }
}
