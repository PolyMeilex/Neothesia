use iced_core::{clipboard::Kind, Clipboard};

pub struct DummyClipboard {}

impl Clipboard for DummyClipboard {
    fn read(&self, _kind: Kind) -> Option<String> {
        None
    }

    fn write(&mut self, _kind: Kind, _contents: String) {}
}
