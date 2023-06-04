mod midi;
pub mod playback;
mod track;
mod utils;

pub use midly;
pub use {midi::*, playback::*, track::*, utils::*};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load() {
        let midi = Midi::new("../test.mid").unwrap();

        for (_id, _note) in midi.merged_track.notes.iter().enumerate() {
            // println!("{id}: {}", note.start.as_micros(),);
        }
    }
}
