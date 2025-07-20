// Based on: https://ffmpeg.org/doxygen/trunk/mux_8c-example.html

use std::ffi::CString;
use std::path::Path;

use ffmpeg::AVPixelFormat;

mod audio;
mod ff;
mod video;

const FRAME_RATE: i32 = 60;
const STREAM_PIX_FMT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_YUV420P;
const SRC_STREAM_PIX_FMT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_BGRA;

pub fn new(path: impl AsRef<Path>) -> impl FnMut(Option<&[u8]>) {
    let path = path.as_ref().to_str().unwrap();
    let path = CString::new(path).unwrap();

    let format_context = ff::FormatContext::new(&path);

    let output_format = format_context.output_format();

    let video_stream = video::VideoOutputStream::new(&format_context, &output_format);
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

            video_stream.write_frame(format_context, Some(input_frame));
        } else {
            let (mut video_stream, _audio_stream, format_context) =
                ctx.take().expect("Encoder should not be closed");

            video_stream.write_frame(&format_context, None);

            format_context.write_trailer();
        }
    }
}
