use std::path::{Path, PathBuf};

use crate::{
    context::Context,
    scene::menu_scene::{MsgFn, on_async},
    song::Song,
    utils::BoxFuture,
};

use super::UiState;

pub fn open_midi_file_picker(data: &mut UiState, ctx: &Context) -> BoxFuture<MsgFn> {
    data.is_loading = true;

    let dir = ctx
        .config
        .last_opened_song()
        .and_then(|song| song.parent())
        .map(Path::to_path_buf);

    on_async(open_midi_file_picker_fut(dir), |res, data, ctx| {
        if let Some((midi, path)) = res {
            data.song = Some(Song::from_path(midi, &path));
            ctx.config.set_last_opened_song(Some(path));
        }
        data.is_loading = false;
    })
}

async fn open_midi_file_picker_fut(dir: Option<PathBuf>) -> Option<(midi_file::MidiFile, PathBuf)> {
    let mut dialog =
        rfd::AsyncFileDialog::new().add_filter("midi", &["mid", "midi", "MID", "MIDI"]);

    if let Some(dir) = dir {
        dialog = dialog.set_directory(dir);
    }

    let file = dialog.pick_file().await;

    if let Some(file) = file {
        log::info!("File path = {:?}", file.path());

        let thread = crate::utils::task::thread::spawn("midi-loader".into(), move || {
            let midi = midi_file::MidiFile::new(file.path());

            if let Err(e) = &midi {
                log::error!("{e}");
            }

            midi.map(|midi| (midi, file.path().to_path_buf())).ok()
        });

        thread.join().await.ok().flatten()
    } else {
        log::info!("User canceled dialog");
        None
    }
}
