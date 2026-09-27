//! Transkun V2 (Y. Yan, Z. Duan)
//! https://arxiv.org/pdf/2404.09466
//! https://openreview.net/pdf?id=DGA8XbJ8FVd
//! https://github.com/Yujia-Yan/Transkun

use rten::{Model, NodeId};
use rten_tensor::prelude::*;
use rten_tensor::{NdTensor, NdTensorView};

use crate::{args::Args, audio, midi};

const SAMPLE_RATE: u32 = 44100;
const HOP_SIZE: usize = 1024;
const WINDOW_SIZE: usize = 4096;
const SEGMENT_SIZE_IN_SECOND: f64 = 16.0;
const SEGMENT_HOP_SIZE_IN_SECOND: f64 = 8.0;

/// Sustain and soft pedal, followed by 88 piano keys
const N_EVENT_TYPES: usize = 2 + 88;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EventType {
    Pedal(u8),
    Note(u8),
}

impl EventType {
    fn from_idx(idx: usize) -> Self {
        match idx {
            0 => Self::Pedal(64),
            1 => Self::Pedal(67),
            n => Self::Note((n - 2 + 21) as u8),
        }
    }

    /// The `targetMIDIPitch` value used by transkun, pedals are represented as negative numbers
    fn sort_key(self) -> i32 {
        match self {
            Self::Pedal(cc) => -(cc as i32),
            Self::Note(pitch) => pitch as i32,
        }
    }
}

#[derive(Debug, Clone)]
struct Event {
    kind: usize,
    start: f64,
    end: f64,
    velocity: u8,
    has_onset: bool,
    has_offset: bool,
}

impl Event {
    fn event_type(&self) -> EventType {
        EventType::from_idx(self.kind)
    }
}

fn sort_events(events: &mut [Event]) {
    events.sort_by(|a, b| {
        a.start
            .total_cmp(&b.start)
            .then(a.end.total_cmp(&b.end))
            .then(a.event_type().sort_key().cmp(&b.event_type().sort_key()))
    });
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let audio = load_audio(args)?;

    let model = TransKun::load(args)?;
    let events = model.transcribe(&audio)?;

    let mut notes = Vec::new();
    let mut control_changes = Vec::new();

    for e in events {
        match e.event_type() {
            EventType::Note(pitch) => {
                println!("{pitch}: {} - {}", e.start, e.end);
                notes.push(midi::Note {
                    pitch,
                    start: e.start,
                    end: e.end,
                    // Velocity 0 would turn NoteOn into NoteOff
                    velocity: e.velocity.max(1),
                });
            }
            EventType::Pedal(controller) => {
                control_changes.push(midi::ControlChange {
                    controller,
                    value: e.velocity,
                    time: e.start,
                });
                control_changes.push(midi::ControlChange {
                    controller,
                    value: 0,
                    time: e.end,
                });
            }
        }
    }

    let file = midi::create_midi_file(&notes, &control_changes);
    file.save(&args.output)?;

    Ok(())
}

/// Stereo audio, mono files get duplicated into both channels
/// (model downmixes to mono internally, so this is equivalent)
fn load_audio(args: &Args) -> anyhow::Result<[Vec<f32>; 2]> {
    let mut channels = audio::load(&args.input, SAMPLE_RATE)?.into_iter();
    let left = channels.next().unwrap_or_default();
    let right = channels.next().unwrap_or_else(|| left.clone());
    Ok([left, right])
}

struct TransKun {
    model: Model,
    frames_id: NodeId,
    attr_id: NodeId,
    score_id: NodeId,
    ctx_id: NodeId,
    velocity_id: NodeId,
    of_id: NodeId,
}

impl TransKun {
    fn load(args: &Args) -> anyhow::Result<Self> {
        let model = Model::load_file(&args.model)?;

        Ok(Self {
            frames_id: model.node_id("frames")?,
            attr_id: model.node_id("attr")?,
            score_id: model.node_id("score")?,
            ctx_id: model.node_id("ctx")?,
            velocity_id: model.node_id("velocity")?,
            of_id: model.node_id("of")?,
            model,
        })
    }

    /// Port of `TransKun.transcribe`
    fn transcribe(&self, audio: &[Vec<f32>; 2]) -> anyhow::Result<Vec<Event>> {
        let fs = SAMPLE_RATE as f64;
        let hop_size = HOP_SIZE as f64;

        let pad_time_begin = SEGMENT_SIZE_IN_SECOND - SEGMENT_HOP_SIZE_IN_SECOND;
        let pad = (pad_time_begin * fs).ceil() as usize;

        let x: [Vec<f32>; 2] = audio.clone().map(|channel| {
            let mut padded = vec![0.0; pad];
            padded.extend(channel);
            padded.resize(padded.len() + pad, 0.0);
            padded
        });
        let n_sample = x[0].len();

        let start_frame_idx = (pad_time_begin * fs / hop_size).floor() as usize;
        let mut start_pos = vec![start_frame_idx; N_EVENT_TYPES];

        let step_size = (SEGMENT_HOP_SIZE_IN_SECOND * fs / hop_size).ceil() as usize * HOP_SIZE;
        let segment_size = (SEGMENT_SIZE_IN_SECOND * fs).ceil() as usize;
        let last_frame_idx = (segment_size as f64 / hop_size).round() as usize;

        let mut events_by_type: Vec<Vec<Event>> = vec![Vec::new(); N_EVENT_TYPES];

        let n_segments = n_sample.div_ceil(step_size);
        for (segment_id, i) in (0..n_sample).step_by(step_size).enumerate() {
            eprintln!("Segment {}/{n_segments}", segment_id + 1);

            let j = (i + segment_size).min(n_sample);
            let begin_time = i as f64 / fs - pad_time_begin;

            let frames = make_frames(&x, i..j, segment_size);

            let (events, last_p) = self.transcribe_frames(frames, &start_pos, last_frame_idx)?;

            start_pos = last_p
                .iter()
                .map(|k| k.saturating_sub(step_size / HOP_SIZE))
                .collect();

            for mut e in events {
                // shift all notes by begin_time
                e.start = (e.start + begin_time).max(0.0);
                e.end = (e.end + begin_time).max(e.start);

                // merge incomplete events
                let events = &mut events_by_type[e.kind];

                if let Some(last_e) = events.last_mut()
                    && e.start < last_e.end
                {
                    if e.has_onset {
                        *last_e = e;
                    } else {
                        last_e.has_offset = e.has_offset;
                        last_e.end = e.end.max(last_e.end);
                    }
                    continue;
                }

                if e.has_onset {
                    events.push(e);
                }
            }
        }

        // handling incomplete events in the last segment
        for events in events_by_type.iter_mut() {
            if let Some(last) = events.last_mut() {
                last.has_offset = true;
            }
        }

        let events: Vec<Event> = events_by_type
            .into_iter()
            .flatten()
            .filter(|e| e.has_offset)
            .collect();

        Ok(resolve_overlapping(events))
    }

    /// Port of `TransKun.transcribeFrames`
    ///
    /// Returns events (with times relative to the segment begining),
    /// and the last offset position for each event type
    fn transcribe_frames(
        &self,
        frames: NdTensor<f32, 4>,
        forced_start_pos: &[usize],
        last_frame_idx: usize,
    ) -> anyhow::Result<(Vec<Event>, Vec<usize>)> {
        let [score, ctx] = self.model.run_n(
            vec![(self.frames_id, frames.view().into())],
            [self.score_id, self.ctx_id],
            None,
        )?;
        // [P, T_end, T_begin]
        let score: NdTensor<f32, 3> = score.try_into()?;
        // [P, T, D]
        let ctx: NdTensor<f32, 3> = ctx.try_into()?;

        let paths: Vec<Vec<(usize, usize)>> = (0..N_EVENT_TYPES)
            .map(|p| viterbi_backward(score.slice(p), forced_start_pos[p]))
            .collect();

        let n_intervals: usize = paths.iter().map(Vec::len).sum();
        if n_intervals == 0 {
            return Ok((Vec::new(), vec![0; N_EVENT_TYPES]));
        }

        // Interval features: [ctx[begin], ctx[end], ctx[begin] * ctx[end]]
        let d = ctx.size(2);
        let mut attr = NdTensor::<f32, 2>::zeros([n_intervals, 3 * d]);
        {
            let mut row = 0;
            for (p, path) in paths.iter().enumerate() {
                for &(begin, end) in path {
                    let a = ctx.slice((p, begin));
                    let b = ctx.slice((p, end));
                    let mut out = attr.slice_mut(row);
                    for k in 0..d {
                        out[k] = a[k];
                        out[d + k] = b[k];
                        out[2 * d + k] = a[k] * b[k];
                    }
                    row += 1;
                }
            }
        }

        let [velocity, of] = self.model.run_n(
            vec![(self.attr_id, attr.view().into())],
            [self.velocity_id, self.of_id],
            None,
        )?;
        // [N, 128]
        let velocity: NdTensor<f32, 2> = velocity.try_into()?;
        // [N, 4] (onset refine, offset refine, onset presence, offset presence)
        let of: NdTensor<f32, 2> = of.try_into()?;

        let frame_dur = HOP_SIZE as f64 / SAMPLE_RATE as f64;

        let mut events = Vec::with_capacity(n_intervals);
        let mut last_p = Vec::with_capacity(N_EVENT_TYPES);
        let mut row = 0;

        for (kind, path) in paths.iter().enumerate() {
            let mut last_end = 0.0;
            let mut cur_last_p = 0;

            for &(begin, end) in path {
                let velocity = argmax(velocity.slice(row)) as u8;

                let of = of.slice(row);
                let onset_shift = refined_of_value(of[0]) as f64;
                let offset_shift = refined_of_value(of[1]) as f64;

                // Presence prediction is only used to distinguish the corner case
                // that either onset or offset happens exactly on the first/last frame.
                let has_onset = begin > 0 || of[2] > 0.0;
                let has_offset = end < last_frame_idx || of[3] > 0.0;

                let start = ((begin as f64 + onset_shift) * frame_dur).max(last_end);
                let end_time = ((end as f64 + offset_shift) * frame_dur).max(start + 1e-8);
                last_end = end_time;

                events.push(Event {
                    kind,
                    start,
                    end: end_time,
                    velocity,
                    has_onset,
                    has_offset,
                });

                if has_offset {
                    cur_last_p = end;
                }

                row += 1;
            }

            last_p.push(cur_last_p);
        }

        sort_events(&mut events);

        Ok((events, last_p))
    }
}

/// Port of `makeFrame`, frames `x[range]` zero padded to `segment_size`.
///
/// Output: [1, nChannel, nFrame, WINDOW_SIZE]
fn make_frames(
    x: &[Vec<f32>; 2],
    range: std::ops::Range<usize>,
    segment_size: usize,
) -> NdTensor<f32, 4> {
    let n_frame = segment_size.div_ceil(HOP_SIZE) + 1;
    let l_pad = WINDOW_SIZE / 2;

    let mut frames = NdTensor::<f32, 4>::zeros([1, x.len(), n_frame, WINDOW_SIZE]);

    for (ch, channel) in x.iter().enumerate() {
        let segment = &channel[range.clone()];

        for f in 0..n_frame {
            let mut frame = frames.slice_mut((0, ch, f));

            // Frame covers segment[f * HOP_SIZE - l_pad .. f * HOP_SIZE - l_pad + WINDOW_SIZE]
            for t in 0..WINDOW_SIZE {
                let idx = (f * HOP_SIZE + t).checked_sub(l_pad);
                if let Some(sample) = idx.and_then(|idx| segment.get(idx)) {
                    frame[t] = *sample;
                }
            }
        }
    }

    frames
}

/// Port of `viterbiBackward` from `NeuralSemiCRFInterval`, with zero noise score
///
/// `score`: [T_end, T_begin], returns non-overlapping `(begin, end)` intervals
fn viterbi_backward(score: NdTensorView<f32, 2>, forced_start_pos: usize) -> Vec<(usize, usize)> {
    let t = score.size(0);
    let diag = |i: usize| score[[i, i]];

    // q[i]: best score of the path starting at i
    let mut q = vec![0.0f32; t];
    // ptr[t - i - 2]: for position i, -1 means skip, otherwise the offset of interval end from i + 1
    let mut ptr = vec![-1i32; t.saturating_sub(1)];

    q[t - 1] = diag(t - 1).max(0.0);

    for i in 1..t {
        let pos = t - i - 1;

        // skip
        let mut best = q[pos + 1];
        let mut selection = -1;

        // an interval
        for end in pos + 1..t {
            let v = q[end] + score[[end, pos]];
            if v > best {
                best = v;
                selection = (end - pos - 1) as i32;
            }
        }

        ptr[i - 1] = selection;
        q[pos] = best + diag(pos).max(0.0);
    }

    let mut result = Vec::new();
    let mut j = forced_start_pos;

    while j + 1 < t {
        let selection = ptr[t - j - 2];

        if diag(j) > 0.0 {
            result.push((j, j));
        }

        if selection < 0 {
            j += 1;
        } else {
            let i = selection as usize + j + 1;
            result.push((j, i));
            j = i;
        }
    }

    if diag(t - 1) > 0.0 {
        result.push((t - 1, t - 1));
    }

    result
}

/// Mean of `ContinuousBernoulli(logits)`, mapped back to the refined onset/offset
/// shift in frames, as in `transcribeFrames`
fn refined_of_value(logit: f32) -> f32 {
    let p = 1.0 / (1.0 + (-logit).exp());

    let mean = if p <= 0.499 || p > 0.501 {
        p / (2.0 * p - 1.0) + 1.0 / ((-p).ln_1p() - p.ln())
    } else {
        // Taylor expansion around 0.5 for numerical stability
        let x = p - 0.5;
        0.5 + (1.0 / 3.0 + 16.0 / 45.0 * x * x) * x
    };

    ((mean - 0.5) / 0.99).clamp(-0.5, 0.5)
}

fn argmax(v: NdTensorView<f32, 1>) -> usize {
    let mut best = 0;
    for i in 1..v.size(0) {
        if v[i] > v[best] {
            best = i;
        }
    }
    best
}

/// Port of `resolveOverlapping`
fn resolve_overlapping(mut events: Vec<Event>) -> Vec<Event> {
    sort_events(&mut events);

    let mut last_by_type: [Option<usize>; N_EVENT_TYPES] = [None; N_EVENT_TYPES];

    for idx in 0..events.len() {
        let kind = events[idx].kind;
        if let Some(prev) = last_by_type[kind]
            && events[prev].end > events[idx].start
        {
            events[prev].end = events[idx].start;
        }
        last_by_type[kind] = Some(idx);
    }

    sort_events(&mut events);

    // remove all notes that have start == end
    events.retain(|e| e.start < e.end);

    events
}
