use png::{BitDepth, ColorType, Decoder, Transformations};

pub fn load_png(
    file: impl std::io::BufRead + std::io::Seek,
) -> Result<(Vec<u8>, u32, u32), png::DecodingError> {
    let mut decoder = Decoder::new(file);

    decoder.set_transformations(
        Transformations::EXPAND | Transformations::STRIP_16 | Transformations::ALPHA,
    );

    let mut reader = decoder.read_info()?;

    let mut out_buf = vec![0u8; reader.output_buffer_size().unwrap()];
    let out_info = reader.next_frame(&mut out_buf)?;

    let filled = &out_buf[..out_info.buffer_size()];

    let (output_color, output_depth) = reader.output_color_type();

    assert_eq!(
        output_depth,
        BitDepth::Eight,
        "Only BitDepth of 8 supported"
    );

    let rgba = match output_color {
        ColorType::Rgba => filled.to_vec(),
        ColorType::Rgb => {
            let mut v = Vec::with_capacity((filled.len() / 3) * 4);
            for chunk in filled.chunks_exact(3) {
                v.extend_from_slice(chunk);
                v.push(255);
            }
            v
        }
        ColorType::Grayscale => {
            unimplemented!("Grayscale png not supported");
        }
        ColorType::GrayscaleAlpha => {
            unimplemented!("GrayscaleAlpha png not supported");
        }
        ColorType::Indexed => {
            // With EXPAND we normally won't get Indexed?
            unimplemented!("Indexed png not supported");
        }
    };

    Ok((rgba, out_info.width, out_info.height))
}
