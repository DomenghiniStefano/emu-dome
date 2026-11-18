//! Feasibility spike for Trello #16: can EmuDome's host give a libretro core a hardware render
//! context (`SET_HW_RENDER`, Vulkan) and present the core's frames through wgpu?
//!
//! Windowless: no surface, no window. The core renders on a Vulkan device shared with wgpu,
//! wgpu samples the core's image (no copy) into its own target, and the target is optionally
//! read back to the CPU to check the pixels and measure the readback.
//!
//! ```text
//! cargo run --release -p hw-render-spike -- <core> <content> [options]
//!   --opt key=value   core option answered to GET_VARIABLE (repeatable),
//!                     e.g. --opt citra_graphics_api=Vulkan for Azahar
//!   --frames N        frames per measured phase (default 300)
//!   --warmup N        frames run before measuring (default 120)
//!   --png FILE        write the last read-back frame as a PNG
//!   --core-log        print the core's log lines (format strings only)
//!   --adapter NAME    use the Vulkan GPU whose name contains NAME (default: high performance)
//! ```
//!
//! Phases, each `--frames` long and unthrottled: (A) `retro_run` only, the core's image left
//! unused; (B) `retro_run` + wgpu draws the core image + waits for the GPU; (C) as B + copy to
//! the CPU. A software-rendering core runs phase A only.

mod host;
mod present;
mod vk_iface;

use std::ffi::CString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

use host::{Core, FrameKind, STATE};
use present::Gpu;

struct Args {
    core: PathBuf,
    content: PathBuf,
    opts: Vec<(String, String)>,
    frames: u32,
    warmup: u32,
    png: Option<PathBuf>,
    core_log: bool,
    adapter: Option<String>,
}

fn parse_args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut opts = Vec::new();
    let mut frames = 300;
    let mut warmup = 120;
    let mut png = None;
    let mut core_log = false;
    let mut adapter = None;
    while let Some(a) = it.next() {
        let mut value = || it.next().ok_or_else(|| anyhow!("{a} needs a value"));
        match a.as_str() {
            "--opt" => {
                let kv = value()?;
                let (k, v) = kv
                    .split_once('=')
                    .ok_or_else(|| anyhow!("--opt wants key=value, got {kv}"))?;
                opts.push((k.to_owned(), v.to_owned()));
            }
            "--frames" => frames = value()?.parse()?,
            "--warmup" => warmup = value()?.parse()?,
            "--png" => png = Some(value()?.into()),
            "--core-log" => core_log = true,
            "--adapter" => adapter = Some(value()?),
            _ => positional.push(a),
        }
    }
    let [core, content]: [String; 2] = positional
        .try_into()
        .map_err(|_| anyhow!("usage: hw-render-spike <core> <content> [--opt key=value]..."))?;
    Ok(Args {
        core: core.into(),
        content: content.into(),
        opts,
        frames,
        warmup,
        png,
        core_log,
        adapter,
    })
}

#[derive(Default)]
struct Timing {
    frames: u32,
    run: Duration,
    present: Duration,
    total: Duration,
    hw_frames: u32,
    distinct: std::collections::HashSet<u64>,
}

impl Timing {
    fn report(&self, name: &str) {
        let n = f64::from(self.frames.max(1));
        println!(
            "{name}: {} frames in {:.2} s = {:.1} fps; retro_run {:.2} ms/frame, present {:.3} ms/frame; hardware frames {}",
            self.frames,
            self.total.as_secs_f64(),
            f64::from(self.frames) / self.total.as_secs_f64(),
            self.run.as_secs_f64() * 1000.0 / n,
            self.present.as_secs_f64() * 1000.0 / n,
            self.hw_frames
        );
        if !self.distinct.is_empty() {
            println!("{name}: {} distinct read-back frames", self.distinct.len());
        }
    }
}

/// The hardware frame of the last `retro_run`, if it gave one with an image.
fn hw_frame() -> Option<(host::CoreImage, u32, u32)> {
    let st = STATE.lock();
    match (st.last_frame, st.image) {
        (FrameKind::Hardware { width, height }, Some(img)) => Some((img, width, height)),
        (FrameKind::Hardware { .. }, None)
        | (FrameKind::Software { .. }, _)
        | (FrameKind::Dupe, _)
        | (FrameKind::None, _) => None,
    }
}

/// A read-back frame: width, height, tightly packed RGBA.
type Pixels = (u32, u32, Vec<u8>);

/// Runs `n` frames; `mode` 0 = run only, 1 = + draw, 2 = + draw + readback.
unsafe fn phase(
    core: &Core,
    gpu: &mut Option<Gpu>,
    n: u32,
    mode: u8,
) -> Result<(Timing, Option<Pixels>)> {
    let mut t = Timing::default();
    let mut last = None;
    let start = Instant::now();
    for _ in 0..n {
        let r = Instant::now();
        (core.run)();
        t.run += r.elapsed();
        t.frames += 1;
        let Some((img, w, h)) = hw_frame() else {
            continue;
        };
        t.hw_frames += 1;
        if mode == 0 {
            continue;
        }
        let Some(g) = gpu.as_mut() else {
            continue;
        };
        let p = Instant::now();
        let pixels = g.present(&img, w, h, mode == 2)?;
        t.present += p.elapsed();
        if let Some(px) = pixels {
            // Distinct read-back frames: the picture is live, not one stale image.
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&px, &mut hasher);
            t.distinct.insert(std::hash::Hasher::finish(&hasher));
            last = Some((w, h, px));
        }
    }
    t.total = start.elapsed();
    Ok((t, last))
}

fn write_png(path: &std::path::Path, w: u32, h: u32, rgba: &[u8]) -> Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(rgba)?;
    Ok(())
}

/// A rough "is there a picture" check: how many distinct colours, and the share of non-black.
fn describe(rgba: &[u8]) -> String {
    let mut colours = std::collections::HashSet::new();
    let mut lit = 0usize;
    for px in rgba.chunks_exact(4) {
        colours.insert([px[0], px[1], px[2]]);
        if px[0] > 8 || px[1] > 8 || px[2] > 8 {
            lit += 1;
        }
    }
    format!(
        "{} distinct colours, {:.1}% non-black pixels",
        colours.len(),
        100.0 * lit as f64 / (rgba.len() / 4).max(1) as f64
    )
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args = parse_args()?;
    let tmp = std::env::temp_dir().join("emudome-hw-render-spike");
    std::fs::create_dir_all(tmp.join("system"))?;
    std::fs::create_dir_all(tmp.join("saves"))?;
    {
        let mut st = STATE.lock();
        st.system_dir = Some(CString::new(
            tmp.join("system").to_string_lossy().as_bytes(),
        )?);
        st.save_dir = Some(CString::new(
            tmp.join("saves").to_string_lossy().as_bytes(),
        )?);
        st.core_log = args.core_log;
        for (k, v) in &args.opts {
            st.options.push((k.clone(), CString::new(v.as_bytes())?));
        }
    }

    // SAFETY: the spike's whole purpose is to run the given core.
    unsafe {
        let mut core = Core::load(&args.core)?;
        println!("core: {} {}", core.info_name, core.info_version);
        let t = Instant::now();
        let loaded = core.load_game(&args.content);
        let (requests, hw, negotiation) = {
            let st = STATE.lock();
            (st.hw_requests.clone(), st.hw, st.negotiation)
        };
        for (ty, major, minor) in &requests {
            println!("SET_HW_RENDER asked: context type {ty}, version {major}.{minor}");
        }
        loaded?;
        println!(
            "retro_load_game ok in {:.0} ms; av_info {}x{} @ {:.2} fps",
            t.elapsed().as_secs_f64() * 1000.0,
            core.av_info.geometry.base_width,
            core.av_info.geometry.base_height,
            core.av_info.timing.fps
        );

        let mut gpu = None;
        if let Some(hw) = hw {
            let t = Instant::now();
            let g = Gpu::new(negotiation, args.adapter.as_deref())
                .context("Vulkan context for the core")?;
            println!(
                "Vulkan context: {:?} on {} ({:?}, driver {} {}), device created by {:?} in {:.0} ms",
                g.adapter_info.backend,
                g.adapter_info.name,
                g.adapter_info.device_type,
                g.adapter_info.driver,
                g.adapter_info.driver_info,
                g.origin,
                t.elapsed().as_secs_f64() * 1000.0
            );
            println!(
                "extensions wgpu required from the core's device: {:?}",
                g.required_extensions()
            );
            gpu = Some(g);
            let reset = hw
                .context_reset
                .ok_or_else(|| anyhow!("the core gave no context_reset"))?;
            let t = Instant::now();
            reset();
            println!(
                "context_reset done in {:.0} ms",
                t.elapsed().as_secs_f64() * 1000.0
            );
        } else {
            println!("no hardware context requested: software rendering");
        }

        let (warm, _) = phase(&core, &mut gpu, args.warmup, 0)?;
        warm.report("warm-up (run only)");
        {
            let st = STATE.lock();
            println!(
                "after warm-up: last frame {:?}, set_image calls {}, image {:?}",
                st.last_frame, st.set_image_calls, st.image
            );
        }
        let (a, _) = phase(&core, &mut gpu, args.frames, 0)?;
        a.report("A run only");
        if gpu.is_some() {
            let (b, _) = phase(&core, &mut gpu, args.frames, 1)?;
            b.report("B run + wgpu draw (zero copy)");
            let (c, last) = phase(&core, &mut gpu, args.frames, 2)?;
            c.report("C run + wgpu draw + readback");
            match last {
                Some((w, h, px)) => {
                    println!("last read-back frame {w}x{h}: {}", describe(&px));
                    if let Some(p) = &args.png {
                        write_png(p, w, h, &px)?;
                        println!("written {}", p.display());
                    }
                }
                None => bail!("no hardware frame was read back"),
            }
        }
        let msgs = STATE.lock().messages.clone();
        if !msgs.is_empty() {
            println!("core messages: {msgs:?}");
        }
        // Teardown order: our view of the core's image, then the core (it frees its Vulkan
        // objects), then the device.
        if let Some(g) = gpu.as_mut() {
            g.release_image();
        }
        drop(core);
        drop(gpu);
        println!("clean shutdown");
    }
    Ok(())
}
