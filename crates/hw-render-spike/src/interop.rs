//! The wgpu side for an OpenGL core: wgpu's own Vulkan device (opened with the external
//! memory and semaphore extensions), and the two ways a GL frame reaches a wgpu draw:
//!
//! - **readback**: GL `glReadPixels` to the CPU, `queue.write_texture` back up to the GPU;
//! - **zero copy**: one Vulkan image whose memory GL imports as its framebuffer texture
//!   (Win32 handles), synchronised with two exported binary semaphores: GL signals `gl_done`
//!   after the core's frame, Vulkan waits on it before drawing and signals `vk_done` after,
//!   GL waits on `vk_done` before the next frame.

use std::ffi::CStr;

use anyhow::{anyhow, bail, Context, Result};
use ash::khr::{external_memory_win32, external_semaphore_win32};
use ash::vk;
use wgpu_hal::api::Vulkan;

use crate::gl::ExportedHandles;

const SHADER: &str = r#"
struct Out { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

// x, y: the share of the texture the frame covers; z: 1 when rows are bottom-up (GL).
@group(0) @binding(2) var<uniform> rect: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> Out {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: Out;
    o.pos = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
    let v = select(1.0 - p.y, p.y, rect.z > 0.5);
    o.uv = vec2<f32>(p.x * rect.x, v * rect.y);
    return o;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv);
}
"#;

/// A texture the draw samples, with its bind group and the size it was made for.
struct Source {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    rect: wgpu::Buffer,
    width: u32,
    height: u32,
}

/// The Vulkan image shared with GL.
struct Shared {
    image: vk::Image,
    memory: vk::DeviceMemory,
    gl_done: vk::Semaphore,
    vk_done: vk::Semaphore,
    source: Source,
}

struct Target {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    padded_row: u32,
}

pub struct VkSide {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
    raw_instance: ash::Instance,
    raw_device: ash::Device,
    raw_queue: vk::Queue,
    memory_types: vk::PhysicalDeviceMemoryProperties,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    upload: Option<Source>,
    shared: Option<Shared>,
    target: Option<Target>,
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
}

impl VkSide {
    /// # Safety
    /// Creates raw Vulkan objects through wgpu-hal.
    pub unsafe fn new() -> Result<Self> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .context("no Vulkan adapter")?;
        let adapter_info = adapter.get_info();
        let features = wgpu::Features::empty();
        let limits = adapter.limits();

        let hal_instance = instance
            .as_hal::<Vulkan>()
            .ok_or_else(|| anyhow!("not a Vulkan instance"))?;
        let raw_instance = hal_instance.shared_instance().raw_instance().clone();
        let hal_adapter = adapter
            .as_hal::<Vulkan>()
            .ok_or_else(|| anyhow!("not a Vulkan adapter"))?;
        let gpu = hal_adapter.raw_physical_device();
        let available: Vec<String> = raw_instance
            .enumerate_device_extension_properties(gpu)?
            .iter()
            .filter_map(|e| e.extension_name_as_c_str().ok())
            .map(|c| c.to_string_lossy().into_owned())
            .collect();
        let wanted: [&'static CStr; 2] =
            [external_memory_win32::NAME, external_semaphore_win32::NAME];
        for e in wanted {
            if !available.iter().any(|a| a.as_bytes() == e.to_bytes()) {
                bail!(
                    "the GPU lacks {}: the spike implements Win32 handles only (Linux would use the _fd extensions)",
                    e.to_string_lossy()
                );
            }
        }
        // wgpu 30 opens the device with extra extensions through `open_with_callback`.
        let open = hal_adapter
            .open_with_callback(
                features,
                &limits,
                &wgpu::MemoryHints::Performance,
                Some(Box::new(move |args| {
                    for e in wanted {
                        if !args.extensions.contains(&e) {
                            args.extensions.push(e);
                        }
                    }
                })),
            )
            .map_err(|e| anyhow!("wgpu device: {e:?}"))?;
        let memory_types = raw_instance.get_physical_device_memory_properties(gpu);
        drop(hal_adapter);
        let (device, queue) = adapter
            .create_device_from_hal::<Vulkan>(
                open,
                &wgpu::DeviceDescriptor {
                    label: Some("hw-render-spike gl"),
                    required_features: features,
                    required_limits: limits,
                    ..Default::default()
                },
            )
            .context("wgpu device from hal")?;
        let (raw_device, raw_queue) = {
            let d = device
                .as_hal::<Vulkan>()
                .ok_or_else(|| anyhow!("not a Vulkan device"))?;
            (d.raw_device().clone(), d.raw_queue())
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit gl"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit gl"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blit gl"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit gl"),
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
                    format: wgpu::TextureFormat::Rgba8Unorm,
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Ok(Self {
            device,
            queue,
            adapter_info,
            raw_instance,
            raw_device,
            raw_queue,
            memory_types,
            pipeline,
            layout,
            sampler,
            upload: None,
            shared: None,
            target: None,
            _instance: instance,
            _adapter: adapter,
        })
    }

    fn source(&self, texture: wgpu::Texture, width: u32, height: u32) -> Source {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let rect = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("source"),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: rect.as_entire_binding(),
                },
            ],
        });
        Source {
            texture,
            bind_group,
            rect,
            width,
            height,
        }
    }

    /// Creates the Vulkan image GL will draw into (`width` x `height`, RGBA8), with exportable
    /// memory and two exportable semaphores, wraps it for wgpu, and returns the handles for GL.
    ///
    /// # Safety
    /// Raw Vulkan object creation on wgpu's device.
    pub unsafe fn create_shared(&mut self, width: u32, height: u32) -> Result<ExportedHandles> {
        let d = &self.raw_device;
        let handle_type = vk::ExternalMemoryHandleTypeFlags::OPAQUE_WIN32;
        let mut ext_info = vk::ExternalMemoryImageCreateInfo::default().handle_types(handle_type);
        let image = d.create_image(
            &vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(
                    vk::ImageUsageFlags::SAMPLED
                        | vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::TRANSFER_SRC
                        | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .push_next(&mut ext_info),
            None,
        )?;
        let req = d.get_image_memory_requirements(image);
        let type_index = (0..self.memory_types.memory_type_count)
            .find(|&i| {
                req.memory_type_bits & (1 << i) != 0
                    && self.memory_types.memory_types[i as usize]
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .ok_or_else(|| anyhow!("no device-local memory type for the shared image"))?;
        let mut export = vk::ExportMemoryAllocateInfo::default().handle_types(handle_type);
        let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let memory = d.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(req.size)
                .memory_type_index(type_index)
                .push_next(&mut export)
                .push_next(&mut dedicated),
            None,
        )?;
        d.bind_image_memory(image, memory, 0)?;
        let mem_fn = external_memory_win32::Device::new(&self.raw_instance, d);
        let memory_handle = mem_fn.get_memory_win32_handle(
            &vk::MemoryGetWin32HandleInfoKHR::default()
                .memory(memory)
                .handle_type(handle_type),
        )?;

        let sem_fn = external_semaphore_win32::Device::new(&self.raw_instance, d);
        let sem = || -> Result<(vk::Semaphore, isize)> {
            let mut export = vk::ExportSemaphoreCreateInfo::default()
                .handle_types(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32);
            let s = d.create_semaphore(
                &vk::SemaphoreCreateInfo::default().push_next(&mut export),
                None,
            )?;
            let h = sem_fn.get_semaphore_win32_handle(
                &vk::SemaphoreGetWin32HandleInfoKHR::default()
                    .semaphore(s)
                    .handle_type(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32),
            )?;
            Ok((s, h))
        };
        let (gl_done, gl_done_h) = sem()?;
        let (vk_done, vk_done_h) = sem()?;

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let hal_texture = {
            let hal_device = self
                .device
                .as_hal::<Vulkan>()
                .ok_or_else(|| anyhow!("not a Vulkan device"))?;
            hal_device.texture_from_raw(
                image,
                &wgpu_hal::TextureDescriptor {
                    label: Some("shared with GL"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUses::RESOURCE,
                    memory_flags: wgpu_hal::MemoryFlags::empty(),
                    view_formats: vec![],
                },
                // Ours, destroyed in `destroy_shared` after wgpu lets go.
                Some(Box::new(|| {})),
                wgpu_hal::vulkan::TextureMemory::External,
            )
        };
        let texture = self.device.create_texture_from_hal::<Vulkan>(
            hal_texture,
            &wgpu::TextureDescriptor {
                label: Some("shared with GL"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            // GL's semaphore signal leaves it in SHADER_READ_ONLY before every Vulkan read.
            wgpu::TextureUses::RESOURCE,
        );
        let source = self.source(texture, width, height);
        self.shared = Some(Shared {
            image,
            memory,
            gl_done,
            vk_done,
            source,
        });
        Ok(ExportedHandles {
            memory: memory_handle,
            memory_size: req.size,
            gl_done: gl_done_h,
            vk_done: vk_done_h,
        })
    }

    fn ensure_target(&mut self, width: u32, height: u32) {
        if let Some(t) = &self.target {
            if t.width == width && t.height == height {
                return;
            }
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let padded_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.target = Some(Target {
            width,
            height,
            texture,
            view,
            readback,
            padded_row,
        });
    }

    /// Readback path, second half: uploads GL's bottom-up rows and draws them.
    pub fn draw_uploaded(
        &mut self,
        rows: &[u8],
        width: u32,
        height: u32,
        check: bool,
    ) -> Result<Option<Vec<u8>>> {
        let fits = self
            .upload
            .as_ref()
            .is_some_and(|u| u.width == width && u.height == height);
        if !fits {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("upload"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.upload = Some(self.source(texture, width, height));
        }
        let Some(up) = &self.upload else {
            bail!("no upload texture");
        };
        self.queue.write_texture(
            up.texture.as_image_copy(),
            rows,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue
            .write_buffer(&up.rect, 0, &rect_bytes(1.0, 1.0, true));
        self.draw(false, width, height, check)
    }

    /// Zero-copy path: draws the shared image's bottom-left `width` x `height`.
    ///
    /// # Safety
    /// GL has signalled `gl_done` for this frame.
    pub unsafe fn draw_shared(
        &mut self,
        width: u32,
        height: u32,
        check: bool,
    ) -> Result<Option<Vec<u8>>> {
        let Some(sh) = &self.shared else {
            bail!("no shared image");
        };
        let sx = width as f32 / sh.source.width as f32;
        let sy = height as f32 / sh.source.height as f32;
        self.queue
            .write_buffer(&sh.source.rect, 0, &rect_bytes(sx, sy, true));
        self.draw(true, width, height, check)
    }

    fn submit_raw(&self, wait: Option<vk::Semaphore>, signal: Option<vk::Semaphore>) -> Result<()> {
        let waits: Vec<vk::Semaphore> = wait.into_iter().collect();
        let stages = vec![vk::PipelineStageFlags::FRAGMENT_SHADER; waits.len()];
        let signals: Vec<vk::Semaphore> = signal.into_iter().collect();
        let info = vk::SubmitInfo::default()
            .wait_semaphores(&waits)
            .wait_dst_stage_mask(&stages)
            .signal_semaphores(&signals);
        // SAFETY: an empty batch on wgpu's queue, from the only thread that submits to it.
        unsafe {
            self.raw_device
                .queue_submit(self.raw_queue, &[info], vk::Fence::null())
        }?;
        Ok(())
    }

    fn draw(
        &mut self,
        shared: bool,
        width: u32,
        height: u32,
        check: bool,
    ) -> Result<Option<Vec<u8>>> {
        self.ensure_target(width, height);
        let source = if shared {
            self.shared.as_ref().map(|s| &s.source)
        } else {
            self.upload.as_ref()
        };
        let (Some(source), Some(target)) = (source, &self.target) else {
            bail!("no source or target");
        };
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &source.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        if check {
            enc.copy_texture_to_buffer(
                target.texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &target.readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(target.padded_row),
                        rows_per_image: Some(target.height),
                    },
                },
                wgpu::Extent3d {
                    width: target.width,
                    height: target.height,
                    depth_or_array_layers: 1,
                },
            );
        }
        let sems = self.shared.as_ref().map(|s| (s.gl_done, s.vk_done));
        if let (true, Some((gl_done, _))) = (shared, sems) {
            // Everything submitted after this batch waits for GL's frame (Vulkan orders a
            // semaphore wait before all later submissions on the queue).
            self.submit_raw(Some(gl_done), None)?;
        }
        let index = self.queue.submit([enc.finish()]);
        if let (true, Some((_, vk_done))) = (shared, sems) {
            self.submit_raw(None, Some(vk_done))?;
        }
        let slice = target.readback.slice(..);
        let rx = check.then(|| {
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            rx
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            })
            .map_err(|e| anyhow!("poll: {e:?}"))?;
        let Some(rx) = rx else {
            return Ok(None);
        };
        rx.recv()
            .context("map callback")?
            .map_err(|e| anyhow!("map: {e:?}"))?;
        let mut out = Vec::with_capacity((target.width * target.height * 4) as usize);
        {
            let data = slice
                .get_mapped_range()
                .map_err(|e| anyhow!("mapped range: {e:?}"))?;
            for row in data.chunks(target.padded_row as usize) {
                out.extend_from_slice(&row[..(target.width * 4) as usize]);
            }
        }
        target.readback.unmap();
        Ok(Some(out))
    }

    /// Frees the shared image and semaphores, after the GPU is idle.
    pub fn destroy_shared(&mut self) {
        if let Err(e) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            log::warn!("poll at teardown: {e:?}");
        }
        if let Some(sh) = self.shared.take() {
            drop(sh.source);
            if let Err(e) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
                log::warn!("poll at teardown: {e:?}");
            }
            // SAFETY: wgpu dropped its wrapper and the GPU is idle.
            unsafe {
                self.raw_device.destroy_semaphore(sh.gl_done, None);
                self.raw_device.destroy_semaphore(sh.vk_done, None);
                self.raw_device.destroy_image(sh.image, None);
                self.raw_device.free_memory(sh.memory, None);
            }
        }
    }
}

fn rect_bytes(sx: f32, sy: f32, bottom_up: bool) -> [u8; 16] {
    let v = [sx, sy, if bottom_up { 1.0 } else { 0.0 }, 0.0];
    let mut out = [0u8; 16];
    for (i, f) in v.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&f.to_le_bytes());
    }
    out
}
