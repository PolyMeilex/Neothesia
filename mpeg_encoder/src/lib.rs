//! MPEG  video encoder.
//!

// Inspired by the muxing sample: http://ffmpeg.org/doxygen/trunk/muxing_8c-source.html

use ffmpeg_sys as ffmpeg;

use ffmpeg::{
    AVChannelLayout, AVCodec, AVCodecContext, AVCodecID, AVFormatContext, AVFrame, AVPacket,
    AVPixelFormat, AVRational, AVStream, SwsContext, AV_CODEC_FLAG_GLOBAL_HEADER,
};
use std::ffi::CString;
use std::mem;
use std::path::Path;
use std::ptr::{self, NonNull};

#[derive(PartialEq)]
enum ColorFormat {
    Bgra,
}

impl ColorFormat {
    fn has_alpha(&self) -> bool {
        match self {
            Self::Bgra => true,
        }
    }
}

impl From<&ColorFormat> for AVPixelFormat {
    fn from(v: &ColorFormat) -> AVPixelFormat {
        match v {
            ColorFormat::Bgra => AVPixelFormat::AV_PIX_FMT_BGRA,
        }
    }
}

/// Initializes the recorder if needed.
#[allow(clippy::too_many_arguments)]
fn init_context(
    format_context: &NonNull<AVFormatContext>,
    video_st: &NonNull<AVStream>,
    time_base: AVRational,
    gop_size: i32,
    max_b_frames: i32,
    pix_fmt: AVPixelFormat,
    crf: Option<f32>,
    preset: Option<&str>,
    target_width: usize,
    target_height: usize,
) -> NonNull<AVCodecContext> {
    unsafe {
        let video_codec = (*(*format_context.as_ptr()).oformat).video_codec;

        if video_codec == AVCodecID::AV_CODEC_ID_NONE {
            panic!("The selected output container does not support video encoding.")
        }

        let codec: *const AVCodec = ffmpeg::avcodec_find_encoder(video_codec);

        if codec.is_null() {
            panic!("Codec not found.");
        }

        let context = NonNull::new(ffmpeg::avcodec_alloc_context3(codec))
            .expect("Could not allocate video codec context.");

        if let Some(crf) = crf {
            let val = CString::new(crf.to_string()).unwrap();
            let _ = ffmpeg::av_opt_set(
                (*context.as_ptr()).priv_data,
                c"crf".as_ptr(),
                val.as_ptr(),
                0,
            );
        }

        if let Some(preset) = preset {
            let val = CString::new(preset).unwrap();
            let _ = ffmpeg::av_opt_set(
                (*context.as_ptr()).priv_data,
                c"preset".as_ptr(),
                val.as_ptr(),
                0,
            );
        }

        (*context.as_ptr()).codec_id = video_codec;

        // Resolution must be a multiple of two.
        (*context.as_ptr()).width = target_width as i32;
        (*context.as_ptr()).height = target_height as i32;

        // frames per second.
        (*context.as_ptr()).time_base = time_base;
        (*context.as_ptr()).gop_size = gop_size;
        (*context.as_ptr()).max_b_frames = max_b_frames;
        (*context.as_ptr()).pix_fmt = pix_fmt;

        if (*context.as_ptr()).codec_id == AVCodecID::AV_CODEC_ID_MPEG1VIDEO {
            // Needed to avoid using macroblocks in which some coeffs overflow.
            // This does not happen with normal video, it just happens here as
            // the motion of the chroma plane does not match the luma plane.
            (*context.as_ptr()).mb_decision = 2;
        }

        // Open the codec.
        if ffmpeg::avcodec_open2(context.as_ptr(), codec, ptr::null_mut()) < 0 {
            panic!("Could not open the codec.");
        }

        if ffmpeg::avcodec_parameters_from_context((*video_st.as_ptr()).codecpar, context.as_ptr())
            < 0
        {
            panic!("Failed to set codec parameters.");
        }

        context
    }
}

struct AudioCtx {
    stream: NonNull<AVStream>,
    context: NonNull<AVCodecContext>,
    frame: NonNull<AVFrame>,
    frame_size: usize,
    next_pts: i64,
}

/// MPEG video recorder.
pub struct Encoder {
    tmp_frame_buf: Vec<u8>,
    _frame_buf: Vec<u8>,

    _target_width: usize,
    _target_height: usize,
    src_width: usize,
    src_height: usize,

    tmp_frame: NonNull<AVFrame>,
    frame: NonNull<AVFrame>,
    context: NonNull<AVCodecContext>,
    format_context: NonNull<AVFormatContext>,
    video_st: NonNull<AVStream>,
    scale_context: NonNull<SwsContext>,

    audio: Option<AudioCtx>,
}

impl Encoder {
    pub fn new(
        path: impl AsRef<Path>,
        src_width: usize,
        src_height: usize,
        crf: Option<f32>,
        preset: Option<&str>,
        audio_sample_rate: Option<u32>,
    ) -> Self {
        let path = path.as_ref().to_str().unwrap();
        let path_str = CString::new(path).unwrap();

        let time_base = AVRational { num: 1, den: 60 };

        let gop_size = 10;
        let max_b_frames = 1;
        let pix_fmt = AVPixelFormat::AV_PIX_FMT_YUV420P;

        // width and height must be a multiple of two.
        let target_width = if src_width % 2 == 0 {
            src_width
        } else {
            src_width + 1
        };
        let target_height = if src_height % 2 == 0 {
            src_height
        } else {
            src_height + 1
        };

        // sws scaling context
        let scale_context = unsafe {
            NonNull::new(ffmpeg::sws_getContext(
                src_width as i32,
                src_height as i32,
                (&ColorFormat::Bgra).into(),
                target_width as i32,
                target_height as i32,
                pix_fmt,
                ffmpeg::SWS_BICUBIC,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            ))
            .expect("Failed to create scale context")
        };

        // Init the temporary video frame.
        let tmp_frame = unsafe {
            let frame = NonNull::new(ffmpeg::av_frame_alloc())
                .expect("Could not allocate the video frame.");

            (*frame.as_ptr()).format = pix_fmt as i32;
            // the rest (width, height, data, linesize) are set at the moment of the snapshot.

            frame
        };

        // Init the destination video frame.
        let (frame, frame_buf) = unsafe {
            let frame = NonNull::new(ffmpeg::av_frame_alloc())
                .expect("Could not allocate the video frame.");

            (*frame.as_ptr()).format = pix_fmt as i32;
            (*frame.as_ptr()).width = target_width as i32;
            (*frame.as_ptr()).height = target_height as i32;
            (*frame.as_ptr()).pts = 0;

            // alloc the buffer
            let nframe_bytes = ffmpeg::av_image_get_buffer_size(
                pix_fmt,
                target_width as i32,
                target_height as i32,
                16,
            );

            let frame_buf = vec![0u8; nframe_bytes as usize];

            let _ = ffmpeg::av_image_fill_arrays(
                (*frame.as_ptr()).data.as_mut_ptr(),
                (*frame.as_ptr()).linesize.as_mut_ptr(),
                frame_buf.as_ptr(),
                pix_fmt,
                target_width as i32,
                target_height as i32,
                1,
            );

            (frame, frame_buf)
        };

        // try to guess the container type from the path.
        let format_context = unsafe {
            let mut fmt = ptr::null_mut();

            let _ = ffmpeg::avformat_alloc_output_context2(
                &mut fmt,
                ptr::null_mut(),
                ptr::null(),
                path_str.as_ptr(),
            );

            NonNull::new(fmt)
                .or_else(|| {
                    // could not guess, default to MPEG
                    let mpeg = CString::new(&b"mpeg"[..]).unwrap();

                    let _ = ffmpeg::avformat_alloc_output_context2(
                        &mut fmt,
                        ptr::null_mut(),
                        mpeg.as_ptr(),
                        path_str.as_ptr(),
                    );

                    NonNull::new(fmt)
                })
                .expect("Unable to create the output context.")
        };

        let video_st = unsafe {
            let stream = NonNull::new(ffmpeg::avformat_new_stream(
                format_context.as_ptr(),
                ptr::null(),
            ))
            .expect("Failed to allocate the video stream.");

            (*stream.as_ptr()).id = ((*format_context.as_ptr()).nb_streams - 1) as i32;
            (*stream.as_ptr()).time_base = time_base;

            stream
        };

        let context = init_context(
            &format_context,
            &video_st,
            time_base,
            gop_size,
            max_b_frames,
            pix_fmt,
            crf,
            preset,
            target_width,
            target_height,
        );

        let audio = audio_sample_rate.map(|sample_rate| {
            let (audio_st, audio_context, audio_frame, audio_frame_size) =
                Self::init_audio_context(&format_context, sample_rate);

            AudioCtx {
                stream: audio_st,
                context: audio_context,
                frame: audio_frame,
                frame_size: audio_frame_size,
                next_pts: 0,
            }
        });

        // Print detailed information about the input or output format, such as duration, bitrate, streams, container, programs, metadata, side data, codec and time base
        unsafe {
            ffmpeg::av_dump_format(format_context.as_ptr(), 0, path_str.as_ptr(), 1);
        }

        // Finalize and Write Header
        unsafe {
            // Open the output file.
            if ffmpeg::avio_open(
                &mut (*format_context.as_ptr()).pb,
                path_str.as_ptr(),
                ffmpeg::AVIO_FLAG_WRITE,
            ) < 0
            {
                panic!("Failed to open the output file.");
            }

            if ffmpeg::avformat_write_header(format_context.as_ptr(), ptr::null_mut()) < 0 {
                panic!("Failed to open the output file.");
            }
        }

        Self {
            tmp_frame_buf: Vec::new(),

            _target_width: target_width,
            _target_height: target_height,
            src_width,
            src_height,

            tmp_frame,

            frame,
            _frame_buf: frame_buf,

            context,
            format_context,
            video_st,
            scale_context,
            audio,
        }
    }

    /// Adds a image with a BGRA pixel format to the video.
    pub fn encode_bgra(&mut self, data: &[u8]) {
        let width: usize = self.src_width;
        let height: usize = self.src_height;
        let color_format = ColorFormat::Bgra;
        let has_alpha = color_format.has_alpha();

        let mut pkt = unsafe {
            let mut pkt: mem::MaybeUninit<AVPacket> = mem::MaybeUninit::uninit();
            ffmpeg::av_init_packet(pkt.as_mut_ptr());
            pkt.assume_init()
        };

        pkt.data = ptr::null_mut(); // packet data will be allocated by the encoder
        pkt.size = 0;

        // Fill the snapshot frame.
        let pixel_len = if has_alpha { 4 } else { 3 };

        assert_eq!(data.len(), width * height * pixel_len);

        self.tmp_frame_buf.resize(data.len(), 0);
        self.tmp_frame_buf.clone_from_slice(data);

        unsafe {
            (*self.tmp_frame.as_ptr()).width = width as i32;
            (*self.tmp_frame.as_ptr()).height = height as i32;

            ffmpeg::av_image_fill_arrays(
                (*self.tmp_frame.as_ptr()).data.as_mut_ptr(),
                (*self.tmp_frame.as_ptr()).linesize.as_mut_ptr(),
                self.tmp_frame_buf.as_ptr(),
                (&color_format).into(),
                width as i32,
                height as i32,
                1,
            );
        }

        // Convert the snapshot frame to the right format for the destination frame.
        unsafe {
            ffmpeg::sws_scale(
                self.scale_context.as_ptr(),
                &(*self.tmp_frame.as_ptr()).data[0] as *const *mut u8 as *const *const u8,
                &(*self.tmp_frame.as_ptr()).linesize[0],
                0,
                height as i32,
                &(*self.frame.as_ptr()).data[0] as *const *mut u8,
                &(*self.frame.as_ptr()).linesize[0],
            )
        };

        // Encode the image.

        let ret = unsafe { ffmpeg::avcodec_send_frame(self.context.as_ptr(), self.frame.as_ptr()) };

        if ret < 0 {
            panic!("Error encoding frame.");
        }

        unsafe {
            if ffmpeg::avcodec_receive_packet(self.context.as_ptr(), &mut pkt) == 0 {
                ffmpeg::av_interleaved_write_frame(self.format_context.as_ptr(), &mut pkt);
                ffmpeg::av_packet_unref(&mut pkt);
            }
        }

        unsafe {
            (*self.frame.as_ptr()).pts += ffmpeg::av_rescale_q(
                1,
                (*self.context.as_ptr()).time_base,
                (*self.video_st.as_ptr()).time_base,
            );
        }
    }
}

/// TODO: Here be AI dragons
/// This impl block is AI slop, not sure if it makes any sense.
/// Replace with more conscious implementation based on: http://ffmpeg.org/doxygen/trunk/muxing_8c-source.html
impl Encoder {
    fn init_audio_context(
        format_context: &NonNull<AVFormatContext>,
        sample_rate: u32,
    ) -> (
        NonNull<AVStream>,
        NonNull<AVCodecContext>,
        NonNull<AVFrame>,
        usize,
    ) {
        unsafe {
            // Steps 1-8 are the same as before...
            let audio_codec_id = (*(*format_context.as_ptr()).oformat).audio_codec;
            let codec = ffmpeg::avcodec_find_encoder(audio_codec_id);

            if codec.is_null() {
                panic!("Audio codec not found.");
            }

            let stream = NonNull::new(ffmpeg::avformat_new_stream(
                format_context.as_ptr(),
                ptr::null(),
            ))
            .expect("Failed to allocate audio stream.");

            (*stream.as_ptr()).id = ((*format_context.as_ptr()).nb_streams - 1) as i32;
            let codecpar = (*stream.as_ptr()).codecpar;

            (*codecpar).codec_type = ffmpeg::AVMediaType::AVMEDIA_TYPE_AUDIO;
            (*codecpar).codec_id = audio_codec_id;
            (*codecpar).sample_rate = sample_rate as i32;
            (*codecpar).format = ffmpeg::AVSampleFormat::AV_SAMPLE_FMT_FLTP as i32;
            (*codecpar).bit_rate = 128_000;
            (*codecpar).frame_size = 1024;

            let mut ch_layout: AVChannelLayout = mem::zeroed();
            let layout_str = CString::new("stereo").unwrap();
            if ffmpeg::av_channel_layout_from_string(&mut ch_layout, layout_str.as_ptr()) < 0 {
                panic!("Failed to create stereo channel layout.");
            }

            (*codecpar).ch_layout = ch_layout;

            let context = NonNull::new(ffmpeg::avcodec_alloc_context3(codec))
                .expect("Could not alloc audio context.");

            if ffmpeg::avcodec_parameters_to_context(context.as_ptr(), codecpar) < 0 {
                panic!("Failed to copy codec parameters to context");
            }

            (*context.as_ptr()).time_base = AVRational {
                num: 1,
                den: sample_rate as i32,
            };

            if (*(*format_context.as_ptr()).oformat).flags & ffmpeg::AVFMT_GLOBALHEADER != 0 {
                (*context.as_ptr()).flags |= AV_CODEC_FLAG_GLOBAL_HEADER as i32;
            }

            if ffmpeg::avcodec_open2(context.as_ptr(), codec, ptr::null_mut()) < 0 {
                panic!("Could not open audio codec.");
            }

            // 9. Allocate and configure the audio frame we will reuse for encoding
            let frame =
                NonNull::new(ffmpeg::av_frame_alloc()).expect("Could not allocate audio frame.");

            let frame_size_val = (*context.as_ptr()).frame_size;

            (*frame.as_ptr()).nb_samples = frame_size_val;
            (*frame.as_ptr()).format = (*context.as_ptr()).sample_fmt as i32;

            ffmpeg::av_channel_layout_copy(
                &mut (*frame.as_ptr()).ch_layout,
                &(*context.as_ptr()).ch_layout,
            );

            // Allocate the data buffers for the frame
            if ffmpeg::av_frame_get_buffer(frame.as_ptr(), 0) < 0 {
                panic!("Could not allocate audio frame data buffers.");
            }

            (stream, context, frame, frame_size_val as usize)
        }
    }

    /// Adds a stereo audio buffer to the stream.
    /// The audio data is expected to be in 32-bit floating point format.
    /// NOTE: The sample rate must match the `audio_sample_rate` provided to `new()`.
    pub fn encode_audio_f32(&mut self, left_channel: &[f32], right_channel: &[f32]) {
        assert_eq!(
            left_channel.len(),
            right_channel.len(),
            "Left and right channels must have the same number of samples."
        );

        let Some(audio) = self.audio.as_mut() else {
            return;
        };

        let frame = audio.frame.as_ptr();
        let frame_size = audio.frame_size;
        let mut offset = 0;

        while offset < left_channel.len() {
            let samples_to_write = (left_channel.len() - offset).min(frame_size);

            unsafe {
                if ffmpeg::av_frame_make_writable(frame) < 0 {
                    panic!("Audio frame not writable.");
                }

                // Copy data into the planar audio frame
                let left_ptr = (*frame).data[0] as *mut f32;
                let right_ptr = (*frame).data[1] as *mut f32;

                for i in 0..samples_to_write {
                    *left_ptr.add(i) = left_channel[offset + i];
                    *right_ptr.add(i) = right_channel[offset + i];
                }

                (*frame).nb_samples = samples_to_write as i32;
                (*frame).pts = audio.next_pts;
                audio.next_pts += samples_to_write as i64;

                Self::write_audio_frame(
                    self.format_context,
                    audio.context,
                    audio.stream,
                    Some(audio.frame),
                );
            }
            offset += samples_to_write;
        }
    }

    fn write_audio_frame(
        format_context: NonNull<AVFormatContext>,
        audio_context: NonNull<AVCodecContext>,
        stream: NonNull<AVStream>,
        frame_opt: Option<NonNull<AVFrame>>,
    ) {
        let frame_ptr = frame_opt.map_or(ptr::null(), |f| f.as_ptr());

        unsafe {
            if ffmpeg::avcodec_send_frame(audio_context.as_ptr(), frame_ptr) < 0 {
                panic!("Error sending an audio frame for encoding");
            }

            loop {
                let mut pkt: AVPacket = mem::zeroed();
                let ret = ffmpeg::avcodec_receive_packet(audio_context.as_ptr(), &mut pkt);

                if ret == ffmpeg::AVERROR(libc::EAGAIN) || ret == ffmpeg::AVERROR_EOF {
                    break;
                } else if ret < 0 {
                    panic!("Error encoding an audio frame");
                }

                pkt.stream_index = (*stream.as_ptr()).id;
                ffmpeg::av_packet_rescale_ts(
                    &mut pkt,
                    (*audio_context.as_ptr()).time_base,
                    (*stream.as_ptr()).time_base,
                );

                if ffmpeg::av_interleaved_write_frame(format_context.as_ptr(), &mut pkt) < 0 {
                    eprintln!("Warning: Error while writing audio frame");
                }

                ffmpeg::av_packet_unref(&mut pkt);
            }
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // Get the delayed frames.

        let ret = unsafe { ffmpeg::avcodec_send_frame(self.context.as_ptr(), ptr::null()) };

        if ret < 0 {
            panic!("Error encoding frame.");
        }

        loop {
            let mut pkt = unsafe {
                let mut pkt: mem::MaybeUninit<AVPacket> = mem::MaybeUninit::uninit();
                ffmpeg::av_init_packet(pkt.as_mut_ptr());
                pkt.assume_init()
            };

            pkt.data = ptr::null_mut(); // packet data will be allocated by the encoder
            pkt.size = 0;

            unsafe {
                match ffmpeg::avcodec_receive_packet(self.context.as_ptr(), &mut pkt) {
                    0 => {
                        let _ = ffmpeg::av_interleaved_write_frame(
                            self.format_context.as_ptr(),
                            &mut pkt,
                        );
                        ffmpeg::av_packet_unref(&mut pkt);
                    }
                    ffmpeg::AVERROR_EOF => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        if let Some(audio) = self.audio.as_ref() {
            Self::write_audio_frame(self.format_context, audio.context, audio.stream, None);
        }

        // Write trailer
        unsafe {
            if ffmpeg::av_write_trailer(self.format_context.as_ptr()) < 0 {
                panic!("Error writing trailer.");
            }
        }

        // Free things and stuffs.
        unsafe {
            // Free audio resources
            if let Some(audio) = self.audio.as_ref() {
                ffmpeg::avcodec_free_context(&mut audio.context.as_ptr());
                ffmpeg::av_frame_free(&mut audio.frame.as_ptr());
            }

            // Free video resources
            ffmpeg::avcodec_free_context(&mut self.context.as_ptr());
            ffmpeg::av_frame_free(&mut self.frame.as_ptr());
            ffmpeg::av_frame_free(&mut self.tmp_frame.as_ptr());
            ffmpeg::sws_freeContext(self.scale_context.as_ptr());
            if ffmpeg::avio_closep(&mut (*self.format_context.as_ptr()).pb) < 0 {
                println!("Warning: failed closing output file");
            }
            ffmpeg::avformat_free_context(self.format_context.as_ptr());
        }
    }
}
