//! The wgpu side: a Vulkan device shared with the core, the core's image wrapped as a wgpu
//! texture (no copy), drawn by a wgpu render pass into our own target, and optionally read
//! back to the CPU to check the pixels and measure the readback.

use std::ffi::c_void;
use std::os::raw::c_char;

use anyhow::{anyhow, bail, Context, Result};
use ash::vk;
use wgpu_hal::api::Vulkan;

use crate::host::{self, CoreImage, QUEUE_LOCK};
use crate::vk_iface::{
    RetroHwRenderInterfaceVulkan, RetroNegotiationVulkanV1, RetroNegotiationVulkanV2,
    RetroVulkanContext, RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION,
};

const SHADER: &str = r#"
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
    return textureSample(tex, samp, in.uv);
}
"#;

/// How the Vulkan device came to be: the core's `create_device`, or wgpu's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceOrigin {
    CoreCreateDevice,
    Wgpu,
}

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
    pub origin: DeviceOrigin,
    /// The queue family shared with the core.
    family: u32,
    /// Extensions the frontend required from the core's device (wgpu's list).
    required_extensions: Vec<String>,
    raw_device: ash::Device,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    wrapped: Option<Wrapped>,
    target: Option<Target>,
    // Kept alive: the core holds pointers to them for the whole session.
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
}

struct Wrapped {
    image: vk::Image,
    width: u32,
    height: u32,
    bind_group: wgpu::BindGroup,
    _texture: wgpu::Texture,
}

struct Target {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    padded_row: u32,
}

/// Maps the core's image format to wgpu's (the formats a libretro core is expected to use).
fn map_format(f: vk::Format) -> Result<wgpu::TextureFormat> {
    Ok(match f {
        vk::Format::R8G8B8A8_UNORM => wgpu::TextureFormat::Rgba8Unorm,
        vk::Format::R8G8B8A8_SRGB => wgpu::TextureFormat::Rgba8UnormSrgb,
        vk::Format::B8G8R8A8_UNORM => wgpu::TextureFormat::Bgra8Unorm,
        vk::Format::B8G8R8A8_SRGB => wgpu::TextureFormat::Bgra8UnormSrgb,
        vk::Format::A2B10G10R10_UNORM_PACK32 => wgpu::TextureFormat::Rgb10a2Unorm,
        other => bail!("core image format {other:?} not mapped by the spike"),
    })
}

impl Gpu {
    /// Creates the Vulkan instance and adapter with wgpu, lets the core create the device
    /// through its negotiation interface (when it has one), and wraps that device in wgpu.
    /// Then fills the `retro_hw_render_interface_vulkan` the core will ask for.
    ///
    /// # Safety
    /// `negotiation` is null or the core's interface, valid while the core is loaded.
    pub unsafe fn new(
        negotiation: *const RetroNegotiationVulkanV1,
        adapter_name: Option<&str>,
    ) -> Result<Self> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let adapter = match adapter_name {
            // An explicit GPU by (part of) its name, e.g. the integrated one.
            Some(name) => {
                let all = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
                let names: Vec<String> = all.iter().map(|a| a.get_info().name).collect();
                all.into_iter()
                    .find(|a| {
                        a.get_info()
                            .name
                            .to_lowercase()
                            .contains(&name.to_lowercase())
                    })
                    .ok_or_else(|| {
                        anyhow!("no Vulkan adapter named like {name}; there are {names:?}")
                    })?
            }
            None => pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            }))
            .context("no Vulkan adapter")?,
        };
        let adapter_info = adapter.get_info();
        let features = wgpu::Features::empty();
        let limits = adapter.limits();

        let hal_instance = instance
            .as_hal::<Vulkan>()
            .ok_or_else(|| anyhow!("not a Vulkan instance"))?;
        let shared = hal_instance.shared_instance();
        let raw_instance = shared.raw_instance().clone();
        let gipa = shared.entry().static_fn().get_instance_proc_addr;
        let gdpa = raw_instance.fp_v1_0().get_device_proc_addr;

        let hal_adapter = adapter
            .as_hal::<Vulkan>()
            .ok_or_else(|| anyhow!("not a Vulkan adapter"))?;
        let gpu = hal_adapter.raw_physical_device();
        let extensions = hal_adapter.required_device_extensions(features);
        let required_extensions = extensions
            .iter()
            .map(|e| e.to_string_lossy().into_owned())
            .collect();
        let phd_features = hal_adapter.physical_device_features(&extensions, features);
        let core_features = phd_features.get_core();
        println!(
            "Vulkan 1.0 features required from the core's device: robustBufferAccess {}, independentBlend {}, sampleRateShading {}, imageCubeArray {}",
            core_features.robust_buffer_access,
            core_features.independent_blend,
            core_features.sample_rate_shading,
            core_features.image_cube_array
        );

        let create_device = if negotiation.is_null() {
            None
        } else {
            let n = &*negotiation;
            if n.interface_version >= 2 {
                let v2 = &*(negotiation as *const RetroNegotiationVulkanV2);
                println!(
                    "negotiation v{}: create_device {}, create_instance {}, create_device2 {}",
                    n.interface_version,
                    n.create_device.is_some(),
                    v2.create_instance.is_some(),
                    v2.create_device2.is_some()
                );
            }
            if let Some(info) = n.get_application_info {
                let app = info();
                if !app.is_null() {
                    let v = (*app).api_version;
                    println!(
                        "core asks for Vulkan API {}.{} (application info)",
                        vk::api_version_major(v),
                        vk::api_version_minor(v)
                    );
                }
            }
            n.create_device
        };

        let (open, family, origin) = match create_device {
            Some(create) => {
                let mut ctx = RetroVulkanContext::default();
                let ptrs: Vec<*const c_char> = extensions.iter().map(|e| e.as_ptr()).collect();
                let ok = create(
                    &mut ctx,
                    raw_instance.handle(),
                    gpu,
                    vk::SurfaceKHR::null(),
                    gipa,
                    ptrs.as_ptr(),
                    ptrs.len() as u32,
                    std::ptr::null(),
                    0,
                    &core_features,
                );
                if !ok || ctx.device == vk::Device::null() {
                    bail!("the core's create_device failed");
                }
                if ctx.gpu != gpu {
                    bail!("the core picked another GPU than the one offered");
                }
                let raw = ash::Device::load(raw_instance.fp_v1_0(), ctx.device);
                // wgpu-hal takes ownership (drop_callback None): Azahar leaves the device to
                // the frontend (`destroy_device` is null).
                let open = hal_adapter
                    .device_from_raw(
                        raw,
                        None,
                        &extensions,
                        features,
                        &limits,
                        &wgpu::MemoryHints::Performance,
                        ctx.queue_family_index,
                        0,
                    )
                    .map_err(|e| anyhow!("wgpu could not wrap the core's device: {e:?}"))?;
                (open, ctx.queue_family_index, DeviceOrigin::CoreCreateDevice)
            }
            None => {
                let open = wgpu_hal::Adapter::open(
                    &*hal_adapter,
                    features,
                    &limits,
                    &wgpu::MemoryHints::Performance,
                )
                .map_err(|e| anyhow!("wgpu device: {e:?}"))?;
                (open, 0, DeviceOrigin::Wgpu)
            }
        };
        drop(hal_adapter);

        let (device, queue) = adapter
            .create_device_from_hal::<Vulkan>(
                open,
                &wgpu::DeviceDescriptor {
                    label: Some("hw-render-spike"),
                    required_features: features,
                    required_limits: limits,
                    ..Default::default()
                },
            )
            .context("wgpu device from hal")?;

        let (raw_device, raw_queue) = {
            let hal_device = device
                .as_hal::<Vulkan>()
                .ok_or_else(|| anyhow!("not a Vulkan device"))?;
            (hal_device.raw_device().clone(), hal_device.raw_queue())
        };

        host::STATE.lock().iface = Some(Box::new(RetroHwRenderInterfaceVulkan {
            interface_type: libretro_sys::RETRO_HW_RENDER_INTERFACE_VULKAN,
            interface_version: RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION,
            handle: std::ptr::null_mut::<c_void>(),
            instance: raw_instance.handle(),
            gpu,
            device: raw_device.handle(),
            get_device_proc_addr: gdpa,
            get_instance_proc_addr: gipa,
            queue: raw_queue,
            queue_index: family,
            set_image: host::set_image,
            get_sync_index: host::get_sync_index,
            get_sync_index_mask: host::get_sync_index_mask,
            set_command_buffers: host::set_command_buffers,
            wait_sync_index: host::wait_sync_index,
            lock_queue: host::lock_queue,
            unlock_queue: host::unlock_queue,
            set_signal_semaphore: host::set_signal_semaphore,
        }));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit"),
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
            label: Some("blit"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit"),
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
            origin,
            family,
            required_extensions,
            raw_device,
            pipeline,
            layout,
            sampler,
            wrapped: None,
            target: None,
            _instance: instance,
            _adapter: adapter,
        })
    }

    /// Wraps the core's `VkImage` as a wgpu texture (no copy, no ownership: the core keeps
    /// destroying it). Rewrapped only when the image or its size changes.
    unsafe fn wrap(&mut self, img: &CoreImage, width: u32, height: u32) -> Result<()> {
        if let Some(w) = &self.wrapped {
            if w.image == img.image && w.width == width && w.height == height {
                return Ok(());
            }
        }
        let format = map_format(img.format)?;
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
                img.image,
                &wgpu_hal::TextureDescriptor {
                    label: Some("core image"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUses::RESOURCE | wgpu::TextureUses::COPY_SRC,
                    memory_flags: wgpu_hal::MemoryFlags::empty(),
                    view_formats: vec![],
                },
                // Not ours: wgpu must not destroy it.
                Some(Box::new(|| {})),
                wgpu_hal::vulkan::TextureMemory::External,
            )
        };
        let texture = self.device.create_texture_from_hal::<Vulkan>(
            hal_texture,
            &wgpu::TextureDescriptor {
                label: Some("core image"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            // The layout the core leaves it in (checked by the caller).
            wgpu::TextureUses::RESOURCE,
        );
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
        self.wrapped = Some(Wrapped {
            image: img.image,
            width,
            height,
            bind_group,
            _texture: texture,
        });
        Ok(())
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

    /// Draws the core's last image with a wgpu render pass into our target and waits for the
    /// GPU (what the compositor will do, minus the swapchain). With `readback`, also copies the
    /// target to the CPU and returns tightly packed RGBA rows.
    ///
    /// # Safety
    /// `img` is the core's current image, in `SHADER_READ_ONLY_OPTIMAL`.
    pub unsafe fn present(
        &mut self,
        img: &CoreImage,
        width: u32,
        height: u32,
        readback: bool,
    ) -> Result<Option<Vec<u8>>> {
        if img.layout != vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
            bail!(
                "core image in layout {:?}, the spike expects SHADER_READ_ONLY_OPTIMAL",
                img.layout
            );
        }
        if img.src_queue_family != self.family && img.src_queue_family != vk::QUEUE_FAMILY_IGNORED {
            bail!(
                "core image owned by queue family {}: ownership transfer not handled",
                img.src_queue_family
            );
        }
        if img.semaphores != 0 {
            bail!(
                "core passed {} semaphores to set_image: not handled by the spike",
                img.semaphores
            );
        }
        self.wrap(img, width, height)?;
        self.ensure_target(width, height);
        let (Some(wrapped), Some(target)) = (&self.wrapped, &self.target) else {
            bail!("no image or target");
        };

        // The core ends its render pass without an outgoing dependency (Azahar's
        // `CreateRenderpass`), and wgpu sees a read-after-read (RESOURCE to RESOURCE) and
        // inserts nothing. So the frontend makes the core's writes visible to its fragment
        // shader with one raw barrier, in a command buffer of its own submitted first
        // (wgpu forbids mixing raw and wgpu commands in one encoder).
        let mut barrier_enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("core barrier"),
            });
        let raw_device = self.raw_device.clone();
        let image = img.image;
        barrier_enc.as_hal_mut::<Vulkan, _, _>(|e| {
            if let Some(e) = e {
                let barrier = vk::ImageMemoryBarrier::default()
                    .src_access_mask(
                        vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::TRANSFER_WRITE,
                    )
                    .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::TRANSFER_READ)
                    .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                raw_device.cmd_pipeline_barrier(
                    e.raw_handle(),
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                        | vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            }
        });
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
            pass.set_bind_group(0, &wrapped.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        if readback {
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
        let cmds = [barrier_enc.finish(), enc.finish()];
        // Same VkQueue as the core: external synchronisation through the interface's lock.
        parking_lot::lock_api::RawMutex::lock(&QUEUE_LOCK);
        let index = self.queue.submit(cmds);
        parking_lot::lock_api::RawMutex::unlock(&QUEUE_LOCK);

        if !readback {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(index),
                    timeout: None,
                })
                .map_err(|e| anyhow!("poll: {e:?}"))?;
            return Ok(None);
        }
        let slice = target.readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            })
            .map_err(|e| anyhow!("poll: {e:?}"))?;
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

    /// Names of the extensions the frontend asked the core to enable.
    pub fn required_extensions(&self) -> &[String] {
        &self.required_extensions
    }

    /// Drops our wrapper of the core's image (before the core destroys it).
    pub fn release_image(&mut self) {
        self.wrapped = None;
        // Let wgpu finish and free the wrapper's view now, while the image still exists.
        if let Err(e) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            log::warn!("poll at release: {e:?}");
        }
    }
}
