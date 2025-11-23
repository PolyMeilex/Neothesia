use std::{
    ffi::CStr,
    ptr::{self, NonNull},
};

use ffmpeg::{
    AVChannelLayout, AVCodec, AVCodecContext, AVCodecID, AVDictionary, AVFormatContext, AVFrame,
    AVOutputFormat, AVPacket, AVPixelFormat, AVRational, AVSampleFormat, AVStream,
};

pub struct FormatContext(NonNull<AVFormatContext>);

impl Drop for FormatContext {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::avio_closep(&mut (*self.0.as_ptr()).pb);
            ffmpeg::avformat_free_context(self.0.as_ptr());
        }
    }
}

impl FormatContext {
    pub fn new(path: &CStr) -> Self {
        let mut output_context = ptr::null_mut();

        unsafe {
            ffmpeg::avformat_alloc_output_context2(
                &mut output_context,
                ptr::null_mut(),
                ptr::null(),
                path.as_ptr(),
            );

            Self(
                NonNull::new(output_context)
                    .or_else(|| {
                        ffmpeg::avformat_alloc_output_context2(
                            &mut output_context,
                            ptr::null_mut(),
                            ptr::null(),
                            c"mpeg".as_ptr(),
                        );
                        NonNull::new(output_context)
                    })
                    .expect("Unable to create the output context."),
            )
        }
    }

    #[allow(unused)]
    pub fn as_ptr(&self) -> *mut AVFormatContext {
        self.0.as_ptr()
    }

    pub fn open(&self, path: impl AsRef<CStr>) {
        unsafe {
            // open the output file, if needed
            if ffmpeg::avio_open(
                &mut (*self.0.as_ptr()).pb,
                path.as_ref().as_ptr(),
                ffmpeg::AVIO_FLAG_WRITE,
            ) < 0
            {
                panic!("Failed to open the output file.");
            }
        }
    }

    /// Write the compressed frame to the media file.
    pub fn interleaved_write_frame(&self, packet: &Packet) {
        unsafe {
            if ffmpeg::av_interleaved_write_frame(self.0.as_ptr(), packet.as_ptr()) < 0 {
                panic!("Error while writing output packet",);
            }
        }
    }

    pub fn write_trailer(&self) {
        unsafe {
            ffmpeg::av_write_trailer(self.0.as_ptr());
        }
    }

    pub fn write_header(&self) {
        unsafe {
            // Write the stream header, if any.
            if ffmpeg::avformat_write_header(self.0.as_ptr(), ptr::null_mut()) < 0 {
                panic!("Failed to open the output file.");
            }
        }
    }

    pub fn output_format(&self) -> OutputFormat {
        OutputFormat(unsafe { (*self.0.as_ptr()).oformat })
    }

    pub fn new_stream(&self) -> Option<Stream> {
        unsafe {
            let stream = ffmpeg::avformat_new_stream(self.0.as_ptr(), ptr::null_mut());
            let stream = NonNull::new(stream)?;

            (*stream.as_ptr()).id = ((*self.0.as_ptr()).nb_streams - 1) as i32;

            Some(Stream(stream))
        }
    }

    pub fn dump_format(&self, path: impl AsRef<CStr>) {
        unsafe {
            ffmpeg::av_dump_format(self.0.as_ptr(), 0, path.as_ref().as_ptr(), 1);
        }
    }
}

pub struct Stream(NonNull<AVStream>);

impl Stream {
    pub fn as_ptr(&self) -> *mut AVStream {
        self.0.as_ptr()
    }

    pub fn set_time_base(&self, time_base: AVRational) {
        unsafe { (*self.0.as_ptr()).time_base = time_base };
    }

    pub fn time_base(&self) -> AVRational {
        unsafe { (*self.0.as_ptr()).time_base }
    }

    pub fn index(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).index }
    }
}

pub struct OutputFormat(*const AVOutputFormat);

impl OutputFormat {
    pub fn as_ptr(&self) -> *const AVOutputFormat {
        self.0
    }

    pub fn video_codec_id(&self) -> AVCodecID {
        unsafe { (*self.0).video_codec }
    }

    pub fn audio_codec_id(&self) -> AVCodecID {
        unsafe { (*self.0).audio_codec }
    }

    pub fn video_codec(&self) -> Codec {
        let codec_id = self.video_codec_id();
        let codec = unsafe { ffmpeg::avcodec_find_encoder(codec_id) };
        assert!(!codec.is_null(), "Could not find video encoder");
        Codec(codec)
    }

    pub fn audio_codec(&self) -> Codec {
        let codec_id = self.audio_codec_id();
        let codec = unsafe { ffmpeg::avcodec_find_encoder(codec_id) };
        assert!(!codec.is_null(), "Could not find audio encoder");
        Codec(codec)
    }
}

pub struct Codec(*const AVCodec);

impl Codec {
    pub fn as_ptr(&self) -> *const AVCodec {
        self.0
    }

    pub fn context(&self) -> CodecContext {
        let codec_context = unsafe { ffmpeg::avcodec_alloc_context3(self.0) };
        CodecContext(NonNull::new(codec_context).expect("Could not alloc an encoding context"))
    }

    pub fn capabilities(&self) -> i32 {
        unsafe { (*self.0).capabilities }
    }
}

pub struct CodecContext(NonNull<AVCodecContext>);

impl Drop for CodecContext {
    fn drop(&mut self) {
        unsafe { ffmpeg::avcodec_free_context(&mut self.0.as_ptr()) };
    }
}

impl CodecContext {
    pub fn as_ptr(&self) -> *mut AVCodecContext {
        self.0.as_ptr()
    }

    pub fn open_video(&self) {
        unsafe {
            let mut opt: *mut AVDictionary = ptr::null_mut();

            // The range of the CRF scale is 0–51, where 0 is lossless
            ffmpeg::av_dict_set(&mut opt, c"crf".as_ptr(), c"0".as_ptr(), 0);
            ffmpeg::av_dict_set(&mut opt, c"preset".as_ptr(), c"medium".as_ptr(), 0);

            if ffmpeg::avcodec_open2(self.0.as_ptr(), ptr::null_mut(), &mut opt) < 0 {
                panic!("Could not open video codec.");
            }

            ffmpeg::av_dict_free(&mut opt);
        }
    }

    pub fn open(&self) {
        unsafe {
            if ffmpeg::avcodec_open2(self.0.as_ptr(), ptr::null_mut(), &mut ptr::null_mut()) < 0 {
                panic!("Could not open video codec.");
            }
        }
    }

    pub fn send_frame(&self, frame: Option<&Frame>) {
        if unsafe {
            ffmpeg::avcodec_send_frame(
                self.0.as_ptr(),
                frame.map(Frame::as_const_ptr).unwrap_or(ptr::null()),
            )
        } < 0
        {
            panic!("Error sending a frame to the encoder",);
        }
    }

    pub fn receive_packet(&self, packet: &Packet) -> i32 {
        unsafe { ffmpeg::avcodec_receive_packet(self.0.as_ptr(), packet.as_ptr()) }
    }

    pub fn copy_parameters_to_stream(&self, stream: &Stream) {
        let codecpar = unsafe { (*stream.as_ptr()).codecpar };
        unsafe {
            // copy the stream parameters to the muxer
            if ffmpeg::avcodec_parameters_from_context(codecpar, self.0.as_ptr()) < 0 {
                panic!("Could not copy the stream parameters");
            }
        }
    }

    pub fn frame_size(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).frame_size }
    }

    pub fn sample_rate(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).sample_rate }
    }

    pub fn sample_fmt(&self) -> AVSampleFormat {
        unsafe { (*self.0.as_ptr()).sample_fmt }
    }

    pub fn channel_layout(&self) -> &AVChannelLayout {
        unsafe { &(*self.0.as_ptr()).ch_layout }
    }

    pub fn channel_layout_mut(&mut self) -> &mut AVChannelLayout {
        unsafe { &mut (*self.0.as_ptr()).ch_layout }
    }

    pub fn time_base(&self) -> AVRational {
        unsafe { (*self.0.as_ptr()).time_base }
    }

    pub fn pix_fmt(&self) -> AVPixelFormat {
        unsafe { (*self.0.as_ptr()).pix_fmt }
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).height }
    }
}

pub struct Packet(NonNull<AVPacket>);

impl Drop for Packet {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::av_packet_free(&mut self.0.as_ptr());
        }
    }
}

impl Packet {
    pub fn new() -> Self {
        let packet = unsafe { ffmpeg::av_packet_alloc() };
        Self(NonNull::new(packet).expect("Could not allocate AVPacket"))
    }

    pub fn as_ptr(&self) -> *mut AVPacket {
        self.0.as_ptr()
    }

    pub fn rescale_ts(&self, src: AVRational, dst: AVRational) {
        unsafe {
            ffmpeg::av_packet_rescale_ts(self.0.as_ptr(), src, dst);
        }
    }

    pub fn set_stream_index(&self, index: i32) {
        unsafe {
            (*self.0.as_ptr()).stream_index = index;
        }
    }
}

pub struct Frame(NonNull<AVFrame>);

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { ffmpeg::av_frame_free(&mut self.0.as_ptr()) };
    }
}

impl Frame {
    pub fn as_ptr(&self) -> *mut AVFrame {
        self.0.as_ptr()
    }

    pub fn as_const_ptr(&self) -> *const AVFrame {
        self.0.as_ptr() as *const AVFrame
    }

    pub fn new_raw() -> Self {
        let frame = unsafe { ffmpeg::av_frame_alloc() };
        let frame = NonNull::new(frame).expect("Could not allocate frame.");
        Self(frame)
    }

    pub fn new_video(pix_fmt: AVPixelFormat, width: i32, height: i32) -> Self {
        let frame = Frame::new_raw();

        unsafe {
            let frame = frame.as_ptr();
            (*frame).format = pix_fmt as i32;
            (*frame).width = width;
            (*frame).height = height;

            /* allocate the buffers for the frame data */
            if ffmpeg::av_frame_get_buffer(frame, 0) < 0 {
                panic!("Could not allocate frame data.");
            }
        }

        frame
    }

    pub fn new_audio(
        sample_fmt: AVSampleFormat,
        channel_layout: *const AVChannelLayout,
        sample_rate: i32,
        nb_samples: i32,
    ) -> Self {
        let frame = Frame::new_raw();

        unsafe {
            let frame = frame.as_ptr();
            (*frame).format = sample_fmt as i32;
            ffmpeg::av_channel_layout_copy(&mut (*frame).ch_layout, channel_layout);
            (*frame).sample_rate = sample_rate;
            (*frame).nb_samples = nb_samples;

            /* allocate the buffers for the frame data */
            if nb_samples > 0 && ffmpeg::av_frame_get_buffer(frame, 0) < 0 {
                panic!("Error allocating an audio buffer");
            }
        }

        frame
    }

    pub fn make_writable(&self) {
        unsafe {
            if ffmpeg::av_frame_make_writable(self.0.as_ptr()) < 0 {
                panic!("Could not make frame writable");
            }
        }
    }

    pub fn image_fill_arrays(&self, frame_bytes: &[u8], pix_fmt: AVPixelFormat) {
        unsafe {
            ffmpeg::av_image_fill_arrays(
                (*self.0.as_ptr()).data.as_mut_ptr(),
                (*self.0.as_ptr()).linesize.as_mut_ptr(),
                frame_bytes.as_ptr(),
                pix_fmt,
                self.width(),
                self.height(),
                1,
            );
        }
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.0.as_ptr()).height }
    }

    #[allow(unused)]
    pub fn presentation_timestamp(&self) -> i64 {
        unsafe { (*self.0.as_ptr()).pts }
    }

    pub fn set_presentation_timestamp(&self, pts: i64) {
        unsafe {
            (*self.0.as_ptr()).pts = pts;
        }
    }
}

pub struct SwrContext(NonNull<ffmpeg::SwrContext>);

impl Drop for SwrContext {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::swr_free(&mut self.0.as_ptr());
        };
    }
}

#[allow(unused)]
impl SwrContext {
    pub fn new() -> Self {
        let swr = unsafe { ffmpeg::swr_alloc() };
        let swr = NonNull::new(swr).expect("Could not allocate resampler context");

        Self(swr)
    }

    pub fn as_ptr(&self) -> *mut ffmpeg::SwrContext {
        self.0.as_ptr()
    }

    pub fn init(&self) {
        if unsafe { ffmpeg::swr_init(self.0.as_ptr()) } < 0 {
            panic!("Failed to initialize the resampling context");
        }
    }
}

pub struct SwsContext(NonNull<ffmpeg::SwsContext>);

impl Drop for SwsContext {
    fn drop(&mut self) {
        unsafe { ffmpeg::sws_freeContext(self.0.as_ptr()) };
    }
}

impl SwsContext {
    pub fn new(
        src_width: i32,
        src_height: i32,
        src_pix_fmt: AVPixelFormat,
        dist_width: i32,
        dist_height: i32,
        dist_pix_fmt: AVPixelFormat,
    ) -> Self {
        let ctx = unsafe {
            ffmpeg::sws_getContext(
                src_width,
                src_height,
                src_pix_fmt,
                dist_width,
                dist_height,
                dist_pix_fmt,
                // TODO:
                // - ffmpeg::SWS_BICUBIC,
                // + ffmpeg_sys::SwsFlags::SWS_BICUBIC as i32,
                4,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        Self(NonNull::new(ctx).expect("Could not initialize the conversion context"))
    }

    #[allow(unused)]
    pub fn as_ptr(&self) -> *mut ffmpeg::SwsContext {
        self.0.as_ptr()
    }

    pub fn scale(&self, src_frame: &Frame, dest_frame: &Frame, height: i32) {
        unsafe {
            ffmpeg::sws_scale(
                self.0.as_ptr(),
                (*src_frame.as_ptr()).data.as_ptr() as *const *const u8,
                (*src_frame.as_ptr()).linesize.as_ptr(),
                0,
                height,
                (*dest_frame.as_ptr()).data.as_mut_ptr(),
                (*dest_frame.as_ptr()).linesize.as_ptr(),
            );
        }
    }
}
