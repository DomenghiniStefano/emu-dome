//! Audio output: takes the core's samples and resamples them to the device's
//! rate, with a slight speed control so delay neither builds up nor
//! drains the queue.

use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::CurrentShared;

/// The cpal stream is not `Send` on every platform: it lives on a thread of its own.
pub struct AudioOutput {
    _stop: mpsc::Sender<()>,
}

impl AudioOutput {
    pub fn start(current: CurrentShared) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        std::thread::Builder::new()
            .name("emudome-audio".into())
            .spawn(move || match build_stream(current) {
                Ok(stream) => {
                    if let Err(e) = stream.play() {
                        log::error!("audio not started: {e}");
                    }
                    // Stays alive until the app closes.
                    let _ = stop_rx.recv();
                }
                Err(e) => log::error!("no audio output: {e:#}"),
            })
            .expect("audio thread");
        Self { _stop: stop_tx }
    }
}

/// State of the linear resampler.
struct Resampler {
    pos: f64,
    prev: [f32; 2],
    next: [f32; 2],
}

impl Resampler {
    fn new() -> Self {
        Self {
            pos: 1.0,
            prev: [0.0; 2],
            next: [0.0; 2],
        }
    }
}

fn build_stream(current: CurrentShared) -> anyhow::Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("no audio device"))?;
    let supported = device.default_output_config()?;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let channels = config.channels as usize;
    let out_rate = config.sample_rate as f64;
    log::info!(
        "audio: {} Hz, {} channels, {:?}",
        out_rate,
        channels,
        sample_format
    );

    let mut rs = Resampler::new();
    let mut fill = move |out: &mut [f32]| {
        let shared = current.lock().clone();
        let Some(shared) = shared else {
            out.fill(0.0);
            return;
        };
        let mut q = shared.audio.lock();
        // Base ratio plus a correction proportional to the queue's fill level (±0.5%).
        let target = q.sample_rate * 0.06 * 2.0;
        let fill_err = ((q.samples.len() as f64 - target) / target).clamp(-1.0, 1.0);
        let step = q.sample_rate / out_rate * (1.0 + 0.005 * fill_err);
        for frame in out.chunks_mut(channels) {
            while rs.pos >= 1.0 {
                rs.prev = rs.next;
                rs.next = match (q.samples.pop_front(), q.samples.pop_front()) {
                    (Some(l), Some(r)) => [l as f32 / 32768.0, r as f32 / 32768.0],
                    _ => [rs.prev[0] * 0.98, rs.prev[1] * 0.98], // empty queue: fade out
                };
                rs.pos -= 1.0;
            }
            let t = rs.pos as f32;
            let l = rs.prev[0] + (rs.next[0] - rs.prev[0]) * t;
            let r = rs.prev[1] + (rs.next[1] - rs.prev[1]) * t;
            match frame.len() {
                1 => frame[0] = (l + r) * 0.5,
                _ => {
                    frame[0] = l;
                    frame[1] = r;
                    for c in frame.iter_mut().skip(2) {
                        *c = 0.0;
                    }
                }
            }
            rs.pos += step;
        }
    };

    let err = |e| log::error!("audio error: {e}");
    // cpal::SampleFormat is `#[non_exhaustive]`: a fallback arm is the only way to match it, and
    // every format without an arm here is reported as an error, never silently mis-played.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "cpal::SampleFormat is non_exhaustive: the fallback arm is required"
    )]
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            device.build_output_stream(config, move |data: &mut [f32], _| fill(data), err, None)?
        }
        cpal::SampleFormat::I16 => {
            let mut buf = Vec::new();
            device.build_output_stream(
                config,
                move |data: &mut [i16], _| {
                    buf.resize(data.len(), 0.0);
                    fill(&mut buf);
                    for (o, s) in data.iter_mut().zip(&buf) {
                        *o = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                    }
                },
                err,
                None,
            )?
        }
        other => anyhow::bail!("unhandled audio format: {other:?}"),
    };
    Ok(stream)
}
