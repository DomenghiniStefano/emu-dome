//! Hardware rendering, the GPU side: a game whose core draws with Vulkan (3DS, Azahar) on the
//! app's one device ([`crate::video::GpuService`]). The libretro side is
//! `emudome_core::hw_render::vulkan`; everything under wgpu is `gpu_interop.rs`.
//!
//! [`video_setup`] gives a session the core's handler (the shell's device in the interface
//! table) and [`GpuFrameSink`], which takes each frame the core signals:
//! - the core's image is copied on the GPU into a texture EmuDome owns (the game texture), right
//!   after the `retro_run` that drew it ([`CoreImageCopier`]): the core renders its next frame
//!   into the same image while the renderer may still be drawing;
//! - the game texture is read back into `Shared::frame` ([`Readback`]): the webview path, the
//!   screenshots and the slot thumbnails read it, and the native renderer uploads it as it does
//!   a software frame, until it samples the game texture itself.

use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use emudome_core::hw_render::vulkan::{QueueLock, VulkanHandler};
use emudome_core::screens::Split;
use emudome_core::video_sink::{GpuFrame, SharedFrameSink, VideoFrame, VideoSink};
use emudome_core::{Shared, VideoSetup};

use crate::gpu_interop::{
    core_device, core_image_barriers, core_image_texture, wait_core_semaphores, wgpu_format,
};
use crate::video::{GpuService, VulkanLease};

/// The session's video for a core that renders with Vulkan on the app's device.
pub fn video_setup(lease: VulkanLease) -> VideoSetup {
    let handler = VulkanHandler::new(core_device(&lease.handles, lease.queue_lock.clone()));
    VideoSetup {
        sink: Some(Box::new(move |shared: &Arc<Shared>| {
            Box::new(GpuFrameSink::new(lease, shared.clone())) as Box<dyn VideoSink>
        })),
        hw_render: Some(Box::new(handler)),
    }
}

/// For the headless examples (`examples/headless_gpu.rs`): the app's GPU service with no window,
/// and the video setup of each session that asks (`None` when there is no Vulkan GPU).
pub fn headless_setup() -> impl Fn() -> Option<VideoSetup> {
    let gpu = GpuService::new();
    move || match gpu.vulkan_lease() {
        Ok(lease) => {
            log::info!(
                "hw render: Vulkan device {:?}, API {:#x}",
                lease.handles.device,
                lease.handles.api_version
            );
            Some(video_setup(lease))
        }
        Err(e) => {
            log::warn!("hw render: no Vulkan device ({e:#}): software");
            None
        }
    }
}

/// The game texture's format: the bytes as the core wrote them (sRGB-encoded), what the
/// renderer and librashader expect (`video.rs`'s `RAW_FORMAT`), and what `Shared::frame` holds.
pub const GAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The wgpu format of the core's image (a raw `VkFormat`), and the view of the game texture to
/// draw it into so the bytes stay as the core wrote them: an sRGB image is decoded when sampled,
/// so it is encoded again through an sRGB view.
pub fn formats(vk_format: u32) -> Result<(wgpu::TextureFormat, wgpu::TextureFormat)> {
    let Some(format) = wgpu_format(vk_format) else {
        bail!("core image format {vk_format} is not supported");
    };
    let view = if format.is_srgb() {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        GAME_FORMAT
    };
    Ok((format, view))
}

/// Takes a hardware-rendering core's frames: the GPU copy, then the readback into
/// `Shared::frame`. Software frames (a core that switched renderer) go to `Shared::frame` as
/// usual.
pub struct GpuFrameSink {
    lease: VulkanLease,
    copier: CoreImageCopier,
    readback: Readback,
    /// The game texture, remade when the size changes.
    game: Option<(wgpu::Texture, (u32, u32))>,
    frames: SharedFrameSink,
    /// The last failure, logged once until another one.
    failed: Option<String>,
}

impl GpuFrameSink {
    pub fn new(lease: VulkanLease, shared: Arc<Shared>) -> Self {
        let copier = CoreImageCopier::new(
            lease.device.clone(),
            lease.queue.clone(),
            lease.queue_lock.clone(),
        );
        Self {
            lease,
            copier,
            readback: Readback::default(),
            game: None,
            frames: SharedFrameSink::new(shared),
            failed: None,
        }
    }

    fn game_texture(&mut self, size: (u32, u32)) -> wgpu::Texture {
        if let Some((t, s)) = &self.game {
            if *s == size {
                return t.clone();
            }
        }
        let texture = self.lease.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("game (hardware)"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: GAME_FORMAT,
            // COPY_SRC: the readback, and effects that keep past frames.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });
        self.game = Some((texture.clone(), size));
        texture
    }

    /// The frame copied into the game texture and read back into `Shared::frame`.
    fn gpu_frame(&mut self, frame: &GpuFrame, split: Split) -> Result<()> {
        let size = (frame.width, frame.height);
        anyhow::ensure!(size.0 > 0 && size.1 > 0, "empty frame");
        let target = self.game_texture(size);
        let device = self.lease.device.clone();
        let readback = &mut self.readback;
        // SAFETY: the image is the core's current one (`set_image` of this `retro_run`), alive
        // until its next `retro_run`; its layout and queue family were checked by the handler.
        let submitted = unsafe {
            self.copier.copy(frame, &target, |enc| {
                readback.record(&device, enc, &target, size);
            })?
        };
        let back = self.frames.back_buffer();
        self.readback.finish(&device, submitted, back)?;
        self.frames.publish(size.0, size.1, split);
        Ok(())
    }
}

impl VideoSink for GpuFrameSink {
    fn frame(&mut self, frame: VideoFrame<'_>, split: Split) {
        match frame {
            VideoFrame::Cpu(_) => self.frames.frame(frame, split),
            VideoFrame::Gpu(gpu) => {
                if let Err(e) = self.gpu_frame(&gpu, split) {
                    let text = format!("{e:#}");
                    if self.failed.as_ref() != Some(&text) {
                        log::error!("hw render: frame {}x{} lost: {text}", gpu.width, gpu.height);
                    }
                    self.failed = Some(text);
                } else {
                    self.failed = None;
                }
            }
        }
    }

    fn release(&mut self) {
        self.copier.release();
    }
}

const BLIT: &str = r#"
struct Out { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> Out {
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
    return vec4<f32>(textureSample(tex, samp, in.uv).rgb, 1.0);
}
"#;

/// The core's image as a wgpu texture (no copy, not owned: the core destroys it), for one frame.
struct Wrapped {
    target_view_format: wgpu::TextureFormat,
    bind_group: wgpu::BindGroup,
    _texture: wgpu::Texture,
}

/// Copies the core's image into the game texture, on the GPU, on the queue the core submits to.
///
/// The copy is a draw (not a transfer), so the core's image stays in
/// `SHADER_READ_ONLY_OPTIMAL`, the layout it hands it in and expects back. In one submit, under
/// the queue lock:
/// 1. (only if `set_image` gave semaphores) an empty submit waiting for them;
/// 2. a raw barrier making the core's colour writes visible to the fragment shader (its render
///    pass ends without an outgoing dependency, and wgpu sees read-after-read and inserts
///    nothing: measured in the spike);
/// 3. the draw into the game texture (and what the caller adds: the readback's copy);
/// 4. a raw barrier ordering every later command (the core's next frame) after the draw's reads,
///    so the core can overwrite its image.
pub struct CoreImageCopier {
    device: wgpu::Device,
    queue: wgpu::Queue,
    queue_lock: Arc<QueueLock>,
    layout: wgpu::BindGroupLayout,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    /// One pipeline per view format drawn into (plain or sRGB).
    pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
    sampler: wgpu::Sampler,
}

impl CoreImageCopier {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, queue_lock: Arc<QueueLock>) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("core image copy"),
            source: wgpu::ShaderSource::Wgsl(BLIT.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("core image copy"),
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
            label: Some("core image copy"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        // Same size in and out: nearest copies each texel as it is.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("core image copy"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            device,
            queue,
            queue_lock,
            layout,
            shader,
            pipeline_layout,
            pipelines: Vec::new(),
            sampler,
        }
    }

    fn pipeline(&mut self, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        if let Some((_, p)) = self.pipelines.iter().find(|(f, _)| *f == format) {
            return p.clone();
        }
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("core image copy"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
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
        self.pipelines.push((format, pipeline.clone()));
        pipeline
    }

    /// Wraps the core's image for this frame. Never kept for the next one: the core may destroy
    /// its image and make another, possibly with the same handle (Azahar does on every state
    /// load), so a wrapper (and its view) is never reused.
    ///
    /// # Safety
    /// `frame`'s image is the core's current one, alive until its next `retro_run`, at least
    /// `frame.width` × `frame.height`.
    unsafe fn wrap(&self, frame: &GpuFrame) -> Result<Wrapped> {
        let size = (frame.width, frame.height);
        let (format, target_view_format) = formats(frame.image.format)?;
        let texture = core_image_texture(
            &self.device,
            frame.image.handle,
            &wgpu::TextureDescriptor {
                label: Some("core image"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        )
        .ok_or_else(|| anyhow!("not a Vulkan device"))?;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("core image"),
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
        Ok(Wrapped {
            target_view_format,
            bind_group,
            _texture: texture,
        })
    }

    /// Copies `frame`'s image into `target` (its size, [`GAME_FORMAT`] with an sRGB view format)
    /// and submits, with the barriers of the type's notes. `then` records more work into the
    /// same submit (the readback), after the copy.
    ///
    /// # Safety
    /// As [`Self::wrap`].
    pub unsafe fn copy(
        &mut self,
        frame: &GpuFrame,
        target: &wgpu::Texture,
        then: impl FnOnce(&mut wgpu::CommandEncoder),
    ) -> Result<wgpu::SubmissionIndex> {
        // Dropped at the end of this call: wgpu keeps what the submit uses until it is done.
        let wrapped = self.wrap(frame)?;
        let (bind_group, view_format) = (wrapped.bind_group.clone(), wrapped.target_view_format);
        let pipeline = self.pipeline(view_format);
        let view = target.create_view(&wgpu::TextureViewDescriptor {
            format: Some(view_format),
            ..Default::default()
        });

        let (before, after) = core_image_barriers(&self.device, frame.image.handle)
            .context("barriers around the copy")?;
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("core image copy"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("core image copy"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        then(&mut enc);

        let _queue = self.queue_lock.lock();
        if !frame.sync.wait_semaphores.is_empty() {
            wait_core_semaphores(&self.device, &frame.sync.wait_semaphores)?;
        }
        Ok(self.queue.submit([before, enc.finish(), after]))
    }

    /// Waits until the GPU no longer reads the core's image (before the core destroys it).
    pub fn release(&mut self) {
        if let Err(e) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            log::warn!("hw render: poll at release: {e:?}");
        }
    }
}

/// The game texture back to the CPU, as tightly packed RGBA8 rows. Synchronous: it waits for
/// the GPU (measured cost in `docs/fase0-esito.md`, 3DS entry).
#[derive(Default)]
pub struct Readback {
    buffer: Option<(wgpu::Buffer, (u32, u32), u32)>,
}

/// Bytes per row in the buffer: wgpu wants rows aligned to 256 bytes.
pub fn padded_row(width: u32) -> u32 {
    (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

/// Copies the padded rows of `data` (`padded` bytes each) into `out` as `width` × 4 bytes each.
pub fn unpad(data: &[u8], width: u32, padded: u32, out: &mut Vec<u8>) {
    out.clear();
    for row in data.chunks(padded as usize) {
        out.extend_from_slice(&row[..(width * 4) as usize]);
    }
}

impl Readback {
    /// Records the copy of `texture` (`size`) into the buffer; [`Self::finish`] reads it once the
    /// encoder's submit is done.
    pub fn record(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        size: (u32, u32),
    ) {
        if self.buffer.as_ref().map(|b| b.1) != Some(size) {
            let row = padded_row(size.0);
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("frame readback"),
                size: u64::from(row) * u64::from(size.1),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            self.buffer = Some((buffer, size, row));
        }
        let Some((buffer, _, row)) = &self.buffer else {
            return;
        };
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(*row),
                    rows_per_image: Some(size.1),
                },
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Waits for `submitted` and copies the recorded frame into `out`.
    pub fn finish(
        &self,
        device: &wgpu::Device,
        submitted: wgpu::SubmissionIndex,
        out: &mut Vec<u8>,
    ) -> Result<()> {
        let (buffer, size, row) = self.buffer.as_ref().context("nothing recorded")?;
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submitted),
                timeout: Some(std::time::Duration::from_secs(5)),
            })
            .map_err(|e| anyhow!("poll: {e:?}"))?;
        rx.recv()
            .context("map callback")?
            .map_err(|e| anyhow!("map: {e:?}"))?;
        {
            let data = slice
                .get_mapped_range()
                .map_err(|e| anyhow!("mapped range: {e:?}"))?;
            unpad(&data, size.0, *row, out);
        }
        buffer.unmap();
        Ok(())
    }
}

/// Headless GPU checks (no window): `cargo test -p emudome --lib gpu -- --ignored`.
#[cfg(test)]
mod gpu_tests {
    use std::sync::Arc;

    use emudome_core::screens::Split;
    use emudome_core::video_sink::{GpuFrame, GpuImage, GpuSync, VideoFrame, VideoSink};
    use emudome_core::Shared;

    use super::GpuFrameSink;
    use crate::gpu_interop::gpu_tests::painted_core_image;
    use crate::video::GpuService;

    /// A core's image (cleared to a colour, as a core leaves it) reaches `Shared::frame` through
    /// the GPU copy and the readback, on the app's device opened for cores; a new image each
    /// frame (a core that made another, as Azahar does on a state load) is followed.
    #[test]
    #[ignore = "needs a Vulkan GPU"]
    fn a_core_image_reaches_the_shared_frame() {
        let gpu = GpuService::new();
        let lease = match gpu.vulkan_lease() {
            Ok(lease) => lease,
            Err(e) => {
                eprintln!("no Vulkan device ({e:#}): skipped");
                return;
            }
        };
        let (device, queue) = (lease.device.clone(), lease.queue.clone());
        let shared = Arc::new(Shared::default());
        let mut sink = GpuFrameSink::new(lease, shared.clone());
        let size = (400, 480);
        for (i, colour) in [[1.0, 0.0, 0.0, 1.0], [0.0, 0.5, 1.0, 1.0]]
            .into_iter()
            .enumerate()
        {
            let (image, destroy) = painted_core_image(&device, &queue, size, colour);
            let started = std::time::Instant::now();
            sink.frame(
                VideoFrame::Gpu(GpuFrame {
                    width: size.0,
                    height: size.1,
                    // VK_FORMAT_R8G8B8A8_UNORM, SHADER_READ_ONLY_OPTIMAL.
                    image: GpuImage {
                        handle: image,
                        format: 37,
                        layout: 5,
                    },
                    sync: GpuSync::default(),
                }),
                Split::Whole,
            );
            eprintln!(
                "frame {i}: copy + readback {} µs",
                started.elapsed().as_micros()
            );
            {
                let f = shared.frame.lock();
                assert_eq!((f.width, f.height, f.seq), (size.0, size.1, i as u64 + 1));
                let want = colour.map(|c| (c * 255.0).round() as u8);
                let at = |x: u32, y: u32| {
                    let o = ((y * f.width + x) * 4) as usize;
                    [f.rgba[o], f.rgba[o + 1], f.rgba[o + 2], f.rgba[o + 3]]
                };
                for (x, y) in [(0, 0), (399, 479), (200, 240)] {
                    let got = at(x, y);
                    assert!(
                        got.iter().zip(want).all(|(g, w)| g.abs_diff(w) <= 1),
                        "frame {i} pixel ({x}, {y}): {got:?}, want {want:?}"
                    );
                }
            }
            sink.release();
            destroy();
        }
    }
}

#[cfg(test)]
mod tests {
    use wgpu::TextureFormat as T;

    use super::{formats, padded_row, unpad};

    #[test]
    fn srgb_images_are_drawn_through_an_srgb_view() {
        // VK_FORMAT_R8G8B8A8_UNORM = 37, B8G8R8A8_UNORM = 44, B8G8R8A8_SRGB = 50,
        // R16G16B16A16_SFLOAT = 97.
        assert_eq!(formats(37).ok(), Some((T::Rgba8Unorm, T::Rgba8Unorm)));
        assert_eq!(formats(44).ok(), Some((T::Bgra8Unorm, T::Rgba8Unorm)));
        assert_eq!(
            formats(50).ok(),
            Some((T::Bgra8UnormSrgb, T::Rgba8UnormSrgb))
        );
        assert!(formats(97).is_err());
    }

    #[test]
    fn rows_are_padded_to_256_bytes_and_back() {
        assert_eq!(padded_row(400), 1792);
        assert_eq!(padded_row(64), 256);
        assert_eq!(padded_row(65), 512);
        let (width, padded) = (3u32, 256u32);
        let data: Vec<u8> = (0..2 * padded).map(|i| (i % 256) as u8).collect();
        let mut out = vec![9];
        unpad(&data, width, padded, &mut out);
        assert_eq!(out.len(), 2 * 12);
        assert_eq!(&out[..12], &data[..12]);
        assert_eq!(&out[12..], &data[256..268]);
    }
}
