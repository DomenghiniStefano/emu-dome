//! Native renderer: draws the core's latest frame onto the window with wgpu,
//! under the interface's transparent webview. Runs on a thread of its own.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use anyhow::{Context, Result};
use emudome_core::Shared;

enum Msg {
    Resize(u32, u32),
    Source(Option<Arc<Shared>>),
}

#[derive(Clone)]
pub struct Renderer {
    tx: Sender<Msg>,
}

impl Renderer {
    /// Must be called on the main thread: on Windows and Linux the window's native
    /// handle is only available there (from another thread Tauri blocks waiting for it).
    /// Drawing then continues on a dedicated thread.
    pub fn spawn(window: tauri::Window, width: u32, height: u32) -> Result<Self> {
        let gpu = Gpu::new(window, width, height)?;
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("emudome-video".into())
            .spawn(move || gpu.run(rx))?;
        Ok(Self { tx })
    }

    pub fn resize(&self, width: u32, height: u32) {
        let _ = self.tx.send(Msg::Resize(width, height));
    }

    pub fn set_source(&self, shared: Option<Arc<Shared>>) {
        let _ = self.tx.send(Msg::Source(shared));
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
    source: Option<Arc<Shared>>,
    last_seq: u64,
}

impl Gpu {
    fn new(window: tauri::Window, width: u32, height: u32) -> Result<Self> {
        // No display handle: Windows does not need it, and on Linux Tauri's one
        // does not match the surface's (MismatchingDisplayHandle).
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance
            .create_surface(window)
            .map_err(|e| anyhow::anyhow!("wgpu surface not created: {e:?}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
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
                ..Default::default()
            }))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: caps.alpha_modes[0],
            color_space: wgpu::SurfaceColorSpace::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

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
            source: None,
            last_seq: u64::MAX,
        })
    }

    fn run(mut self, rx: Receiver<Msg>) {
        loop {
            // Incoming messages; with no game, wait instead of spinning.
            let mut msgs: Vec<Msg> = rx.try_iter().collect();
            if self.source.is_none() && msgs.is_empty() {
                match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(m) => msgs.push(m),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            for m in msgs {
                match m {
                    Msg::Resize(w, h) if w > 0 && h > 0 => {
                        self.config.width = w;
                        self.config.height = h;
                        self.surface.configure(&self.device, &self.config);
                    }
                    Msg::Resize(..) => {}
                    Msg::Source(s) => {
                        self.source = s;
                        self.game = None;
                        self.last_seq = u64::MAX;
                    }
                }
            }
            self.upload_frame();
            self.draw();
        }
    }

    fn upload_frame(&mut self) {
        let Some(source) = self.source.clone() else {
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

    /// The game's rectangle: integer scale when it fits, otherwise the largest that keeps the aspect ratio.
    fn viewport(&self, w: u32, h: u32) -> (f32, f32, f32, f32) {
        let (sw, sh) = (self.config.width as f32, self.config.height as f32);
        let fit = (sw / w as f32).min(sh / h as f32);
        let scale = if fit >= 1.0 { fit.floor() } else { fit };
        let (vw, vh) = (w as f32 * scale, h as f32 * scale);
        (((sw - vw) / 2.0).floor(), ((sh - vh) / 2.0).floor(), vw, vh)
    }

    fn draw(&mut self) {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            other @ (wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation) => {
                log::debug!("frame skipped: {other:?}");
                std::thread::sleep(std::time::Duration::from_millis(8));
                return;
            }
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
                let (x, y, w, h) = self.viewport(game.width, game.height);
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &game.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(output);
    }
}
