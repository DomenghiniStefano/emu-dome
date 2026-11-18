//! Developer and bench knobs, read once at startup from the environment (Trello #19).
//!
//! They are not user settings (those live in [`crate::settings`]): they exist to measure the
//! video and audio paths, and what the measures settle can then become a setting. With none
//! of them set, the defaults are exactly the app's normal behaviour.
//!
//! | Variable | Values | Default | What it changes |
//! | --- | --- | --- | --- |
//! | `EMUDOME_VIDEO` | `native`, `webview` | per OS (`emudome_platform::native_video_default`) | where the game is drawn |
//! | `EMUDOME_PRESENT_MODE` | `fifo`, `fiforelaxed`, `mailbox`, `immediate`, `autovsync`, `autonovsync` | `autovsync` | wgpu present mode of the native surface |
//! | `EMUDOME_FRAME_LATENCY` | `1`..`3` | `1` | frames the GPU may queue ahead (`desired_maximum_frame_latency`) |
//! | `EMUDOME_AUDIO_TARGET_MS` | `10`..`500` | `60` | audio queue fill the resampler steers towards |
//! | `EMUDOME_AUDIO_BUFFER` | `default` or frames (`256`, `512`...) | `default` | device buffer requested from cpal |
//! | `EMUDOME_AUDIO_RATE` | Hz (`44100`, `48000`...) | the device's | output sample rate requested |
//! | `EMUDOME_AUDIO_CHANNELS` | `1`..`8` | the device's | output channels requested |
//! | `EMUDOME_AUDIO_DEVICE` | part of a device name | the system default | output device |
//! | `EMUDOME_DATA_DIR` | folder | the app data folder | everything: saves, states, system files, settings, library, covers, first-run marker (under `local/`), webview profile |
//! | `EMUDOME_BENCH` | seconds | off | bench mode: measure for N s, write a report, exit |
//! | `EMUDOME_BENCH_OUT` | file | `emudome-bench.json` | where the bench report goes |
//! | `EMUDOME_BENCH_WARMUP` | seconds | `2` | time after the first frame before measuring |
//! | `EMUDOME_BENCH_MENU` | `1` | off | opens the quick menu 5 s after the first frame |
//! | `EMUDOME_BENCH_TONE` | amplitude `0`..`1` | off | replaces the game's audio with a 440 Hz tone |
//!
//! Read by wgpu itself, recorded here for the reports: `WGPU_BACKEND` (`vulkan`, `dx12`, `gl`...)
//! and `WGPU_DX12_PRESENTATION_SYSTEM` (`Hwnd`, `Visual`).
//!
//! A value that does not parse is ignored (the default stays) and listed in [`Tuning::warnings`].

use std::path::PathBuf;

use serde::Serialize;

/// Where the game is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoPath {
    /// wgpu on the window, under the transparent webview.
    Native,
    /// A canvas inside the webview.
    Webview,
}

/// Present mode of the native surface, mirror of `wgpu::PresentMode` (the core does not
/// depend on wgpu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PresentMode {
    Fifo,
    FifoRelaxed,
    Mailbox,
    Immediate,
    AutoVsync,
    AutoNoVsync,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoTuning {
    /// `None`: the operating system's default.
    pub path: Option<VideoPath>,
    pub present_mode: PresentMode,
    /// Frames the GPU may queue ahead of the one on screen.
    pub frame_latency: u32,
    /// `WGPU_BACKEND` as given (wgpu reads it itself).
    pub wgpu_backend: Option<String>,
    /// `WGPU_DX12_PRESENTATION_SYSTEM` as given (wgpu reads it itself).
    pub dx12_presentation: Option<String>,
}

/// Device buffer requested from cpal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioBuffer {
    /// Whatever the system chooses.
    Default,
    /// A fixed size in frames.
    Frames(u32),
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioTuning {
    /// Queue fill, in milliseconds, the resampler's speed control steers towards.
    pub target_ms: u32,
    pub buffer: AudioBuffer,
    /// `None`: the device's own rate.
    pub rate: Option<u32>,
    /// `None`: the device's own channel count.
    pub channels: Option<u16>,
    /// Part of the output device's name (case-insensitive); `None`: the system default.
    pub device: Option<String>,
}

/// Bench mode: open the game, measure, write a JSON report, exit.
#[derive(Debug, Clone, Serialize)]
pub struct BenchTuning {
    /// Measured time, after the warm-up.
    pub seconds: f64,
    pub out: PathBuf,
    pub warmup_seconds: f64,
    /// Opens the quick menu this long after the first frame (to capture the native video under it).
    pub open_menu_after: Option<f64>,
    /// Amplitude (0..1) of the 440 Hz tone that replaces the game's audio.
    pub tone: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tuning {
    pub video: VideoTuning,
    pub audio: AudioTuning,
    /// `None`: the app data folder.
    pub data_dir: Option<PathBuf>,
    pub bench: Option<BenchTuning>,
    /// Values that did not parse, as `NAME=value`: they were ignored.
    pub warnings: Vec<String>,
}

/// Frames the GPU may queue: the app's value. The Windows runs suggest 2 presents at a
/// steadier cadence with the same CPU (docs/bench-windows.md); not adopted yet.
pub const DEFAULT_FRAME_LATENCY: u32 = 1;
/// The audio queue target of the app (60 ms).
pub const DEFAULT_AUDIO_TARGET_MS: u32 = 60;
/// When the bench opens the quick menu, after the first frame.
const BENCH_MENU_AFTER_SECONDS: f64 = 5.0;

impl Tuning {
    /// Reads the process environment.
    pub fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the knobs through `lookup` (the environment, or a table in the tests).
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let mut r = Reader {
            lookup: &lookup,
            warnings: Vec::new(),
        };
        let video = VideoTuning {
            path: r.parse("EMUDOME_VIDEO", |v| match v {
                "native" => Some(VideoPath::Native),
                "webview" => Some(VideoPath::Webview),
                _ => None,
            }),
            present_mode: r
                .parse("EMUDOME_PRESENT_MODE", parse_present_mode)
                .unwrap_or(PresentMode::AutoVsync),
            frame_latency: r
                .parse("EMUDOME_FRAME_LATENCY", |v| {
                    v.parse().ok().filter(|n| (1..=3).contains(n))
                })
                .unwrap_or(DEFAULT_FRAME_LATENCY),
            wgpu_backend: r.raw("WGPU_BACKEND"),
            dx12_presentation: r.raw("WGPU_DX12_PRESENTATION_SYSTEM"),
        };
        let audio = AudioTuning {
            target_ms: r
                .parse("EMUDOME_AUDIO_TARGET_MS", |v| {
                    v.parse().ok().filter(|n| (10..=500).contains(n))
                })
                .unwrap_or(DEFAULT_AUDIO_TARGET_MS),
            buffer: r
                .parse("EMUDOME_AUDIO_BUFFER", |v| match v {
                    "default" => Some(AudioBuffer::Default),
                    n => n
                        .parse()
                        .ok()
                        .filter(|n| (16..=16384).contains(n))
                        .map(AudioBuffer::Frames),
                })
                .unwrap_or(AudioBuffer::Default),
            rate: r.parse("EMUDOME_AUDIO_RATE", |v| {
                v.parse().ok().filter(|n| (8000..=384_000).contains(n))
            }),
            channels: r.parse("EMUDOME_AUDIO_CHANNELS", |v| {
                v.parse().ok().filter(|n| (1..=8).contains(n))
            }),
            device: r.raw("EMUDOME_AUDIO_DEVICE"),
        };
        let data_dir = r.raw("EMUDOME_DATA_DIR").map(PathBuf::from);
        let bench = r
            .parse("EMUDOME_BENCH", |v| {
                v.parse::<f64>().ok().filter(|s| *s > 0.0 && s.is_finite())
            })
            .map(|seconds| BenchTuning {
                seconds,
                out: r
                    .raw("EMUDOME_BENCH_OUT")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("emudome-bench.json")),
                warmup_seconds: r
                    .parse("EMUDOME_BENCH_WARMUP", |v| {
                        v.parse::<f64>().ok().filter(|s| *s >= 0.0 && s.is_finite())
                    })
                    .unwrap_or(2.0),
                open_menu_after: r
                    .parse("EMUDOME_BENCH_MENU", |v| match v {
                        "1" => Some(true),
                        "0" => Some(false),
                        _ => None,
                    })
                    .filter(|open| *open)
                    .map(|_| BENCH_MENU_AFTER_SECONDS),
                tone: r.parse("EMUDOME_BENCH_TONE", |v| {
                    v.parse::<f32>().ok().filter(|a| (0.0..=1.0).contains(a))
                }),
            });
        Self {
            video,
            audio,
            data_dir,
            bench,
            warnings: r.warnings,
        }
    }
}

fn parse_present_mode(v: &str) -> Option<PresentMode> {
    match v {
        "fifo" => Some(PresentMode::Fifo),
        "fiforelaxed" => Some(PresentMode::FifoRelaxed),
        "mailbox" => Some(PresentMode::Mailbox),
        "immediate" => Some(PresentMode::Immediate),
        "autovsync" => Some(PresentMode::AutoVsync),
        "autonovsync" => Some(PresentMode::AutoNoVsync),
        _ => None,
    }
}

struct Reader<'a, F: Fn(&str) -> Option<String>> {
    lookup: &'a F,
    warnings: Vec<String>,
}

impl<F: Fn(&str) -> Option<String>> Reader<'_, F> {
    /// The value, trimmed; `None` when unset or empty.
    fn raw(&self, name: &str) -> Option<String> {
        (self.lookup)(name)
            .map(|v| v.trim().to_owned())
            .filter(|v| !v.is_empty())
    }

    /// The value parsed (lowercase); a value that does not parse is recorded and ignored.
    fn parse<T>(&mut self, name: &str, f: impl FnOnce(&str) -> Option<T>) -> Option<T> {
        let value = self.raw(name)?;
        let parsed = f(&value.to_lowercase());
        if parsed.is_none() {
            self.warnings.push(format!("{name}={value}"));
        }
        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning(vars: &[(&str, &str)]) -> Tuning {
        Tuning::from_lookup(|name| {
            vars.iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| (*v).to_owned())
        })
    }

    #[test]
    fn defaults_are_the_app_behaviour() {
        let t = tuning(&[]);
        assert_eq!(t.video.path, None);
        assert_eq!(t.video.present_mode, PresentMode::AutoVsync);
        assert_eq!(t.video.frame_latency, DEFAULT_FRAME_LATENCY);
        assert_eq!(t.audio.target_ms, 60);
        assert_eq!(t.audio.buffer, AudioBuffer::Default);
        assert_eq!(t.audio.rate, None);
        assert_eq!(t.audio.channels, None);
        assert!(t.bench.is_none());
        assert!(t.warnings.is_empty());
    }

    #[test]
    fn parses_every_knob() {
        let t = tuning(&[
            ("EMUDOME_VIDEO", "webview"),
            ("EMUDOME_PRESENT_MODE", "Mailbox"),
            ("EMUDOME_FRAME_LATENCY", "3"),
            ("EMUDOME_AUDIO_TARGET_MS", "100"),
            ("EMUDOME_AUDIO_BUFFER", "512"),
            ("EMUDOME_AUDIO_RATE", "48000"),
            ("EMUDOME_AUDIO_CHANNELS", "2"),
            ("EMUDOME_BENCH", "30"),
            ("EMUDOME_BENCH_OUT", "out.json"),
            ("EMUDOME_BENCH_MENU", "1"),
            ("EMUDOME_BENCH_TONE", "0.05"),
        ]);
        assert_eq!(t.video.path, Some(VideoPath::Webview));
        assert_eq!(t.video.present_mode, PresentMode::Mailbox);
        assert_eq!(t.video.frame_latency, 3);
        assert_eq!(t.audio.target_ms, 100);
        assert_eq!(t.audio.buffer, AudioBuffer::Frames(512));
        assert_eq!(t.audio.rate, Some(48000));
        assert_eq!(t.audio.channels, Some(2));
        let bench = t.bench.expect("bench on");
        assert_eq!(bench.seconds, 30.0);
        assert_eq!(bench.out, PathBuf::from("out.json"));
        assert_eq!(bench.open_menu_after, Some(5.0));
        assert_eq!(bench.tone, Some(0.05));
        assert!(t.warnings.is_empty());
    }

    #[test]
    fn invalid_values_keep_the_default_and_warn() {
        let t = tuning(&[
            ("EMUDOME_PRESENT_MODE", "vsync-please"),
            ("EMUDOME_FRAME_LATENCY", "9"),
            ("EMUDOME_AUDIO_BUFFER", "huge"),
        ]);
        assert_eq!(t.video.present_mode, PresentMode::AutoVsync);
        assert_eq!(t.video.frame_latency, DEFAULT_FRAME_LATENCY);
        assert_eq!(t.audio.buffer, AudioBuffer::Default);
        assert_eq!(
            t.warnings,
            [
                "EMUDOME_PRESENT_MODE=vsync-please",
                "EMUDOME_FRAME_LATENCY=9",
                "EMUDOME_AUDIO_BUFFER=huge"
            ]
        );
    }
}
