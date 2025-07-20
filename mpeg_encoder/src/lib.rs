//! MPEG  video encoder.
//!

// Inspired by the muxing sample: http://ffmpeg.org/doxygen/trunk/muxing_8c-source.html

use ffmpeg::{
    self as ffmpeg, AVChannelLayout, AVChannelLayout__bindgen_ty_1, AVChannelOrder, AVCodec,
    AVCodecContext, AVCodecID, AVFormatContext, AVFrame, AVPacket, AVPixelFormat, AVRational,
    AVSampleFormat, AVStream, SwrContext, SwsContext, AVERROR, AVERROR_EOF, AV_CH_LAYOUT_STEREO,
    AV_CODEC_CAP_VARIABLE_FRAME_SIZE, EAGAIN,
};

use std::cell::OnceCell;
use std::ffi::{c_void, CString};
use std::mem;
use std::path::Path;
use std::ptr::{self, NonNull};

mod ff;

const FRAME_RATE: i32 = 60;
const STREAM_PIX_FMT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_YUV420P;
const SRC_STREAM_PIX_FMT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_BGRA;

struct VideoOutputStream {
    stream: ff::Stream,
    codec_ctx: ff::CodecContext,
    tmp_pkt: ff::Packet,

    frame: ff::Frame,
    tmp_frame: Option<ff::Frame>,

    next_pts: i64,

    sws_ctx: OnceCell<ff::SwsContext>,
}

struct AudioOutputStream {
    stream: *mut AVStream,
    codec_context: *mut AVCodecContext,
    tmp_pkt: ff::Packet,

    frame: NonNull<AVFrame>,
    tmp_frame: NonNull<AVFrame>,

    swr_ctx: *mut SwrContext,
}

/// MPEG video recorder.
pub struct Encoder {}

impl Encoder {
    fn new_video_streams(
        format_context: &ff::FormatContext,
        output_format: &ff::OutputFormat,
    ) -> VideoOutputStream {
        let codec_id = output_format.video_codec_id();
        assert_ne!(
            codec_id,
            AVCodecID::AV_CODEC_ID_NONE,
            "The selected output container does not support video encoding"
        );

        let codec = output_format.video_codec();

        let output_format = output_format.as_ptr();

        let tmp_pkt = ff::Packet::new();

        let stream = format_context
            .new_stream()
            .expect("Could not allocate stream");

        let codec_ctx = codec.context();

        unsafe {
            let codec_ctx = codec_ctx.as_ptr();

            (*codec_ctx).codec_id = codec_id;
            (*codec_ctx).bit_rate = 400000;

            // Resolution must be a multiple of two.
            (*codec_ctx).width = 1920;
            (*codec_ctx).height = 1080;

            // timebase: This is the fundamental unit of time (in seconds) in terms
            // of which frame timestamps are represented. For fixed-fps content,
            // timebase should be 1/framerate and timestamp increments should be
            // identical to 1.
            let time_base = AVRational {
                num: 1,
                den: FRAME_RATE,
            };
            (*stream.as_ptr()).time_base = time_base;
            (*codec_ctx).time_base = time_base;

            (*codec_ctx).gop_size = 12; // emit one intra frame every twelve frames at most
            (*codec_ctx).pix_fmt = STREAM_PIX_FMT;

            if (*codec_ctx).codec_id == AVCodecID::AV_CODEC_ID_MPEG2VIDEO {
                // just for testing, we also add B-frames
                // (*video_codec_context).mb_decision = 2;
            }

            if (*codec_ctx).codec_id == AVCodecID::AV_CODEC_ID_MPEG1VIDEO {
                // Needed to avoid using macroblocks in which some coeffs overflow.
                // This does not happen with normal video, it just happens here as
                // the motion of the chroma plane does not match the luma plane.
                (*codec_ctx).mb_decision = 2;
            }

            // Some formats want stream headers to be separate.
            if (*output_format).flags & ffmpeg::AVFMT_GLOBALHEADER != 0 {
                (*codec_ctx).flags |= ffmpeg::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
            }
        }

        codec_ctx.open();

        let video_frame =
            ff::Frame::new(codec_ctx.pix_fmt(), codec_ctx.width(), codec_ctx.height());

        video_frame.set_pts(0);

        // If the output format is not YUV420P, then a temporary YUV420P
        // picture is needed too. It is then converted to the required
        // output format.
        let tmp_frame = if codec_ctx.pix_fmt() != SRC_STREAM_PIX_FMT {
            Some(ff::Frame::new(
                SRC_STREAM_PIX_FMT,
                codec_ctx.width(),
                codec_ctx.height(),
            ))
        } else {
            None
        };

        // copy the stream parameters to the muxer
        codec_ctx.copy_parameters_to_stream(&stream);

        VideoOutputStream {
            stream,
            codec_ctx,
            tmp_pkt,
            frame: video_frame,
            tmp_frame,
            sws_ctx: OnceCell::new(),
            next_pts: 0,
        }
    }

    fn new_audio_streams(
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

            let nb_samples = if (*codec).capabilities & AV_CODEC_CAP_VARIABLE_FRAME_SIZE as i32 != 0
            {
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

            if ffmpeg::avcodec_parameters_from_context((*stream.as_ptr()).codecpar, codec_context)
                < 0
            {
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

    fn next_video_frame(video: &mut VideoOutputStream, frame_bytes: &[u8]) {
        let codec_ctx = &video.codec_ctx;

        video.frame.make_writable();

        if codec_ctx.pix_fmt() == SRC_STREAM_PIX_FMT {
            todo!()
        } else {
            let sws_ctx = video.sws_ctx.get_or_init(|| {
                ff::SwsContext::new(
                    codec_ctx.width(),
                    codec_ctx.height(),
                    SRC_STREAM_PIX_FMT,
                    codec_ctx.width(),
                    codec_ctx.height(),
                    codec_ctx.pix_fmt(),
                )
            });

            assert_eq!(
                frame_bytes.len(),
                codec_ctx.width() as usize * codec_ctx.height() as usize * 4
            );

            unsafe {
                ffmpeg::av_image_fill_arrays(
                    (*video.tmp_frame.as_ref().unwrap().as_ptr())
                        .data
                        .as_mut_ptr(),
                    (*video.tmp_frame.as_ref().unwrap().as_ptr())
                        .linesize
                        .as_mut_ptr(),
                    frame_bytes.as_ptr(),
                    SRC_STREAM_PIX_FMT,
                    codec_ctx.width(),
                    codec_ctx.height(),
                    1,
                );
            }

            sws_ctx.scale(
                video.tmp_frame.as_ref().unwrap(),
                &video.frame,
                codec_ctx.height(),
            );
        }

        video.frame.set_pts(video.next_pts);
        video.next_pts += 1;
    }

    /// Encode one frame and send it to the muxer.
    /// Returns true when encoding is finished, false otherwise.
    fn write_frame(
        format_ctx: &ff::FormatContext,
        codec_ctx: &ff::CodecContext,
        stream: &ff::Stream,
        frame: Option<&ff::Frame>,
        packet: &ff::Packet,
    ) -> bool {
        // Send the frame to the encoder
        codec_ctx.send_frame(frame);

        let mut ret = 0;
        while ret >= 0 {
            ret = codec_ctx.receive_packet(packet);

            if ret == AVERROR(EAGAIN) || ret == AVERROR_EOF {
                break;
            } else if ret < 0 {
                panic!("Error encoding a frame",);
            }

            // Rescale output packet timestamp values from codec to stream timebase
            packet.rescale_ts(codec_ctx.time_base(), stream.time_base());
            packet.set_stream_index(stream.index());

            // Write the compressed frame to the media file.
            format_ctx.interleaved_write_frame(packet);
        }

        ret == AVERROR_EOF
    }

    pub fn new2(path: impl AsRef<Path>) -> impl FnMut(Option<&[u8]>) {
        let path = path.as_ref().to_str().unwrap();
        let path = CString::new(path).unwrap();

        let format_context = ff::FormatContext::new(&path);

        let output_format = format_context.output_format();

        let mut video_stream = Self::new_video_streams(&format_context, &output_format);
        let audio_stream = Self::new_audio_streams(&format_context, &output_format);

        format_context.dump_format(&path);
        format_context.open(&path);
        // Write the stream header, if any.
        format_context.write_header();

        move |input_frame| {
            if let Some(input_frame) = input_frame {
                Self::next_video_frame(&mut video_stream, input_frame);
                Self::write_frame(
                    &format_context,
                    &video_stream.codec_ctx,
                    &video_stream.stream,
                    Some(&video_stream.frame),
                    &video_stream.tmp_pkt,
                );
            } else {
                Self::write_frame(
                    &format_context,
                    &video_stream.codec_ctx,
                    &video_stream.stream,
                    None,
                    &video_stream.tmp_pkt,
                );

                format_context.write_trailer();
                format_context.closep();
            }
        }
    }
}
