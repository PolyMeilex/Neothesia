use std::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use ffmpeg::{
    AVChannelLayout, AVChannelLayout__bindgen_ty_1, AVChannelOrder, AVCodecContext, AVCodecID,
    AVFrame, AVRational, AVSampleFormat, AVStream, SwrContext, AV_CH_LAYOUT_STEREO,
    AV_CODEC_CAP_VARIABLE_FRAME_SIZE,
};

use crate::ff;

pub struct AudioOutputStream {
    pub stream: *mut AVStream,
    pub codec_context: *mut AVCodecContext,
    pub tmp_pkt: ff::Packet,

    pub frame: NonNull<AVFrame>,
    pub tmp_frame: NonNull<AVFrame>,

    pub swr_ctx: *mut SwrContext,
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
    let codec_context = codec_context.as_ptr();

    unsafe {
        let sample_fmts = (*codec).sample_fmts;

        (*codec_context).sample_fmt = if sample_fmts.is_null() {
            AVSampleFormat::AV_SAMPLE_FMT_FLTP
        } else {
            *(*codec).sample_fmts
        };

        (*codec_context).bit_rate = 64000;
        (*codec_context).sample_rate = 44100;

        let supported_samplerates = (*codec).supported_samplerates;

        if !supported_samplerates.is_null() {
            (*codec_context).sample_rate = *supported_samplerates.offset(0);
            let mut i = 0;
            while *supported_samplerates.offset(i) != 0 {
                if *supported_samplerates.offset(i) == 44100 {
                    (*codec_context).sample_rate = 44100;
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
        ffmpeg::av_channel_layout_copy(&mut (*codec_context).ch_layout, &stereo_layout);

        (*stream.as_ptr()).time_base = AVRational {
            num: 1,
            den: (*codec_context).sample_rate,
        };

        // Some formats want stream headers to be separate.
        if (*output_format).flags & ffmpeg::AVFMT_GLOBALHEADER != 0 {
            (*codec_context).flags |= ffmpeg::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }

        if ffmpeg::avcodec_open2(codec_context, codec, ptr::null_mut()) < 0 {
            panic!("Could not open audio codec.");
        }

        let nb_samples = if (*codec).capabilities & AV_CODEC_CAP_VARIABLE_FRAME_SIZE as i32 != 0 {
            10000
        } else {
            (*codec_context).frame_size
        };

        // TODO: Make sure this is right
        unsafe fn alloc_audio_frame(
            sample_fmt: AVSampleFormat,
            channel_layout: *const AVChannelLayout,
            sample_rate: i32,
            nb_samples: i32,
        ) -> NonNull<AVFrame> {
            let frame = ffmpeg::av_frame_alloc();
            if frame.is_null() {
                panic!("Error allocating an audio frame");
            }
            (*frame).format = sample_fmt as i32;
            ffmpeg::av_channel_layout_copy(&mut (*frame).ch_layout, channel_layout);
            (*frame).sample_rate = sample_rate;
            (*frame).nb_samples = nb_samples;

            if nb_samples > 0 && ffmpeg::av_frame_get_buffer(frame, 0) < 0 {
                panic!("Error allocating an audio buffer");
            }

            NonNull::new_unchecked(frame)
        }

        let frame = alloc_audio_frame(
            (*codec_context).sample_fmt,
            &(*codec_context).ch_layout,
            (*codec_context).sample_rate,
            nb_samples,
        );
        let tmp_frame = alloc_audio_frame(
            AVSampleFormat::AV_SAMPLE_FMT_S16,
            &(*codec_context).ch_layout,
            (*codec_context).sample_rate,
            nb_samples,
        );

        if ffmpeg::avcodec_parameters_from_context((*stream.as_ptr()).codecpar, codec_context) < 0 {
            panic!("Could not copy the stream parameters");
        }

        let swr_ctx = ffmpeg::swr_alloc();
        if swr_ctx.is_null() {
            panic!("Could not allocate resampler context");
        }

        ffmpeg::av_opt_set_chlayout(
            swr_ctx as *mut c_void,
            c"in_chlayout".as_ptr(),
            &(*codec_context).ch_layout,
            0,
        );
        ffmpeg::av_opt_set_int(
            swr_ctx as *mut c_void,
            c"in_sample_rate".as_ptr(),
            (*codec_context).sample_rate as i64,
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
            &(*codec_context).ch_layout,
            0,
        );
        ffmpeg::av_opt_set_int(
            swr_ctx as *mut c_void,
            c"out_sample_rate".as_ptr(),
            (*codec_context).sample_rate as i64,
            0,
        );
        ffmpeg::av_opt_set_sample_fmt(
            swr_ctx as *mut c_void,
            c"out_sample_fmt".as_ptr(),
            (*codec_context).sample_fmt,
            0,
        );

        if ffmpeg::swr_init(swr_ctx) < 0 {
            panic!("Failed to initialize the resampling context");
        }

        AudioOutputStream {
            stream: stream.as_ptr(),
            codec_context,
            tmp_pkt,
            frame,
            tmp_frame,
            swr_ctx,
        }
    }
}
