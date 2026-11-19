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
//! - `--rewind N`: at the end, record N seconds of rewind and report its memory, rewind for a
//!   second, and take a screenshot (written under the temp folder's `saves/screenshots`);
//! - `--dir <folder>`: the system and save folders go under it (default: the temp folder's
//!   `emudome-headless`);
//! - `--screens <prefix>`: write each screen of the frame as its own PNG (`<prefix>-top.png`,
//!   `<prefix>-bottom.png`; `<prefix>-main.png` on one-screen consoles);
//! - `--touch`: NDS, touch the bottom screen through the shell's path (window point → composed
//!   rectangle → pointer value) and check what the test ROM shows (`tools/test-rom-nds`).
//!
//! A `.nds` game runs as the shell runs it: melonDS DS's forced options and the two-screen split
//! (`emudome_core::nds`).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use emudome_core::input::{Button, Source};
use emudome_core::screens::{self, Area, Layout, ScreenId, Split};
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
    })
}

fn write_png(session: &Session, path: &std::path::Path) -> anyhow::Result<()> {
    let f = session.shared().frame.lock();
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, f.width, f.height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&f.rgba)?;
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
        let file = std::io::BufWriter::new(std::fs::File::create(&path)?);
        let mut enc = png::Encoder::new(file, s.src.w, s.src.h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()?.write_image_data(&rgba)?;
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
    rom: &str,
    secs: u32,
) -> anyhow::Result<()> {
    use emudome_core::rewind;

    // The cost of one difference, between two states a frame apart and a second apart.
    let files = emudome_core::GameFiles::new(&tmp.join("saves"), std::path::Path::new(rom));
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

fn main() -> anyhow::Result<()> {
    let args = parse_args()?;
    let tmp = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("emudome-headless"));
    // A DS game runs as the shell runs it: melonDS DS's forced options and the split.
    let nds = args.rom.to_lowercase().ends_with(".nds");
    let mut core_options: Vec<(String, String)> = if nds {
        emudome_core::nds::FORCED_OPTIONS
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    } else {
        Vec::new()
    };
    core_options.extend(args.opts.iter().cloned());
    let (tx, rx) = mpsc::channel();
    let session = Session::start(
        SessionOptions {
            core_path: args.core.clone().into(),
            game_path: args.rom.clone().into(),
            system_dir: tmp.join("system"),
            save_dir: tmp.join("saves"),
            autosave_every: None,
            autosave_on_exit: false,
            resume_auto: false,
            app_version: env!("CARGO_PKG_VERSION").into(),
            core_options,
            rewind_seconds: args.rewind,
            split: if nds {
                emudome_core::nds::SPLIT
            } else {
                Split::Whole
            },
            username: None,
        },
        tx,
    )?;
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(5))?);

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
        measure_rewind(&session, &rx, &tmp, &args.rom, secs)?;
    }

    drop(session);
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);
    Ok(())
}
