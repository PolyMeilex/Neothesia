use std::path::Path;

use ::image::{
    self as image_rs,
    error::{ImageError, LimitError, LimitErrorKind},
    ImageBuffer, ImageResult,
};

use bytes::Bytes;

pub fn load_from_path(path: &Path) -> ImageResult<ImageBuffer<image_rs::Rgba<u8>, Bytes>> {
    let (width, height, pixels) = {
        let image = ::image::open(path)?;

        let rgba = image.into_rgba8();

        (rgba.width(), rgba.height(), Bytes::from(rgba.into_raw()))
    };

    if let Some(image) = ImageBuffer::from_raw(width, height, pixels) {
        Ok(image)
    } else {
        Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )))
    }
}

pub fn load_from_bytes(
    bytes: &[u8],
) -> image_rs::ImageResult<ImageBuffer<::image::Rgba<u8>, Bytes>> {
    let (width, height, pixels) = {
        let image = ::image::load_from_memory(bytes)?;
        let rgba = image.into_rgba8();

        (rgba.width(), rgba.height(), Bytes::from(rgba.into_raw()))
    };

    if let Some(image) = ImageBuffer::from_raw(width, height, pixels) {
        Ok(image)
    } else {
        Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )))
    }
}

pub fn load_from_rgba(
    width: u32,
    height: u32,
    pixels: &Bytes,
) -> image_rs::ImageResult<ImageBuffer<::image::Rgba<u8>, Bytes>> {
    let (width, height, pixels) = (width, height, pixels.clone());

    if let Some(image) = ImageBuffer::from_raw(width, height, pixels) {
        Ok(image)
    } else {
        Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )))
    }
}
