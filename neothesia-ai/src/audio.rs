use std::path::Path;

use symphonium::{ResampleQuality, SymphoniumLoader};

use crate::{SAMPLE_RATE, SEGMENT_SAMPLES};

pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Vec<f32>> {
    // A struct used to load audio files.
    let mut loader = SymphoniumLoader::new();

    let mut audio_data_f32 = loader
        .load_f32(path, Some(SAMPLE_RATE), ResampleQuality::High, None)
        .unwrap();

    let left = audio_data_f32.data.remove(0);
    let right = audio_data_f32.data.remove(0);

    let mut mono: Vec<f32> = left
        .into_iter()
        .zip(right)
        .map(|(l, r)| (l + r) / 2.0)
        .collect();

    let pad_len =
        (mono.len() as f32 / SEGMENT_SAMPLES as f32).ceil() as usize * SEGMENT_SAMPLES - mono.len();

    mono.resize(mono.len() + pad_len, 0.0);

    Ok(mono)
}
