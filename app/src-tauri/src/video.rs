//! Native renderer: draws the core's latest frame with wgpu under the interface's
//! transparent webview. Runs on a thread of its own.
//!
//! The surface is either Tauri's window (desktop) or one the system hands over later
//! and may take back (Android's `SurfaceView`): [`Renderer::attach`] / [`Renderer::detach`].
//!
//! The GPU outlives the surfaces ([`GpuService`]): instance, adapter, device and queue are made
//! with the first surface and kept for the whole run, so a detach (Android going to the
//! background) or a new surface never destroys the device a running core may render with
//! (hardware rendering, `docs/feasibility-hw-render.md`). Only a surface the current GPU cannot
//! present to makes a new device. The painter (pipelines, game texture, effect chain) is kept
//! too while the surface format stays the same.
//!
//! Screen effects ([`Renderer::set_effect`]): the game texture goes through a librashader
//! filter chain into a texture the size of the game's rectangle, then one pass of ours mixes it
//! over the plain image (`fx.fxint`) and brightens it (`fx.fxbright`) on the surface.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use emudome_core::tuning::{PresentMode, VideoTuning};
use emudome_core::video_sink::Wake;
use emudome_core::{ErrorCode, Shared};
use librashader::presets::ShaderFeatures;
use librashader::runtime::wgpu::{FilterChain, FilterChainOptions, WgpuOutputView};
use librashader::runtime::{Size, Viewport};
use parking_lot::Mutex;
use serde::Serialize;

use crate::health::{Part, SharedHealth};

/// How the surface presents, from the tuning knobs.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceSettings {
    pub present_mode: wgpu::PresentMode,
    /// `desired_maximum_frame_latency`.
    pub frame_latency: u32,
}

impl SurfaceSettings {
    pub fn from_tuning(t: &VideoTuning) -> Self {
        Self {
            present_mode: match t.present_mode {
                PresentMode::Fifo => wgpu::PresentMode::Fifo,
                PresentMode::FifoRelaxed => wgpu::PresentMode::FifoRelaxed,
                PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
                PresentMode::Immediate => wgpu::PresentMode::Immediate,
                PresentMode::AutoVsync => wgpu::PresentMode::AutoVsync,
                PresentMode::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
            },
            frame_latency: t.frame_latency,
        }
    }
}

/// What the GPU and the surface actually are, for logs and bench reports.
#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub adapter: String,
    pub backend: String,
    pub device_type: String,
    pub driver: String,
    pub driver_info: String,
    pub present_mode_requested: String,
    /// Differs from the requested one when the surface does not support it (`AutoVsync` then).
    pub present_mode_used: String,
    pub present_modes_supported: Vec<String>,
    pub frame_latency: u32,
    pub format: String,
    pub alpha_mode: String,
}

impl GpuInfo {
    #[cfg_attr(
        not(feature = "bench"),
        expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
    )]
    pub fn present_mode_supported(&self) -> bool {
        self.present_mode_requested == self.present_mode_used
    }
}

/// One frame presented on the surface.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    not(feature = "bench"),
    expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
)]
pub struct Present {
    pub at: Instant,
    /// The game frame it showed (`Frame::seq`).
    pub seq: u64,
}

/// What the video thread measures. Presents are kept only while `presents` is `Some`
/// (bench mode), so a normal session does not grow it.
#[derive(Debug, Default)]
pub struct VideoMetrics {
    pub gpu: Option<GpuInfo>,
    /// Why the last surface handed over could not be used.
    pub surface_error: Option<String>,
    /// `get_current_texture` failures, by kind.
    pub surface_errors: BTreeMap<&'static str, u64>,
    /// First present that showed a game frame.
    pub first_game_present: Option<Instant>,
    pub presents: Option<Vec<Present>>,
}

pub type SharedVideoMetrics = Arc<Mutex<VideoMetrics>>;

/// A screen effect: a `.slangp` preset and how it is mixed over the plain image.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectSpec {
    pub preset: PathBuf,
    /// 0..=1, share of the effect over the plain image.
    pub intensity: f32,
    /// Multiplies the effect's colours (1 = unchanged).
    pub brightness: f32,
}

/// Told when a preset cannot be used: its path and the reason (librashader's or wgpu's text,
/// for the logs and the `detail` of an error code). The plain image is drawn meanwhile.
pub type EffectListener = Box<dyn Fn(&Path, String) + Send>;

enum Msg {
    Resize(u32, u32),
    Source(Option<Arc<Shared>>),
    Attach(wgpu::SurfaceTarget<'static>, u32, u32),
    Detach(Sender<()>),
    /// Share of the surface's height, at the bottom, the game keeps clear of (G09's bar).
    Inset(f32),
    Effect(Option<EffectSpec>),
    EffectListener(EffectListener),
}

#[derive(Clone)]
pub struct Renderer {
    tx: Sender<Msg>,
    /// Wakes the video thread for a message (the game's frames wake it through
    /// `Shared::frame_wake`).
    wake: Arc<Wake>,
    metrics: SharedVideoMetrics,
}

impl Renderer {
    /// Draws on Tauri's window. Must be called on the main thread: on Windows and Linux
    /// the window's native handle is only available there (from another thread Tauri
    /// blocks waiting for it). Drawing then continues on a dedicated thread.
    pub fn spawn(
        window: tauri::Window,
        width: u32,
        height: u32,
        settings: SurfaceSettings,
        health: SharedHealth,
    ) -> Result<Self> {
        let mut service = GpuService::new();
        let mut painter = None;
        let gpu = Gpu::new(
            &mut service,
            &mut painter,
            window.into(),
            width,
            height,
            settings,
        )?;
        Self::start(service, Some(gpu), painter, settings, health)
    }

    /// Starts with no surface: it arrives with [`Self::attach`].
    pub fn spawn_detached(settings: SurfaceSettings, health: SharedHealth) -> Result<Self> {
        Self::start(GpuService::new(), None, None, settings, health)
    }

    fn start(
        service: GpuService,
        gpu: Option<Gpu>,
        painter: Option<Painter>,
        settings: SurfaceSettings,
        health: SharedHealth,
    ) -> Result<Self> {
        let metrics = SharedVideoMetrics::default();
        metrics.lock().gpu = gpu.as_ref().map(|g| g.info.clone());
        let (tx, rx) = mpsc::channel();
        let wake = Arc::new(Wake::default());
        let video = Video::new(
            service,
            gpu,
            painter,
            settings,
            metrics.clone(),
            wake.clone(),
            health.clone(),
        );
        std::thread::Builder::new()
            .name("emudome-video".into())
            .spawn(move || health.guard(Part::NativeVideo, || video.run(rx)))?;
        Ok(Self { tx, wake, metrics })
    }

    fn send(&self, msg: Msg) {
        if self.tx.send(msg).is_ok() {
            self.wake.notify();
        }
    }

    /// What the video thread measures (GPU in use, surface errors, presents in bench mode).
    #[cfg_attr(
        not(feature = "bench"),
        expect(dead_code, reason = "read by the bench mode only (feature `bench`)")
    )]
    pub fn metrics(&self) -> &SharedVideoMetrics {
        &self.metrics
    }

    /// The window's new size; 0 × 0 (minimized) stops drawing until it has a size again.
    pub fn resize(&self, width: u32, height: u32) {
        self.send(Msg::Resize(width, height));
    }

    pub fn set_source(&self, shared: Option<Arc<Shared>>) {
        self.send(Msg::Source(shared));
    }

    /// A new surface to draw on (it replaces the current one, if any).
    pub fn attach(&self, target: wgpu::SurfaceTarget<'static>, width: u32, height: u32) {
        self.send(Msg::Attach(target, width, height));
    }

    /// Keeps the bottom `fraction` of the surface clear: the game shrinks into the rest
    /// (quick menu G09, whose bar sits at the bottom). 0 = the whole surface.
    pub fn set_inset(&self, fraction: f32) {
        self.send(Msg::Inset(fraction));
    }

    /// The screen effect to draw the game with (`None`: the plain image). Kept across
    /// surfaces; a preset is loaded again only when its path changes.
    pub fn set_effect(&self, effect: Option<EffectSpec>) {
        self.send(Msg::Effect(effect));
    }

    /// Who is told when a preset cannot be loaded or drawn.
    pub fn on_effect_error(&self, listener: EffectListener) {
        self.send(Msg::EffectListener(listener));
    }

    /// Lets go of the surface; returns once the GPU no longer uses it (Android destroys it right
    /// after). Waits for as long as the video thread takes: a surface destroyed under a present in
    /// flight is a crash, a slow detach only a slow pause. Returns at once if the thread is gone.
    pub fn detach(&self) {
        let (ack_tx, ack_rx) = mpsc::channel();
        let started = Instant::now();
        self.send(Msg::Detach(ack_tx));
        loop {
            match ack_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => log::warn!(
                    "native video: the surface is still in use after {:.0} s",
                    started.elapsed().as_secs_f64()
                ),
            }
        }
    }
}

/// With nothing new to show, the video thread sleeps until woken, at most this long (it then
/// looks at its messages and logs the rates).
const IDLE_WAIT: Duration = Duration::from_millis(100);

/// After a frame that could not be presented (surface outdated, lost, occluded, timed out),
/// the next try waits this long, so a surface that keeps failing never spins a core.
const RETRY_WAIT: Duration = Duration::from_millis(8);

/// How often the video thread logs the frame rates (to measure FPS from the logs too).
const FPS_LOG_EVERY: Duration = Duration::from_secs(10);

/// The video thread: the GPU, the surface (while there is one) and the game it shows.
struct Video {
    service: GpuService,
    /// The surface drawn on; `None` while detached.
    gpu: Option<Gpu>,
    /// Draws the game; kept across surfaces of the same format.
    painter: Option<Painter>,
    settings: SurfaceSettings,
    metrics: SharedVideoMetrics,
    source: Option<Arc<Shared>>,
    presented: u32,
    since: Instant,
    /// See [`Renderer::set_inset`]; kept across surfaces.
    inset: f32,
    /// See [`Renderer::set_effect`]; kept across surfaces.
    effect: Option<EffectSpec>,
    effect_listener: Option<EffectListener>,
    /// Woken by the messages and by the game's new frames.
    wake: Arc<Wake>,
    /// Something changed since the last present (size, surface, game, inset, effect).
    dirty: bool,
    /// Frames not presented because nothing changed, since the last rate log.
    unchanged: u32,
    /// Where a surface that cannot be used is reported (`videoSurfaceFailed`).
    health: SharedHealth,
}

impl Video {
    fn new(
        service: GpuService,
        gpu: Option<Gpu>,
        painter: Option<Painter>,
        settings: SurfaceSettings,
        metrics: SharedVideoMetrics,
        wake: Arc<Wake>,
        health: SharedHealth,
    ) -> Self {
        Self {
            service,
            gpu,
            painter,
            settings,
            metrics,
            source: None,
            presented: 0,
            since: Instant::now(),
            inset: 0.0,
            effect: None,
            effect_listener: None,
            wake,
            dirty: true,
            unchanged: 0,
            health,
        }
    }

    /// Whether the game has a frame the painter has not uploaded yet.
    fn new_frame(&self) -> bool {
        let (Some(source), Some(painter)) = (&self.source, &self.painter) else {
            return false;
        };
        let frame = source.frame.lock();
        !frame.rgba.is_empty() && frame.seq != painter.last_seq
    }

    /// Presents only when something changed (a new game frame, the size, the surface, the
    /// game, the inset, the effect); otherwise sleeps until woken. A paused game, a menu over a
    /// still picture, a minimized window cost no GPU work and no CPU core, and every game frame
    /// is presented once (Trello #79; the pacing itself is `docs/video-pacing.md`).
    fn run(mut self, rx: Receiver<Msg>) {
        loop {
            loop {
                match rx.try_recv() {
                    Ok(m) => self.handle(m),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => return,
                }
            }
            let drawable = self.gpu.as_ref().is_some_and(Gpu::drawable) && self.painter.is_some();
            if !(drawable && (self.dirty || self.new_frame())) {
                if drawable {
                    self.unchanged += 1;
                }
                self.wake.wait(IDLE_WAIT);
                self.log_fps();
                continue;
            }
            if let Err(kind) = self.present() {
                *self.metrics.lock().surface_errors.entry(kind).or_default() += 1;
                // Tried again shortly (or at the next message, a resize for one), never at once.
                self.wake.wait(RETRY_WAIT);
            }
            self.log_fps();
        }
    }

    fn handle(&mut self, m: Msg) {
        // Every message but the listener's changes what is on screen.
        self.dirty |= !matches!(m, Msg::EffectListener(_));
        match m {
            Msg::Resize(w, h) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(&self.service, w, h);
                }
            }
            Msg::Source(s) => {
                // The new game's frames wake this thread; the old game's no longer do.
                if let Some(old) = &self.source {
                    old.frame_wake.lock().take();
                }
                if let Some(new) = &s {
                    *new.frame_wake.lock() = Some(self.wake.clone());
                }
                self.source = s;
                if let Some(painter) = &mut self.painter {
                    painter.reset_game();
                }
            }
            Msg::Attach(target, w, h) => {
                // The old surface goes first; the GPU and the painter stay.
                self.gpu = None;
                match Gpu::new(
                    &mut self.service,
                    &mut self.painter,
                    target,
                    w,
                    h,
                    self.settings,
                ) {
                    Ok(gpu) => {
                        {
                            let mut m = self.metrics.lock();
                            m.gpu = Some(gpu.info.clone());
                            m.surface_error = None;
                        }
                        self.gpu = Some(gpu);
                        self.health.set(Part::Surface, None);
                    }
                    Err(e) => {
                        log::error!("native video: surface not usable: {e:#}");
                        let detail = format!("{e:#}");
                        self.metrics.lock().surface_error = Some(detail.clone());
                        self.health.set(
                            Part::Surface,
                            Some(ErrorCode::VideoSurfaceFailed { detail }),
                        );
                    }
                }
            }
            Msg::Detach(ack) => {
                // Only the surface: the device lives on for the next one. The GPU finishes
                // what it was given first (a present in flight), then the surface goes.
                if let Some(d) = self.service.device() {
                    if let Err(e) = d.device.poll(wgpu::PollType::Wait {
                        submission_index: None,
                        timeout: Some(Duration::from_secs(5)),
                    }) {
                        log::warn!("native video: the GPU did not finish before the detach: {e}");
                    }
                }
                self.gpu = None;
                let _ = ack.send(());
            }
            Msg::Inset(fraction) => self.inset = fraction,
            Msg::Effect(effect) => self.effect = effect,
            Msg::EffectListener(listener) => self.effect_listener = Some(listener),
        }
    }

    /// Uploads the newest frame and presents; the kind of surface error when nothing was
    /// presented (the picture stays dirty and is tried again).
    fn present(&mut self) -> Result<(), &'static str> {
        let (Some(gpu), Some(painter)) = (&mut self.gpu, &mut self.painter) else {
            return Err("no surface");
        };
        painter.sync_effect(self.effect.as_ref());
        if let Some((preset, detail)) = painter.effect_error.take() {
            log::error!("screen effect {}: {detail}", preset.display());
            if let Some(listener) = &self.effect_listener {
                listener(&preset, detail);
            }
        }
        painter.upload_frame(self.source.as_deref());
        gpu.draw(&self.service, painter, self.inset)?;
        self.dirty = false;
        self.presented += 1;
        let at = Instant::now();
        let mut m = self.metrics.lock();
        if painter.game.is_some() && m.first_game_present.is_none() {
            m.first_game_present = Some(at);
        }
        if let Some(presents) = &mut m.presents {
            presents.push(Present {
                at,
                seq: painter.last_seq,
            });
        }
        Ok(())
    }

    fn log_fps(&mut self) {
        let elapsed = self.since.elapsed();
        if elapsed < FPS_LOG_EVERY {
            return;
        }
        if let Some(source) = &self.source {
            let stats = source.stats.lock();
            // `core` is the rate of the last whole second of running game; paused, it is 0.
            log::info!(
                "fps: core {:.2}{}, presented {:.2} (target {:.2}), {} idle waits",
                stats.emu_fps,
                if stats.paused { " (paused)" } else { "" },
                f64::from(self.presented) / elapsed.as_secs_f64(),
                stats.av_info.timing.fps,
                self.unchanged,
            );
        }
        self.presented = 0;
        self.unchanged = 0;
        self.since = Instant::now();
    }
}

/// Background colour with no game: the design system's `bg` (#0e0f13), in linear space.
const BG: wgpu::Color = wgpu::Color {
    r: 0.0044,
    g: 0.0048,
    b: 0.0065,
    a: 1.0,
};

/// The game's shaders. The game texture holds the core's sRGB bytes as they are (`Rgba8Unorm`,
/// what librashader's presets expect), so the colours are decoded here when the surface encodes
/// them again (`DECODE`, an sRGB surface).
const SHADER: &str = r#"
struct Out { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> Out {
    // Triangle covering the whole viewport.
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: Out;
    o.pos = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
    o.uv = vec2<f32>(p.x, 1.0 - p.y);
    return o;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

fn to_surface(c: vec3<f32>) -> vec4<f32> {
    if (!DECODE) {
        return vec4<f32>(c, 1.0);
    }
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return vec4<f32>(select(hi, lo, c <= vec3<f32>(0.04045)), 1.0);
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    return to_surface(textureSample(tex, samp, in.uv).rgb);
}

// The effect's output, one texel per pixel of the game's rectangle, mixed over the plain image.
struct Mix { intensity: f32, brightness: f32, origin: vec2<f32> };
@group(0) @binding(2) var fx_tex: texture_2d<f32>;
@group(0) @binding(3) var<uniform> mix_: Mix;

@fragment
fn fs_effect(in: Out) -> @location(0) vec4<f32> {
    let plain = textureSample(tex, samp, in.uv).rgb;
    let size = vec2<i32>(textureDimensions(fx_tex)) - vec2<i32>(1);
    let p = clamp(vec2<i32>(floor(in.pos.xy - mix_.origin)), vec2<i32>(0), size);
    let fx = min(textureLoad(fx_tex, p, 0).rgb * mix_.brightness, vec3<f32>(1.0));
    return to_surface(mix(plain, fx, mix_.intensity));
}
"#;

/// Format of the game texture and of the effect's output: the bytes as they are, sRGB-encoded.
const RAW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The effect's filter chain and the texture it draws into (the size of the game's rectangle).
struct Effect {
    preset: PathBuf,
    chain: FilterChain,
    target: Option<EffectTarget>,
}

struct EffectTarget {
    width: u32,
    height: u32,
    view: wgpu::TextureView,
}

struct GameTexture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

/// The GPU for the whole run: made with the first surface (an adapter that can present to it),
/// kept when surfaces come and go. The one owner of the device; the painter holds clones.
struct GpuService {
    instance: wgpu::Instance,
    device: Option<ServiceDevice>,
}

struct ServiceDevice {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuService {
    fn new() -> Self {
        // No display handle: Windows and Android do not need it, and on Linux Tauri's
        // one does not match the surface's (MismatchingDisplayHandle).
        // The environment (WGPU_BACKEND, WGPU_DX12_PRESENTATION_SYSTEM...) picks the backend.
        Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            device: None,
        }
    }

    /// A surface for `target`, and whether the device was (re)made for it: on the first
    /// surface, or when the GPU in use cannot present to this one.
    fn surface(
        &mut self,
        target: wgpu::SurfaceTarget<'static>,
    ) -> Result<(wgpu::Surface<'static>, bool)> {
        let surface = self
            .instance
            .create_surface(target)
            .map_err(|e| anyhow::anyhow!("wgpu surface not created: {e:?}"))?;
        if let Some(d) = &self.device {
            if d.adapter.is_surface_supported(&surface) {
                return Ok((surface, false));
            }
            log::warn!("native video: the GPU in use cannot present to the new surface");
        }
        let adapter =
            pollster::block_on(self.instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            }))
            .context("no compatible GPU")?;
        log::info!("GPU: {:?}", adapter.get_info());
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("emudome"),
                // Mobile GPUs do not reach the desktop defaults.
                required_limits:
                    wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            }))?;
        self.device = Some(ServiceDevice {
            adapter,
            device,
            queue,
        });
        Ok((surface, true))
    }

    fn device(&self) -> Option<&ServiceDevice> {
        self.device.as_ref()
    }
}

/// A surface the game is presented on (the GPU is [`GpuService`]'s).
struct Gpu {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    info: GpuInfo,
    /// The window has no size (minimized): nothing is drawn.
    minimized: bool,
}

/// Draws the game, with its effect, on any texture view of the surface's format: the surface
/// in the app, a plain texture in the GPU test.
struct Painter {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// The format it draws in (the surface's): another format needs another painter.
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    /// The pass that mixes the effect over the plain image.
    effect_pipeline: wgpu::RenderPipeline,
    effect_layout: wgpu::BindGroupLayout,
    /// `Mix` of the shader: intensity, brightness, origin of the game's rectangle.
    effect_uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    game: Option<GameTexture>,
    last_seq: u64,
    effect: Option<Effect>,
    /// The preset that failed: not tried again until another one is asked for.
    effect_failed: Option<PathBuf>,
    /// A preset that could not be loaded or drawn, for the listener.
    effect_error: Option<(PathBuf, String)>,
    /// wgpu errors no scope caught (librashader's pipelines): the latest, instead of a panic.
    gpu_error: Arc<Mutex<Option<String>>>,
    frame_count: usize,
    /// Intensity and brightness of the effect (see [`EffectSpec`]).
    mix: (f32, f32),
}

impl Gpu {
    /// Attaches `target`: a surface on the service's GPU (made now if there is none or it cannot
    /// present here), and a painter for its format (`painter` kept when the format is the same
    /// and the device was not remade).
    fn new(
        service: &mut GpuService,
        painter: &mut Option<Painter>,
        target: wgpu::SurfaceTarget<'static>,
        width: u32,
        height: u32,
        settings: SurfaceSettings,
    ) -> Result<Self> {
        let (surface, remade) = service.surface(target)?;
        if remade {
            *painter = None;
        }
        let d = service
            .device()
            .context("no GPU device after making the surface")?;
        let (adapter, device, queue) = (&d.adapter, &d.device, &d.queue);
        let adapter_info = adapter.get_info();

        let caps = surface.get_capabilities(adapter);
        // A surface the adapter cannot present to lists nothing: refused, never a panic.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .context("the surface offers no format to this GPU")?;
        let alpha_mode = caps
            .alpha_modes
            .first()
            .copied()
            .context("the surface offers no alpha mode to this GPU")?;
        // The Auto modes always resolve; an explicit one the surface lacks would be a
        // validation error (a panic in wgpu): fall back to AutoVsync and say so.
        let requested = settings.present_mode;
        let present_mode = if matches!(
            requested,
            wgpu::PresentMode::AutoVsync | wgpu::PresentMode::AutoNoVsync
        ) || caps.present_modes.contains(&requested)
        {
            requested
        } else {
            log::warn!(
                "present mode {requested:?} not supported (supported: {:?}), using AutoVsync",
                caps.present_modes
            );
            wgpu::PresentMode::AutoVsync
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            desired_maximum_frame_latency: settings.frame_latency,
            alpha_mode,
            color_space: wgpu::SurfaceColorSpace::Auto,
            view_formats: vec![],
        };
        surface.configure(device, &config);
        if painter.as_ref().map(|p| p.format) != Some(format) {
            *painter = Some(Painter::new(device.clone(), queue.clone(), format));
        }
        let info = GpuInfo {
            adapter: adapter_info.name.clone(),
            backend: format!("{:?}", adapter_info.backend),
            device_type: format!("{:?}", adapter_info.device_type),
            driver: adapter_info.driver.clone(),
            driver_info: adapter_info.driver_info.clone(),
            present_mode_requested: format!("{:?}", settings.present_mode),
            present_mode_used: format!("{present_mode:?}"),
            present_modes_supported: caps
                .present_modes
                .iter()
                .map(|m| format!("{m:?}"))
                .collect(),
            frame_latency: settings.frame_latency,
            format: format!("{format:?}"),
            alpha_mode: format!("{:?}", config.alpha_mode),
        };
        log::info!(
            "surface: {format:?}, {present_mode:?}, frame latency {}",
            settings.frame_latency
        );
        Ok(Self {
            surface,
            config,
            info,
            minimized: width == 0 || height == 0,
        })
    }

    /// A new size; 0 in either direction (a minimized window) stops drawing until a real size
    /// comes, instead of presenting to a surface that answers `Outdated` every time.
    fn resize(&mut self, service: &GpuService, width: u32, height: u32) {
        self.minimized = width == 0 || height == 0;
        if let (false, Some(d)) = (self.minimized, service.device()) {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&d.device, &self.config);
        }
    }

    /// Whether there is something to draw on (not minimized).
    fn drawable(&self) -> bool {
        !self.minimized
    }

    /// Draws and presents with `painter`; when no frame was presented, the kind of surface
    /// error.
    fn draw(
        &mut self,
        service: &GpuService,
        painter: &mut Painter,
        inset: f32,
    ) -> Result<(), &'static str> {
        let Some(d) = service.device() else {
            return Err("no device");
        };
        // A frame not presented: the caller waits before trying again (`RETRY_WAIT`).
        let skipped = |kind: &'static str| {
            log::debug!("frame skipped: {kind}");
            Err(kind)
        };
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&d.device, &self.config);
                return skipped("outdated");
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&d.device, &self.config);
                return skipped("lost");
            }
            wgpu::CurrentSurfaceTexture::Timeout => return skipped("timeout"),
            wgpu::CurrentSurfaceTexture::Occluded => return skipped("occluded"),
            wgpu::CurrentSurfaceTexture::Validation => return skipped("validation"),
        };
        let view = output.texture.create_view(&Default::default());
        painter.paint(&view, (self.config.width, self.config.height), inset);
        d.queue.present(output);
        Ok(())
    }
}

impl Painter {
    fn new(device: wgpu::Device, queue: wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // A validation error (a user's preset librashader let through) is logged and makes the
        // effect fail, instead of wgpu's default panic on the video thread.
        let gpu_error: Arc<Mutex<Option<String>>> = Arc::default();
        let errors = gpu_error.clone();
        device.on_uncaptured_error(Arc::new(move |e: wgpu::Error| {
            log::error!("wgpu: {e}");
            *errors.lock() = Some(e.to_string());
        }));

        let source = format!("const DECODE: bool = {};\n{SHADER}", format.is_srgb());
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("game"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("game"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("game"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("game"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let effect_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("effect"),
            entries: &[
                texture_entry(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let effect_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("effect"),
                bind_group_layouts: &[Some(&effect_layout)],
                ..Default::default()
            });
        let effect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("effect"),
            layout: Some(&effect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_effect"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let effect_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("effect mix"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Sharp pixels: the "smooth" filter and the shaders come with librashader.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            device,
            queue,
            format,
            pipeline,
            layout,
            effect_pipeline,
            effect_layout,
            effect_uniform,
            sampler,
            game: None,
            last_seq: u64::MAX,
            effect: None,
            effect_failed: None,
            effect_error: None,
            gpu_error,
            frame_count: 0,
            mix: (1.0, 1.0),
        }
    }

    /// Another game (or none): the next frame is uploaded whatever its number.
    fn reset_game(&mut self) {
        self.game = None;
        self.last_seq = u64::MAX;
    }

    fn upload_frame(&mut self, source: Option<&Shared>) {
        let Some(source) = source else {
            return;
        };
        // The pixels' `Arc` is taken under the lock, the upload runs after it: the core's next
        // frame never waits for `write_texture`.
        let (width, height, rgba) = {
            let frame = source.frame.lock();
            if frame.seq == self.last_seq || frame.rgba.is_empty() {
                return;
            }
            self.last_seq = frame.seq;
            (frame.width, frame.height, frame.rgba.clone())
        };
        self.upload(width, height, &rgba);
    }

    /// Puts `rgba` (`w` x `h`) in the game texture.
    fn upload(&mut self, w: u32, h: u32, rgba: &[u8]) {
        if self.game.as_ref().map(|g| (g.width, g.height)) != Some((w, h)) {
            self.game = Some(self.create_texture(w, h));
        }
        let Some(game) = self.game.as_ref() else {
            return;
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &game.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    fn create_texture(&self, width: u32, height: u32) -> GameTexture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("game"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: RAW_FORMAT,
            // COPY_SRC: presets that keep past frames copy the input.
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("game"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        GameTexture {
            width,
            height,
            texture,
            view,
            bind_group,
        }
    }

    /// Draws the game (or the background) on `view`, a target of `size`, and submits.
    fn paint(&mut self, view: &wgpu::TextureView, size: (u32, u32), inset: f32) {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // The effect first draws into its own texture; `None`: the plain image this frame.
        let effect_group = self.record_effect(&mut encoder, size, inset);
        {
            let clear = if self.game.is_some() {
                wgpu::Color::BLACK
            } else {
                BG
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("game"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            if let Some(game) = &self.game {
                let (x, y, w, h) = game_rect(size, game, inset);
                if let Some((group, (ew, eh))) = &effect_group {
                    // The effect's texture is exactly the rectangle's whole pixels.
                    pass.set_viewport(x, y, *ew as f32, *eh as f32, 0.0, 1.0);
                    pass.set_pipeline(&self.effect_pipeline);
                    pass.set_bind_group(0, group, &[]);
                } else {
                    pass.set_viewport(x, y, w, h, 0.0, 1.0);
                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, &game.bind_group, &[]);
                }
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
        self.frame_count = self.frame_count.wrapping_add(1);
        if let Some(e) = self.gpu_error.lock().take() {
            if let Some(effect) = self.effect.take() {
                // Most likely the effect (a user's preset): drop it, draw the plain image.
                self.effect_failed = Some(effect.preset.clone());
                self.effect_error = Some((effect.preset, e));
            }
        }
    }

    /// Loads the preset `want` asks for, when it is not the one loaded (or the one that just
    /// failed); `None` drops the effect. A failure goes to `effect_error`.
    fn sync_effect(&mut self, want: Option<&EffectSpec>) {
        let Some(spec) = want else {
            self.effect = None;
            self.effect_failed = None;
            return;
        };
        self.mix = (spec.intensity.clamp(0.0, 1.0), spec.brightness.max(0.0));
        if self
            .effect
            .as_ref()
            .is_some_and(|e| e.preset == spec.preset)
            || self.effect_failed.as_ref() == Some(&spec.preset)
        {
            return;
        }
        self.effect = None;
        self.effect_failed = None;
        let started = Instant::now();
        match self.load_chain(&spec.preset) {
            Ok(chain) => {
                log::info!(
                    "screen effect {} loaded in {} ms",
                    spec.preset.display(),
                    started.elapsed().as_millis()
                );
                self.effect = Some(Effect {
                    preset: spec.preset.clone(),
                    chain,
                    target: None,
                });
            }
            Err(e) => {
                self.effect_failed = Some(spec.preset.clone());
                self.effect_error = Some((spec.preset.clone(), e));
            }
        }
    }

    fn load_chain(&self, preset: &Path) -> Result<FilterChain, String> {
        self.gpu_error.lock().take();
        let options = FilterChainOptions {
            // The game texture has no mipmaps, and no preset of ours needs them.
            force_no_mipmaps: true,
            enable_cache: false,
            adapter_info: None,
        };
        let chain = FilterChain::load_from_path(
            preset,
            ShaderFeatures::NONE,
            &self.device,
            &self.queue,
            Some(&options),
        )
        .map_err(|e| e.to_string())?;
        match self.gpu_error.lock().take() {
            Some(e) => Err(e),
            None => Ok(chain),
        }
    }

    /// Records the effect's chain into its texture and returns the bind group of the mixing
    /// pass with the texture's size; `None` without an effect or a game (or when it failed).
    fn record_effect(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_size: (u32, u32),
        inset: f32,
    ) -> Option<(wgpu::BindGroup, (u32, u32))> {
        let game = self.game.as_ref()?;
        self.effect.as_ref()?;
        let (x, y, w, h) = game_rect(target_size, game, inset);
        // Whole pixels, at least one.
        let size = (w.floor().max(1.0) as u32, h.floor().max(1.0) as u32);
        let effect = self.effect.as_mut()?;
        if effect.target.as_ref().map(|t| (t.width, t.height)) != Some(size) {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("effect output"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: RAW_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            effect.target = Some(EffectTarget {
                width: size.0,
                height: size.1,
                view: texture.create_view(&Default::default()),
            });
        }
        let target = effect.target.as_ref()?;
        let output = WgpuOutputView::new_from_raw(
            &target.view,
            Size::new(target.width, target.height),
            RAW_FORMAT,
        );
        let viewport = Viewport {
            x: 0.0,
            y: 0.0,
            mvp: None,
            output,
            size: Size::new(target.width, target.height),
        };
        if let Err(e) =
            effect
                .chain
                .frame(&game.texture, &viewport, encoder, self.frame_count, None)
        {
            let preset = effect.preset.clone();
            self.effect = None;
            self.effect_failed = Some(preset.clone());
            self.effect_error = Some((preset, e.to_string()));
            return None;
        }
        let (intensity, brightness) = self.mix;
        self.queue.write_buffer(
            &self.effect_uniform,
            0,
            bytemuck::cast_slice(&[intensity, brightness, x, y]),
        );
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("effect"),
            layout: &self.effect_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&game.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&target.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.effect_uniform.as_entire_binding(),
                },
            ],
        });
        Some((group, size))
    }
}

/// [`viewport`] of `game` on a target of `size`.
fn game_rect(size: (u32, u32), game: &GameTexture, inset: f32) -> (f32, f32, f32, f32) {
    viewport(
        (size.0 as f32, size.1 as f32),
        (game.width as f32, game.height as f32),
        inset,
    )
}

/// The game's rectangle `(x, y, w, h)` on a `surface` of `(width, height)`: integer scale when it
/// fits, otherwise the largest that keeps the aspect ratio, centred in the part of the surface
/// above the bottom `inset` share (0..0.9) that the quick menu's bar keeps clear.
///
/// The rule is `emudome_core::screens::fit`, the one the touch mapping composes the screens
/// with: the whole frame drawn here is the default screen layout (NDS: top above bottom).
/// TODO(#28): draw each `screens::compose` placement once the layout comes from `screens.*`.
fn viewport(surface: (f32, f32), game: (f32, f32), inset: f32) -> (f32, f32, f32, f32) {
    let r = emudome_core::screens::fit(
        emudome_core::screens::Area {
            width: surface.0,
            height: surface.1,
            inset,
        },
        game,
    );
    (r.x, r.y, r.w, r.h)
}

#[cfg(test)]
mod tests {
    use super::viewport;

    #[test]
    fn the_game_is_centred_at_an_integer_scale() {
        // GBA 240x160 on 1440x900: 5x = 1200x800.
        assert_eq!(
            viewport((1440.0, 900.0), (240.0, 160.0), 0.0),
            (120.0, 50.0, 1200.0, 800.0)
        );
    }

    #[test]
    fn an_inset_keeps_the_bottom_clear() {
        // A bar over the bottom 30 %: 630 px left, 3x = 720x480 centred in them.
        let (x, y, w, h) = viewport((1440.0, 900.0), (240.0, 160.0), 0.3);
        assert_eq!((x, y, w, h), (360.0, 75.0, 720.0, 480.0));
        assert!(y + h <= 900.0 * 0.7);
        // Out-of-range insets are clamped, never a negative height.
        let (_, _, _, h) = viewport((1440.0, 900.0), (240.0, 160.0), 5.0);
        assert!(h > 0.0);
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::{EffectSpec, Painter};
    use emudome_core::effects::BUILTIN;
    use emudome_core::Frame;

    const GAME: (u32, u32) = (240, 160);
    const TARGET: (u32, u32) = (960, 640);
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    /// A GBA-sized picture with flat colours, edges and diagonals, like a game's.
    fn picture() -> Vec<u8> {
        let mut rgba = Vec::new();
        for y in 0..GAME.1 {
            for x in 0..GAME.0 {
                let band = (x / 30) as u8;
                let check = ((x / 8 + y / 8) % 2) as u8;
                let diag = (x + y) % 40 < 20;
                let (r, g, b) = if y < 60 {
                    (band * 36, 255 - band * 30, 80 + check * 100)
                } else if diag {
                    (240, 200, 40)
                } else {
                    (30, 60 + (y as u8 / 2), 160)
                };
                rgba.extend_from_slice(&[r, g, b, 255]);
            }
        }
        rgba
    }

    fn painter() -> Painter {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .expect("a GPU");
        eprintln!("GPU: {:?}", adapter.get_info());
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("emudome test"),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .expect("a device");
        Painter::new(device, queue, FORMAT)
    }

    /// Paints one frame on a texture of `TARGET` and reads it back as RGBA.
    fn paint(painter: &mut Painter) -> Vec<u8> {
        let texture = painter.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test target"),
            size: wgpu::Extent3d {
                width: TARGET.0,
                height: TARGET.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        painter.paint(&view, TARGET, 0.0);
        let row = TARGET.0 * 4; // 3840, a multiple of 256
        let buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(row * TARGET.1),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = painter.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(TARGET.1),
                },
            },
            texture.size(),
        );
        painter.queue.submit([encoder.finish()]);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.expect("mapped"));
        painter
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("GPU done");
        let bytes = buffer
            .slice(..)
            .get_mapped_range()
            .expect("mapped range")
            .to_vec();
        bytes
    }

    fn dump(name: &str, rgba: Vec<u8>) {
        let Some(dir) = std::env::var_os("EMUDOME_FX_DUMP") else {
            return;
        };
        let frame = Frame {
            width: TARGET.0,
            height: TARGET.1,
            rgba: rgba.into(),
            seq: 0,
            split: emudome_core::screens::Split::Whole,
        };
        let path = std::path::Path::new(&dir).join(format!("{name}.png"));
        std::fs::write(&path, frame.to_png().expect("png")).expect("written");
        eprintln!("wrote {}", path.display());
    }

    /// Every built-in effect loads and changes the picture, strength and brightness work, and
    /// a preset that does not exist is reported instead of drawn. Offscreen: no window.
    /// `EMUDOME_FX_DUMP=<folder>` also writes one PNG per effect.
    #[test]
    #[ignore = "needs a GPU: cargo test -p emudome --lib gpu_tests -- --ignored"]
    fn built_in_effects_draw_offscreen() {
        let base = std::env::temp_dir().join(format!("emudome-fx-gpu-{}", std::process::id()));
        let dir = crate::effects::write_builtins(&base).expect("presets written");
        let mut painter = painter();
        painter.upload(GAME.0, GAME.1, &picture());

        painter.sync_effect(None);
        let plain = paint(&mut painter);
        dump("none", plain.clone());

        for id in BUILTIN {
            let spec = EffectSpec {
                preset: dir.join(format!("{id}.slangp")),
                intensity: 1.0,
                brightness: 1.0,
            };
            let started = std::time::Instant::now();
            painter.sync_effect(Some(&spec));
            assert!(
                painter.effect_error.is_none(),
                "{id}: {:?}",
                painter.effect_error
            );
            assert!(painter.effect.is_some(), "{id} loaded");
            eprintln!("{id}: loaded in {} ms", started.elapsed().as_millis());
            paint(&mut painter);
            let out = paint(&mut painter);
            assert!(
                painter.effect_error.is_none(),
                "{id}: {:?}",
                painter.effect_error
            );
            assert!(painter.effect.is_some(), "{id} still drawn");
            assert_ne!(out, plain, "{id} changes the picture");
            dump(id, out);
        }

        // Strength 0.5 sits between the plain picture and the full effect; brightness lifts it.
        let spec = |intensity, brightness| EffectSpec {
            preset: dir.join("lcd_grid.slangp"),
            intensity,
            brightness,
        };
        let sum = |px: &[u8]| px.iter().map(|&b| u64::from(b)).sum::<u64>();
        painter.sync_effect(Some(&spec(1.0, 1.0)));
        let full = sum(&paint(&mut painter));
        painter.sync_effect(Some(&spec(0.5, 1.0)));
        let half = paint(&mut painter);
        dump("lcd_grid_half", half.clone());
        let half = sum(&half);
        let plain_sum = sum(&plain);
        assert!(half < plain_sum.max(full) && half > plain_sum.min(full));
        painter.sync_effect(Some(&spec(1.0, 1.5)));
        assert!(sum(&paint(&mut painter)) > full, "brighter");

        // A preset that is not there: reported once, the plain picture drawn.
        let missing = EffectSpec {
            preset: dir.join("missing.slangp"),
            intensity: 1.0,
            brightness: 1.0,
        };
        painter.sync_effect(Some(&missing));
        assert!(painter.effect_error.take().is_some());
        assert!(painter.effect.is_none());
        assert_eq!(paint(&mut painter), plain);
        painter.sync_effect(Some(&missing));
        assert!(painter.effect_error.is_none(), "not tried again");

        let _ = std::fs::remove_dir_all(&base);
    }
}
