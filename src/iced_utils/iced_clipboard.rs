use iced_native::Clipboard;

pub struct DummyClipboard {}

impl Clipboard for DummyClipboard {
    fn read(&self) -> Option<String> {
        None
    }

    fn write(&mut self, _contents: String) {}
}
