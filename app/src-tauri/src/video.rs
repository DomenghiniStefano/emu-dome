//! Native renderer: draws the core's latest frame with wgpu under the interface's
//! transparent webview. Runs on a thread of its own.
//!
//! The surface is either Tauri's window (desktop) or one the system hands over later
//! and may take back (Android's `SurfaceView`): [`Renderer::attach`] / [`Renderer::detach`].

use std::collections::BTreeMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use emudome_core::tuning::{PresentMode, VideoTuning};
use emudome_core::Shared;
use parking_lot::Mutex;
use serde::Serialize;

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
    pub fn present_mode_supported(&self) -> bool {
        self.present_mode_requested == self.present_mode_used
    }
}

/// One frame presented on the surface.
#[derive(Debug, Clone, Copy)]
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

enum Msg {
    Resize(u32, u32),
    Source(Option<Arc<Shared>>),
    Attach(wgpu::SurfaceTarget<'static>, u32, u32),
    Detach(Sender<()>),
    /// Share of the surface's height, at the bottom, the game keeps clear of (G09's bar).
    Inset(f32),
}

#[derive(Clone)]
pub struct Renderer {
    tx: Sender<Msg>,
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
    ) -> Result<Self> {
        let gpu = Gpu::new(window.into(), width, height, settings)?;
        Self::start(Some(gpu), settings)
    }

    /// Starts with no surface: it arrives with [`Self::attach`].
    pub fn spawn_detached(settings: SurfaceSettings) -> Result<Self> {
        Self::start(None, settings)
    }

    fn start(gpu: Option<Gpu>, settings: SurfaceSettings) -> Result<Self> {
        let metrics = SharedVideoMetrics::default();
        metrics.lock().gpu = gpu.as_ref().map(|g| g.info.clone());
        let (tx, rx) = mpsc::channel();
        let video = Video::new(gpu, settings, metrics.clone());
        std::thread::Builder::new()
            .name("emudome-video".into())
            .spawn(move || video.run(rx))?;
        Ok(Self { tx, metrics })
    }

    /// What the video thread measures (GPU in use, surface errors, presents in bench mode).
    pub fn metrics(&self) -> &SharedVideoMetrics {
        &self.metrics
    }

    pub fn resize(&self, width: u32, height: u32) {
        let _ = self.tx.send(Msg::Resize(width, height));
    }

    pub fn set_source(&self, shared: Option<Arc<Shared>>) {
        let _ = self.tx.send(Msg::Source(shared));
    }

    /// A new surface to draw on (it replaces the current one, if any).
    pub fn attach(&self, target: wgpu::SurfaceTarget<'static>, width: u32, height: u32) {
        let _ = self.tx.send(Msg::Attach(target, width, height));
    }

    /// Keeps the bottom `fraction` of the surface clear: the game shrinks into the rest
    /// (quick menu G09, whose bar sits at the bottom). 0 = the whole surface.
    pub fn set_inset(&self, fraction: f32) {
        let _ = self.tx.send(Msg::Inset(fraction));
    }

    /// Lets go of the surface; returns once it is no longer used.
    pub fn detach(&self) {
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.tx.send(Msg::Detach(ack_tx)).is_ok() {
            let _ = ack_rx.recv_timeout(Duration::from_secs(2));
        }
    }
}

/// How often the video thread logs the frame rates (to measure FPS from the logs too).
const FPS_LOG_EVERY: Duration = Duration::from_secs(10);

/// The video thread: the GPU (while there is a surface) and the game it shows.
struct Video {
    gpu: Option<Gpu>,
    settings: SurfaceSettings,
    metrics: SharedVideoMetrics,
    source: Option<Arc<Shared>>,
    presented: u32,
    since: Instant,
    /// See [`Renderer::set_inset`]; kept across surfaces.
    inset: f32,
}

impl Video {
    fn new(gpu: Option<Gpu>, settings: SurfaceSettings, metrics: SharedVideoMetrics) -> Self {
        Self {
            gpu,
            settings,
            metrics,
            source: None,
            presented: 0,
            since: Instant::now(),
            inset: 0.0,
        }
    }

    fn run(mut self, rx: Receiver<Msg>) {
        loop {
            // Incoming messages; with nothing to draw, wait instead of spinning.
            let mut msgs: Vec<Msg> = rx.try_iter().collect();
            if (self.source.is_none() || self.gpu.is_none()) && msgs.is_empty() {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(m) => msgs.push(m),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            for m in msgs {
                match m {
                    Msg::Resize(w, h) => {
                        if let Some(gpu) = &mut self.gpu {
                            gpu.resize(w, h);
                        }
                    }
                    Msg::Source(s) => {
                        self.source = s;
                        if let Some(gpu) = &mut self.gpu {
                            gpu.reset_game();
                        }
                    }
                    Msg::Attach(target, w, h) => {
                        self.gpu = None;
                        match Gpu::new(target, w, h, self.settings) {
                            Ok(gpu) => {
                                let mut m = self.metrics.lock();
                                m.gpu = Some(gpu.info.clone());
                                m.surface_error = None;
                                self.gpu = Some(gpu);
                            }
                            Err(e) => {
                                log::error!("native video: surface not usable: {e:#}");
                                self.metrics.lock().surface_error = Some(format!("{e:#}"));
                            }
                        }
                    }
                    Msg::Detach(ack) => {
                        self.gpu = None;
                        let _ = ack.send(());
                    }
                    Msg::Inset(fraction) => self.inset = fraction,
                }
            }
            if let Some(gpu) = &mut self.gpu {
                gpu.upload_frame(self.source.as_deref());
                match gpu.draw(self.inset) {
                    Ok(()) => {
                        self.presented += 1;
                        let at = Instant::now();
                        let mut m = self.metrics.lock();
                        if gpu.game.is_some() && m.first_game_present.is_none() {
                            m.first_game_present = Some(at);
                        }
                        if let Some(presents) = &mut m.presents {
                            presents.push(Present {
                                at,
                                seq: gpu.last_seq,
                            });
                        }
                    }
                    Err(kind) => *self.metrics.lock().surface_errors.entry(kind).or_default() += 1,
                }
            }
            self.log_fps();
        }
    }

    fn log_fps(&mut self) {
        let elapsed = self.since.elapsed();
        if elapsed < FPS_LOG_EVERY {
            return;
        }
        if let Some(source) = &self.source {
            let stats = source.stats.lock();
            log::info!(
                "fps: core {:.2}, presented {:.2} (target {:.2})",
                stats.emu_fps,
                f64::from(self.presented) / elapsed.as_secs_f64(),
                stats.av_info.timing.fps,
            );
        }
        self.presented = 0;
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

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv);
}
"#;

struct GameTexture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    game: Option<GameTexture>,
    last_seq: u64,
    info: GpuInfo,
}

impl Gpu {
    fn new(
        target: wgpu::SurfaceTarget<'static>,
        width: u32,
        height: u32,
        settings: SurfaceSettings,
    ) -> Result<Self> {
        // No display handle: Windows and Android do not need it, and on Linux Tauri's
        // one does not match the surface's (MismatchingDisplayHandle).
        // The environment (WGPU_BACKEND, WGPU_DX12_PRESENTATION_SYSTEM...) picks the backend.
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance
            .create_surface(target)
            .map_err(|e| anyhow::anyhow!("wgpu surface not created: {e:?}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .context("no compatible GPU")?;
        let adapter_info = adapter.get_info();
        log::info!("GPU: {adapter_info:?}");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("emudome"),
                // Mobile GPUs do not reach the desktop defaults.
                required_limits:
                    wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            }))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
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
            alpha_mode: caps.alpha_modes[0],
            color_space: wgpu::SurfaceColorSpace::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("game"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
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
        // Sharp pixels: the "smooth" filter and the shaders come with librashader.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            layout,
            sampler,
            game: None,
            last_seq: u64::MAX,
            info,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
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
        let frame = source.frame.lock();
        if frame.seq == self.last_seq || frame.rgba.is_empty() {
            return;
        }
        self.last_seq = frame.seq;
        let (w, h) = (frame.width, frame.height);
        if self.game.as_ref().map(|g| (g.width, g.height)) != Some((w, h)) {
            self.game = Some(self.create_texture(w, h));
        }
        let game = self.game.as_ref().expect("texture just created");
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &game.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &frame.rgba,
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
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
            bind_group,
        }
    }

    fn viewport(&self, w: u32, h: u32, inset: f32) -> (f32, f32, f32, f32) {
        viewport(
            (self.config.width as f32, self.config.height as f32),
            (w as f32, h as f32),
            inset,
        )
    }

    /// Draws and presents; when no frame was presented, the kind of surface error.
    fn draw(&mut self, inset: f32) -> Result<(), &'static str> {
        let skipped = |kind: &'static str| {
            log::debug!("frame skipped: {kind}");
            std::thread::sleep(Duration::from_millis(8));
            Err(kind)
        };
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Err("outdated");
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Err("lost");
            }
            wgpu::CurrentSurfaceTexture::Timeout => return skipped("timeout"),
            wgpu::CurrentSurfaceTexture::Occluded => return skipped("occluded"),
            wgpu::CurrentSurfaceTexture::Validation => return skipped("validation"),
        };
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let clear = if self.game.is_some() {
                wgpu::Color::BLACK
            } else {
                BG
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("game"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                let (x, y, w, h) = self.viewport(game.width, game.height, inset);
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &game.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(output);
        Ok(())
    }
}

/// The game's rectangle `(x, y, w, h)` on a `surface` of `(width, height)`: integer scale when it
/// fits, otherwise the largest that keeps the aspect ratio, centred in the part of the surface
/// above the bottom `inset` share (0..0.9) that the quick menu's bar keeps clear.
fn viewport(surface: (f32, f32), game: (f32, f32), inset: f32) -> (f32, f32, f32, f32) {
    let (sw, full_h) = surface;
    let (w, h) = game;
    let sh = full_h * (1.0 - inset.clamp(0.0, 0.9));
    let fit = (sw / w).min(sh / h);
    let scale = if fit >= 1.0 { fit.floor() } else { fit };
    let (vw, vh) = (w * scale, h * scale);
    (((sw - vw) / 2.0).floor(), ((sh - vh) / 2.0).floor(), vw, vh)
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
