use std::io::Cursor;

use bytemuck::{Pod, Zeroable};
use neothesia_core::Rect;
use wgpu_jumpstart::{Instances, Shape, wgpu};

use crate::context::Context;

#[derive(Clone, Copy)]
pub enum SpriteKind {
    TrebleClef,
    BassClef,
    QuarterNote,
    HalfNote,
    WholeNote,
    Sharp,
}

impl SpriteKind {
    fn uv(self) -> ([f32; 2], [f32; 2]) {
        let cell = [1.0 / 3.0, 1.0 / 2.0];
        let origin = match self {
            Self::TrebleClef => [0.0, 0.0],
            Self::BassClef => [cell[0], 0.0],
            Self::QuarterNote => [cell[0] * 2.0, 0.0],
            Self::HalfNote => [0.0, cell[1]],
            Self::WholeNote => [cell[0], cell[1]],
            Self::Sharp => [cell[0] * 2.0, cell[1]],
        };
        (origin, cell)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SpriteInstance {
    position: [f32; 2],
    size: [f32; 2],
    uv_origin: [f32; 2],
    uv_size: [f32; 2],
    tint: [f32; 4],
}

impl SpriteInstance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            1 => Float32x2,
            2 => Float32x2,
            3 => Float32x2,
            4 => Float32x2,
            5 => Float32x4,
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}

pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    transform_bind_group: wgpu::BindGroup,
    atlas_bind_group: wgpu::BindGroup,
    shape: Shape,
    instances: Instances<SpriteInstance>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    scissor_rect: Rect<u32>,
}

impl SpriteRenderer {
    pub fn new(ctx: &Context) -> Self {
        let device = &ctx.gpu.device;
        let queue = &ctx.gpu.queue;

        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sheet music atlas layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sheet music sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sheet music sprite pipeline layout"),
            bind_group_layouts: &[&ctx.transform.bind_group_layout, &atlas_layout],
            immediate_size: 0,
        });
        let target = wgpu_jumpstart::default_color_target_state(ctx.gpu.texture_format);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sheet music sprite pipeline"),
            layout: Some(&pipeline_layout),
            fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
            ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                &shader,
                &[Shape::layout(), SpriteInstance::layout()],
            ))
        });

        let atlas_bytes = include_bytes!("../../../../../assets/sheet-music/notation-atlas.png");
        let (rgba, width, height) =
            neothesia_image::load_png(Cursor::new(atlas_bytes.as_slice())).unwrap();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sheet music notation atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sheet music atlas sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sheet music atlas bind group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            pipeline,
            transform_bind_group: ctx.transform.bind_group.clone(),
            atlas_bind_group,
            shape: Shape::new_quad(device),
            instances: Instances::new(device, 256),
            device: device.clone(),
            queue: queue.clone(),
            scissor_rect: Rect::zero(),
        }
    }

    pub fn clear(&mut self) {
        self.instances.data.clear();
    }

    pub fn set_scissor_rect(&mut self, rect: Rect<u32>) {
        self.scissor_rect = rect;
    }

    pub fn push(&mut self, kind: SpriteKind, position: [f32; 2], size: [f32; 2], tint: [f32; 4]) {
        let (uv_origin, uv_size) = kind.uv();
        self.instances.data.push(SpriteInstance {
            position,
            size,
            uv_origin,
            uv_size,
            tint,
        });
    }

    pub fn prepare(&mut self) {
        self.instances.update(&self.device, &self.queue);
    }

    pub fn render<'pass>(&'pass self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        let pass_size = rpass.size();
        let rect = self.scissor_rect;
        rpass.set_scissor_rect(rect.origin.x, rect.origin.y, rect.width(), rect.height());
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.transform_bind_group, &[]);
        rpass.set_bind_group(1, &self.atlas_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.shape.vertex_buffer.slice(..));
        rpass.set_vertex_buffer(1, self.instances.buffer.slice(..));
        rpass.set_index_buffer(self.shape.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.shape.indices_len, 0, 0..self.instances.len());
        rpass.set_scissor_rect(0, 0, pass_size.width, pass_size.height);
    }
}
