//! Load and operate on images.
#[cfg(feature = "image")]
pub use ::image as image_rs;

use crate::core::image;
use crate::core::svg;
use crate::core::Rectangle;

/// A raster or vector image.
#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    /// A raster image.
    Raster(image::Image, Rectangle),

    /// A vector image.
    Vector(svg::Svg, Rectangle),
}

impl Image {
    /// Returns the bounds of the [`Image`].
    pub fn bounds(&self) -> Rectangle {
        match self {
            Image::Raster(image, bounds) => bounds.rotate(image.rotation),
            Image::Vector(svg, bounds) => bounds.rotate(svg.rotation),
        }
    }
}

#[cfg(feature = "image")]
/// Tries to load an image by its [`Handle`].
///
/// [`Handle`]: image::Handle
pub fn load(
    handle: &image::Handle,
) -> ::image::ImageResult<::image::ImageBuffer<::image::Rgba<u8>, image::Bytes>> {
    let (width, height, pixels) = match handle {
        image::Handle::Path(_, path) => {
            let image = ::image::open(path)?;

            let rgba = image.into_rgba8();

            (
                rgba.width(),
                rgba.height(),
                image::Bytes::from(rgba.into_raw()),
            )
        }
        image::Handle::Bytes(_, bytes) => {
            let image = ::image::load_from_memory(bytes)?;

            let rgba = image.into_rgba8();

            (
                rgba.width(),
                rgba.height(),
                image::Bytes::from(rgba.into_raw()),
            )
        }
        image::Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => (*width, *height, pixels.clone()),
    };

    if let Some(image) = ::image::ImageBuffer::from_raw(width, height, pixels) {
        Ok(image)
    } else {
        Err(::image::error::ImageError::Limits(
            ::image::error::LimitError::from_kind(::image::error::LimitErrorKind::DimensionError),
        ))
    }
}
