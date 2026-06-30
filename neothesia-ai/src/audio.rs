use std::num::NonZeroU32;
use std::path::Path;

use crate::{SAMPLE_RATE, SEGMENT_SAMPLES};

pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Vec<f32>> {
    let probed = symphonium::probe_from_file(path.as_ref(), None)?;

    let mut audio_data_f32 = symphonium::decode_f32(
        probed,
        &Default::default(),
        NonZeroU32::new(SAMPLE_RATE),
        None,
        None,
    )?;

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
