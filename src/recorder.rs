// use ffmpeg_next as ffmpeg;

// use ffmpeg::{
//     codec::{id::Id, Context},
//     ffi,
//     util::frame::Frame,
// };

// use gif::SetParameter;
// use std::fs::File;

// use image::{Bgra, ImageBuffer};

use mpeg_encoder::Encoder;

pub struct Recorder {
    // pub images: Vec<ImageBuffer<Bgra<u8>, &'a [u8]>>,
    // pub decoder: gif::Decoder<File>,
    pub encoder: Encoder,
}

impl Recorder {
    pub fn new() -> Self {
        // let mut codec = ffmpeg_next::codec::encoder::find(Id::MPEG4).unwrap();

        // unsafe {
        //     let mut ctx = Context::wrap(ffi::avcodec_alloc_context3(codec.as_mut_ptr()), None);

        //     let ctx = ctx.as_mut_ptr();

        //     (*ctx).codec_id = Id::MPEG4.into();

        //     let frame = Frame::empty();

        //     Self {} }
        let encoder = Encoder::new(
            "/home/poly/Documents/Programing/rust/Neothesia/out/test.mp4",
            1920,
            1080,
        );
        Self { encoder }
    }
}
