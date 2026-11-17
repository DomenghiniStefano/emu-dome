//! Windowless test: loads core and game, reads frames, presses A, saves and reloads a state.
//!
//! cargo run -p emudome-core --example headless -- <core> <rom>

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

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let tmp = std::env::temp_dir().join("emudome-headless");
    let (tx, rx) = mpsc::channel();
    let session = Session::start(
        SessionOptions {
            core_path: args[1].clone().into(),
            game_path: args[2].clone().into(),
            system_dir: tmp.join("system"),
            save_dir: tmp.join("saves"),
            autosave_every: None,
            autosave_on_exit: false,
            resume_auto: false,
        },
        tx,
    )?;
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(5))?);

    std::thread::sleep(Duration::from_millis(1000));
    let stats = session.shared().stats.lock().clone();
    println!(
        "1x: emulated fps {:.1}, center pixel {:?}",
        stats.emu_fps,
        center_pixel(&session)
    );

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
    let seq0 = center_pixel(&session).3;
    std::thread::sleep(Duration::from_millis(1000));
    let seq1 = center_pixel(&session).3;
    println!(
        "Max: {:.0} frames/s",
        (seq1 - seq0) as f64 / t.elapsed().as_secs_f64()
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

    drop(session);
    println!("event: {:?}", rx.recv_timeout(Duration::from_secs(2))?);
    Ok(())
}
