//! Windowless test: loads core and game, reads frames, presses A, saves and reloads a state.
//!
//! cargo run -p emudome-core --example headless -- <core> <rom> [options]
//!
//! Options:
//! - `--opt key=value` (repeatable): a core option answered to `GET_VARIABLE`,
//!   e.g. `--opt citra_graphics_api=Software` for Azahar;
//! - `--secs N`: measure the 1x and Max speeds over N seconds each (default 1);
//! - `--png <file>`: write the last frame before the state tests as a PNG;
//! - `--press-a N`: before measuring, tap A N times, one tap per second (to leave a title screen);
//! - `--live key=value` (repeatable): after the 1x measure, change a core option while the
//!   game runs and print the center pixel before and after (does the core take it live?);
//! - `--fast-audio mute|pitch_up|same_pitch`: `audio.ff`; then also count the audio frames
//!   queued per second of wall time at 1x, 4x and 0.5x (about the core's rate when converted);
//! - `--list-options`: print the options the core declared (key, default, values, label) and
//!   stop, without the speed and state tests;
//! - `--rewind N`: at the end, record N seconds of rewind and report its memory, rewind for a
//!   second, and take a screenshot (written under the temp folder's `saves/screenshots`);
//! - `--dir <folder>`: the system and save folders go under it (default: the temp folder's
//!   `emudome-headless`);
//! - `--screens <prefix>`: write each screen of the frame as its own PNG (`<prefix>-top.png`,
//!   `<prefix>-bottom.png`; `<prefix>-main.png` on one-screen consoles);
//! - `--touch`: NDS, touch the bottom screen through the shell's path (window point → composed
//!   rectangle → pointer value) and check what the test ROM shows (`tools/test-rom-nds`);
//! - `--set key=value` (repeatable): a text setting the BIOS rule reads (`consoles.dsmode`,
//!   `cores.mdsboot`), as the shell would resolve it for the game;
//! - `--tap X,Y` (repeatable): after `--press-a`, touch pixel (X, Y) of the bottom screen for
//!   200 ms, then wait a second (through the shell's pointer value for that screen);
//! - `--timing N`: run N seconds at 1x, a quick save into slot 5 every 2 s, and print the
//!   emulation thread's work per frame (p50 / p99 / max, frames over budget), then stop. With
//!   `--rewind S` rewind records meanwhile, as in the app. Build with `--release` to measure;
//! - `--max-path DIR,FILE`: the longest folder and file path the system allows (default
//!   `247,259`, Windows without long paths); a game whose core's files would not fit under its
//!   save folder is refused before the core loads, as in the shell (Trello #94).
//!
//! A game runs as the shell runs it: its core's forced options and screen split
//! (`emudome_core::core_profile`). Every game gets the BIOS rule of its core (`emudome_core::bios`): the
//! system folder is `<dir>/system/<core>`, as in the shell, and the files in place, the options
//! and the fallbacks are printed. `--opt` values come last and win.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use emudome_core::bios;
use emudome_core::core_profile::{CoreId, Renderer};
use emudome_core::input::{Button, Source};
use emudome_core::save_path::{PathLimits, PathUnit};
use emudome_core::screens::{self, Area, Layout, ScreenId};
use emudome_core::settings::{ConsoleId, Value};
use emudome_core::{Command, Session, SessionOptions, Speed};

fn center_pixel(session: &Session) -> (u32, u32, [u8; 4], u64) {
    let f = session.shared().frame.lock();
    if f.rgba.is_empty() {
        return (0, 0, [0; 4], f.seq);
    }
    let i = (((f.height / 2) * f.width + f.width / 2) * 4) as usize;
    (
        f.width,
        f.height,
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]],
        f.seq,
    )
}

struct Args {
    core: String,
    rom: String,
    opts: Vec<(String, String)>,
    secs: f64,
    png: Option<std::path::PathBuf>,
    press_a: u32,
    rewind: Option<u32>,
    dir: Option<std::path::PathBuf>,
    screens: Option<String>,
    touch: bool,
    list_options: bool,
    live: Vec<(String, String)>,
    fast_audio: Option<emudome_core::speed_audio::FastAudio>,
    sets: Vec<(String, String)>,
    timing: Option<u32>,
    taps: Vec<(u32, u32)>,
    max_path: PathLimits,
}

/// `--max-path` by default: Windows without long paths (folder 247, file 259 characters). This
/// example carries no long-path manifest, so on Windows these are its limits whatever the
/// system policy says; elsewhere pass the system's (`--max-path 4095,4095` on Linux).
const LEGACY_MAX_PATH: PathLimits = PathLimits {
    dir: 247,
    file: 259,
    unit: PathUnit::Utf16,
};

/// `--tap X,Y`: a touch on pixel (X, Y) of the bottom screen, as the shell would send it.
fn tap(session: &Session, (x, y): (u32, u32)) -> anyhow::Result<()> {
    let (frame, bottom) = {
        let f = session.shared().frame.lock();
        let bottom = f
            .screens()
            .into_iter()
            .find(|s| s.id == ScreenId::Bottom)
            .ok_or_else(|| anyhow::anyhow!("--tap: the frame has no bottom screen"))?;
        ((f.width, f.height), bottom.src)
    };
    let pointer = screens::pointer_value(bottom, frame, x, y);
    session.shared().input.set_pointer(Some(pointer));
    std::thread::sleep(Duration::from_millis(200));
    session.shared().input.set_pointer(None);
    std::thread::sleep(Duration::from_secs(1));
    println!("tapped bottom screen ({x}, {y}): pointer {pointer:?}");
    Ok(())
}

/// `--timing N`: the emulation thread's work per frame over N seconds at 1x, with a quick save
/// every 2 s (rewind records too when it is on).
fn measure_timing(
    session: &Session,
    rx: &mpsc::Receiver<emudome_core::Event>,
    secs: u32,
) -> anyhow::Result<()> {
    *session.shared().frame_times.lock() = Some(Vec::new());
    let t = Instant::now();
    let mut saves = 0;
    while t.elapsed() < Duration::from_secs(u64::from(secs)) {
        std::thread::sleep(Duration::from_secs(2));
        session.send(Command::SaveState(5));
        saves += 1;
    }
    // The last save's event, then the frames recorded.
    for _ in 0..saves {
        rx.recv_timeout(Duration::from_secs(5))?;
    }
    let mut times = session
        .shared()
        .frame_times
        .lock()
        .take()
        .unwrap_or_default();
    anyhow::ensure!(!times.is_empty(), "no frame recorded");
    times.sort_unstable();
    let at = |q: f64| times[((times.len() - 1) as f64 * q).round() as usize];
    let mean = times.iter().map(|&t| f64::from(t)).sum::<f64>() / times.len() as f64;
    let (states, bytes) = {
        let s = session.shared().stats.lock();
        (s.rewind_states, s.rewind_bytes)
    };
    println!(
        "timing: {} frames, {saves} quick saves; work per frame µs: mean {mean:.0}, p50 {}, p90 {}, p99 {}, p99.9 {}, max {}; over 8 ms {}, over 16.7 ms {}; rewind {states} states, {:.1} MB",
        times.len(),
        at(0.5),
        at(0.9),
        at(0.99),
        at(0.999),
        times[times.len() - 1],
        times.iter().filter(|&&t| t > 8_000).count(),
        times.iter().filter(|&&t| t > 16_700).count(),
        bytes as f64 / 1e6,
    );
    Ok(())
}

fn key_value(kv: &str) -> anyhow::Result<(String, String)> {
    let (k, v) = kv
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("wanted key=value, got {kv}"))?;
    Ok((k.to_owned(), v.to_owned()))
}

/// Audio frames queued per second of wall time over `secs` (the queue drained every 5 ms).
fn audio_rate(session: &Session, secs: f64) -> f64 {
    let ring = &session.shared().audio;
    let drain = || ring.reader().map_or(0, |mut r| r.drain());
    drain();
    let t = Instant::now();
    let mut samples = 0usize;
    while t.elapsed().as_secs_f64() < secs {
        std::thread::sleep(Duration::from_millis(5));
        samples += drain();
    }
    samples as f64 / 2.0 / t.elapsed().as_secs_f64()
}

fn parse_args() -> anyhow::Result<Args> {
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut opts = Vec::new();
    let mut secs = 1.0;
    let mut png = None;
    let mut press_a = 0;
    let mut rewind = None;
    let mut dir = None;
    let mut screens = None;
    let mut touch = false;
    let mut list_options = false;
    let mut live = Vec::new();
    let mut fast_audio = None;
    let mut sets = Vec::new();
    let mut timing = None;
    let mut taps = Vec::new();
    let mut max_path = LEGACY_MAX_PATH;
    while let Some(a) = it.next() {
        let mut value = || {
            it.next()
                .ok_or_else(|| anyhow::anyhow!("{a} needs a value"))
        };
        match a.as_str() {
            "--opt" => {
                let kv = value()?;
                let (k, v) = kv
                    .split_once('=')
                    .ok_or_else(|| anyhow::anyhow!("--opt wants key=value, got {kv}"))?;
                opts.push((k.to_owned(), v.to_owned()));
            }
            "--secs" => secs = value()?.parse()?,
            "--png" => png = Some(value()?.into()),
            "--press-a" => press_a = value()?.parse()?,
            "--rewind" => rewind = Some(value()?.parse()?),
            "--dir" => dir = Some(value()?.into()),
            "--screens" => screens = Some(value()?),
            "--touch" => touch = true,
            "--list-options" => list_options = true,
            "--live" => live.push(key_value(&value()?)?),
            "--set" => sets.push(key_value(&value()?)?),
            "--timing" => timing = Some(value()?.parse()?),
            "--tap" => {
                let xy = value()?;
                let (x, y) = xy
                    .split_once(',')
                    .ok_or_else(|| anyhow::anyhow!("--tap wants X,Y, got {xy}"))?;
                taps.push((x.trim().parse()?, y.trim().parse()?));
            }
            "--max-path" => {
                let v = value()?;
                let (d, f) = v
                    .split_once(',')
                    .ok_or_else(|| anyhow::anyhow!("--max-path wants DIR,FILE, got {v}"))?;
                max_path = PathLimits {
                    dir: d.trim().parse()?,
                    file: f.trim().parse()?,
                    unit: max_path.unit,
                };
            }
            "--fast-audio" => {
                let id = value()?;
                fast_audio = Some(
                    emudome_core::speed_audio::FastAudio::from_option(&id)
                        .ok_or_else(|| anyhow::anyhow!("unknown --fast-audio {id}"))?,
                );
            }
            _ => positional.push(a),
        }
    }
    let [core, rom]: [String; 2] = positional
        .try_into()
        .map_err(|_| anyhow::anyhow!("usage: headless <core> <rom> [--opt key=value]..."))?;
    Ok(Args {
        core,
        rom,
        opts,
        secs,
        png,
        press_a,
        rewind,
        dir,
        screens,
        touch,
        list_options,
        live,
        fast_audio,
        sets,
        timing,
        taps,
        max_path,
    })
}

fn write_png(session: &Session, path: &std::path::Path) -> anyhow::Result<()> {
    let f = session.shared().frame.lock().clone();
    std::fs::write(path, f.to_png()?)?;
    Ok(())
}

/// `--screens <prefix>`: each screen of the frame as its own PNG.
fn write_screens(session: &Session, prefix: &str) -> anyhow::Result<()> {
    let f = session.shared().frame.lock();
    for s in f.screens() {
        let name = match s.id {
            ScreenId::Main => "main",
            ScreenId::Top => "top",
            ScreenId::Bottom => "bottom",
        };
        let mut rgba = Vec::with_capacity((s.src.w * s.src.h * 4) as usize);
        for y in s.src.y..s.src.y + s.src.h {
            let row = ((y * f.width + s.src.x) * 4) as usize;
            rgba.extend_from_slice(&f.rgba[row..row + (s.src.w * 4) as usize]);
        }
        let path = format!("{prefix}-{name}.png");
        std::fs::write(&path, emudome_core::rgba_png(s.src.w, s.src.h, &rgba)?)?;
        println!("{name} screen {:?} written to {path}", s.src);
    }
    Ok(())
}

/// The colour of pixel (x, y) of a screen, in 5-bit components as the DS wrote them.
fn screen_rgb5(session: &Session, id: ScreenId, x: u32, y: u32) -> Option<[u8; 3]> {
    let f = session.shared().frame.lock();
    let s = f.screens().into_iter().find(|s| s.id == id)?;
    let i = (((s.src.y + y) * f.width + s.src.x + x) * 4) as usize;
    let px = f.rgba.get(i..i + 3)?;
    Some([px[0] >> 3, px[1] >> 3, px[2] >> 3])
}

/// `--touch`: window points on a 1920 x 1080 window with the default layout go through
/// `screens::pointer_at` to the core; the test ROM paints its bottom screen with the point it
/// read (red = x / 8, green = y / 8, blue = 16), so the colour proves the whole path.
fn check_touch(session: &Session) -> anyhow::Result<()> {
    let area = Area {
        width: 1920.0,
        height: 1080.0,
        inset: 0.0,
    };
    let layout = Layout::default();
    let (frame, split) = {
        let f = session.shared().frame.lock();
        ((f.width, f.height), f.split)
    };
    let placements = screens::compose(&layout, &split.screens(frame.0, frame.1), area);
    for p in &placements {
        println!(
            "composed {:?}: frame {:?} -> window {:?}",
            p.id, p.src, p.dst
        );
    }
    let mut failures = 0;
    for (wx, wy) in [
        (704.0, 540.0),
        (960.0, 732.0),
        (1215.0, 923.0),
        (800.0, 900.0),
        (1100.0, 560.0),
        (960.0, 300.0), // the top screen: not a touch
    ] {
        let want = screens::touch_at(&placements, wx, wy);
        let pointer = screens::pointer_at(&layout, split, frame, area, wx, wy);
        session.shared().input.set_pointer(pointer);
        std::thread::sleep(Duration::from_millis(150));
        let got = screen_rgb5(session, ScreenId::Bottom, 5, 5);
        let ok = match (want, got) {
            (Some((x, y)), Some(c)) => c == [(x / 8) as u8, (y / 8) as u8, 16],
            (None, Some(c)) => c == [0, 0, 31],
            (Some(_) | None, None) => false,
        };
        failures += u32::from(!ok);
        println!(
            "touch at window ({wx}, {wy}): touch-screen pixel {want:?}, pointer {pointer:?}, ROM shows rgb5 {got:?} -> {}",
            if ok { "ok" } else { "MISMATCH" }
        );
    }
    session.shared().input.set_pointer(None);
    std::thread::sleep(Duration::from_millis(150));
    println!(
        "released: bottom screen rgb5 {:?} (want [0, 0, 31])",
        screen_rgb5(session, ScreenId::Bottom, 5, 5)
    );
    anyhow::ensure!(failures == 0, "{failures} touch points did not match");
    Ok(())
}

/// `--rewind N`: memory of N seconds of rewind, a second of rewinding, a screenshot.
fn measure_rewind(
    session: &Session,
    rx: &mpsc::Receiver<emudome_core::Event>,
    tmp: &std::path::Path,
    game: &emudome_core::GameKey,
    secs: u32,
) -> anyhow::Result<()> {
    use emudome_core::rewind;

    // The cost of one difference, between two states a frame apart and a second apart.
    let files = emudome_core::GameFiles::new(&tmp.join("saves"), game);
    session.send(Command::SetPaused(false));
    let mut slots = Vec::new();
    for (slot, wait) in [(7u8, 0u64), (8, 17), (9, 1000)] {
        std::thread::sleep(Duration::from_millis(wait));
        session.send(Command::SaveState(slot));
        rx.recv_timeout(Duration::from_secs(2))?;
        slots.push(std::fs::read(files.slot(slot))?);
    }
    let state = slots[0].len();
    let t = Instant::now();
    let frame_delta = rewind::encode_delta(&slots[0], &slots[1]).len();
    let encode = t.elapsed();
    let second_delta = rewind::encode_delta(&slots[0], &slots[2]).len();
    println!(
        "rewind: state {state} bytes; difference ~1 frame apart {frame_delta} bytes (encoded in {:.2} ms), 1 s apart {second_delta} bytes",
        encode.as_secs_f64() * 1000.0
    );

    let fps = session.shared().stats.lock().av_info.timing.fps;
    let interval = rewind::interval_for(state);
    for s in [10, 30, 60, 120] {
        let cap = rewind::capacity(s, fps, interval);
        println!(
            "rewind {s:>3} s: {cap} states; whole states {:.0} MB, measured-size differences {:.1} MB",
            rewind::estimate_bytes(cap, state, state) as f64 / 1e6,
            rewind::estimate_bytes(cap, state, frame_delta.max(second_delta)) as f64 / 1e6,
        );
    }

    std::thread::sleep(Duration::from_secs(u64::from(secs) + 1));
    let (states, bytes) = {
        let s = session.shared().stats.lock();
        (s.rewind_states, s.rewind_bytes)
    };
    println!(
        "rewind after {} s of play: {states} states, {bytes} bytes ({:.1} MB)",
        secs + 1,
        bytes as f64 / 1e6
    );

    let f0 = session.shared().stats.lock().frames;
    session.send(Command::Rewind(true));
    std::thread::sleep(Duration::from_secs(1));
    session.send(Command::Rewind(false));
    let s = session.shared().stats.lock().clone();
    println!(
        "rewound 1 s: {} ticks, {} states left ({} used)",
        s.frames - f0,
        s.rewind_states,
        states - s.rewind_states
    );

    session.send(Command::Screenshot);
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);
    session.send(Command::SetPaused(true));
    Ok(())
}

/// The host's and the core's log lines on stderr, at `RUST_LOG`'s level (`error`, `warn`,
/// `info`, `debug`, `trace`); nothing without it.
struct StderrLog;

impl log::Log for StderrLog {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("[{} {}] {}", record.level(), record.target(), record.args());
        }
    }

    fn flush(&self) {}
}

fn init_log() {
    let level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|l| l.parse::<log::LevelFilter>().ok());
    if let Some(level) = level {
        static LOGGER: StderrLog = StderrLog;
        if log::set_logger(&LOGGER).is_ok() {
            log::set_max_level(level);
        }
    }
}

fn main() -> anyhow::Result<()> {
    run(&|| None)
}

/// A host's GPU for the session, when it has one: the video setup of a Vulkan device (the app's
/// `headless_gpu` example passes the shell's). `None`: software frames only.
pub type GpuSetup<'a> = &'a dyn Fn() -> Option<emudome_core::VideoSetup>;

/// The whole test; `gpu` gives the session a Vulkan device when the core can use one.
pub fn run(gpu: GpuSetup<'_>) -> anyhow::Result<()> {
    init_log();
    let args = parse_args()?;
    let tmp = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("emudome-headless"));
    // A game runs as the shell runs it: its core's forced options and split, by its console.
    let console = std::path::Path::new(&args.rom)
        .extension()
        .and_then(|e| emudome_core::library::console_of_extension(&e.to_string_lossy()))
        .unwrap_or(ConsoleId::Gba);
    let core = CoreId::for_console(console);
    let profile = core.profile();
    // Vulkan when the host has a device and the core can draw with it, as in the shell.
    let wants_gpu = profile.hw_render.renderer(true) == Renderer::Vulkan;
    let video = if wants_gpu { gpu() } else { None };
    let renderer = profile.hw_render.renderer(video.is_some());
    println!("renderer: {renderer:?}");
    let mut core_options = profile.forced_pairs(renderer);
    let system_dir = bios::files::core_dir(&tmp.join("system"), core.name());
    let in_place = bios::table(core)
        .map(|t| bios::files::in_place(&system_dir, t))
        .unwrap_or_default();
    let setting = |key: &str| -> Option<Value> {
        args.sets
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| Value::Text(v.clone()))
    };
    let plan = bios::plan(core, console, &in_place, &setting);
    println!(
        "bios: in place {in_place:?}, options {:?}, fallbacks {:?}",
        plan.options, plan.fallbacks
    );
    core_options.extend(
        plan.options
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
    );
    core_options.extend(args.opts.iter().cloned());
    // The game's key names its saves, as in the shell.
    let game = emudome_core::library::identify(std::path::Path::new(&args.rom))
        .map_err(|e| anyhow::anyhow!("{}: {e}", args.rom))?
        .id;
    let saves_root = tmp.join("saves");
    // As the shell: a save folder too deep for the core's own files is refused before the core
    // loads (Azahar would abort the process, Trello #94).
    if let Err(e) = profile.check_save_dir(
        emudome_core::GameFiles::new(&saves_root, &game).dir(),
        args.max_path,
    ) {
        println!(
            "refused: {:?} ({e})",
            emudome_core::ErrorCode::from(e.clone())
        );
        return Ok(());
    }
    let (tx, rx) = mpsc::channel();
    let session = Session::start(
        SessionOptions {
            core_path: args.core.clone().into(),
            game_path: args.rom.clone().into(),
            system_dir,
            save_dir: saves_root,
            game: game.clone(),
            autosave_every: None,
            autosave_on_exit: false,
            resume_auto: false,
            app_version: env!("CARGO_PKG_VERSION").into(),
            core_options,
            rewind_seconds: args.rewind,
            // As the shell: loads that would crash the core are refused (Trello #93).
            state_load_block: emudome_core::core_profile::state_load_block(core, renderer, &game),
            split: profile.split,
            username: None,
            fast_audio: args.fast_audio.unwrap_or_default(),
            video: video.unwrap_or_default(),
        },
        tx,
    )?;
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(5))?);

    if args.list_options {
        for o in session.shared().core_options.lock().iter() {
            println!(
                "{} = {:?} of {:?} ({})",
                o.key, o.default, o.values, o.label
            );
        }
        return Ok(());
    }

    for _ in 0..args.press_a {
        std::thread::sleep(Duration::from_millis(850));
        session
            .shared()
            .input
            .set(Source::Keyboard, Button::A, true);
        std::thread::sleep(Duration::from_millis(150));
        session
            .shared()
            .input
            .set(Source::Keyboard, Button::A, false);
    }

    for xy in &args.taps {
        tap(&session, *xy)?;
    }

    if let Some(secs) = args.timing {
        measure_timing(&session, &rx, secs)?;
        // The picture at the end of the measure.
        if let Some(png) = &args.png {
            write_png(&session, png)?;
        }
        if let Some(prefix) = &args.screens {
            write_screens(&session, prefix)?;
        }
        drop(session);
        return Ok(());
    }

    // 1x: average over the whole window, from the frame counter.
    let measure = Duration::from_secs_f64(args.secs);
    let f0 = session.shared().stats.lock().frames;
    let t = Instant::now();
    std::thread::sleep(measure);
    let f1 = session.shared().stats.lock().frames;
    let stats = session.shared().stats.lock().clone();
    println!(
        "1x: {:.1} frames/s over {:.1} s (last second {:.1}, core target {:.2}), center pixel {:?}",
        (f1 - f0) as f64 / t.elapsed().as_secs_f64(),
        t.elapsed().as_secs_f64(),
        stats.emu_fps,
        stats.av_info.timing.fps,
        center_pixel(&session)
    );
    for (key, value) in &args.live {
        let before = center_pixel(&session);
        session.send(Command::SetCoreOption {
            key: key.clone(),
            value: value.clone(),
        });
        std::thread::sleep(Duration::from_millis(200));
        // Frames drawn against frames run over the next half second (frameskip taken live?).
        let (seq0, run0) = (
            center_pixel(&session).3,
            session.shared().stats.lock().frames,
        );
        std::thread::sleep(Duration::from_millis(500));
        let (seq1, run1) = (
            center_pixel(&session).3,
            session.shared().stats.lock().frames,
        );
        println!(
            "live {key}={value}: center pixel {:?} -> {:?}, then {} frames drawn of {} run",
            before.2,
            center_pixel(&session).2,
            seq1 - seq0,
            run1 - run0
        );
    }
    if args.fast_audio.is_some() {
        for (label, speed) in [
            ("1x", Speed::Factor(1.0)),
            ("4x", Speed::Factor(4.0)),
            ("0.5x", Speed::Factor(0.5)),
            ("Max", Speed::Max),
        ] {
            session.send(Command::SetSpeed(speed));
            std::thread::sleep(Duration::from_millis(300));
            let rate = session.shared().audio.sample_rate();
            println!(
                "audio at {label}: {:.0} frames/s queued (core rate {rate})",
                audio_rate(&session, 1.0)
            );
        }
        session.send(Command::SetSpeed(Speed::Factor(1.0)));
    }
    if let Some(png) = &args.png {
        write_png(&session, png)?;
        println!("frame written to {}", png.display());
    }
    if let Some(prefix) = &args.screens {
        write_screens(&session, prefix)?;
    }
    if args.touch {
        check_touch(&session)?;
    }

    session
        .shared()
        .input
        .set(Source::Keyboard, Button::A, true);
    std::thread::sleep(Duration::from_millis(200));
    println!(
        "A pressed: center pixel {:?}, top screen rgb5 {:?}",
        center_pixel(&session),
        screen_rgb5(&session, ScreenId::Top, 5, 5)
    );
    session
        .shared()
        .input
        .set(Source::Keyboard, Button::A, false);
    std::thread::sleep(Duration::from_millis(200));
    println!(
        "A released: center pixel {:?}, top screen rgb5 {:?}",
        center_pixel(&session),
        screen_rgb5(&session, ScreenId::Top, 5, 5)
    );

    session.send(Command::SetSpeed(Speed::Max));
    let t = Instant::now();
    let f0 = session.shared().stats.lock().frames;
    std::thread::sleep(measure);
    let f1 = session.shared().stats.lock().frames;
    println!(
        "Max: {:.0} frames/s over {:.1} s",
        (f1 - f0) as f64 / t.elapsed().as_secs_f64(),
        t.elapsed().as_secs_f64()
    );

    session.send(Command::SetSpeed(Speed::Factor(2.0)));
    std::thread::sleep(Duration::from_millis(1200));
    println!(
        "2x: emulated fps {:.1}",
        session.shared().stats.lock().emu_fps
    );
    session.send(Command::SetSpeed(Speed::Factor(1.0)));

    session.send(Command::SaveState(1));
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);
    session.send(Command::LoadState(1));
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);
    session.send(Command::LoadState(9));
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);

    session.send(Command::SetPaused(true));
    let s0 = center_pixel(&session).3;
    std::thread::sleep(Duration::from_millis(300));
    println!("paused: new frames {}", center_pixel(&session).3 - s0);

    if let Some(secs) = args.rewind {
        measure_rewind(&session, &rx, &tmp, &game, secs)?;
    }

    drop(session);
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);
    Ok(())
}
