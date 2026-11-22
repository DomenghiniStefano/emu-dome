//! Console controller test, no windows: opens the app's own controller source
//! (`emudome_platform::pads`: SDL3's gamepad subsystem with the app's hints on the desktop) and
//! prints connections, buttons by position and axes past half travel for 20 seconds.
//!
//! `cargo run -p emudome --example gamepad_probe`

use std::collections::HashMap;
use std::time::{Duration, Instant};

use emudome_platform::pads::{PadButton, PadEvent, PadReading, PadSource};

const DURATION: Duration = Duration::from_secs(20);

const AXES: [&str; 6] = [
    "leftx",
    "lefty",
    "rightx",
    "righty",
    "lefttrigger",
    "righttrigger",
];

fn main() {
    let mut source = match PadSource::open() {
        Ok(source) => source,
        Err(e) => {
            eprintln!("{} failed: {e}", PadSource::BACKEND);
            std::process::exit(1);
        }
    };
    println!(
        "Connect a controller and press its buttons ({}): listening for {} seconds.",
        PadSource::BACKEND,
        DURATION.as_secs()
    );

    let mut pads: HashMap<u32, PadReading> = HashMap::new();
    let mut events = Vec::new();
    let start = Instant::now();
    while start.elapsed() < DURATION {
        source.poll(&mut events);
        for event in events.drain(..) {
            match event {
                PadEvent::Added { device, at_start } => {
                    println!(
                        "[{}] connected{}: {} | type {} | kind {:?} | vid {:04x} pid {:04x} | battery {:?}",
                        device.id,
                        if at_start { " (at start)" } else { "" },
                        device.name.as_deref().unwrap_or("(unnamed)"),
                        device.type_name,
                        device.kind,
                        device.vendor,
                        device.product,
                        device.battery,
                    );
                    pads.insert(device.id, PadReading::default());
                }
                PadEvent::Removed(id) => {
                    println!("[{id}] disconnected");
                    pads.remove(&id);
                }
                PadEvent::Battery { id, battery } => println!("[{id}] battery {battery:?}"),
            }
        }
        for (id, last) in &mut pads {
            let Some(now) = source.read(*id) else {
                continue;
            };
            for button in PadButton::ALL {
                let (was, is) = (
                    last.buttons & button.bit() != 0,
                    now.buttons & button.bit() != 0,
                );
                if was != is {
                    let state = if is { "pressed" } else { "released" };
                    println!("[{id}] {} {state}", button.name());
                }
            }
            // Only when an axis crosses half travel either way, so the console does not fill up.
            for (i, name) in AXES.iter().enumerate() {
                let past_half = |v: i16| v.unsigned_abs() > 16000;
                if past_half(last.axes[i]) != past_half(now.axes[i]) {
                    println!("[{id}] axis {name} = {}", now.axes[i]);
                }
            }
            *last = now;
        }
        std::thread::sleep(Duration::from_millis(4));
    }
    println!("Done.");
}
