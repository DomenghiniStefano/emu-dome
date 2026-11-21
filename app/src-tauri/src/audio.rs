//! Audio output: takes the core's samples and resamples them to the device's
//! rate, with a slight speed control so delay neither builds up nor
//! drains the queue.
//!
//! The device, its rate, channels and buffer and the queue target come from
//! [`AudioTuning`]. The real-time callback takes no lock another thread may hold: the open game
//! comes from an `ArcSwapOption` ([`CurrentShared`]), its samples from the lock-free ring
//! (`emudome_core::audio`), and the measurements ([`AudioMetrics`]) are recorded only in bench
//! mode, with `try_lock` (a callback never waits for the bench reading them).

use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use emudome_core::audio::TEST_TONE_HZ;
use emudome_core::tuning::{AudioBuffer, AudioTuning};
use emudome_core::Shared;
use parking_lot::Mutex;
use serde::Serialize;

use crate::CurrentShared;

/// The cpal stream is not `Send` on every platform: it lives on a thread of its own.
pub struct AudioOutput {
    _stop: mpsc::Sender<()>,
    metrics: SharedAudioMetrics,
}

/// [`CallbackStats::windows`] keeps the last this many windows (an hour of 10 s windows).
const MAX_WINDOWS: usize = 360;

/// The output as opened, for logs and bench reports.
#[derive(Debug, Clone, Serialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    /// Every output device the system lists.
    pub devices: Vec<String>,
    pub default_rate: u32,
    pub default_channels: u16,
    pub rate: u32,
    pub channels: u16,
    pub sample_format: String,
    pub buffer: String,
    /// The configurations the device declares (`channels, rates, format, buffer`).
    pub supported: Vec<String>,
}

/// Length of the windows of [`CallbackStats::windows`].
const WINDOW: Duration = Duration::from_secs(10);

/// What the audio callbacks measured since the last [`AudioMetrics::reset`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct CallbackStats {
    pub callbacks: u64,
    pub frames: u64,
    pub frames_per_callback_min: u64,
    pub frames_per_callback_max: u64,
    /// Callbacks that found the queue empty before filling their buffer.
    pub underrun_callbacks: u64,
    /// Output frames written from an empty queue (fade-out instead of the game's audio).
    pub underrun_frames: u64,
    /// Queue fill when a callback starts, in ms of audio at the core's rate.
    pub fill_ms_min: f64,
    pub fill_ms_max: f64,
    pub fill_ms_sum: f64,
    /// Time between the callback and the moment its first frame is played, as cpal predicts it.
    pub device_latency_ms_sum: f64,
    pub device_latency_ms_max: f64,
    /// Longest time between two callbacks.
    pub interval_ms_max: f64,
    /// The resampler's speed correction (1.0 = none).
    pub ratio_min: f64,
    pub ratio_max: f64,
    pub ratio_sum: f64,
    /// Breaks in the test tone (bench mode with `EMUDOME_BENCH_TONE`): each is an audible click.
    pub discontinuities: u64,
    /// Averages per 10 s window, to see latency drift over a long run (the last
    /// [`MAX_WINDOWS`]).
    pub windows: std::collections::VecDeque<WindowStats>,
    #[serde(skip)]
    last_callback: Option<Instant>,
    #[serde(skip)]
    window: WindowAcc,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WindowStats {
    pub fill_ms_avg: f64,
    pub device_latency_ms_avg: f64,
    pub underrun_frames: u64,
}

#[derive(Debug, Clone, Default)]
struct WindowAcc {
    start: Option<Instant>,
    callbacks: u64,
    fill_ms_sum: f64,
    latency_ms_sum: f64,
    underrun_frames: u64,
}

struct CallbackSample {
    at: Instant,
    frames: u64,
    underrun_frames: u64,
    fill_ms: f64,
    device_latency_ms: f64,
    ratio: f64,
    discontinuities: u64,
}

impl CallbackStats {
    fn record(&mut self, s: CallbackSample) {
        if self.callbacks == 0 {
            self.frames_per_callback_min = s.frames;
            self.fill_ms_min = s.fill_ms;
            self.ratio_min = s.ratio;
            self.ratio_max = s.ratio;
        }
        self.callbacks += 1;
        self.frames += s.frames;
        self.frames_per_callback_min = self.frames_per_callback_min.min(s.frames);
        self.frames_per_callback_max = self.frames_per_callback_max.max(s.frames);
        if s.underrun_frames > 0 {
            self.underrun_callbacks += 1;
            self.underrun_frames += s.underrun_frames;
        }
        self.fill_ms_min = self.fill_ms_min.min(s.fill_ms);
        self.fill_ms_max = self.fill_ms_max.max(s.fill_ms);
        self.fill_ms_sum += s.fill_ms;
        self.device_latency_ms_sum += s.device_latency_ms;
        self.device_latency_ms_max = self.device_latency_ms_max.max(s.device_latency_ms);
        self.ratio_min = self.ratio_min.min(s.ratio);
        self.ratio_max = self.ratio_max.max(s.ratio);
        self.ratio_sum += s.ratio;
        self.discontinuities += s.discontinuities;
        if let Some(last) = self.last_callback {
            let ms = s.at.duration_since(last).as_secs_f64() * 1000.0;
            self.interval_ms_max = self.interval_ms_max.max(ms);
        }
        self.last_callback = Some(s.at);

        let w = &mut self.window;
        let start = *w.start.get_or_insert(s.at);
        w.callbacks += 1;
        w.fill_ms_sum += s.fill_ms;
        w.latency_ms_sum += s.device_latency_ms;
        w.underrun_frames += s.underrun_frames;
        if s.at.duration_since(start) >= WINDOW {
            let n = w.callbacks as f64;
            if self.windows.len() >= MAX_WINDOWS {
                self.windows.pop_front();
            }
            self.windows.push_back(WindowStats {
                fill_ms_avg: w.fill_ms_sum / n,
                device_latency_ms_avg: w.latency_ms_sum / n,
                underrun_frames: w.underrun_frames,
            });
            self.window = WindowAcc::default();
        }
    }
}

#[derive(Debug, Default)]
pub struct AudioMetrics {
    pub device: Option<AudioDeviceInfo>,
    /// Why there is no audio output (device missing, configuration refused...).
    pub error: Option<String>,
    pub stats: CallbackStats,
}

impl AudioMetrics {
    /// Starts measuring again (the bench calls it after the warm-up).
    #[cfg_attr(
        not(feature = "bench"),
        expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
    )]
    pub fn reset(&mut self) {
        self.stats = CallbackStats::default();
    }
}

pub type SharedAudioMetrics = Arc<Mutex<AudioMetrics>>;

impl AudioOutput {
    /// `tone`: the amplitude of the bench's test tone, when the game's audio is replaced by
    /// it; the callbacks then count its breaks. `measure`: the callbacks record
    /// [`CallbackStats`] (bench mode only).
    pub fn start(
        current: CurrentShared,
        tuning: AudioTuning,
        tone: Option<f32>,
        measure: bool,
    ) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let metrics = SharedAudioMetrics::default();
        let thread_metrics = metrics.clone();
        let spawned = std::thread::Builder::new()
            .name("emudome-audio".into())
            .spawn(move || {
                let measured = measure.then(|| thread_metrics.clone());
                match build_stream(current, &tuning, tone, &thread_metrics, measured) {
                    Ok(stream) => {
                        if let Err(e) = stream.play() {
                            log::error!("audio not started: {e}");
                            thread_metrics.lock().error = Some(format!("play: {e}"));
                        }
                        // Stays alive until the app closes.
                        let _ = stop_rx.recv();
                    }
                    Err(e) => {
                        log::error!("no audio output: {e:#}");
                        thread_metrics.lock().error = Some(format!("{e:#}"));
                    }
                }
            });
        if let Err(e) = spawned {
            log::error!("audio thread not started: {e}");
            metrics.lock().error = Some(format!("thread: {e}"));
        }
        Self {
            _stop: stop_tx,
            metrics,
        }
    }

    #[cfg_attr(
        not(feature = "bench"),
        expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
    )]
    pub fn metrics(&self) -> &SharedAudioMetrics {
        &self.metrics
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

/// Finds breaks in a sine of known frequency: for a pure sine
/// `y[n] - 2 cos(w) y[n-1] + y[n-2] = 0`; a residual well above the resampler's own error
/// is a click (an underrun's fade, dropped samples, a jump).
struct ToneCheck {
    two_cos_w: f32,
    threshold: f32,
    y1: f32,
    y2: f32,
    /// Samples left before another break can be counted (one click, one count).
    hold: u32,
}

impl ToneCheck {
    fn new(amplitude: f32, out_rate: f64) -> Self {
        let w = std::f64::consts::TAU * TEST_TONE_HZ / out_rate;
        Self {
            two_cos_w: (2.0 * w.cos()) as f32,
            // Linear interpolation of a 440 Hz sine sampled at 32 kHz errs by ~0.1% of the
            // amplitude; a fade-out or a jump moves the residual by several %.
            threshold: amplitude * 0.01,
            y1: 0.0,
            y2: 0.0,
            hold: 64,
        }
    }

    /// True when `y` starts a new break.
    fn step(&mut self, y: f32) -> bool {
        let residual = (y - self.two_cos_w * self.y1 + self.y2).abs();
        self.y2 = self.y1;
        self.y1 = y;
        if self.hold > 0 {
            self.hold -= 1;
            return false;
        }
        if residual > self.threshold {
            self.hold = 64;
            return true;
        }
        false
    }
}

fn describe_buffer(b: &cpal::SupportedBufferSize) -> String {
    match b {
        cpal::SupportedBufferSize::Range { min, max } => format!("{min}..{max} frames"),
        cpal::SupportedBufferSize::Unknown => "unknown".into(),
    }
}

fn device_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|d| d.name().to_owned())
        .unwrap_or_else(|e| format!("? ({e})"))
}

/// What a stream's callback keeps between two calls.
struct Fill {
    current: CurrentShared,
    target_seconds: f64,
    out_rate: f64,
    channels: usize,
    rs: Resampler,
    /// The game the resampler's state belongs to (its `Shared`'s address): another game starts
    /// from a clean resampler, not from the last game's samples.
    game: usize,
    tone: Option<f32>,
    check: Option<ToneCheck>,
    /// Bench mode only: where the callbacks' measurements go.
    metrics: Option<SharedAudioMetrics>,
}

impl Fill {
    /// One callback: `out` filled with interleaved frames of `channels` samples. Takes no lock
    /// another thread may hold (the reader's is only ever tried by this callback; the metrics'
    /// with `try_lock`, in bench mode).
    fn fill(&mut self, out: &mut [f32], info: &cpal::OutputCallbackInfo) {
        let at = Instant::now();
        let current = self.current.load();
        let Some(shared) = current.as_ref() else {
            out.fill(0.0);
            return;
        };
        let game = Arc::as_ptr(shared) as usize;
        if game != self.game {
            self.game = game;
            self.rs = Resampler::new();
            self.check = self.tone.map(|a| ToneCheck::new(a, self.out_rate));
        }
        self.play(shared, out, info, at);
    }

    fn play(
        &mut self,
        shared: &Shared,
        out: &mut [f32],
        info: &cpal::OutputCallbackInfo,
        at: Instant,
    ) {
        let ring = &shared.audio;
        let Some(mut reader) = ring.reader() else {
            out.fill(0.0);
            return;
        };
        if shared.muted.load(std::sync::atomic::Ordering::Relaxed) {
            // «Muto»: the samples are dropped, so unmuting does not play stale sound.
            reader.drain();
            out.fill(0.0);
            return;
        }
        let queued = reader.begin(ring) as f64;
        let rate = ring.sample_rate();
        let fill_ms = queued / 2.0 / rate * 1000.0;
        // Base ratio plus a correction proportional to the queue's fill level (±0.5%).
        let target = rate * self.target_seconds * 2.0;
        let fill_err = ((queued - target) / target).clamp(-1.0, 1.0);
        let ratio = 1.0 + 0.005 * fill_err;
        let step = rate / self.out_rate * ratio;
        let mut underrun_frames = 0u64;
        let mut discontinuities = 0u64;
        let rs = &mut self.rs;
        for frame in out.chunks_mut(self.channels) {
            while rs.pos >= 1.0 {
                rs.prev = rs.next;
                rs.next = match reader.pop_frame() {
                    Some([l, r]) => [f32::from(l) / 32768.0, f32::from(r) / 32768.0],
                    None => {
                        underrun_frames += 1;
                        [rs.prev[0] * 0.98, rs.prev[1] * 0.98] // empty queue: fade out
                    }
                };
                rs.pos -= 1.0;
            }
            let t = rs.pos as f32;
            let l = rs.prev[0] + (rs.next[0] - rs.prev[0]) * t;
            let r = rs.prev[1] + (rs.next[1] - rs.prev[1]) * t;
            if let Some(c) = &mut self.check {
                discontinuities += u64::from(c.step(l));
            }
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
        drop(reader);
        let Some(metrics) = &self.metrics else {
            return;
        };
        let ts = info.timestamp();
        let device_latency_ms = ts
            .playback
            .checked_duration_since(ts.callback)
            .map_or(0.0, |d| d.as_secs_f64() * 1000.0);
        // Never waits: a callback the bench is reading at that moment goes unrecorded.
        if let Some(mut m) = metrics.try_lock() {
            m.stats.record(CallbackSample {
                at,
                frames: (out.len() / self.channels.max(1)) as u64,
                underrun_frames,
                fill_ms,
                device_latency_ms,
                ratio,
                discontinuities,
            });
        }
    }
}

fn build_stream(
    current: CurrentShared,
    tuning: &AudioTuning,
    tone: Option<f32>,
    metrics: &SharedAudioMetrics,
    measured: Option<SharedAudioMetrics>,
) -> anyhow::Result<cpal::Stream> {
    let host = cpal::default_host();
    let devices: Vec<cpal::Device> = host.output_devices()?.collect();
    let device = match &tuning.device {
        Some(want) => {
            let want = want.to_lowercase();
            devices
                .iter()
                .find(|d| device_name(d).to_lowercase().contains(&want))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("no output device named like {want:?}"))?
        }
        None => host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("no audio device"))?,
    };
    let default = device.default_output_config()?;
    let sample_format = default.sample_format();
    let supported = device
        .supported_output_configs()
        .map(|configs| {
            // Only the device's own sample format: the others repeat the same rates.
            configs
                .filter(|c| c.sample_format() == sample_format)
                .map(|c| {
                    format!(
                        "{} ch, {}-{} Hz, {:?}, {}",
                        c.channels(),
                        c.min_sample_rate(),
                        c.max_sample_rate(),
                        c.sample_format(),
                        describe_buffer(c.buffer_size())
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let config = cpal::StreamConfig {
        channels: tuning.channels.unwrap_or(default.channels()),
        sample_rate: tuning.rate.unwrap_or(default.sample_rate()),
        buffer_size: match tuning.buffer {
            AudioBuffer::Default => cpal::BufferSize::Default,
            AudioBuffer::Frames(n) => cpal::BufferSize::Fixed(n),
        },
    };
    let channels = config.channels as usize;
    let out_rate = f64::from(config.sample_rate);
    let info = AudioDeviceInfo {
        name: device_name(&device),
        devices: devices.iter().map(device_name).collect(),
        default_rate: default.sample_rate(),
        default_channels: default.channels(),
        rate: config.sample_rate,
        channels: config.channels,
        sample_format: format!("{sample_format:?}"),
        buffer: match tuning.buffer {
            AudioBuffer::Default => "default".into(),
            AudioBuffer::Frames(n) => format!("{n} frames"),
        },
        supported,
    };
    log::info!(
        "audio: {} — {} Hz, {} channels, {:?}, buffer {}, queue target {} ms",
        info.name,
        out_rate,
        channels,
        sample_format,
        info.buffer,
        tuning.target_ms
    );
    metrics.lock().device = Some(info);

    let mut state = Fill {
        current,
        target_seconds: f64::from(tuning.target_ms) / 1000.0,
        out_rate,
        channels,
        rs: Resampler::new(),
        game: 0,
        tone,
        check: tone.map(|a| ToneCheck::new(a, out_rate)),
        metrics: measured,
    };
    let mut fill = move |out: &mut [f32], info: &cpal::OutputCallbackInfo| state.fill(out, info);

    let err = |e| log::error!("audio error: {e}");
    // cpal::SampleFormat is `#[non_exhaustive]`: a fallback arm is the only way to match it, and
    // every format without an arm here is reported as an error, never silently mis-played.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "cpal::SampleFormat is non_exhaustive: the fallback arm is required"
    )]
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            config,
            move |data: &mut [f32], info| fill(data, info),
            err,
            None,
        )?,
        cpal::SampleFormat::I16 => {
            let mut buf = Vec::new();
            device.build_output_stream(
                config,
                move |data: &mut [i16], info| {
                    buf.resize(data.len(), 0.0);
                    fill(&mut buf, info);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clean_tone_has_no_breaks_and_a_cut_has_one() {
        let rate = 48000.0;
        let amp = 0.05_f32;
        let w = std::f64::consts::TAU * TEST_TONE_HZ / rate;
        let mut check = ToneCheck::new(amp, rate);
        let sample = |n: u32| (f64::from(n) * w).sin() as f32 * amp;
        let clean: u64 = (0..48000).map(|n| u64::from(check.step(sample(n)))).sum();
        assert_eq!(clean, 0);
        // 200 samples skipped: a jump in phase.
        let cut: u64 = (48200..49000)
            .map(|n| u64::from(check.step(sample(n))))
            .sum();
        assert_eq!(cut, 1);
    }
}
