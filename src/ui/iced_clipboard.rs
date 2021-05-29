use iced_native::Clipboard;

pub struct DumyClipboard {}

impl Clipboard for DumyClipboard {
    fn read(&self) -> Option<String> {
        None
    }

    fn write(&mut self, _contents: String) {}
}
