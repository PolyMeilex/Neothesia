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

    on_async(open_midi_file_picker_fut(dir), on_midi_file_loaded)
}

pub fn open_midi_file(data: &mut UiState, path: PathBuf) -> BoxFuture<MsgFn> {
    data.is_loading = true;
    on_async(load_midi_file(path), on_midi_file_loaded)
}

pub fn is_midi_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("mid") || ext.eq_ignore_ascii_case("midi"))
}

fn on_midi_file_loaded(
    res: Option<(midi_file::MidiFile, PathBuf)>,
    data: &mut UiState,
    ctx: &mut Context,
) {
    if let Some((midi, path)) = res {
        ctx.config.set_last_opened_song(Some(path));
        data.song = Some(Song::new(midi));
    }
    data.is_loading = false;
}

async fn open_midi_file_picker_fut(dir: Option<PathBuf>) -> Option<(midi_file::MidiFile, PathBuf)> {
    let mut dialog =
        rfd::AsyncFileDialog::new().add_filter("midi", &["mid", "midi", "MID", "MIDI"]);

    if let Some(dir) = dir {
        dialog = dialog.set_directory(dir);
    }

    let file = dialog.pick_file().await;

    if let Some(file) = file {
        load_midi_file(file.path().to_path_buf()).await
    } else {
        log::info!("User canceled dialog");
        None
    }
}

async fn load_midi_file(path: PathBuf) -> Option<(midi_file::MidiFile, PathBuf)> {
    log::info!("File path = {path:?}");

    let thread = crate::utils::task::thread::spawn("midi-loader".into(), move || {
        let midi = midi_file::MidiFile::new(&path);

        if let Err(e) = &midi {
            log::error!("{e}");
        }

        midi.map(|midi| (midi, path)).ok()
    });

    thread.join().await.ok().flatten()
}
