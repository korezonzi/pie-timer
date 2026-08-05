// Rust-side sound engine for Pie Timer.
//
// This is a faithful port of the Web Audio synthesis in `src/lib/sound.ts`
// (kept as the source of truth — see comments on each synth function).
// `rodio::OutputStream` is `!Send`, so audio playback happens on a dedicated
// thread; the rest of the app talks to it via an mpsc channel.

use rodio::buffer::SamplesBuffer;
use rodio::DeviceSinkBuilder;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

const SAMPLE_RATE: u32 = 44100;
const SAMPLE_RATE_F: f32 = SAMPLE_RATE as f32;
const MASTER_GAIN: f32 = 0.8;

pub enum SoundKind {
    Tick,
    Start,
    Pause,
    Bell,
}

pub struct SoundPlayer {
    tx: mpsc::Sender<SoundKind>,
    muted: Arc<AtomicBool>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<SoundKind>();
        let muted = Arc::new(AtomicBool::new(false));

        std::thread::spawn(move || {
            // Pre-synthesize all four sound effects once at startup.
            let tick = apply_master_gain(synth_tick());
            let pause = apply_master_gain(synth_pause());
            let start = apply_master_gain(synth_start());
            let bell = apply_master_gain(synth_bell());

            // rodio 0.22 exposes `DeviceSinkBuilder` / `MixerDeviceSink` rather than the
            // `OutputStreamBuilder` name used in earlier drafts of this module — the
            // crate resolved by Cargo has this API, so we target it directly.
            match DeviceSinkBuilder::open_default_sink() {
                Ok(sink) => {
                    let mixer = sink.mixer();
                    let channels = std::num::NonZero::new(1u16).unwrap();
                    let sample_rate = std::num::NonZero::new(SAMPLE_RATE).unwrap();
                    for kind in rx {
                        let buf = match kind {
                            SoundKind::Tick => &tick,
                            SoundKind::Start => &start,
                            SoundKind::Pause => &pause,
                            SoundKind::Bell => &bell,
                        };
                        // Fire-and-forget: hand the buffer to the mixer and move on.
                        mixer.add(SamplesBuffer::new(channels, sample_rate, buf.clone()));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open default audio output sink: {e}");
                    // Drain the channel so senders never block; app keeps running silently.
                    for _ in rx {}
                }
            }
        });

        Self { tx, muted }
    }

    pub fn play(&self, kind: SoundKind) {
        if !self.muted.load(Ordering::Relaxed) {
            let _ = self.tx.send(kind);
        }
    }

    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn set_muted(engine: tauri::State<'_, crate::timer::TimerEngine>, muted: bool) {
    engine.sound.set_muted(muted);
}

fn apply_master_gain(mut buf: Vec<f32>) -> Vec<f32> {
    for sample in buf.iter_mut() {
        *sample = (*sample * MASTER_GAIN).clamp(-1.0, 1.0);
    }
    buf
}

/// Exponential ramp matching Web Audio's `exponentialRampToValueAtTime`:
/// v(t) = v0 * (v1/v0)^(t/duration), clamped to [0, duration].
fn exp_ramp(v0: f32, v1: f32, t: f32, duration: f32) -> f32 {
    if duration <= 0.0 {
        return v1;
    }
    let frac = (t / duration).clamp(0.0, 1.0);
    v0 * (v1 / v0).powf(frac)
}

fn samples_for(duration_sec: f32) -> usize {
    (duration_sec * SAMPLE_RATE_F).round() as usize
}

// Short tick sound — like a clock tick.
// sine 800 -> 600 Hz exponential sweep over 0.05s (holds at 600 after),
// gain 0.3 -> 0.01 exponential decay over 0.08s, total duration 0.08s.
fn synth_tick() -> Vec<f32> {
    let duration = 0.08_f32;
    let freq_sweep_dur = 0.05_f32;
    let n = samples_for(duration);
    let mut buf = vec![0.0_f32; n];
    let mut phase = 0.0_f32;

    for (i, sample) in buf.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE_F;
        let freq = if t < freq_sweep_dur {
            exp_ramp(800.0, 600.0, t, freq_sweep_dur)
        } else {
            600.0
        };
        phase += 2.0 * PI * freq / SAMPLE_RATE_F;
        let gain = exp_ramp(0.3, 0.01, t, duration);
        *sample = phase.sin() * gain;
    }
    buf
}

// Pause sound — short descending tone.
// sine 660 -> 330 Hz exponential sweep over 0.12s (holds at 330 after),
// gain 0.25 -> 0.01 exponential decay over 0.15s, total duration 0.15s.
fn synth_pause() -> Vec<f32> {
    let duration = 0.15_f32;
    let freq_sweep_dur = 0.12_f32;
    let n = samples_for(duration);
    let mut buf = vec![0.0_f32; n];
    let mut phase = 0.0_f32;

    for (i, sample) in buf.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE_F;
        let freq = if t < freq_sweep_dur {
            exp_ramp(660.0, 330.0, t, freq_sweep_dur)
        } else {
            330.0
        };
        phase += 2.0 * PI * freq / SAMPLE_RATE_F;
        let gain = exp_ramp(0.25, 0.01, t, duration);
        *sample = phase.sin() * gain;
    }
    buf
}

// Additively mix one bell-like note (fundamental + two inharmonic partials)
// into `buf`, starting at `start_time` seconds. Fast (4ms) exponential attack
// then a long exponential decay gives a metallic, chime-like ring.
fn synth_bell_note(buf: &mut Vec<f32>, start_time: f32, freq: f32, peak: f32, decay: f32) {
    const PARTIALS: [(f32, f32); 3] = [(1.0, 1.0), (2.01, 0.45), (2.99, 0.2)];
    const ATTACK: f32 = 0.004;

    let start_sample = samples_for(start_time);
    let end_sample = samples_for(start_time + decay);
    if buf.len() < end_sample {
        buf.resize(end_sample, 0.0);
    }

    for &(mult, level) in PARTIALS.iter() {
        let mut phase = 0.0_f32;
        for (i, sample) in buf
            .iter_mut()
            .enumerate()
            .take(end_sample)
            .skip(start_sample)
        {
            let local_t = i as f32 / SAMPLE_RATE_F - start_time;
            let amp = if local_t < ATTACK {
                exp_ramp(0.0001, peak * level, local_t, ATTACK)
            } else {
                exp_ramp(peak * level, 0.0001, local_t - ATTACK, decay - ATTACK)
            };
            phase += 2.0 * PI * (freq * mult) / SAMPLE_RATE_F;
            *sample += phase.sin() * amp;
        }
    }
}

// Start sound — bright ascending bell chime (two layered notes).
// E5 (659.25Hz) at t=0, peak 0.3, decay 0.6s.
// B5 (987.77Hz) at t=0.13s, peak 0.3, decay 0.85s.
fn synth_start() -> Vec<f32> {
    let mut buf: Vec<f32> = Vec::new();
    synth_bell_note(&mut buf, 0.0, 659.25, 0.3, 0.6);
    synth_bell_note(&mut buf, 0.13, 987.77, 0.3, 0.85);
    buf
}

// Bell sound — pleasant chime for session completion.
// Two layered sine tones (880Hz + 1320Hz), gain held at 0.4 for 0.1s then
// exponentially decayed to 0.01 by t=1.2s.
fn synth_bell() -> Vec<f32> {
    let hold_until = 0.1_f32;
    let duration = 1.2_f32;
    let n = samples_for(duration);
    let mut buf = vec![0.0_f32; n];

    for freq in [880.0_f32, 1320.0_f32] {
        let mut phase = 0.0_f32;
        for (i, sample) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE_F;
            let gain = if t < hold_until {
                0.4
            } else {
                exp_ramp(0.4, 0.01, t - hold_until, duration - hold_until)
            };
            phase += 2.0 * PI * freq / SAMPLE_RATE_F;
            *sample += phase.sin() * gain;
        }
    }
    buf
}
