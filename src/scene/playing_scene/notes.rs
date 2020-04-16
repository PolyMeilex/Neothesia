use super::notes_pipeline::{NoteInstance, NotesPipeline};
use crate::wgpu_jumpstart::gpu::Gpu;
use crate::MainState;

pub struct Notes {
    notes_pipeline: NotesPipeline,
}

impl Notes {
    pub fn new(state: &MainState, gpu: &Gpu, midi: &lib_midi::Midi) -> Self {
        let notes_pipeline = NotesPipeline::new(state, &gpu.device, midi);
        Self { notes_pipeline }
    }
    pub fn resize(
        &mut self,
        state: &crate::MainState,
        gpu: &mut Gpu,
        keys: &Vec<super::keyboard::Key>,
        midi: &lib_midi::Midi,
    ) {
        let mut instances = Vec::new();

        let mut longer_than_88 = false;
        for note in midi.merged_track.notes.iter() {
            if note.note >= 21 && note.note <= 108 {
                let key = &keys[note.note as usize - 21];
                let ar = state.window_size.0 / state.window_size.1;

                // let colors: [[[f32; 3]; 2]; 2] = [
                //     [
                //         [146.0 / 255.0, 255.0 / 255.0, 48.0 / 255.0],
                //         [87.0 / 255.0, 183.0 / 255.0, 12.0 / 255.0],
                //     ],
                //     [
                //         [118.0 / 255.0, 166.0 / 255.0, 211.0 / 255.0],
                //         [54.0 / 255.0, 109.0 / 255.0, 173.0 / 255.0],
                //     ],
                // ];
                let colors: [[[f32; 3]; 2]; 2] = [
                    [
                        [93.0 / 255.0, 188.0 / 255.0, 255.0 / 255.0],
                        [48.0 / 255.0, 124.0 / 255.0, 255.0 / 255.0],
                        // [49.0 / 255.0, 151.0 / 255.0, 255.0 / 255.0],
                        // [5.0 / 255.0, 92.0 / 255.0, 182.0 / 255.0],
                    ],
                    [
                        [210.0 / 255.0, 89.0 / 255.0, 222.0 / 255.0],
                        [125.0 / 255.0, 69.0 / 255.0, 134.0 / 255.0],
                        // [165.0 / 255.0, 84.0 / 255.0, 255.0 / 255.0],
                        // [114.0 / 255.0, 0.0 / 255.0, 219.0 / 255.0],
                    ],
                ];

                let color = colors[note.track_id % 2];
                let color = if key.is_black { color[1] } else { color[0] };

                let h = if note.duration >= 0.1 {
                    note.duration
                } else {
                    0.1
                };

                instances.push(NoteInstance {
                    position: [key.x, note.start],
                    size: [key.w - 1.0, h],
                    color,
                    radius: 4.0 * ar,
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
    pub fn render(&mut self, state: &MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
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
            self.notes_pipeline.render(state, &mut render_pass);
        }
    }
}
