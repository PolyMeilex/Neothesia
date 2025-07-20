//! MPEG  video encoder.
//!

// Inspired by the muxing sample: https://ffmpeg.org/doxygen/trunk/mux_8c-example.html

use ffmpeg::{AVPixelFormat, AVERROR, AVERROR_EOF, EAGAIN};
use video::VideoOutputStream;

use std::ffi::CString;
use std::path::Path;

mod audio;
mod ff;
mod video;

const FRAME_RATE: i32 = 60;
const STREAM_PIX_FMT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_YUV420P;
const SRC_STREAM_PIX_FMT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_BGRA;

/// MPEG video recorder.
pub struct Encoder {}

impl Encoder {
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

        let video_stream = video::new_video_streams(&format_context, &output_format);
        let audio_stream = audio::new_audio_streams(&format_context, &output_format);

        format_context.dump_format(&path);
        format_context.open(&path);
        // Write the stream header, if any.
        format_context.write_header();

        let mut ctx = Some((video_stream, audio_stream, format_context));

        move |input_frame| {
            if let Some(input_frame) = input_frame {
                let (video_stream, _audio_stream, format_context) =
                    ctx.as_mut().expect("Encoder should not be closed");

                Self::next_video_frame(video_stream, input_frame);
                Self::write_frame(
                    format_context,
                    &video_stream.codec_ctx,
                    &video_stream.stream,
                    Some(&video_stream.frame),
                    &video_stream.tmp_pkt,
                );
            } else {
                let (video_stream, audio_stream, format_context) =
                    ctx.take().expect("Encoder should not be closed");

                Self::write_frame(
                    &format_context,
                    &video_stream.codec_ctx,
                    &video_stream.stream,
                    None,
                    &video_stream.tmp_pkt,
                );

                format_context.write_trailer();

                std::mem::drop(video_stream);
                std::mem::drop(audio_stream);
                std::mem::drop(format_context);
            }
        }
    }
}
