use std::num::NonZeroU32;
use std::path::Path;

/// Decode audio file and resample it to `sample_rate`, returns samples of each channel
pub fn load(path: impl AsRef<Path>, sample_rate: u32) -> anyhow::Result<Vec<Vec<f32>>> {
    let probed = symphonium::probe_from_file(path.as_ref(), None)?;

    let audio_data_f32 = symphonium::decode_f32(
        probed,
        &Default::default(),
        NonZeroU32::new(sample_rate),
        None,
        None,
    )?;

    anyhow::ensure!(
        !audio_data_f32.data.is_empty(),
        "audio file has no channels"
    );

    Ok(audio_data_f32.data)
}

pub fn to_mono(channels: &[Vec<f32>]) -> Vec<f32> {
    let len = channels.iter().map(Vec::len).min().unwrap_or(0);
    let count = channels.len() as f32;

    (0..len)
        .map(|i| channels.iter().map(|c| c[i]).sum::<f32>() / count)
        .collect()
}
