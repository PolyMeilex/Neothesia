use super::notes_pipeline::{NoteInstance, NotesPipeline};
use crate::wgpu_jumpstart::gpu::Gpu;

pub struct Notes {
    notes_pipeline: NotesPipeline,
}

impl Notes {
    pub fn new(gpu: &Gpu) -> Self {
        let notes_pipeline = NotesPipeline::new(&gpu.device);
        Self { notes_pipeline }
    }
    pub fn resize(
        &mut self,
        state: &crate::MainState,
        gpu: &mut Gpu,
        keys: &Vec<super::keyboard::Key>,
        midi: &lib_midi::Midi,
    ) {
        self.notes_pipeline.resize(
            &mut gpu.encoder,
            &gpu.device,
            (state.window_size.0, state.window_size.1),
        );

        let mut instances = Vec::new();

        let mut longer_than_88 = false;
        for note in midi.merged_track.notes.iter() {
            if note.note >= 21 && note.note <= 108 {
                let key = &keys[note.note as usize - 21];
                let ar = state.window_size.0 / state.window_size.1;

                let color = if key.is_black {
                    [91.0 / 255.0, 55.0 / 255.0, 165.0 / 255.0]
                } else {
                    [121.0 / 255.0, 85.0 / 255.0, 195.0 / 255.0]
                };

                instances.push(NoteInstance {
                    position: [key.x, note.start],
                    size: [key.w - 1.0, note.duration],
                    color,
                    radius: 5.0 * ar,
                });
            } else {
                longer_than_88 = true;
            }
        }

        if longer_than_88 {
            log::warn!("Midi Wider Than 88 Keys!");
        }

        self.notes_pipeline
            .update_instance_buffer(&mut gpu.encoder, &gpu.device, instances);
    }
    pub fn update(&mut self, gpu: &mut Gpu, time: f32) {
        self.notes_pipeline
            .update_time(&mut gpu.encoder, &gpu.device, time);
    }
    pub fn render(&mut self, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
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
            self.notes_pipeline.render(&mut render_pass);
        }
    }
}
