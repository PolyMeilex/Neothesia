use bytes::Bytes;

mod load;
pub use load::{load_from_bytes, load_from_path, load_from_rgba};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ImageIdentifier {
    Ptr(usize),
}

impl ImageIdentifier {
    pub fn from_bytes_ptr(bytes: &Bytes) -> Self {
        Self::from_ptr(bytes.as_ptr())
    }

    pub fn from_ptr(ptr: *const u8) -> Self {
        Self::Ptr(ptr as usize)
    }
}
