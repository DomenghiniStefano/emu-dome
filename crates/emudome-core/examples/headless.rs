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
//!   second, and take a screenshot (written under the temp folder's `saves/screenshots`).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use emudome_core::input::{Button, Source};
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
}

fn parse_args() -> anyhow::Result<Args> {
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut opts = Vec::new();
    let mut secs = 1.0;
    let mut png = None;
    let mut press_a = 0;
    let mut rewind = None;
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
    let tmp = std::env::temp_dir().join("emudome-headless");
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
            core_options: args.opts.clone(),
            rewind_seconds: args.rewind,
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

    session
        .shared()
        .input
        .set(Source::Keyboard, Button::A, true);
    std::thread::sleep(Duration::from_millis(200));
    println!("A pressed: center pixel {:?}", center_pixel(&session));
    session
        .shared()
        .input
        .set(Source::Keyboard, Button::A, false);
    std::thread::sleep(Duration::from_millis(200));
    println!("A released: center pixel {:?}", center_pixel(&session));

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
