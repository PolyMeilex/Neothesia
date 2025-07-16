use std::{future::Future, pin::Pin};

pub use neothesia_core::utils::*;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub mod task;
pub mod window;
