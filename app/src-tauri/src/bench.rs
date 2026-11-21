//! Bench mode (`EMUDOME_BENCH=<seconds>`, see `tuning.rs`): the app opens the game
//! given on the command line, waits for the first frame and a warm-up, measures for the given
//! time, writes a JSON report (`EMUDOME_BENCH_OUT`) and exits. Nothing here runs otherwise.
//!
//! What it measures:
//! - native video: every present of the video thread (time, game frame shown);
//! - webview video: every round of the canvas loop, sent by the interface (`bench_frames`);
//! - the core's frames, the process CPU time, the audio callbacks (`audio::AudioMetrics`),
//!   the time from `main` to the first game frame on screen.
//!
//! The window is created without taking the focus, so a bench can run while the PC is in use.
//!
//! Built only with the `bench` cargo feature (`bench_off.rs` otherwise).

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::tuning::{BenchTuning, Tuning};
use tauri::{Emitter, Manager};

use crate::{AppState, VideoMode};

/// The most canvas rounds one `bench_frames` call may add (the interface sends about a second's
/// worth: under 250 at 240 Hz), and the most the log keeps (about 24 MB): the rest is dropped.
const MAX_FRAMES_PER_CALL: usize = 4096;
const MAX_FRAMES: usize = 1 << 20;

/// How long to wait for the first game frame before giving up.
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(30);

/// One round of the webview's canvas loop, as the interface reports it.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct WebviewFrame {
    /// Wall clock, ms since the Unix epoch (`performance.timeOrigin + performance.now()`).
    t: f64,
    /// The game frame on the canvas after this round.
    seq: f64,
    /// Whether this round drew a new game frame.
    drawn: bool,
}

#[derive(Default)]
struct WebviewLog {
    recording: bool,
    frames: Vec<WebviewFrame>,
    first_drawn_unix_ms: Option<f64>,
}

pub struct Bench {
    tuning: Tuning,
    config: BenchTuning,
    main_start: Instant,
    main_unix_ms: f64,
    webview: Mutex<WebviewLog>,
}

fn unix_ms(t: SystemTime) -> f64 {
    t.duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

impl Bench {
    /// `None` when bench mode is off. `main_start` is taken as early as possible in `run`.
    pub fn new(tuning: &Tuning, main_start: Instant, main_wall: SystemTime) -> Option<Arc<Self>> {
        let config = tuning.bench.clone()?;
        Some(Arc::new(Self {
            tuning: tuning.clone(),
            config,
            main_start,
            main_unix_ms: unix_ms(main_wall),
            webview: Mutex::default(),
        }))
    }

    /// Amplitude of the test tone that replaces the game's audio, if any.
    pub fn tone(&self) -> Option<f32> {
        self.config.tone
    }
}

/// Whether the interface should report its canvas rounds (`bench_frames`).
#[tauri::command]
pub fn bench_active(state: tauri::State<'_, AppState>) -> bool {
    state.bench.is_some()
}

/// The interface's canvas rounds since the previous call (webview video mode).
#[tauri::command]
pub fn bench_frames(state: tauri::State<'_, AppState>, frames: Vec<WebviewFrame>) {
    let Some(bench) = &state.bench else {
        return;
    };
    let mut log = bench.webview.lock();
    if log.first_drawn_unix_ms.is_none() {
        log.first_drawn_unix_ms = frames.iter().find(|f| f.drawn).map(|f| f.t);
    }
    if log.recording {
        let room = MAX_FRAMES
            .saturating_sub(log.frames.len())
            .min(MAX_FRAMES_PER_CALL);
        if frames.len() > room {
            log::warn!("bench: {} canvas rounds dropped", frames.len() - room);
        }
        log.frames.extend(frames.into_iter().take(room));
    }
}

/// Runs the bench on a thread of its own; the app exits when it is done.
pub fn spawn(app: tauri::AppHandle, bench: Arc<Bench>) {
    let spawned = std::thread::Builder::new()
        .name("emudome-bench".into())
        .spawn(move || {
            let (code, report) = run(&app, &bench);
            if let Err(e) = write_report(&bench.config.out, &report) {
                log::error!(
                    "bench: report not written to {}: {e}",
                    bench.config.out.display()
                );
            } else {
                log::info!("bench: report in {}", bench.config.out.display());
            }
            app.exit(code);
        });
    if let Err(e) = spawned {
        log::error!("bench thread not started: {e}");
    }
}

fn write_report(path: &Path, report: &Value) -> std::io::Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(report).map_err(std::io::Error::other)?;
    std::fs::write(path, text)
}

/// When the first game frame reached the screen, in ms since `main`.
fn first_frame_ms(state: &AppState, bench: &Bench, mode: VideoMode) -> Option<f64> {
    match mode {
        VideoMode::Native => {
            let at = state
                .renderer
                .as_ref()?
                .metrics()
                .lock()
                .first_game_present?;
            Some(at.duration_since(bench.main_start).as_secs_f64() * 1000.0)
        }
        VideoMode::Webview => bench
            .webview
            .lock()
            .first_drawn_unix_ms
            .map(|t| t - bench.main_unix_ms),
    }
}

/// The bench itself: exit code and report.
fn run(app: &tauri::AppHandle, bench: &Bench) -> (i32, Value) {
    let state = app.state::<AppState>();
    let mode = *state.video_mode.lock();
    let mut report = json!({
        "schema": 1,
        "os": emudome_platform::OS_NAME,
        "build": if cfg!(debug_assertions) { "debug" } else { "release" },
        "tuning": bench.tuning,
        "video_mode": mode,
        "native_error": state.native_error,
        "main_unix_ms": bench.main_unix_ms,
        "logical_cpus": std::thread::available_parallelism().map_or(0, |n| n.get()),
    });
    let fail = |mut report: Value, status: &str, reason: String| {
        log::error!("bench: {status}: {reason}");
        report["status"] = json!(status);
        report["reason"] = json!(reason);
        report
    };

    if let Some(renderer) = &state.renderer {
        let gpu = renderer.metrics().lock().gpu.clone();
        report["gpu"] = json!(gpu);
        if let Some(gpu) = gpu.filter(|g| !g.present_mode_supported()) {
            let reason = format!(
                "present mode {} not supported by this surface (supported: {})",
                gpu.present_mode_requested,
                gpu.present_modes_supported.join(", ")
            );
            return (3, fail(report, "unsupported", reason));
        }
    }

    // The game opens from the interface (`startup_game`): wait for its first frame on screen.
    let waited = Instant::now();
    let first_ms = loop {
        if let Some(ms) = first_frame_ms(&state, bench, mode) {
            break ms;
        }
        if waited.elapsed() > FIRST_FRAME_TIMEOUT {
            let reason = "no game frame on screen within 30 s (game given on the command line?)";
            return (2, fail(report, "error", reason.into()));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    report["startup"] = json!({ "first_frame_ms_since_main": first_ms });
    log::info!("bench: first frame {first_ms:.0} ms after main");
    let first_at = Instant::now();

    let Some(shared) = state.games.current().load_full() else {
        return (2, fail(report, "error", "game closed".into()));
    };

    if let Some(after) = bench.config.open_menu_after {
        let wait = Duration::from_secs_f64(after).saturating_sub(first_at.elapsed());
        std::thread::sleep(wait);
        log::info!("bench: opening the quick menu");
        let _ = app.emit(crate::events::MENU, ());
    }
    let warmup = Duration::from_secs_f64(bench.config.warmup_seconds);
    std::thread::sleep(warmup.saturating_sub(first_at.elapsed()));

    // Start measuring.
    let video_metrics = state.renderer.as_ref().map(|r| r.metrics().clone());
    if let Some(m) = &video_metrics {
        m.lock().presents = Some(Vec::new());
    }
    {
        let mut log = bench.webview.lock();
        log.recording = true;
        log.frames.clear();
    }
    let audio_metrics = state.audio.metrics().clone();
    audio_metrics.lock().reset();
    let surface_errors0 = video_metrics
        .as_ref()
        .map(|m| m.lock().surface_errors.clone());
    let frames0 = shared.stats.lock().frames;
    let dropped0 = shared.audio.dropped();
    let cpu0 = emudome_platform::process_cpu_time();
    let t0 = Instant::now();

    std::thread::sleep(Duration::from_secs_f64(bench.config.seconds));

    let elapsed = t0.elapsed().as_secs_f64();
    let cpu1 = emudome_platform::process_cpu_time();
    let frames1 = shared.stats.lock().frames;
    let dropped1 = shared.audio.dropped();
    let presents = video_metrics
        .as_ref()
        .and_then(|m| m.lock().presents.take())
        .unwrap_or_default();
    let webview_frames = std::mem::take(&mut bench.webview.lock().frames);
    let audio = std::mem::take(&mut *audio_metrics.lock());
    let target_fps = shared.stats.lock().av_info.timing.fps;

    report["status"] = json!("ok");
    report["seconds"] = json!(elapsed);
    report["core"] = json!({
        "frames": frames1 - frames0,
        "fps": (frames1 - frames0) as f64 / elapsed,
        "target_fps": target_fps,
    });
    let rounds: Vec<(f64, u64)> = match mode {
        VideoMode::Native => presents
            .iter()
            .map(|p| (p.at.duration_since(t0).as_secs_f64() * 1000.0, p.seq))
            .collect(),
        // seq -1: no game frame on the canvas yet (skipped, like the native u64::MAX).
        VideoMode::Webview => webview_frames
            .iter()
            .map(|f| (f.t, if f.seq < 0.0 { u64::MAX } else { f.seq as u64 }))
            .collect(),
    };
    report["video"] = presentation(&rounds, elapsed, target_fps);
    if let (Some(m), Some(before)) = (&video_metrics, surface_errors0) {
        let now = m.lock().surface_errors.clone();
        let diff: serde_json::Map<String, Value> = now
            .iter()
            .map(|(k, v)| {
                (
                    (*k).to_owned(),
                    json!(v - before.get(k).copied().unwrap_or(0)),
                )
            })
            .collect();
        report["surface_errors"] = Value::Object(diff);
    }
    if let (Some(c0), Some(c1)) = (cpu0, cpu1) {
        let pct = (c1 - c0).as_secs_f64() / elapsed * 100.0;
        report["cpu"] = json!({
            "process_pct_of_one_core": pct,
            "process_pct_of_machine": pct / report["logical_cpus"].as_f64().unwrap_or(1.0).max(1.0),
        });
    }
    report["audio"] = audio_report(audio, dropped1.saturating_sub(dropped0));
    (0, report)
}

/// Percentiles of a list of intervals in ms.
fn intervals(times: &[f64]) -> Value {
    let mut d: Vec<f64> = times.windows(2).map(|w| w[1] - w[0]).collect();
    if d.is_empty() {
        return Value::Null;
    }
    d.sort_by(f64::total_cmp);
    let pct = |p: f64| d[((d.len() - 1) as f64 * p).round() as usize];
    json!({
        "count": d.len(),
        "mean": d.iter().sum::<f64>() / d.len() as f64,
        "p50": pct(0.50),
        "p95": pct(0.95),
        "p99": pct(0.99),
        "max": d[d.len() - 1],
    })
}

/// Presentation statistics from `(time ms, game frame shown)` per present / canvas round.
fn presentation(rounds: &[(f64, u64)], seconds: f64, target_fps: f64) -> Value {
    let all: Vec<f64> = rounds.iter().map(|r| r.0).collect();
    let mut new_frames = Vec::new();
    let mut dropped = 0u64;
    let mut duplicated = 0u64;
    let mut prev: Option<u64> = None;
    for &(t, seq) in rounds {
        if seq == u64::MAX {
            continue;
        }
        match prev {
            Some(p) if seq == p => duplicated += 1,
            Some(p) => {
                dropped += seq.saturating_sub(p).saturating_sub(1);
                new_frames.push(t);
            }
            None => new_frames.push(t),
        }
        prev = Some(seq);
    }
    let period = 1000.0 / target_fps.max(1.0);
    let uneven = new_frames
        .windows(2)
        .filter(|w| ((w[1] - w[0]) - period).abs() > period * 0.25)
        .count();
    json!({
        "presents": all.len(),
        "present_fps": all.len() as f64 / seconds,
        "present_interval_ms": intervals(&all),
        "new_frames": new_frames.len(),
        "new_frame_fps": new_frames.len() as f64 / seconds,
        "expected_new_frames": seconds * target_fps,
        "new_frame_interval_ms": intervals(&new_frames),
        "new_frame_intervals_off_by_25pct": uneven,
        "dropped_frames": dropped,
        "duplicated_presents": duplicated,
    })
}

fn audio_report(audio: crate::audio::AudioMetrics, queue_dropped: u64) -> Value {
    let s = &audio.stats;
    let n = s.callbacks.max(1) as f64;
    json!({
        "device": audio.device,
        "error": audio.error,
        "callbacks": s.callbacks,
        "frames_per_callback": [s.frames_per_callback_min, s.frames_per_callback_max],
        "underrun_callbacks": s.underrun_callbacks,
        "underrun_frames": s.underrun_frames,
        "queue_overflow_samples": queue_dropped,
        "fill_ms": { "min": s.fill_ms_min, "avg": s.fill_ms_sum / n, "max": s.fill_ms_max },
        "device_latency_ms": { "avg": s.device_latency_ms_sum / n, "max": s.device_latency_ms_max },
        "callback_interval_ms_max": s.interval_ms_max,
        "ratio": { "min": s.ratio_min, "avg": s.ratio_sum / n, "max": s.ratio_max },
        "tone_discontinuities": s.discontinuities,
        "windows_10s": s.windows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_counts_drops_and_duplicates() {
        // 4 presents per game frame on a 240 Hz display, one game frame skipped (seq 3).
        let rounds: Vec<(f64, u64)> = [1u64, 1, 1, 1, 2, 2, 2, 2, 4, 4]
            .iter()
            .enumerate()
            .map(|(i, &s)| (i as f64 * 4.1667, s))
            .collect();
        let v = presentation(&rounds, 1.0, 60.0);
        assert_eq!(v["presents"], 10);
        assert_eq!(v["new_frames"], 3);
        assert_eq!(v["dropped_frames"], 1);
        assert_eq!(v["duplicated_presents"], 7);
    }

    #[test]
    fn intervals_percentiles() {
        let t: Vec<f64> = (0..101).map(|i| f64::from(i) * 10.0).collect();
        let v = intervals(&t);
        assert_eq!(v["count"], 100);
        assert_eq!(v["p50"], 10.0);
        assert_eq!(v["max"], 10.0);
    }
}
