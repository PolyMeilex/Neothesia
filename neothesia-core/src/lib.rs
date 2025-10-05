#![allow(clippy::collapsible_match, clippy::single_match)]

pub use dpi;
pub use piano_layout;
pub use wgpu_jumpstart::{Color, Gpu, TransformUniform, Uniform};

pub mod config;
pub mod font_system;
pub mod render;
pub mod utils;

pub use euclid;

pub type Point<T = f32> = euclid::default::Point2D<T>;
pub type Size<T = f32> = euclid::default::Size2D<T>;
pub type Box2D<T = f32> = euclid::default::Box2D<T>;
pub type Rect<T = f32> = euclid::default::Rect<T>;
