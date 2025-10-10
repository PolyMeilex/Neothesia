use std::cell::OnceCell;

use ffmpeg::{AVCodecID, AVRational};

use crate::{FRAME_RATE, SRC_STREAM_PIX_FMT, STREAM_PIX_FMT, ff};

pub struct VideoOutputStream {
    pub stream: ff::Stream,
    pub codec_ctx: ff::CodecContext,
    pub tmp_pkt: ff::Packet,

    pub frame: ff::Frame,
    pub tmp_frame: Option<ff::Frame>,

    pub next_pts: i64,

    pub sws_ctx: OnceCell<ff::SwsContext>,
}

impl VideoOutputStream {
    pub fn new(
        format_context: &ff::FormatContext,
        output_format: &ff::OutputFormat,
        width: i32,
        height: i32,
    ) -> Self {
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
            (*codec_ctx).width = width;
            (*codec_ctx).height = height;

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

        codec_ctx.open_video();

        let video_frame =
            ff::Frame::new_video(codec_ctx.pix_fmt(), codec_ctx.width(), codec_ctx.height());

        video_frame.set_presentation_timestamp(0);

        // If the output format is not YUV420P, then a temporary YUV420P
        // picture is needed too. It is then converted to the required
        // output format.
        let tmp_frame = if codec_ctx.pix_fmt() != SRC_STREAM_PIX_FMT {
            Some(ff::Frame::new_video(
                SRC_STREAM_PIX_FMT,
                codec_ctx.width(),
                codec_ctx.height(),
            ))
        } else {
            None
        };

        // copy the stream parameters to the muxer
        codec_ctx.copy_parameters_to_stream(&stream);

        Self {
            stream,
            codec_ctx,
            tmp_pkt,
            frame: video_frame,
            tmp_frame,
            sws_ctx: OnceCell::new(),
            next_pts: 0,
        }
    }

    fn next_frame(&mut self, frame_bytes: &[u8]) {
        let video = self;
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
    pub fn write_frame(&mut self, format_ctx: &ff::FormatContext, frame_bytes: &[u8]) -> bool {
        self.next_frame(frame_bytes);

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
