//! MPEG  video encoder.
//!

// Inspired by the muxing sample: https://ffmpeg.org/doxygen/trunk/mux_8c-example.html

use ffmpeg::{AVCodecID, AVPixelFormat, AVRational, AVERROR, AVERROR_EOF, EAGAIN};

use std::cell::OnceCell;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

mod audio;
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

        video_frame.set_presentation_timestamp(0);

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

            video
                .tmp_frame
                .as_ref()
                .unwrap()
                .image_fill_arrays(frame_bytes, SRC_STREAM_PIX_FMT);

            sws_ctx.scale(
                video.tmp_frame.as_ref().unwrap(),
                &video.frame,
                codec_ctx.height(),
            );
        }

        video.frame.set_presentation_timestamp(video.next_pts);
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
        let audio_stream = audio::new_audio_streams(&format_context, &output_format);

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

                unsafe {
                    ffmpeg::avcodec_free_context(&mut video_stream.codec_ctx.as_ptr());
                    ffmpeg::av_frame_free(&mut video_stream.frame.as_ptr());
                    ffmpeg::av_frame_free(
                        &mut video_stream
                            .tmp_frame
                            .as_ref()
                            .map(ff::Frame::as_ptr)
                            .unwrap_or(ptr::null_mut()),
                    );
                    ffmpeg::av_packet_free(&mut video_stream.tmp_pkt.as_ptr());
                    ffmpeg::sws_freeContext(
                        video_stream
                            .sws_ctx
                            .get()
                            .map(ff::SwsContext::as_ptr)
                            .unwrap_or(ptr::null_mut()),
                    );
                }

                unsafe {
                    ffmpeg::avcodec_free_context(&mut { audio_stream.codec_ctx });
                    ffmpeg::av_frame_free(&mut audio_stream.frame.as_ptr());
                    ffmpeg::av_frame_free(&mut audio_stream.tmp_frame.as_ptr());
                    ffmpeg::av_packet_free(&mut audio_stream.tmp_pkt.as_ptr());
                    ffmpeg::swr_free(&mut { audio_stream.swr_ctx });
                }

                format_context.closep();
                format_context.free();
            }
        }
    }
}
