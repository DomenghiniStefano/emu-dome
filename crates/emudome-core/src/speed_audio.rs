//! The game's audio at speeds other than 1× (fast forward, slow motion), as `audio.ff` asks:
//!
//! - `mute` ([`FastAudio::Mute`]): the samples are dropped;
//! - `pitch_up` ([`FastAudio::Pitch`]): resampled by the speed, so the pitch follows it (higher
//!   in fast forward, lower in slow motion), like a tape played faster;
//! - `same_pitch` ([`FastAudio::KeepPitch`]): time-stretched with WSOLA (waveform-similarity
//!   overlap-add): 30 ms grains with a Hann window, overlapped by half, each one taken where it
//!   best continues the previous one within ±8 ms of where the speed puts it. The pitch stays,
//!   the tempo follows the speed.
//!
//! Either way the output is about real time: at N× the core makes N seconds of audio per second
//! and [`SpeedAudio`] turns them into one, so the audio output keeps its queue at the usual fill
//! and its ±0.5 % rate correction works as at 1×. At 1× the samples pass untouched. At Max the
//! speed is measured ([`SpeedMeter`]); until it is known the audio is dropped.
//!
//! Pure and single-threaded: the emulation thread calls it from the audio callbacks.

use std::time::{Duration, Instant};

/// What the game's audio does at a speed other than 1× (`audio.ff`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FastAudio {
    /// Silent.
    #[default]
    Mute,
    /// Played at the speed: the pitch moves with it.
    Pitch,
    /// Played at the speed with the pitch kept (time-stretch).
    KeepPitch,
}

impl FastAudio {
    /// The option id of `audio.ff`; None for anything else.
    pub fn from_option(id: &str) -> Option<Self> {
        match id {
            "mute" => Some(Self::Mute),
            "pitch_up" => Some(Self::Pitch),
            "same_pitch" => Some(Self::KeepPitch),
            _ => None,
        }
    }
}

/// A speed this close to 1 is 1×.
const NORMAL: f64 = 1e-3;

fn is_normal(speed: f64) -> bool {
    (speed - 1.0).abs() < NORMAL
}

/// Turns the samples a core makes at some speed into samples for real-time playback.
#[derive(Debug, Default)]
pub struct SpeedAudio {
    policy: FastAudio,
    /// The emulation speed (1 = normal); None = not known yet (Max, before it is measured).
    speed: Option<f64>,
    pitch: Pitch,
    stretch: Option<Stretch>,
}

impl SpeedAudio {
    pub fn new(policy: FastAudio) -> Self {
        Self {
            policy,
            speed: Some(1.0),
            ..Self::default()
        }
    }

    pub fn policy(&self) -> FastAudio {
        self.policy
    }

    pub fn set_policy(&mut self, policy: FastAudio) {
        if policy != self.policy {
            self.policy = policy;
            self.reset();
        }
    }

    /// The emulation speed, None while it is not known.
    pub fn set_speed(&mut self, speed: Option<f64>) {
        let speed = speed.filter(|s| s.is_finite() && *s > 0.0);
        let was_normal = self.speed.is_some_and(is_normal);
        if was_normal && !speed.is_some_and(is_normal) {
            // Leaving 1×: the converters start clean.
            self.reset();
        }
        self.speed = speed;
    }

    /// Whether the samples go through untouched (1×).
    pub fn is_passthrough(&self) -> bool {
        self.speed.is_some_and(is_normal)
    }

    fn reset(&mut self) {
        self.pitch = Pitch::default();
        self.stretch = None;
    }

    /// Appends to `out` the real-time samples for `input`: interleaved stereo at `rate` Hz,
    /// made at the current speed.
    pub fn process(&mut self, rate: f64, input: &[i16], out: &mut Vec<i16>) {
        let Some(speed) = self.speed else {
            return;
        };
        if is_normal(speed) {
            out.extend_from_slice(input);
            return;
        }
        match self.policy {
            FastAudio::Mute => {}
            FastAudio::Pitch => self.pitch.process(speed, input, out),
            FastAudio::KeepPitch => {
                let stretch = match &mut self.stretch {
                    Some(s) if s.rate == rate => s,
                    other => other.insert(Stretch::new(rate)),
                };
                stretch.process(speed, input, out);
            }
        }
    }
}

fn frames(input: &[i16]) -> impl Iterator<Item = [f32; 2]> + '_ {
    input
        .chunks_exact(2)
        .map(|f| [f32::from(f[0]), f32::from(f[1])])
}

fn push_frame(out: &mut Vec<i16>, f: [f32; 2]) {
    let s = |v: f32| v.round().clamp(-32768.0, 32767.0) as i16;
    out.extend([s(f[0]), s(f[1])]);
}

/// Resampling by the speed: `speed` input frames per output frame. Faster: each output frame is
/// the average of the input it covers (a box filter, so the dropped highs do not fold back as
/// noise). Slower: linear interpolation.
#[derive(Debug, Default)]
struct Pitch {
    // Faster than 1×.
    acc: [f32; 2],
    acc_weight: f64,
    /// Input still needed to complete the output frame (0 = start a new one).
    need: f64,
    // Slower than 1×.
    pos: f64,
    prev: [f32; 2],
}

impl Pitch {
    fn process(&mut self, speed: f64, input: &[i16], out: &mut Vec<i16>) {
        if speed >= 1.0 {
            for f in frames(input) {
                let mut left = 1.0f64;
                while left > 1e-9 {
                    if self.need <= 1e-9 {
                        self.need = speed;
                    }
                    let take = left.min(self.need);
                    for c in 0..2 {
                        self.acc[c] += f[c] * take as f32;
                    }
                    self.acc_weight += take;
                    self.need -= take;
                    left -= take;
                    if self.need <= 1e-9 {
                        let w = self.acc_weight as f32;
                        push_frame(out, [self.acc[0] / w, self.acc[1] / w]);
                        self.acc = [0.0; 2];
                        self.acc_weight = 0.0;
                    }
                }
            }
        } else {
            for next in frames(input) {
                while self.pos < 1.0 {
                    let t = self.pos as f32;
                    push_frame(
                        out,
                        [
                            self.prev[0] + (next[0] - self.prev[0]) * t,
                            self.prev[1] + (next[1] - self.prev[1]) * t,
                        ],
                    );
                    self.pos += speed;
                }
                self.pos -= 1.0;
                self.prev = next;
            }
        }
    }
}

/// WSOLA time-stretch (see the module comment).
#[derive(Debug)]
struct Stretch {
    rate: f64,
    /// Grain length, frames (even).
    len: usize,
    /// Output hop: half a grain.
    hop: usize,
    /// How far from its nominal place a grain may be taken.
    search: usize,
    /// One sample in this many is compared when looking for the best grain (about 8 kHz).
    compare_step: usize,
    window: Vec<f32>,
    /// Input not used up yet; `buf[0]` is input frame number `base`.
    buf: Vec<[f32; 2]>,
    base: usize,
    /// Where the speed puts the next grain (input frame number).
    pos: f64,
    /// Where the previous grain was taken.
    prev: Option<usize>,
    /// The second half of the previous grain, windowed: added to the next grain's first half.
    tail: Vec<[f32; 2]>,
}

impl Stretch {
    const GRAIN: f64 = 0.030;
    const SEARCH: f64 = 0.008;

    fn new(rate: f64) -> Self {
        let rate = if rate.is_finite() && rate > 1000.0 {
            rate
        } else {
            32768.0
        };
        let len = (((rate * Self::GRAIN) as usize) / 2 * 2).max(64);
        let hop = len / 2;
        // Periodic Hann: two windows half a grain apart add up to exactly 1.
        let window = (0..len)
            .map(|i| {
                let x = std::f64::consts::TAU * i as f64 / len as f64;
                (0.5 - 0.5 * x.cos()) as f32
            })
            .collect();
        Self {
            rate,
            len,
            hop,
            search: (rate * Self::SEARCH) as usize,
            compare_step: ((rate / 8192.0).round() as usize).max(1),
            window,
            buf: Vec::new(),
            base: 0,
            pos: 0.0,
            prev: None,
            tail: vec![[0.0; 2]; hop],
        }
    }

    fn at(&self, frame: usize) -> [f32; 2] {
        self.buf[frame - self.base]
    }

    /// The start in `lo..=hi` whose first half best matches the input following the previous
    /// grain (`natural`): highest correlation over the candidate's energy.
    fn best_start(&self, lo: usize, hi: usize, natural: usize) -> usize {
        let mono = |f: [f32; 2]| f[0] + f[1];
        let target: Vec<f32> = (0..self.hop)
            .step_by(self.compare_step)
            .map(|i| mono(self.at(natural + i)))
            .collect();
        let mut best = (natural.clamp(lo, hi), f32::MIN);
        for c in lo..=hi {
            let (mut dot, mut energy) = (0.0f32, 0.0f32);
            for (k, t) in target.iter().enumerate() {
                let v = mono(self.at(c + k * self.compare_step));
                dot += v * t;
                energy += v * v;
            }
            if energy > 1.0 {
                let score = dot / energy.sqrt();
                if score > best.1 {
                    best = (c, score);
                }
            }
        }
        best.0
    }

    fn process(&mut self, speed: f64, input: &[i16], out: &mut Vec<i16>) {
        self.buf.extend(frames(input));
        let analysis_hop = speed * self.hop as f64;
        loop {
            let nominal = (self.pos.round() as usize).max(self.base);
            let lo = nominal.saturating_sub(self.search).max(self.base);
            let hi = nominal + self.search;
            let natural = self.prev.map(|p| p + self.hop);
            let end = self.base + self.buf.len();
            if end < hi + self.len || natural.is_some_and(|n| end < n + self.hop) {
                break;
            }
            let start = match natural {
                Some(n) => self.best_start(lo, hi, n),
                None => nominal,
            };
            for i in 0..self.hop {
                let (a, b) = (self.at(start + i), self.window[i]);
                let t = self.tail[i];
                push_frame(out, [t[0] + a[0] * b, t[1] + a[1] * b]);
            }
            for i in 0..self.hop {
                let (a, b) = (self.at(start + self.hop + i), self.window[self.hop + i]);
                self.tail[i] = [a[0] * b, a[1] * b];
            }
            self.prev = Some(start);
            self.pos += analysis_hop;
            // What the next grain can still use: its search range and the previous grain's
            // natural continuation.
            let next = (self.pos.round() as usize).saturating_sub(self.search);
            let keep_from = next.min(start + self.hop).max(self.base);
            self.buf.drain(..keep_from - self.base);
            self.base = keep_from;
        }
    }
}

/// The speed at Max, measured: frames run × the frame's period over the time they took, over
/// windows of 100 ms.
#[derive(Debug)]
pub struct SpeedMeter {
    period: f64,
    since: Instant,
    frames: u32,
}

impl SpeedMeter {
    const WINDOW: Duration = Duration::from_millis(100);

    /// `fps`: the core's frame rate at 1×.
    pub fn new(fps: f64, now: Instant) -> Self {
        Self {
            period: 1.0 / fps,
            since: now,
            frames: 0,
        }
    }

    /// One frame run at `now`: the speed when a window just closed.
    pub fn frame(&mut self, now: Instant) -> Option<f64> {
        self.frames += 1;
        let elapsed = now.saturating_duration_since(self.since);
        if elapsed < Self::WINDOW {
            return None;
        }
        let speed = f64::from(self.frames) * self.period / elapsed.as_secs_f64();
        self.since = now;
        self.frames = 0;
        Some(speed)
    }
}

#[cfg(test)]
mod tests {
    use super::{FastAudio, SpeedAudio, SpeedMeter};
    use std::time::{Duration, Instant};

    const RATE: f64 = 32768.0;

    /// `secs` of a stereo sine at `hz`, amplitude 8000.
    fn sine(hz: f64, secs: f64) -> Vec<i16> {
        let n = (RATE * secs) as usize;
        (0..n)
            .flat_map(|i| {
                let s = ((std::f64::consts::TAU * hz * i as f64 / RATE).sin() * 8000.0) as i16;
                [s, s]
            })
            .collect()
    }

    /// Feeds `input` in chunks of one 60 Hz frame, as a core does.
    fn run(audio: &mut SpeedAudio, input: &[i16]) -> Vec<i16> {
        let mut out = Vec::new();
        for chunk in input.chunks(1094) {
            audio.process(RATE, chunk, &mut out);
        }
        out
    }

    /// Left channel, skipping the first `skip_secs` (a converter's start).
    fn left(out: &[i16], skip_secs: f64) -> Vec<f32> {
        out.iter()
            .step_by(2)
            .skip((RATE * skip_secs) as usize)
            .map(|s| f32::from(*s))
            .collect()
    }

    /// Frequency from the rising zero crossings, at the output's rate (real time).
    fn frequency(signal: &[f32]) -> f64 {
        let crossings: Vec<usize> = signal
            .windows(2)
            .enumerate()
            .filter(|(_, w)| w[0] < 0.0 && w[1] >= 0.0)
            .map(|(i, _)| i)
            .collect();
        let (first, last) = (crossings[0], crossings[crossings.len() - 1]);
        (crossings.len() - 1) as f64 * RATE / (last - first) as f64
    }

    fn rms(signal: &[f32]) -> f64 {
        (signal.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / signal.len() as f64).sqrt()
    }

    /// Largest jump between two neighbouring samples: a click shows as a jump far above what
    /// the sine itself moves in one sample.
    fn max_step(signal: &[f32]) -> f32 {
        signal
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn the_settings_ids_are_known() {
        assert_eq!(FastAudio::from_option("mute"), Some(FastAudio::Mute));
        assert_eq!(FastAudio::from_option("pitch_up"), Some(FastAudio::Pitch));
        assert_eq!(
            FastAudio::from_option("same_pitch"),
            Some(FastAudio::KeepPitch)
        );
        assert_eq!(FastAudio::from_option("loud"), None);
    }

    #[test]
    fn at_normal_speed_every_policy_passes_the_samples_untouched() {
        let input = sine(440.0, 0.1);
        for policy in [FastAudio::Mute, FastAudio::Pitch, FastAudio::KeepPitch] {
            let mut a = SpeedAudio::new(policy);
            assert!(a.is_passthrough());
            assert_eq!(run(&mut a, &input), input);
        }
    }

    #[test]
    fn mute_drops_everything_off_normal_speed() {
        let mut a = SpeedAudio::new(FastAudio::Mute);
        a.set_speed(Some(4.0));
        assert!(run(&mut a, &sine(440.0, 0.5)).is_empty());
    }

    #[test]
    fn an_unknown_speed_drops_the_audio() {
        let mut a = SpeedAudio::new(FastAudio::KeepPitch);
        a.set_speed(None);
        assert!(run(&mut a, &sine(440.0, 0.5)).is_empty());
    }

    #[test]
    fn pitch_at_double_speed_halves_the_length_and_doubles_the_frequency() {
        let mut a = SpeedAudio::new(FastAudio::Pitch);
        a.set_speed(Some(2.0));
        let input = sine(440.0, 2.0);
        let out = run(&mut a, &input);
        assert_eq!(out.len(), input.len() / 2);
        let l = left(&out, 0.05);
        let f = frequency(&l);
        assert!((f - 880.0).abs() < 5.0, "frequency {f}");
        assert!(rms(&l) > 5000.0, "level {}", rms(&l));
    }

    #[test]
    fn pitch_at_fractional_speeds_keeps_the_count() {
        for speed in [3.0, 4.0, 7.3, 0.5, 0.25] {
            let mut a = SpeedAudio::new(FastAudio::Pitch);
            a.set_speed(Some(speed));
            let input = sine(300.0, 2.0);
            let frames_out = run(&mut a, &input).len() / 2;
            let expected = input.len() as f64 / 2.0 / speed;
            assert!(
                (frames_out as f64 - expected).abs() <= 2.0,
                "speed {speed}: {frames_out} frames, expected {expected}"
            );
        }
        // Slow motion lowers the pitch.
        let mut a = SpeedAudio::new(FastAudio::Pitch);
        a.set_speed(Some(0.5));
        let f = frequency(&left(&run(&mut a, &sine(440.0, 1.0)), 0.05));
        assert!((f - 220.0).abs() < 3.0, "frequency {f}");
    }

    #[test]
    fn same_pitch_keeps_the_frequency_and_follows_the_speed() {
        for (speed, hz) in [(2.0, 440.0), (4.0, 440.0), (3.0, 1000.0), (0.5, 440.0)] {
            let mut a = SpeedAudio::new(FastAudio::KeepPitch);
            a.set_speed(Some(speed));
            let input = sine(hz, 4.0);
            let out = run(&mut a, &input);
            // Length: input / speed, within the 50 ms of input the stretcher holds back (the
            // first grain fades in, the next one needs its search range).
            let expected = input.len() as f64 / 2.0 / speed;
            let got = out.len() as f64 / 2.0;
            assert!(
                (got - expected).abs() < RATE * 0.05 / speed,
                "speed {speed}: {got} frames, expected about {expected}"
            );
            let l = left(&out, 0.1);
            let f = frequency(&l);
            assert!(
                (f - hz).abs() < hz * 0.01,
                "speed {speed}: frequency {f}, wanted {hz}"
            );
            // The level stays (no holes between grains) and nothing clicks: a 1 kHz sine of
            // 8000 moves at most ~1530 per sample at 32 kHz.
            let level = rms(&l);
            assert!(
                (5000.0..6500.0).contains(&level),
                "speed {speed}: level {level}"
            );
            assert!(
                max_step(&l) < 8000.0 * 0.25,
                "speed {speed}: jump {}",
                max_step(&l)
            );
        }
    }

    #[test]
    fn leaving_and_coming_back_to_normal_speed() {
        let mut a = SpeedAudio::new(FastAudio::KeepPitch);
        a.set_speed(Some(4.0));
        assert!(!a.is_passthrough());
        run(&mut a, &sine(440.0, 1.0));
        a.set_speed(Some(1.0));
        let input = sine(440.0, 0.1);
        assert_eq!(run(&mut a, &input), input);
        // Changing the policy while fast keeps working.
        a.set_speed(Some(2.0));
        a.set_policy(FastAudio::Pitch);
        assert_eq!(run(&mut a, &sine(440.0, 1.0)).len() / 2, 16384);
    }

    #[test]
    fn the_meter_measures_the_speed_over_windows() {
        let t0 = Instant::now();
        let mut m = SpeedMeter::new(60.0, t0);
        // 4× of 60 fps: a frame every 1/240 s.
        let mut last = None;
        for i in 1..=60u32 {
            let at = t0 + Duration::from_secs_f64(f64::from(i) / 240.0);
            if let Some(s) = m.frame(at) {
                last = Some(s);
            }
        }
        let s = last.expect("a window closed");
        assert!((s - 4.0).abs() < 0.01, "speed {s}");
    }
}
