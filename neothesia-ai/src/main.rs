use ndarray::{Array2, Array3, ArrayView1, ArrayView2, Axis, concatenate, s};
use rten::{NodeId, ValueOrView};
use rten_tensor::{prelude::*, *};

const FRAMES_PER_SECOND: f32 = 100.0;
const SAMPLE_RATE: u32 = 16000;
const SEGMENT_SAMPLES: usize = SAMPLE_RATE as usize * 10;

mod args;
mod audio;

fn main() -> anyhow::Result<()> {
    let args = args::Args::get_from_env()?;

    let input = audio::load(&args.input)?;

    let input = ArrayView2::from_shape([1, input.len()], &input)?;
    let input = enframe(&input, SEGMENT_SAMPLES);
    let input = input.as_slice().unwrap().to_vec();

    let input = Tensor::from_data(&[input.len() / SEGMENT_SAMPLES, SEGMENT_SAMPLES], input);

    let model = rten::Model::load_file(&args.model)?;

    let inputs: Vec<(NodeId, ValueOrView)> = vec![(model.input_ids()[0], input.view().into())];

    let [
        reg_onset_output,
        reg_offset_output,
        frame_output,
        _velocity_output,
        _reg_pedal_onset_output,
        _reg_pedal_offset_output,
        _pedal_frame_output,
    ] = model.run_n::<7>(inputs, model.output_ids().try_into()?, None)?;

    let (onset_output, onset_shift_output) = {
        let output = reg_onset_output.into_tensor::<f32>().unwrap();
        let shape: [usize; 3] = output.shape().try_into().unwrap();
        let reg_onset_output = Array3::from_shape_vec(shape, output.to_vec()).unwrap();
        let reg_onset_output: Array2<_> = deframe(&reg_onset_output);

        let onset_threshold = 0.3;
        get_binarized_output_from_regression(&reg_onset_output.view(), onset_threshold, 2)
    };

    let (offset_output, offset_shift_output) = {
        let output = reg_offset_output.into_tensor::<f32>().unwrap();
        let shape: [usize; 3] = output.shape().try_into().unwrap();
        let reg_offset_output: Array3<_> = Array3::from_shape_vec(shape, output.to_vec()).unwrap();
        let reg_offset_output: Array2<_> = deframe(&reg_offset_output);

        let offset_threshold = 0.2;
        get_binarized_output_from_regression(&reg_offset_output.view(), offset_threshold, 4)
    };

    let frame_output: Array3<_> = {
        let output = frame_output.into_tensor::<f32>().unwrap();
        let shape: [usize; 3] = output.shape().try_into().unwrap();
        Array3::from_shape_vec(shape, output.to_vec()).unwrap()
    };
    let frame_output: Array2<_> = deframe(&frame_output);

    let frame_threshold = 0.1;

    let file = note_detection_with_onset_offset_regress(
        frame_output.view(),
        onset_output.view(),
        onset_shift_output.view(),
        offset_output.view(),
        offset_shift_output.view(),
        (), // velocity_output,
        frame_threshold,
    );

    file.save(args.output)?;

    Ok(())
}

fn enframe(x: &ArrayView2<f32>, segment_samples: usize) -> Array2<f32> {
    // Ensure that the number of audio samples is divisible by segment_samples
    assert!(x.shape()[1].is_multiple_of(segment_samples));

    let mut batch: Vec<Array2<f32>> = Vec::new();
    let mut pointer = 0;

    let total_samples = x.shape()[1];

    // Enframe the sequence into smaller segments
    while pointer + segment_samples <= total_samples {
        let segment = x
            .slice(s![.., pointer..(pointer + segment_samples)])
            .to_owned();
        batch.push(segment);
        pointer += segment_samples / 2;
    }

    // Concatenate the segments along the first axis (the segment axis)
    concatenate(Axis(0), &batch.iter().map(|a| a.view()).collect::<Vec<_>>()).unwrap()
}

// TODO: Rewrite this madness
fn deframe(x: &Array3<f32>) -> Array2<f32> {
    // Get the shape of the input (N, segment_frames, classes_num)
    let (n_segments, segment_frames, _classes_num) = x.dim();

    // If there is only one segment, return it as is (removing the outer dimension)
    if n_segments == 1 {
        return x.index_axis(Axis(0), 0).to_owned(); // Equivalent to `x[0]` in Python
    }

    // Remove the last frame from each segment
    let x = x.slice(s![.., 0..segment_frames - 1, ..]).to_owned();

    // Ensure that segment_frames is divisible by 4
    let segment_samples = segment_frames - 1;
    assert!(segment_samples % 4 == 0);

    // Collect segments into a vector to concatenate them later
    let mut y: Vec<Array2<f32>> = Vec::new();

    // Append the first 75% of the first segment
    y.push(x.slice(s![0, 0..(segment_samples * 3 / 4), ..]).to_owned());

    // Append the middle part (25% to 75%) of the middle segments
    for i in 1..(n_segments - 1) {
        y.push(
            x.slice(s![i, (segment_samples / 4)..(segment_samples * 3 / 4), ..])
                .to_owned(),
        );
    }

    // Append the last 75% of the last segment
    y.push(
        x.slice(s![n_segments - 1, (segment_samples / 4).., ..])
            .to_owned(),
    );

    // Concatenate all parts along the first axis (frames axis)
    concatenate(Axis(0), &y.iter().map(|a| a.view()).collect::<Vec<_>>()).unwrap()
}

fn get_binarized_output_from_regression(
    reg_output: &ArrayView2<f32>,
    threshold: f32,
    neighbour: usize,
) -> (Array2<bool>, Array2<f32>) {
    let (frames_num, classes_num) = reg_output.dim();

    let mut binary_output = Array2::<bool>::default((frames_num, classes_num));
    let mut shift_output = Array2::<f32>::zeros((frames_num, classes_num));

    for k in 0..classes_num {
        let x: ArrayView1<f32> = reg_output.slice(ndarray::s![.., k]);

        for n in neighbour..(frames_num - neighbour) {
            if x[n] > threshold && is_monotonic_neighbour(&x, n, neighbour) {
                binary_output[[n, k]] = true;

                // See Section III-D in [1] for deduction.
                // [1] Q. Kong, et al., High-resolution Piano Transcription
                // with Pedals by Regressing Onsets and Offsets Times, 2020.
                let shift = if x[n - 1] > x[n + 1] {
                    (x[n + 1] - x[n - 1]) / (x[n] - x[n + 1]) / 2.0
                } else {
                    (x[n + 1] - x[n - 1]) / (x[n] - x[n - 1]) / 2.0
                };
                shift_output[[n, k]] = shift;
            }
        }
    }

    (binary_output, shift_output)
}

fn is_monotonic_neighbour(x: &ArrayView1<f32>, n: usize, neighbour: usize) -> bool {
    // Ensure the value of 'n' is within a valid range
    if n < neighbour || n + neighbour >= x.len() {
        todo!();
    }

    for i in 0..neighbour {
        if x[n - i] < x[n - i - 1] {
            return false;
        }
        if x[n + i] < x[n + i + 1] {
            return false;
        }
    }

    true
}

fn note_detection_with_onset_offset_regress(
    frame: ArrayView2<f32>,
    onset: ArrayView2<bool>,
    onset_shift: ArrayView2<f32>,
    offset: ArrayView2<bool>,
    offset_shift: ArrayView2<f32>,
    velocity: (),
    frame_threshold: f32,
) -> midly::Smf<'static> {
    let classes_num = frame.dim().1;

    let mut notes = Vec::new();
    for piano_note in 0..classes_num {
        let res = note_detection_with_onset_offset_regress_inner(
            frame.slice(ndarray::s![.., piano_note]),
            onset.slice(ndarray::s![.., piano_note]),
            onset_shift.slice(ndarray::s![.., piano_note]),
            offset.slice(ndarray::s![.., piano_note]),
            offset_shift.slice(ndarray::s![.., piano_note]),
            velocity,
            frame_threshold,
        );

        for (bgn, fin, bgn_shift, fin_shift) in res {
            let onset_time = (bgn as f32 + bgn_shift) / FRAMES_PER_SECOND;
            let offset_time = (fin as f32 + fin_shift) / FRAMES_PER_SECOND;

            let labels: [&str; 12] = [
                "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "H",
            ];

            let label = labels[(piano_note + 9) % labels.len()];

            // 21 is the first note in 88 keys layout
            let piano_note = piano_note + 21;

            notes.push((piano_note, onset_time, offset_time));
            println!("{piano_note} {label}: {onset_time} - {offset_time}");
        }
    }

    create_midi_file(notes)
}

fn note_detection_with_onset_offset_regress_inner(
    frame: ArrayView1<f32>,
    onset: ArrayView1<bool>,
    onset_shift: ArrayView1<f32>,
    offset: ArrayView1<bool>,
    offset_shift: ArrayView1<f32>,
    _velocity: (),
    frame_threshold: f32,
) -> Vec<(usize, usize, f32, f32)> {
    let iter = frame
        .into_iter()
        .zip(onset)
        .zip(onset_shift)
        .zip(offset)
        .zip(offset_shift)
        .enumerate()
        // God forgive my sins
        .map(|(id, ((((a, b), c), d), e))| (id, a, b, c, d, e));

    let mut output_tuples = Vec::new();
    let mut bgn: Option<(usize, f32)> = None;
    let mut frame_disappear: Option<(usize, f32)> = None;
    let mut offset_occur: Option<(usize, f32)> = None;

    let len = onset.shape()[0];

    for (i, frame, onset, onset_shift, offset, offset_shift) in iter {
        if *onset {
            // Onset detected
            if let Some((bgn, bgn_offset)) = bgn {
                // Consecutive onsets. E.g., pedal is not released, but two
                // consecutive notes being played.
                let fin = i.saturating_sub(1);
                output_tuples.push((bgn, fin, bgn_offset, 0.0));

                frame_disappear = None;
                offset_occur = None;
            }

            bgn = Some((i, *onset_shift));
        }

        if let Some((bgn_time, bgn_shift)) = bgn
            && i > bgn_time
        {
            // If onset found, then search offset

            if *frame <= frame_threshold && frame_disappear.is_none() {
                // Frame disappear detected
                frame_disappear = Some((i, *offset_shift));
            }

            if *offset && offset_occur.is_none() {
                // Offset detected
                offset_occur = Some((i, *offset_shift));
            }

            if let Some((frame_disappear_time, frame_disappear_shift)) = frame_disappear {
                let (fin, fin_shift) = match offset_occur {
                    Some((offset_occur, shift))
                        if offset_occur - bgn_time > frame_disappear_time - offset_occur =>
                    {
                        // bgn --------- offset_occur --- frame_disappear
                        (offset_occur, shift)
                    }
                    _ => {
                        // bgn --- offset_occur --------- frame_disappear
                        (frame_disappear_time, frame_disappear_shift)
                    }
                };
                output_tuples.push((bgn_time, fin, bgn_shift, fin_shift));

                bgn = None;
                frame_disappear = None;
                offset_occur = None;
            }

            if let Some((bgn_time, bgn_shift)) = bgn
                && (i - bgn_time >= 600 || i == len - 1)
            {
                // Offset not detected
                let fin = i;
                output_tuples.push((bgn_time, fin, bgn_shift, *offset_shift));

                bgn = None;
                frame_disappear = None;
                offset_occur = None;
            }
        }
    }

    output_tuples.sort_by_key(|v| v.0);

    output_tuples
}

fn create_midi_file(notes: Vec<(usize, f32, f32)>) -> midly::Smf<'static> {
    let ticks_per_beat = 384;
    let beats_per_second = 2;
    let ticks_per_second = ticks_per_beat * beats_per_second;
    let microseconds_per_beat = (1_000_000.0 / beats_per_second as f64) as u32;

    let mut track1 = vec![];

    let mut message_roll = vec![];

    for (midi_note, start, end) in notes {
        message_roll.push((start, midi_note, 100));
        message_roll.push((end, midi_note, 0));
    }

    message_roll.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut previous_ticks = 0;

    let start_time = 0.0;
    for message in message_roll {
        let this_ticks = ((message.0 - start_time) * ticks_per_second as f32) as i32;

        if this_ticks >= 0 {
            let diff_ticks = this_ticks - previous_ticks;
            previous_ticks = this_ticks;

            track1.push(midly::TrackEvent {
                delta: (diff_ticks as u32).into(),
                kind: midly::TrackEventKind::Midi {
                    channel: 0.into(),
                    message: midly::MidiMessage::NoteOn {
                        key: (message.1 as u8).into(),
                        vel: message.2.into(),
                    },
                },
            });
        }
    }

    track1.push(midly::TrackEvent {
        delta: 1.into(),
        kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    midly::Smf {
        header: midly::Header {
            format: midly::Format::Parallel,
            timing: midly::Timing::Metrical(ticks_per_beat.into()),
        },
        tracks: vec![
            vec![
                midly::TrackEvent {
                    delta: 0.into(),
                    kind: midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(
                        microseconds_per_beat.into(),
                    )),
                },
                midly::TrackEvent {
                    delta: 0.into(),
                    kind: midly::TrackEventKind::Meta(midly::MetaMessage::TimeSignature(
                        4, 2, 24, 8,
                    )),
                },
                midly::TrackEvent {
                    delta: 1.into(),
                    kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
                },
            ],
            track1,
        ],
    }
}
