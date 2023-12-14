use std::path::PathBuf;

use iced_runtime::Command;

use crate::{song::Song, target::Target};

use super::{Data, Message};

#[derive(Debug, Clone)]
pub enum MidiFilePickerMessage {
    OpenMidiFilePicker,
    MidiFileLoaded(Option<(midi_file::MidiFile, PathBuf)>),
}

impl From<MidiFilePickerMessage> for Message {
    fn from(msg: MidiFilePickerMessage) -> Self {
        Message::MidiFilePicker(msg)
    }
}

pub(super) fn update(
    data: &mut Data,
    msg: MidiFilePickerMessage,
    target: &mut Target,
) -> Command<Message> {
    match msg {
        MidiFilePickerMessage::OpenMidiFilePicker => {
            data.is_loading = true;
            return open_midi_file_picker(|v| MidiFilePickerMessage::MidiFileLoaded(v).into());
        }
        MidiFilePickerMessage::MidiFileLoaded(midi) => {
            if let Some((midi, path)) = midi {
                target.config.last_opened_song = Some(path);
                target.song = Some(Song::new(midi));
            }
            data.is_loading = false;
        }
    }

    Command::none()
}

pub(super) fn open() -> MidiFilePickerMessage {
    MidiFilePickerMessage::OpenMidiFilePicker
}

fn open_midi_file_picker(
    f: impl FnOnce(Option<(midi_file::MidiFile, PathBuf)>) -> Message + 'static + Send,
) -> Command<Message> {
    Command::perform(
        async {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("midi", &["mid", "midi"])
                .pick_file()
                .await;

            if let Some(file) = file {
                log::info!("File path = {:?}", file.path());

                let thread = async_thread::Builder::new()
                    .name("midi-loader".into())
                    .spawn(move || {
                        let midi = midi_file::MidiFile::new(file.path());

                        if let Err(e) = &midi {
                            log::error!("{}", e);
                        }

                        midi.map(|midi| (midi, file.path().to_path_buf())).ok()
                    });

                if let Ok(thread) = thread {
                    thread.join().await.ok().flatten()
                } else {
                    None
                }
            } else {
                log::info!("User canceled dialog");
                None
            }
        },
        f,
    )
}
