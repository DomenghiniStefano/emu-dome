//! Feasibility spike for Trello #16: can EmuDome's host give a libretro core a hardware render
//! context (`SET_HW_RENDER`, Vulkan or desktop OpenGL) and present the core's frames through
//! wgpu?
//!
//! No surface and no visible window. Vulkan: the core renders on a Vulkan device shared with
//! wgpu, wgpu samples the core's image (no copy) into its own target, and the target is
//! optionally read back to the CPU to check the pixels and measure the readback. OpenGL
//! (`--gl`): the core renders into our framebuffer on wgpu-hal's GL context (WGL on a window
//! that is never shown), and the frame reaches a wgpu (Vulkan) draw either through
//! `glReadPixels` + upload or through memory shared between GL and Vulkan (`gl.rs`,
//! `interop.rs`).
//!
//! ```text
//! cargo run --release --manifest-path crates/hw-render-spike/Cargo.toml -- <core> <content> [options]
//!   --opt key=value   core option answered to GET_VARIABLE (repeatable),
//!                     e.g. --opt citra_graphics_api=Vulkan for Azahar
//!   --frames N        frames per measured phase (default 300)
//!   --warmup N        frames run before measuring (default 120)
//!   --png FILE        write the last read-back frame as a PNG
//!   --core-log        print the core's log lines (format strings only)
//!   --adapter NAME    use the Vulkan GPU whose name contains NAME (default: high performance)
//!   --gl              accept a desktop OpenGL SET_HW_RENDER (otherwise refused, as before)
//!   --no-interop      with --gl: skip the zero-copy path (readback only)
//! ```
//!
//! Phases, each `--frames` long and unthrottled: (A) `retro_run` only, the core's image left
//! unused; (B) `retro_run` + wgpu draws the core image + waits for the GPU; (C) as B + copy to
//! the CPU. A software-rendering core runs phase A only. OpenGL phases are described at
//! [`run_gl`].

mod gl;
mod host;
mod interop;
mod present;
mod vk_iface;

use std::ffi::CString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use glow::HasContext;
use libretro_sys::{retro_hw_render_callback, RETRO_HW_CONTEXT_VULKAN};

use host::{Core, FrameKind, STATE};
use interop::VkSide;
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
    gl: bool,
    interop: bool,
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
    let mut gl = false;
    let mut interop = true;
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
            "--gl" => gl = true,
            "--no-interop" => interop = false,
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
        gl,
        interop,
    })
}

#[derive(Default)]
struct Timing {
    frames: u32,
    run: Duration,
    /// OpenGL readback: `glReadPixels` alone.
    read: Duration,
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
        if !self.read.is_zero() {
            println!(
                "{name}: of which glReadPixels {:.3} ms/frame",
                self.read.as_secs_f64() * 1000.0 / n
            );
        }
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
        st.accept_gl = args.gl;
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
        if let Some(hw) = hw.filter(|h| h.context_type != RETRO_HW_CONTEXT_VULKAN) {
            return run_gl(core, hw, &args);
        }
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

/// The frame size of the last `video_refresh`, if it was a hardware frame.
fn gl_frame() -> Option<(u32, u32)> {
    match STATE.lock().last_frame {
        FrameKind::Hardware { width, height } => Some((width, height)),
        FrameKind::Software { .. } | FrameKind::Dupe | FrameKind::None => None,
    }
}

/// How an OpenGL phase gets each frame out of GL.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GlMode {
    /// `retro_run` + `glFinish`: the core's own cost, frame left in GL.
    RunOnly,
    /// `glReadPixels` to the CPU, upload to wgpu, draw, wait.
    Readback,
    /// Shared memory and semaphores, draw, wait.
    ZeroCopy,
}

/// The pieces an OpenGL phase works with.
struct GlSession<'a> {
    core: &'a Core,
    gl: &'a glow::Context,
    fb: gl::Framebuffer,
    vk: VkSide,
}

impl GlSession<'_> {
    /// One OpenGL phase of `n` frames; `check` reads the drawn target back each frame.
    unsafe fn phase(
        &mut self,
        n: u32,
        mode: GlMode,
        check: bool,
    ) -> Result<(Timing, Option<Pixels>)> {
        let mut t = Timing::default();
        let mut last = None;
        let mut rows = Vec::new();
        let start = Instant::now();
        for _ in 0..n {
            let r = Instant::now();
            if mode == GlMode::ZeroCopy {
                self.fb.acquire();
            }
            (self.core.run)();
            if mode == GlMode::RunOnly {
                self.gl.finish();
            }
            t.run += r.elapsed();
            t.frames += 1;
            let Some((w, h)) = gl_frame() else {
                continue;
            };
            t.hw_frames += 1;
            if w > self.fb.width || h > self.fb.height {
                bail!(
                    "frame {w}x{h} larger than the framebuffer {}x{}",
                    self.fb.width,
                    self.fb.height
                );
            }
            let p = Instant::now();
            let pixels = match mode {
                GlMode::RunOnly => None,
                GlMode::Readback => {
                    self.fb.read(self.gl, w, h, &mut rows);
                    t.read += p.elapsed();
                    self.vk.draw_uploaded(&rows, w, h, check)?
                }
                GlMode::ZeroCopy => {
                    self.fb.release(self.gl);
                    let px = self.vk.draw_shared(w, h, check)?;
                    self.fb.vk_signalled();
                    px
                }
            };
            t.present += p.elapsed();
            if let Some(px) = pixels {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&px, &mut hasher);
                t.distinct.insert(std::hash::Hasher::finish(&hasher));
                last = Some((w, h, px));
            }
        }
        t.total = start.elapsed();
        Ok((t, last))
    }
}

/// A desktop OpenGL session. The context is wgpu-hal's GL context (`Backends::GL`), kept
/// current on this thread for the whole session; the compositor is a separate wgpu Vulkan
/// device. Phases, each `--frames` long and unthrottled: (A) `retro_run` + `glFinish`;
/// (B) readback: + `glReadPixels` + upload + wgpu draw + wait; (C) zero copy: GL renders into
/// Vulkan memory, semaphores both ways, wgpu draw + wait. Then 60 frames of B and of C with the
/// drawn target read back, to check the pictures are live.
///
/// # Safety
/// Runs the core.
unsafe fn run_gl(core: Core, hw: retro_hw_render_callback, args: &Args) -> Result<()> {
    gl::init_loader()?;
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
    desc.backends = wgpu::Backends::GL;
    let gl_instance = wgpu::Instance::new(desc);
    let t = Instant::now();
    let gl_adapter =
        pollster::block_on(gl_instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .context("no GL adapter")?;
    let hal = gl_adapter
        .as_hal::<wgpu_hal::api::Gles>()
        .ok_or_else(|| anyhow!("not a GL adapter"))?;
    let lock = hal.adapter_context().lock();
    let glc: &glow::Context = &lock;
    let v = glc.version();
    println!(
        "GL context in {:.0} ms: {} (asked {}.{}; embedded {})",
        t.elapsed().as_secs_f64() * 1000.0,
        gl::describe(glc),
        hw.version_major,
        hw.version_minor,
        v.is_embedded
    );
    if v.is_embedded || (v.major, v.minor) < (hw.version_major, hw.version_minor) {
        bail!("the GL context is older than the core asks for");
    }

    let t = Instant::now();
    let mut vk = VkSide::new()?;
    println!(
        "compositor: wgpu {:?} on {} in {:.0} ms",
        vk.adapter_info.backend,
        vk.adapter_info.name,
        t.elapsed().as_secs_f64() * 1000.0
    );
    let geo = STATE.lock().geometry.unwrap_or(core.av_info.geometry);
    let (fw, fh) = (
        geo.max_width.max(core.av_info.geometry.max_width).max(1),
        geo.max_height.max(core.av_info.geometry.max_height).max(1),
    );
    let interop = args.interop && gl::has_win32_interop(glc);
    let handles = if interop {
        Some(vk.create_shared(fw, fh)?)
    } else {
        None
    };
    let fb = gl::Framebuffer::new(glc, fw, fh, handles.as_ref())?;
    println!(
        "framebuffer {fw}x{fh} (core max size), colour on {}",
        if interop {
            "Vulkan memory imported by GL (zero copy available)"
        } else {
            "a GL texture (readback only)"
        }
    );
    let mut s = GlSession {
        core: &core,
        gl: glc,
        fb,
        vk,
    };
    let reset = hw
        .context_reset
        .ok_or_else(|| anyhow!("the core gave no context_reset"))?;
    let t = Instant::now();
    reset();
    println!(
        "context_reset done in {:.0} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );

    let (warm, _) = s.phase(args.warmup, GlMode::RunOnly, false)?;
    warm.report("warm-up (run + glFinish)");
    println!("after warm-up: last frame {:?}", STATE.lock().last_frame);
    let (a, _) = s.phase(args.frames, GlMode::RunOnly, false)?;
    a.report("A run + glFinish");
    let (b, _) = s.phase(args.frames, GlMode::Readback, false)?;
    b.report("B run + glReadPixels + upload + wgpu draw");
    if interop {
        let (c, _) = s.phase(args.frames, GlMode::ZeroCopy, false)?;
        c.report("C run + zero-copy wgpu draw");
    }
    let (bc, mut shown) = s.phase(60, GlMode::Readback, true)?;
    bc.report("B check");
    if let Some((w, h, px)) = &shown {
        println!("readback path, last drawn frame {w}x{h}: {}", describe(px));
    }
    if interop {
        let (cc, last_c) = s.phase(60, GlMode::ZeroCopy, true)?;
        cc.report("C check");
        if let Some((w, h, px)) = &last_c {
            println!("zero-copy path, last drawn frame {w}x{h}: {}", describe(px));
        }
        shown = last_c.or(shown);
    }
    match shown {
        Some((w, h, px)) => {
            if let Some(p) = &args.png {
                write_png(p, w, h, &px)?;
                println!("written {}", p.display());
            }
        }
        None => bail!("no hardware frame was drawn"),
    }
    let msgs = STATE.lock().messages.clone();
    if !msgs.is_empty() {
        println!("core messages: {msgs:?}");
    }
    // Teardown with the context current: the core's GL objects, then ours, then Vulkan's.
    if let Some(destroy) = hw.context_destroy {
        destroy();
    }
    let GlSession { fb, mut vk, .. } = s;
    drop(core);
    fb.destroy(glc);
    vk.destroy_shared();
    drop(lock);
    println!("clean shutdown");
    Ok(())
}
