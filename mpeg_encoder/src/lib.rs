//! MPEG  video encoder.
//!

// Inspired by the muxing sample: http://ffmpeg.org/doxygen/trunk/muxing_8c-source.html

use ffmpeg_sys::{
    AVCodec, AVCodecContext, AVCodecID, AVFormatContext, AVFrame, AVPacket, AVPixelFormat,
    AVRational, AVStream, SwsContext,
};
use std::ffi::{CStr, CString};
use std::mem;
use std::path::Path;
use std::ptr::{self, NonNull};

#[derive(PartialEq)]
enum ColorFormat {
    Rgb,
    Rgba,
    Bgr,
    Bgra,
}

impl From<&ColorFormat> for AVPixelFormat {
    fn from(v: &ColorFormat) -> AVPixelFormat {
        match v {
            ColorFormat::Rgb => AVPixelFormat::AV_PIX_FMT_RGB24,
            ColorFormat::Rgba => AVPixelFormat::AV_PIX_FMT_RGBA,
            ColorFormat::Bgr => AVPixelFormat::AV_PIX_FMT_BGR24,
            ColorFormat::Bgra => AVPixelFormat::AV_PIX_FMT_BGRA,
        }
    }
}

/// Initializes the recorder if needed.
#[allow(clippy::too_many_arguments)]
fn init_context(
    path_str: &CStr,
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

        let codec: *const AVCodec = ffmpeg_sys::avcodec_find_encoder(video_codec);

        if codec.is_null() {
            panic!("Codec not found.");
        }

        let context = NonNull::new(ffmpeg_sys::avcodec_alloc_context3(codec))
            .expect("Could not allocate video codec context.");

        if let Some(crf) = crf {
            let val = CString::new(crf.to_string()).unwrap();
            let _ = ffmpeg_sys::av_opt_set(
                (*context.as_ptr()).priv_data,
                c"crf".as_ptr(),
                val.as_ptr(),
                0,
            );
        }

        if let Some(preset) = preset {
            let val = CString::new(preset).unwrap();
            let _ = ffmpeg_sys::av_opt_set(
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
        if ffmpeg_sys::avcodec_open2(context.as_ptr(), codec, ptr::null_mut()) < 0 {
            panic!("Could not open the codec.");
        }

        if ffmpeg_sys::avcodec_parameters_from_context(
            (*video_st.as_ptr()).codecpar,
            context.as_ptr(),
        ) < 0
        {
            panic!("Failed to set codec parameters.");
        }

        // Open the output file.
        if ffmpeg_sys::avio_open(
            &mut (*format_context.as_ptr()).pb,
            path_str.as_ptr(),
            ffmpeg_sys::AVIO_FLAG_WRITE,
        ) < 0
        {
            panic!("Failed to open the output file.");
        }

        if ffmpeg_sys::avformat_write_header(format_context.as_ptr(), ptr::null_mut()) < 0 {
            panic!("Failed to open the output file.");
        }

        context
    }
}

/// MPEG video recorder.
pub struct Encoder {
    tmp_frame_buf: Vec<u8>,
    _frame_buf: Vec<u8>,
    curr_frame_index: usize,
    target_width: usize,
    target_height: usize,
    tmp_frame: NonNull<AVFrame>,
    frame: NonNull<AVFrame>,
    context: NonNull<AVCodecContext>,
    format_context: NonNull<AVFormatContext>,
    video_st: NonNull<AVStream>,
    scale_context: NonNull<SwsContext>,
}

impl Encoder {
    pub fn new(
        path: impl AsRef<Path>,
        width: usize,
        height: usize,
        crf: Option<f32>,
        preset: Option<&str>,
    ) -> Self {
        let path = path.as_ref().to_str().unwrap();
        let path_str = CString::new(path).unwrap();

        let time_base = AVRational { num: 1, den: 60 };

        let gop_size = 10;
        let max_b_frames = 1;
        let pix_fmt = AVPixelFormat::AV_PIX_FMT_YUV420P;

        // width and height must be a multiple of two.
        let target_width = if width % 2 == 0 { width } else { width + 1 };
        let target_height = if height % 2 == 0 { height } else { height + 1 };

        // sws scaling context
        let scale_context = unsafe {
            NonNull::new(ffmpeg_sys::sws_getContext(
                target_width as i32,
                target_height as i32,
                AVPixelFormat::AV_PIX_FMT_RGB24,
                target_width as i32,
                target_height as i32,
                pix_fmt,
                ffmpeg_sys::SWS_BICUBIC,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            ))
            .expect("Failed to create scale context")
        };

        // Init the temporary video frame.
        let tmp_frame = unsafe {
            let frame = NonNull::new(ffmpeg_sys::av_frame_alloc())
                .expect("Could not allocate the video frame.");

            (*frame.as_ptr()).format = pix_fmt as i32;
            // the rest (width, height, data, linesize) are set at the moment of the snapshot.

            frame
        };

        // Init the destination video frame.
        let (frame, frame_buf) = unsafe {
            let frame = NonNull::new(ffmpeg_sys::av_frame_alloc())
                .expect("Could not allocate the video frame.");

            (*frame.as_ptr()).format = pix_fmt as i32;
            (*frame.as_ptr()).width = target_width as i32;
            (*frame.as_ptr()).height = target_height as i32;
            (*frame.as_ptr()).pts = 0;

            // alloc the buffer
            let nframe_bytes = ffmpeg_sys::av_image_get_buffer_size(
                pix_fmt,
                target_width as i32,
                target_height as i32,
                16,
            );

            let frame_buf = vec![0u8; nframe_bytes as usize];

            let _ = ffmpeg_sys::av_image_fill_arrays(
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

            let _ = ffmpeg_sys::avformat_alloc_output_context2(
                &mut fmt,
                ptr::null_mut(),
                ptr::null(),
                path_str.as_ptr(),
            );

            NonNull::new(fmt)
                .or_else(|| {
                    // could not guess, default to MPEG
                    let mpeg = CString::new(&b"mpeg"[..]).unwrap();

                    let _ = ffmpeg_sys::avformat_alloc_output_context2(
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
            let stream = NonNull::new(ffmpeg_sys::avformat_new_stream(
                format_context.as_ptr(),
                ptr::null(),
            ))
            .expect("Failed to allocate the video stream.");

            (*stream.as_ptr()).id = ((*format_context.as_ptr()).nb_streams - 1) as i32;
            (*stream.as_ptr()).time_base = time_base;

            stream
        };

        let context = init_context(
            &path_str,
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

        // Print detailed information about the input or output format, such as duration, bitrate, streams, container, programs, metadata, side data, codec and time base
        unsafe {
            ffmpeg_sys::av_dump_format(format_context.as_ptr(), 0, path_str.as_ptr(), 1);
        }

        Self {
            tmp_frame_buf: Vec::new(),
            curr_frame_index: 0,
            target_width,
            target_height,
            tmp_frame,

            frame,
            _frame_buf: frame_buf,

            context,
            format_context,
            video_st,
            scale_context,
        }
    }

    /// Adds a image with a RGB pixel format to the video.
    pub fn encode_rgb(&mut self, width: usize, height: usize, data: &[u8], vertical_flip: bool) {
        assert_eq!(data.len(), width * height * 3);
        self.encode(width, height, data, ColorFormat::Rgb, vertical_flip)
    }

    /// Adds a image with a RGBA pixel format to the video.
    pub fn encode_rgba(&mut self, width: usize, height: usize, data: &[u8], vertical_flip: bool) {
        assert_eq!(data.len(), width * height * 4);
        self.encode(width, height, data, ColorFormat::Rgba, vertical_flip)
    }

    /// Adds a image with a BGRA pixel format to the video.
    pub fn encode_bgr(&mut self, width: usize, height: usize, data: &[u8], vertical_flip: bool) {
        assert_eq!(data.len(), width * height * 3);
        self.encode(width, height, data, ColorFormat::Bgr, vertical_flip)
    }

    /// Adds a image with a BGRA pixel format to the video.
    pub fn encode_bgra(&mut self, width: usize, height: usize, data: &[u8], vertical_flip: bool) {
        assert_eq!(data.len(), width * height * 4);
        self.encode(width, height, data, ColorFormat::Bgra, vertical_flip)
    }

    fn encode(
        &mut self,
        width: usize,
        height: usize,
        data: &[u8],
        color_format: ColorFormat,
        vertical_flip: bool,
    ) {
        let has_alpha = color_format == ColorFormat::Rgba || color_format == ColorFormat::Bgra;

        assert!(
            (has_alpha && data.len() == width * height * 4)
                || (!has_alpha && data.len() == width * height * 3)
        );

        let mut pkt = unsafe {
            let mut pkt: mem::MaybeUninit<AVPacket> = mem::MaybeUninit::uninit();
            ffmpeg_sys::av_init_packet(pkt.as_mut_ptr());
            pkt.assume_init()
        };

        pkt.data = ptr::null_mut(); // packet data will be allocated by the encoder
        pkt.size = 0;

        // Fill the snapshot frame.
        //
        //
        let pixel_len = if has_alpha { 4 } else { 3 };

        self.tmp_frame_buf.resize(width * height * pixel_len, 0);
        self.tmp_frame_buf.clone_from_slice(data);

        if vertical_flip {
            vflip(self.tmp_frame_buf.as_mut_slice(), width * pixel_len, height);
        }

        unsafe {
            (*self.tmp_frame.as_ptr()).width = width as i32;
            (*self.tmp_frame.as_ptr()).height = height as i32;

            let _ = ffmpeg_sys::av_image_fill_arrays(
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
        //
        unsafe {
            self.scale_context = NonNull::new(ffmpeg_sys::sws_getCachedContext(
                self.scale_context.as_ptr(),
                width as i32,
                height as i32,
                (&color_format).into(),
                self.target_width as i32,
                self.target_height as i32,
                AVPixelFormat::AV_PIX_FMT_YUV420P,
                ffmpeg_sys::SWS_BICUBIC,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            ))
            .unwrap();

            let _ = ffmpeg_sys::sws_scale(
                self.scale_context.as_ptr(),
                &(*self.tmp_frame.as_ptr()).data[0] as *const *mut u8 as *const *const u8,
                &(*self.tmp_frame.as_ptr()).linesize[0],
                0,
                height as i32,
                &(*self.frame.as_ptr()).data[0] as *const *mut u8,
                &(*self.frame.as_ptr()).linesize[0],
            );
        }

        // Encode the image.

        let ret =
            unsafe { ffmpeg_sys::avcodec_send_frame(self.context.as_ptr(), self.frame.as_ptr()) };

        if ret < 0 {
            panic!("Error encoding frame.");
        }

        unsafe {
            if ffmpeg_sys::avcodec_receive_packet(self.context.as_ptr(), &mut pkt) == 0 {
                let _ =
                    ffmpeg_sys::av_interleaved_write_frame(self.format_context.as_ptr(), &mut pkt);
                ffmpeg_sys::av_packet_unref(&mut pkt);
            }
        }

        unsafe {
            (*self.frame.as_ptr()).pts += ffmpeg_sys::av_rescale_q(
                1,
                (*self.context.as_ptr()).time_base,
                (*self.video_st.as_ptr()).time_base,
            );
            self.curr_frame_index += self.curr_frame_index;
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // Get the delayed frames.

        let ret = unsafe { ffmpeg_sys::avcodec_send_frame(self.context.as_ptr(), ptr::null()) };

        if ret < 0 {
            panic!("Error encoding frame.");
        }

        loop {
            let mut pkt = unsafe {
                let mut pkt: mem::MaybeUninit<AVPacket> = mem::MaybeUninit::uninit();
                ffmpeg_sys::av_init_packet(pkt.as_mut_ptr());
                pkt.assume_init()
            };

            pkt.data = ptr::null_mut(); // packet data will be allocated by the encoder
            pkt.size = 0;

            unsafe {
                match ffmpeg_sys::avcodec_receive_packet(self.context.as_ptr(), &mut pkt) {
                    0 => {
                        let _ = ffmpeg_sys::av_interleaved_write_frame(
                            self.format_context.as_ptr(),
                            &mut pkt,
                        );
                        ffmpeg_sys::av_packet_unref(&mut pkt);
                    }
                    ffmpeg_sys::AVERROR_EOF => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Write trailer
        unsafe {
            if ffmpeg_sys::av_write_trailer(self.format_context.as_ptr()) < 0 {
                panic!("Error writing trailer.");
            }
        }

        // Free things and stuffs.
        unsafe {
            ffmpeg_sys::avcodec_free_context(&mut self.context.as_ptr());
            ffmpeg_sys::av_frame_free(&mut self.frame.as_ptr());
            ffmpeg_sys::av_frame_free(&mut self.tmp_frame.as_ptr());
            ffmpeg_sys::sws_freeContext(self.scale_context.as_ptr());
            if ffmpeg_sys::avio_closep(&mut (*self.format_context.as_ptr()).pb) < 0 {
                println!("Warning: failed closing output file");
            }
            ffmpeg_sys::avformat_free_context(self.format_context.as_ptr());
        }
    }
}

fn vflip(vec: &mut [u8], width: usize, height: usize) {
    for j in 0..height / 2 {
        for i in 0..width {
            vec.swap((height - j - 1) * width + i, j * width + i);
        }
    }
}
