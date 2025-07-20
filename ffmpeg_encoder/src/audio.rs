use std::{ffi::c_void, ptr};

use ffmpeg::{
    av_rescale_q, swr_convert, swr_get_delay, AVChannelLayout, AVChannelLayout__bindgen_ty_1,
    AVChannelOrder, AVCodecID, AVRational, AVSampleFormat, AV_CH_LAYOUT_STEREO,
    AV_CODEC_CAP_VARIABLE_FRAME_SIZE,
};

use crate::ff;

pub struct AudioOutputStream {
    pub stream: ff::Stream,
    pub codec_ctx: ff::CodecContext,
    pub tmp_pkt: ff::Packet,

    pub frame: ff::Frame,
    pub tmp_frame: ff::Frame,

    pub swr_ctx: ff::SwrContext,

    next_pts: i64,
    samples_count: i64,

    t: f32,
    tincr: f32,
    tincr2: f32,
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

    let codec_context = codec.context();

    let codec = codec.as_ptr();
    let codec_ctx = codec_context.as_ptr();

    unsafe {
        let sample_fmts = (*codec).sample_fmts;

        (*codec_ctx).sample_fmt = if sample_fmts.is_null() {
            AVSampleFormat::AV_SAMPLE_FMT_FLTP
        } else {
            *(*codec).sample_fmts
        };

        (*codec_ctx).bit_rate = 64000;
        (*codec_ctx).sample_rate = 44100;

        let supported_samplerates = (*codec).supported_samplerates;

        if !supported_samplerates.is_null() {
            (*codec_ctx).sample_rate = *supported_samplerates.offset(0);
            let mut i = 0;
            while *supported_samplerates.offset(i) != 0 {
                if *supported_samplerates.offset(i) == 44100 {
                    (*codec_ctx).sample_rate = 44100;
                }
                i += 1;
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
        ffmpeg::av_channel_layout_copy(&mut (*codec_ctx).ch_layout, &stereo_layout);

        (*stream.as_ptr()).time_base = AVRational {
            num: 1,
            den: (*codec_ctx).sample_rate,
        };

        // Some formats want stream headers to be separate.
        if (*output_format).flags & ffmpeg::AVFMT_GLOBALHEADER != 0 {
            (*codec_ctx).flags |= ffmpeg::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }

        if ffmpeg::avcodec_open2(codec_ctx, codec, ptr::null_mut()) < 0 {
            panic!("Could not open audio codec.");
        }

        let nb_samples = if (*codec).capabilities & AV_CODEC_CAP_VARIABLE_FRAME_SIZE as i32 != 0 {
            10000
        } else {
            (*codec_ctx).frame_size
        };

        let frame = ff::Frame::new_audio(
            (*codec_ctx).sample_fmt,
            &(*codec_ctx).ch_layout,
            (*codec_ctx).sample_rate,
            nb_samples,
        );
        let tmp_frame = ff::Frame::new_audio(
            AVSampleFormat::AV_SAMPLE_FMT_S16,
            &(*codec_ctx).ch_layout,
            (*codec_ctx).sample_rate,
            nb_samples,
        );

        if ffmpeg::avcodec_parameters_from_context((*stream.as_ptr()).codecpar, codec_ctx) < 0 {
            panic!("Could not copy the stream parameters");
        }

        let swr_ctx = ff::SwrContext::new();

        {
            let swr_ctx = swr_ctx.as_ptr();
            ffmpeg::av_opt_set_chlayout(
                swr_ctx as *mut c_void,
                c"in_chlayout".as_ptr(),
                &(*codec_ctx).ch_layout,
                0,
            );
            ffmpeg::av_opt_set_int(
                swr_ctx as *mut c_void,
                c"in_sample_rate".as_ptr(),
                (*codec_ctx).sample_rate as i64,
                0,
            );
            ffmpeg::av_opt_set_sample_fmt(
                swr_ctx as *mut c_void,
                c"in_sample_fmt".as_ptr(),
                AVSampleFormat::AV_SAMPLE_FMT_S16,
                0,
            );
            ffmpeg::av_opt_set_chlayout(
                swr_ctx as *mut c_void,
                c"out_chlayout".as_ptr(),
                &(*codec_ctx).ch_layout,
                0,
            );
            ffmpeg::av_opt_set_int(
                swr_ctx as *mut c_void,
                c"out_sample_rate".as_ptr(),
                (*codec_ctx).sample_rate as i64,
                0,
            );
            ffmpeg::av_opt_set_sample_fmt(
                swr_ctx as *mut c_void,
                c"out_sample_fmt".as_ptr(),
                (*codec_ctx).sample_fmt,
                0,
            );
        }

        // Init signal generator
        let t = 0.0;
        let tincr = 2.0 * std::f32::consts::PI * 110.0 / codec_context.sample_rate() as f32;
        let tincr2 = 2.0 * std::f32::consts::PI * 110.0
            / (codec_context.sample_rate() as f32 * codec_context.sample_rate() as f32);

        swr_ctx.init();

        AudioOutputStream {
            stream,
            codec_ctx: codec_context,
            tmp_pkt,
            frame,
            tmp_frame,
            swr_ctx,

            next_pts: 0,
            samples_count: 0,

            t,
            tincr,
            tincr2,
        }
    }
}

#[allow(unused)]
impl AudioOutputStream {
    /// Prepare a 16-bit dummy audio frame.
    unsafe fn next_frame(&mut self) {
        let ost = self;
        let frame = &ost.tmp_frame;

        let nb_samples = (*frame.as_ptr()).nb_samples as usize;
        let nb_channels = (*ost.codec_ctx.as_ptr()).ch_layout.nb_channels as usize;
        let q = (*frame.as_ptr()).data[0] as *mut i16;

        for j in 0..nb_samples {
            let v = (ost.t.sin() * 10000.0) as i16;
            for i in 0..nb_channels {
                q.add(j * nb_channels + i).write(v);
            }
            ost.t += ost.tincr;
            ost.tincr += ost.tincr2;
        }

        (*frame.as_ptr()).pts = ost.next_pts;
        ost.next_pts += (*frame.as_ptr()).nb_samples as i64;
    }

    /// Encode one audio frame and send it to the muxer.
    pub fn write_frame(&mut self, format_ctx: &ff::FormatContext) -> bool {
        unsafe { self.next_frame() };

        let frame = self.frame.as_ptr();

        unsafe {
            let dst_nb_samples =
                swr_get_delay(self.swr_ctx.as_ptr(), self.codec_ctx.sample_rate() as i64)
                    + (*self.frame.as_ptr()).nb_samples as i64;

            assert_eq!(dst_nb_samples, (*frame).nb_samples as i64);

            self.frame.make_writable();

            let ret = swr_convert(
                self.swr_ctx.as_ptr(),
                (*frame).data.as_mut_ptr(),
                dst_nb_samples as i32,
                (*frame).data.as_ptr() as *const *const u8,
                (*frame).nb_samples,
            );
            if ret < 0 {
                panic!("Error while converting audio");
            }

            let time_base = AVRational {
                num: 1,
                den: self.codec_ctx.sample_rate(),
            };

            self.frame.set_presentation_timestamp(av_rescale_q(
                self.samples_count,
                time_base,
                self.codec_ctx.time_base(),
            ));
            self.samples_count += dst_nb_samples;
        }

        super::write_frame(
            &self.codec_ctx,
            &self.stream,
            &self.tmp_pkt,
            format_ctx,
            Some(&self.frame),
        )
    }
}
