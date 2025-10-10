use std::ptr;

use ffmpeg::{
    AV_CH_LAYOUT_STEREO, AV_CODEC_CAP_VARIABLE_FRAME_SIZE, AVChannelLayout,
    AVChannelLayout__bindgen_ty_1, AVChannelOrder, AVCodecID, AVRational, AVSampleFormat,
    av_rescale_q,
};

use crate::ff;

pub struct AudioOutputStream {
    pub stream: ff::Stream,
    pub codec_ctx: ff::CodecContext,
    pub tmp_pkt: ff::Packet,

    pub frame: ff::Frame,

    samples_count: i64,
}

pub fn new_audio_streams(
    format_context: &ff::FormatContext,
    output_format: &ff::OutputFormat,
) -> AudioOutputStream {
    let codec_id = output_format.audio_codec_id();
    assert_ne!(
        codec_id,
        AVCodecID::AV_CODEC_ID_NONE,
        "The selected output container does not support audio encoding"
    );

    let codec = output_format.audio_codec();

    let output_format = output_format.as_ptr();

    let tmp_pkt = ff::Packet::new();

    let stream = format_context
        .new_stream()
        .expect("Could not allocate stream");

    let mut codec_ctx = codec.context();

    let codec_ptr = codec.as_ptr();
    let codec_ctx_ptr = codec_ctx.as_ptr();

    {
        let sample_fmts = unsafe { (*codec_ptr).sample_fmts };

        unsafe {
            (*codec_ctx_ptr).sample_fmt = if sample_fmts.is_null() {
                AVSampleFormat::AV_SAMPLE_FMT_FLTP
            } else {
                *(*codec_ptr).sample_fmts
            };

            (*codec_ctx_ptr).bit_rate = 64000;
            (*codec_ctx_ptr).sample_rate = 44100;

            let supported_samplerates = (*codec_ptr).supported_samplerates;

            if !supported_samplerates.is_null() {
                (*codec_ctx_ptr).sample_rate = *supported_samplerates.offset(0);
                let mut i = 0;
                while *supported_samplerates.offset(i) != 0 {
                    if *supported_samplerates.offset(i) == 44100 {
                        (*codec_ctx_ptr).sample_rate = 44100;
                    }
                    i += 1;
                }
            }
        }

        let stereo_layout = AVChannelLayout {
            order: AVChannelOrder::AV_CHANNEL_ORDER_NATIVE,
            nb_channels: 2, // stereo
            u: AVChannelLayout__bindgen_ty_1 {
                mask: AV_CH_LAYOUT_STEREO,
            },
            opaque: ptr::null_mut(),
        };

        unsafe {
            ffmpeg::av_channel_layout_copy(codec_ctx.channel_layout_mut(), &stereo_layout);
        }

        stream.set_time_base(AVRational {
            num: 1,
            den: codec_ctx.sample_rate(),
        });

        // Some formats want stream headers to be separate.
        unsafe {
            if (*output_format).flags & ffmpeg::AVFMT_GLOBALHEADER != 0 {
                (*codec_ctx_ptr).flags |= ffmpeg::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
            }
        }

        codec_ctx.open();

        let nb_samples = if codec.capabilities() & AV_CODEC_CAP_VARIABLE_FRAME_SIZE as i32 != 0 {
            10000
        } else {
            codec_ctx.frame_size()
        };

        let frame = ff::Frame::new_audio(
            codec_ctx.sample_fmt(),
            codec_ctx.channel_layout(),
            codec_ctx.sample_rate(),
            nb_samples,
        );

        codec_ctx.copy_parameters_to_stream(&stream);

        AudioOutputStream {
            stream,
            codec_ctx,
            tmp_pkt,
            frame,
            samples_count: 0,
        }
    }
}

#[allow(unused)]
impl AudioOutputStream {
    /// Prepare a 16-bit dummy audio frame.
    fn next_frame(&mut self, audio_l: &[f32], audio_r: &[f32]) {
        self.frame.make_writable();
        let frame_ptr = self.frame.as_ptr();

        debug_assert_eq!(audio_l.len(), audio_r.len());
        let nb_samples = audio_l.len();

        unsafe {
            (*frame_ptr).nb_samples = nb_samples as i32;

            let data_l = (*frame_ptr).data[0] as *mut f32;
            let data_r = (*frame_ptr).data[1] as *mut f32;

            for i in 0..nb_samples {
                *data_l.add(i) = audio_l[i];
                *data_r.add(i) = audio_r[i];
            }
        }

        let time_base = AVRational {
            num: 1,
            den: self.codec_ctx.sample_rate(),
        };

        self.frame.set_presentation_timestamp(unsafe {
            av_rescale_q(self.samples_count, time_base, self.codec_ctx.time_base())
        });

        self.samples_count += nb_samples as i64;
    }

    /// Encode one audio frame and send it to the muxer.
    pub fn write_frame(
        &mut self,
        format_ctx: &ff::FormatContext,
        audio_l: &[f32],
        audio_r: &[f32],
    ) -> bool {
        self.next_frame(audio_l, audio_r);

        super::write_frame(
            &self.codec_ctx,
            &self.stream,
            &self.tmp_pkt,
            format_ctx,
            Some(&self.frame),
        )
    }

    pub fn write_terminator_frame(&self, format_ctx: &ff::FormatContext) -> bool {
        super::write_frame(
            &self.codec_ctx,
            &self.stream,
            &self.tmp_pkt,
            format_ctx,
            None,
        )
    }
}
