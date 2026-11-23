//! Audio output: takes the core's samples and resamples them to the device's
//! rate, with a slight speed control so delay neither builds up nor
//! drains the queue.
//!
//! The device and the queue come from the settings ([`AudioPrefs`]: `audio.out`, the output
//! device, `audio.lat`, the queue's target and limit), unless a developer knob of
//! [`AudioTuning`] forces them; its rate, channels and buffer come from [`AudioTuning`]. The
//! real-time callback takes no lock another thread may hold: the open game comes from an
//! `ArcSwapOption` ([`CurrentShared`]), its samples from the lock-free ring
//! (`emudome_core::audio`), the latency from an atomic, and the measurements ([`AudioMetrics`])
//! are recorded only in bench mode, with `try_lock` (a callback never waits for the bench reading
//! them).

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use emudome_core::audio::{samples_in, Latency, TEST_TONE_HZ};
use emudome_core::settings::{keys, Scope};
use emudome_core::{ErrorCode, Shared};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::Manager;

use crate::health::{Part, SharedHealth};
use crate::tuning::{AudioBuffer, AudioTuning};
use crate::CurrentShared;

/// The cpal stream is not `Send` on every platform: it lives on a thread of its own.
pub struct AudioOutput {
    _stop: mpsc::Sender<()>,
    metrics: SharedAudioMetrics,
    prefs: Arc<AudioPrefs>,
}

/// The output the user chose (`audio.out`, `audio.lat`), changed while it plays: the latency is
/// read by every callback (an atomic), a new device makes the audio thread open it.
pub struct AudioPrefs {
    /// [`Latency::to_u8`].
    latency: AtomicU8,
    /// The chosen device's exact name; `None`: the system's default.
    device: Mutex<Option<String>>,
    /// Bumped when `device` changes: the playing stream is replaced.
    generation: AtomicU64,
}

impl AudioPrefs {
    pub fn new(device: Option<String>, latency: Latency) -> Self {
        Self {
            latency: AtomicU8::new(latency.to_u8()),
            device: Mutex::new(device),
            generation: AtomicU64::new(0),
        }
    }

    /// The settings changed: the latency applies at the next callback, another device at once.
    pub fn set(&self, device: Option<String>, latency: Latency) {
        self.latency.store(latency.to_u8(), Ordering::Relaxed);
        let mut current = self.device.lock();
        if *current != device {
            log::info!(
                "audio output: {}",
                device.as_deref().unwrap_or("the system's")
            );
            *current = device;
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn latency(&self) -> Latency {
        Latency::from_u8(self.latency.load(Ordering::Relaxed))
    }

    fn device(&self) -> Option<String> {
        self.device.lock().clone()
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
}

/// `audio.out` (`system`: `None`) and `audio.lat` as `read` resolves them.
pub fn prefs_from(read: impl Fn(&str) -> Option<emudome_core::settings::Value>) -> AudioPrefs {
    let (device, latency) = chosen(read);
    AudioPrefs::new(device, latency)
}

fn chosen(
    read: impl Fn(&str) -> Option<emudome_core::settings::Value>,
) -> (Option<String>, Latency) {
    let settings = keys::Reader::new(read);
    let out = settings.text(keys::AUDIO_OUT);
    let device = (out != "system" && !out.trim().is_empty()).then_some(out);
    let latency = Latency::from_option(&settings.text(keys::AUDIO_LATENCY)).unwrap_or_default();
    (device, latency)
}

/// The `audio.*` settings the output follows.
pub fn is_output_key(key: &str) -> bool {
    key == keys::AUDIO_OUT.as_str() || key == keys::AUDIO_LATENCY.as_str()
}

/// `audio.out` or `audio.lat` changed (both per device, «Tutti i giochi» only).
pub fn settings_changed(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<crate::AppState>() else {
        return;
    };
    let (device, latency) = chosen(crate::settings::resolver(app, &Scope::Global));
    state.audio.prefs.set(device, latency);
}

/// The names of the output devices the system lists now (`audio.out`'s options after
/// «system»), each once, in the system's order.
pub fn output_names() -> anyhow::Result<Vec<String>> {
    let mut names: Vec<String> = Vec::new();
    for device in cpal::default_host().output_devices()? {
        // A device that cannot say its name cannot be chosen by it either.
        let Ok(description) = device.description() else {
            continue;
        };
        let name = description.name().to_owned();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    Ok(names)
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
    /// Input frames the queue did not have: the fade-out of an empty queue, then the silence
    /// until it is refilled to its target.
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
    ///
    /// The stream is rebuilt when it fails (headphones unplugged, a Bluetooth device gone and
    /// back) and, when no device is forced, when the system's default output changes; while no
    /// stream plays, `audioUnavailable` is on `health` and a new one is tried with a growing
    /// pause. The thread ends when this value is dropped.
    pub fn start(
        current: CurrentShared,
        tuning: AudioTuning,
        prefs: AudioPrefs,
        tone: Option<f32>,
        measure: bool,
        health: SharedHealth,
    ) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let metrics = SharedAudioMetrics::default();
        let prefs = Arc::new(prefs);
        let thread_metrics = metrics.clone();
        let thread_health = health.clone();
        let thread_prefs = prefs.clone();
        let spawned = std::thread::Builder::new()
            .name("emudome-audio".into())
            .spawn(move || {
                let output = Output {
                    current,
                    tuning,
                    prefs: thread_prefs,
                    tone,
                    metrics: thread_metrics,
                    measure,
                    health: thread_health.clone(),
                };
                thread_health.guard(Part::Audio, || output.run(&stop_rx));
            });
        if let Err(e) = spawned {
            log::error!("audio thread not started: {e}");
            metrics.lock().error = Some(format!("thread: {e}"));
            health.set(Part::Audio, Some(ErrorCode::AudioUnavailable));
        }
        Self {
            _stop: stop_tx,
            metrics,
            prefs,
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

/// How often the default output device is looked at (no event for it on every backend).
const DEVICE_POLL: Duration = Duration::from_secs(2);
/// First and longest pause before trying to open an output again.
const RETRY_FIRST: Duration = Duration::from_millis(500);
const RETRY_MAX: Duration = Duration::from_secs(10);

/// Why a playing stream is given up.
enum Ended {
    /// The app is closing.
    Stop,
    /// The stream failed, or the default device changed: a new one is built.
    Rebuild(String),
}

/// Whether a stream error means the stream must be built again. Underruns, an automatic reroute
/// to the new default device and a refused real-time priority leave it playing.
fn needs_rebuild(kind: cpal::ErrorKind) -> bool {
    !matches!(
        kind,
        cpal::ErrorKind::Xrun | cpal::ErrorKind::DeviceChanged | cpal::ErrorKind::RealtimeDenied
    )
}

/// Which output to open, among the devices the system lists (`names`).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Choice {
    /// The system's default (`audio.out` = system).
    System,
    /// The device at this index.
    Named(usize),
    /// The chosen device is not there now: the system's default until it is back.
    FellBack,
}

impl Choice {
    /// The stream follows the system's default device.
    fn follows_default(&self) -> bool {
        matches!(self, Choice::System | Choice::FellBack)
    }
}

/// The device to open: the developer knob `forced` (part of a name, any case; none matching is
/// an error, as before the setting), else the `chosen` device by its exact name (gone: the
/// system's, the choice kept), else the system's.
fn choose(names: &[String], forced: Option<&str>, chosen: Option<&str>) -> Result<Choice, String> {
    if let Some(want) = forced {
        let want = want.to_lowercase();
        return names
            .iter()
            .position(|n| n.to_lowercase().contains(&want))
            .map(Choice::Named)
            .ok_or_else(|| format!("no output device named like {want:?}"));
    }
    Ok(match chosen {
        Some(name) => names
            .iter()
            .position(|n| n == name)
            .map_or(Choice::FellBack, Choice::Named),
        None => Choice::System,
    })
}

/// A stream playing, and what it was opened for.
struct Opened {
    stream: cpal::Stream,
    /// The device's name.
    device: String,
    choice: Choice,
    /// [`AudioPrefs::generation`] when it was opened.
    generation: u64,
}

/// The audio thread: builds the stream, keeps it playing, builds it again when it fails.
struct Output {
    current: CurrentShared,
    tuning: AudioTuning,
    prefs: Arc<AudioPrefs>,
    tone: Option<f32>,
    metrics: SharedAudioMetrics,
    measure: bool,
    health: SharedHealth,
}

impl Output {
    fn run(&self, stop: &Receiver<()>) {
        let mut retry = RETRY_FIRST;
        loop {
            let (errors_tx, errors) = mpsc::channel::<String>();
            let measured = self.measure.then(|| self.metrics.clone());
            let built = build_stream(
                self.current.clone(),
                &self.tuning,
                &self.prefs,
                self.tone,
                &self.metrics,
                measured,
                errors_tx,
            )
            .and_then(|opened| {
                opened
                    .stream
                    .play()
                    .map_err(|e| anyhow::anyhow!("play: {e}"))?;
                Ok(opened)
            });
            match built {
                Ok(opened) => {
                    retry = RETRY_FIRST;
                    self.metrics.lock().error = None;
                    self.health.set(Part::Audio, None);
                    let ended = self.watch(stop, &errors, &opened);
                    drop(opened);
                    match ended {
                        Ended::Stop => return,
                        Ended::Rebuild(why) => log::info!("audio: opening the output again: {why}"),
                    }
                }
                Err(e) => {
                    let detail = format!("{e:#}");
                    log::error!("no audio output: {detail}");
                    self.metrics.lock().error = Some(detail);
                    self.health
                        .set(Part::Audio, Some(ErrorCode::AudioUnavailable));
                    match stop.recv_timeout(retry) {
                        Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
                        Err(RecvTimeoutError::Timeout) => {}
                    }
                    retry = (retry * 2).min(RETRY_MAX);
                }
            }
        }
    }

    /// While a stream plays: until the app closes, the stream fails, the user chooses another
    /// device, or (following the system's) the default device is another one than the one
    /// playing, or the chosen device that was missing is back.
    fn watch(&self, stop: &Receiver<()>, errors: &Receiver<String>, opened: &Opened) -> Ended {
        let mut looked = Instant::now();
        loop {
            match errors.recv_timeout(Duration::from_millis(250)) {
                Ok(e) => return Ended::Rebuild(e),
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {}
            }
            match stop.try_recv() {
                Ok(()) | Err(TryRecvError::Disconnected) => return Ended::Stop,
                Err(TryRecvError::Empty) => {}
            }
            if self.prefs.generation() != opened.generation {
                return Ended::Rebuild("another output was chosen".into());
            }
            if opened.choice.follows_default() && looked.elapsed() >= DEVICE_POLL {
                looked = Instant::now();
                let host = cpal::default_host();
                let now = host.default_output_device().map(|d| device_name(&d));
                if now.as_deref() != Some(opened.device.as_str()) {
                    return Ended::Rebuild(format!(
                        "the default output is now {}",
                        now.as_deref().unwrap_or("none")
                    ));
                }
                if opened.choice == Choice::FellBack {
                    let chosen = self.prefs.device();
                    let back = output_names()
                        .is_ok_and(|names| chosen.is_some_and(|c| names.contains(&c)));
                    if back {
                        return Ended::Rebuild("the chosen output is back".into());
                    }
                }
            }
        }
    }
}

/// How long the sound takes to come back after a refill (a fade-in, not a step).
const REFILL_FADE_SECONDS: f64 = 0.005;

/// State of the linear resampler, and of the refill after the queue ran dry.
///
/// An empty queue fades the sound out. Then, instead of playing each sample as it arrives (the
/// queue would hover around empty, every late frame another fade and step: a crackle for the
/// many seconds the ±0.5 % control needs to refill it), the output stays silent until the
/// queue holds its target again, then fades back in. A stall of the emulation thread (a 3DS
/// quick save on Vulkan, Trello #98), a pause or a game's start is then one clean gap.
struct Resampler {
    pos: f64,
    prev: [f32; 2],
    next: [f32; 2],
    /// The queue ran dry: nothing is read until it holds the target again.
    refilling: bool,
    /// The output's gain: 1, or rising back to 1 after a refill.
    gain: f32,
}

impl Resampler {
    fn new() -> Self {
        Self {
            pos: 1.0,
            prev: [0.0; 2],
            next: [0.0; 2],
            // A game's start fills the queue first, like any refill.
            refilling: true,
            gain: 0.0,
        }
    }

    /// At the start of a callback, with `queued` samples waiting and the queue's `target`: a
    /// refill ends once the target is reached, and the sound fades back in from there.
    fn begin(&mut self, queued: f64, target: f64) {
        if self.refilling && queued >= target {
            self.refilling = false;
            self.gain = 0.0;
        }
    }

    /// One output frame, `step` input frames after the last one, reading the queue with `pop`
    /// (`None`: empty); the gain rises by `gain_step` per frame after a refill. Counts in
    /// `underruns` the input frames that were not the game's audio (a fade-out or a refill).
    fn frame(
        &mut self,
        step: f64,
        gain_step: f32,
        underruns: &mut u64,
        mut pop: impl FnMut() -> Option<[i16; 2]>,
    ) -> [f32; 2] {
        while self.pos >= 1.0 {
            self.prev = self.next;
            let read = if self.refilling { None } else { pop() };
            self.next = match read {
                Some([l, r]) => [f32::from(l) / 32768.0, f32::from(r) / 32768.0],
                None => {
                    *underruns += 1;
                    self.refilling = true;
                    // Empty queue: fade out.
                    [self.prev[0] * 0.98, self.prev[1] * 0.98]
                }
            };
            self.pos -= 1.0;
        }
        let t = self.pos as f32;
        let g = self.gain;
        if !self.refilling && self.gain < 1.0 {
            self.gain = (self.gain + gain_step).min(1.0);
        }
        self.pos += step;
        [
            (self.prev[0] + (self.next[0] - self.prev[0]) * t) * g,
            (self.prev[1] + (self.next[1] - self.prev[1]) * t) * g,
        ]
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
    /// `audio.lat`, read at every callback (an atomic).
    prefs: Arc<AudioPrefs>,
    /// The developer knob `EMUDOME_AUDIO_TARGET_MS`, in seconds: then the ring's own limit.
    fixed_target: Option<f64>,
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
        let rate = ring.sample_rate();
        let (queued, target_seconds) = match self.fixed_target {
            Some(seconds) => (reader.begin(ring), seconds),
            None => {
                let latency = self.prefs.latency();
                let max_len = samples_in(rate, latency.max_ms());
                (
                    reader.begin_within(ring, max_len),
                    f64::from(latency.target_ms()) / 1000.0,
                )
            }
        };
        let queued = queued as f64;
        let fill_ms = queued / 2.0 / rate * 1000.0;
        // Base ratio plus a correction proportional to the queue's fill level (±0.5%).
        let target = rate * target_seconds * 2.0;
        let fill_err = ((queued - target) / target).clamp(-1.0, 1.0);
        let ratio = 1.0 + 0.005 * fill_err;
        let step = rate / self.out_rate * ratio;
        let gain_step = (1.0 / (self.out_rate * REFILL_FADE_SECONDS)) as f32;
        let mut underrun_frames = 0u64;
        let mut discontinuities = 0u64;
        let rs = &mut self.rs;
        rs.begin(queued, target);
        for frame in out.chunks_mut(self.channels) {
            let [l, r] = rs.frame(step, gain_step, &mut underrun_frames, || reader.pop_frame());
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
    prefs: &Arc<AudioPrefs>,
    tone: Option<f32>,
    metrics: &SharedAudioMetrics,
    measured: Option<SharedAudioMetrics>,
    errors: Sender<String>,
) -> anyhow::Result<Opened> {
    let host = cpal::default_host();
    // Read before the device: a choice made while this stream opens replaces it at once.
    let generation = prefs.generation();
    let chosen = prefs.device();
    let devices: Vec<cpal::Device> = host.output_devices()?.collect();
    let names: Vec<String> = devices.iter().map(device_name).collect();
    let choice = choose(&names, tuning.device.as_deref(), chosen.as_deref())
        .map_err(|e| anyhow::anyhow!(e))?;
    let device = match &choice {
        Choice::Named(i) => devices
            .get(*i)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("output device {i} gone"))?,
        Choice::System | Choice::FellBack => {
            if choice == Choice::FellBack {
                log::warn!(
                    "audio: the chosen output {:?} is not there, the system's plays until it is back",
                    chosen.as_deref().unwrap_or_default()
                );
            }
            host.default_output_device()
                .ok_or_else(|| anyhow::anyhow!("no audio device"))?
        }
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
        devices: names,
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
    let latency = prefs.latency();
    log::info!(
        "audio: {} — {} Hz, {} channels, {:?}, buffer {}, queue target {} ms ({:?})",
        info.name,
        out_rate,
        channels,
        sample_format,
        info.buffer,
        tuning.target_ms.unwrap_or(latency.target_ms()),
        latency
    );
    let name = info.name.clone();
    metrics.lock().device = Some(info);

    let state = Fill {
        current,
        prefs: prefs.clone(),
        fixed_target: tuning.target_ms.map(|ms| f64::from(ms) / 1000.0),
        out_rate,
        channels,
        rs: Resampler::new(),
        game: 0,
        tone,
        check: tone.map(|a| ToneCheck::new(a, out_rate)),
        metrics: measured,
    };
    // cpal::SampleFormat is `#[non_exhaustive]`: a fallback arm is the only way to match it.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "cpal::SampleFormat is non_exhaustive: the fallback arm is required"
    )]
    let stream = match sample_format {
        cpal::SampleFormat::F32 => build_typed::<f32>(&device, config, state, errors),
        cpal::SampleFormat::F64 => build_typed::<f64>(&device, config, state, errors),
        cpal::SampleFormat::I8 => build_typed::<i8>(&device, config, state, errors),
        cpal::SampleFormat::I16 => build_typed::<i16>(&device, config, state, errors),
        cpal::SampleFormat::I32 => build_typed::<i32>(&device, config, state, errors),
        cpal::SampleFormat::I64 => build_typed::<i64>(&device, config, state, errors),
        cpal::SampleFormat::U8 => build_typed::<u8>(&device, config, state, errors),
        cpal::SampleFormat::U16 => build_typed::<u16>(&device, config, state, errors),
        cpal::SampleFormat::U32 => build_typed::<u32>(&device, config, state, errors),
        cpal::SampleFormat::U64 => build_typed::<u64>(&device, config, state, errors),
        other => unhandled(other),
    }?;
    Ok(Opened {
        stream,
        device: name,
        choice,
        generation,
    })
}

/// A format with no arm (24-bit packed, DSD, a future one) is refused, never mis-played.
fn unhandled(format: cpal::SampleFormat) -> anyhow::Result<cpal::Stream> {
    anyhow::bail!("unhandled audio format: {format:?}")
}

/// The stream for samples of type `T`: the callback fills `f32` frames and converts them. A
/// stream error that needs a new stream is sent on `errors`.
fn build_typed<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut state: Fill,
    errors: Sender<String>,
) -> anyhow::Result<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    // Grows once to the device's buffer, then is reused.
    let mut buf: Vec<f32> = Vec::new();
    let on_error = move |e: cpal::Error| {
        log::error!("audio error: {e}");
        if needs_rebuild(e.kind()) {
            let _ = errors.send(e.to_string());
        }
    };
    Ok(device.build_output_stream(
        config,
        move |data: &mut [T], info| {
            buf.resize(data.len(), 0.0);
            state.fill(&mut buf, info);
            for (o, s) in data.iter_mut().zip(&buf) {
                *o = T::from_sample(s.clamp(-1.0, 1.0));
            }
        },
        on_error,
        None,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn the_chosen_output_is_opened_by_its_exact_name_and_a_missing_one_falls_back() {
        let list = names(&["Speakers (Realtek)", "Headphones (USB)"]);
        assert_eq!(choose(&list, None, None), Ok(Choice::System));
        assert_eq!(
            choose(&list, None, Some("Headphones (USB)")),
            Ok(Choice::Named(1))
        );
        // Unplugged: the system's plays, the choice is kept (the stream follows the default
        // and looks for the device to come back).
        let gone = choose(&list, None, Some("Bluetooth (JBL)"));
        assert_eq!(gone, Ok(Choice::FellBack));
        assert!(gone.is_ok_and(|c| c.follows_default()));
        assert!(!Choice::Named(0).follows_default());
        // Exact, not a part of the name.
        assert_eq!(
            choose(&list, None, Some("Headphones")),
            Ok(Choice::FellBack)
        );
    }

    #[test]
    fn the_developer_knob_wins_and_matches_a_part_of_the_name() {
        let list = names(&["Speakers (Realtek)", "Headphones (USB)"]);
        assert_eq!(
            choose(&list, Some("usb"), Some("Speakers (Realtek)")),
            Ok(Choice::Named(1))
        );
        assert!(choose(&list, Some("hdmi"), None).is_err());
    }

    #[test]
    fn system_is_no_device_and_the_latency_defaults_to_balanced() {
        use emudome_core::settings::Value;
        let (device, latency) = chosen(|_| None);
        assert_eq!((device, latency), (None, Latency::Balanced));
        let (device, latency) = chosen(|key| match key {
            "audio.out" => Some(Value::Text("Headphones (USB)".into())),
            "audio.lat" => Some(Value::Text("safe".into())),
            _ => None,
        });
        assert_eq!(device.as_deref(), Some("Headphones (USB)"));
        assert_eq!(latency, Latency::Safe);
        assert!(is_output_key("audio.out") && is_output_key("audio.lat"));
        assert!(!is_output_key("audio.vol"));
    }

    #[test]
    fn a_new_device_bumps_the_generation_a_new_latency_does_not() {
        let prefs = AudioPrefs::new(None, Latency::Balanced);
        let g = prefs.generation();
        prefs.set(None, Latency::Low);
        assert_eq!(prefs.generation(), g, "the latency applies live");
        assert_eq!(prefs.latency(), Latency::Low);
        prefs.set(Some("Headphones (USB)".into()), Latency::Low);
        assert_eq!(prefs.generation(), g + 1);
        assert_eq!(prefs.device().as_deref(), Some("Headphones (USB)"));
    }

    #[test]
    fn a_lost_device_rebuilds_the_stream_an_underrun_does_not() {
        assert!(needs_rebuild(cpal::ErrorKind::DeviceNotAvailable));
        assert!(needs_rebuild(cpal::ErrorKind::StreamInvalidated));
        assert!(needs_rebuild(cpal::ErrorKind::BackendError));
        assert!(!needs_rebuild(cpal::ErrorKind::Xrun));
        assert!(!needs_rebuild(cpal::ErrorKind::DeviceChanged));
        assert!(!needs_rebuild(cpal::ErrorKind::RealtimeDenied));
    }

    /// Trello #98: the emulation thread stops for 0.3 s (a 3DS quick save on Vulkan). The
    /// output fades out once, stays silent until the queue is back at its target, fades in, and
    /// never runs dry again: one gap, no crackle while the queue would refill slowly.
    #[test]
    fn a_stall_is_one_clean_gap_and_the_queue_refills_to_its_target() {
        use std::collections::VecDeque;

        let rate = 48000.0;
        let target = rate * 0.060 * 2.0; // samples, as `play` counts them
        let frame_len = 800u64; // one 60 Hz frame of audio
        let callback_len = 480u64;
        let gain_step = (1.0 / (rate * REFILL_FADE_SECONDS)) as f32;
        let w = std::f64::consts::TAU * TEST_TONE_HZ / rate;
        let (stall_from, stall_to) = (96_000u64, 96_000 + 14_400); // 2 s in, 0.3 s long

        let mut queue: VecDeque<[i16; 2]> = VecDeque::new();
        let mut produced = 0u64;
        let mut rs = Resampler::new();
        let mut out = Vec::new();
        let mut dry_spells = 0;
        let mut underruns = 0u64;
        let mut t = 0u64;
        while t < 6 * 48_000 {
            // The core: a frame of a 440 Hz tone every 800 output frames, except while stalled.
            while produced <= t {
                if !(stall_from..stall_to).contains(&produced) {
                    let start = produced;
                    queue.extend((start..start + frame_len).map(|n| {
                        let s = ((n as f64 * w).sin() * 8000.0) as i16;
                        [s, s]
                    }));
                }
                produced += frame_len;
            }
            // One callback.
            let was_playing = !rs.refilling;
            rs.begin((queue.len() * 2) as f64, target);
            for _ in 0..callback_len {
                out.push(rs.frame(1.0, gain_step, &mut underruns, || queue.pop_front())[0]);
            }
            if was_playing && rs.refilling {
                dry_spells += 1;
            }
            t += callback_len;
        }
        assert_eq!(dry_spells, 1, "the queue ran dry only at the stall");
        assert!(
            queue.len() * 2 >= (target as usize) - (frame_len as usize) * 2,
            "the queue is back near its target: {}",
            queue.len()
        );
        // No click: a 440 Hz tone of 8000 moves at most ~0.014 per sample at 48 kHz; a step
        // from silence to the tone would be up to 0.24.
        let jump = out
            .windows(2)
            .map(|p| (p[1] - p[0]).abs())
            .fold(0.0f32, f32::max);
        assert!(jump < 0.02, "largest step {jump}");
        // Silent for the stall and the refill, then the tone at full level.
        let tail = &out[out.len() - 48_000..];
        let peak = tail.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.24, "peak {peak}");
    }

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
