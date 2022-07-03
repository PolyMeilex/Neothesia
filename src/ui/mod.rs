mod text_renderer;
pub use text_renderer::TextRenderer;

#[cfg(feature = "app")]
mod iced_clipboard;
#[cfg(feature = "app")]
pub use iced_clipboard::DummyClipboard;
#[cfg(feature = "app")]
pub mod iced_conversion;
#[cfg(feature = "app")]
mod iced_manager;
#[cfg(feature = "app")]
pub use iced_manager::IcedManager;
#[cfg(feature = "app")]
pub mod iced_state;
