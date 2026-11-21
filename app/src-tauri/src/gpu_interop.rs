//! The one module that reaches under wgpu into its Vulkan backend (wgpu-hal and ash).
//!
//! wgpu-hal's interop API (`as_hal`, `device_from_raw`, `texture_from_raw`,
//! `create_device_from_hal`, `create_texture_from_hal`, raw commands in an encoder) changes with
//! every wgpu major. Hardware rendering (3DS through Azahar, #46 / #16,
//! `docs/feasibility-hw-render.md`) needs these things from it:
//! - the app's one device, opened with the extensions and features a 3DS core uses on top of
//!   wgpu's own ([`open_device_for_cores`]): the core renders on the app's device, it never makes
//!   its own;
//! - the raw Vulkan objects of that device, for the core ([`VulkanInterop::vulkan_handles`],
//!   [`core_device`]);
//! - an image the core drew into, seen as a wgpu texture with no copy
//!   ([`VulkanInterop::texture_from_vulkan_image`]);
//! - a raw barrier on that image, and a wait on the core's semaphores
//!   ([`VulkanInterop::image_barrier`], [`VulkanInterop::wait_semaphores`]): wgpu cannot see the
//!   core's own writes.
//!
//! A wgpu upgrade changes this file only.
//!
//! Vulkan only, on the platforms that have it (Windows, Linux, Android). macOS (not compiled)
//! would need wgpu's `vulkan-portability`.

use std::ffi::CStr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use ash::vk;
use emudome_core::hw_render::vulkan::{
    PFN_vkGetDeviceProcAddr, PFN_vkGetInstanceProcAddr, QueueLock, VulkanDevice,
};
use wgpu::hal::api::Vulkan;

/// The raw Vulkan objects of a wgpu device (what `retro_hw_render_interface_vulkan` hands a
/// core).
#[derive(Clone, Copy, Debug)]
pub struct VulkanHandles {
    pub instance: vk::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: vk::Device,
    pub queue: vk::Queue,
    pub queue_family: u32,
    /// The physical device's Vulkan version.
    pub api_version: u32,
    pub get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
}

/// `handles` as `emudome_core`'s description of the device the core is handed, with the lock
/// every submit on its queue takes.
pub fn core_device(handles: &VulkanHandles, queue_lock: Arc<QueueLock>) -> VulkanDevice {
    use vk::Handle as _;
    // SAFETY: the same functions under the C types of `libretro_sys::vulkan` (a function
    // pointer into an `Option` of one: same size, never null): dispatchable handles are pointers
    // on both sides (ash's are `#[repr(transparent)]` over one), the calling convention is the
    // system one on both.
    let (gipa, gdpa) = unsafe {
        (
            std::mem::transmute::<vk::PFN_vkGetInstanceProcAddr, PFN_vkGetInstanceProcAddr>(
                handles.get_instance_proc_addr,
            ),
            std::mem::transmute::<vk::PFN_vkGetDeviceProcAddr, PFN_vkGetDeviceProcAddr>(
                handles.get_device_proc_addr,
            ),
        )
    };
    VulkanDevice {
        instance: handles.instance.as_raw() as usize,
        gpu: handles.physical_device.as_raw() as usize,
        device: handles.device.as_raw() as usize,
        queue: handles.queue.as_raw() as usize,
        queue_family: handles.queue_family,
        api_version: handles.api_version,
        get_instance_proc_addr: gipa,
        get_device_proc_addr: gdpa,
        queue_lock,
    }
}

/// A barrier's side: pipeline stages and memory accesses.
pub type Scope = (vk::PipelineStageFlags, vk::AccessFlags);

/// What hardware rendering needs from a wgpu device.
pub trait VulkanInterop {
    /// The device's raw Vulkan objects; `None` when it is not on Vulkan.
    fn vulkan_handles(&self) -> Option<VulkanHandles>;

    /// `image` (made on this device, as `desc` says) as a texture of this device, no copy, in
    /// `SHADER_READ_ONLY_OPTIMAL` (wgpu's `RESOURCE`). It is not owned: `on_drop` runs when wgpu
    /// lets go of it, and the image must live until then. `None` when the device is not on
    /// Vulkan.
    ///
    /// # Safety
    /// `image` is a valid image of this device matching `desc` (size, format, usage).
    unsafe fn texture_from_vulkan_image(
        &self,
        image: vk::Image,
        desc: &wgpu::TextureDescriptor<'_>,
        on_drop: Option<Box<dyn FnOnce() + Send + Sync>>,
    ) -> Option<wgpu::Texture>;

    /// A command buffer holding one barrier on `image`, its layout left in
    /// `SHADER_READ_ONLY_OPTIMAL`: `src` before, `dst` after. In an encoder of its own: wgpu
    /// forbids raw commands in an encoder that has wgpu ones. `None` off Vulkan.
    ///
    /// # Safety
    /// `image` is a valid colour image of this device, in `SHADER_READ_ONLY_OPTIMAL` when the
    /// buffer runs.
    unsafe fn image_barrier(
        &self,
        image: vk::Image,
        src: Scope,
        dst: Scope,
    ) -> Option<wgpu::CommandBuffer>;

    /// An empty submit on the device's queue that waits for `semaphores`: every later submit
    /// runs after it.
    ///
    /// # Safety
    /// The caller holds the queue's lock; the semaphores are the core's, signalled by a submit
    /// already made.
    unsafe fn wait_semaphores(&self, semaphores: &[vk::Semaphore]) -> Result<()>;
}

impl VulkanInterop for wgpu::Device {
    fn vulkan_handles(&self) -> Option<VulkanHandles> {
        // SAFETY: the hal device is only read, while `self` keeps it alive.
        let hal = unsafe { self.as_hal::<Vulkan>() }?;
        let shared = hal.shared_instance();
        let instance = shared.raw_instance();
        let physical_device = hal.raw_physical_device();
        // SAFETY: a query on the device's own physical device.
        let api_version =
            unsafe { instance.get_physical_device_properties(physical_device) }.api_version;
        Some(VulkanHandles {
            instance: instance.handle(),
            physical_device,
            device: hal.raw_device().handle(),
            queue: hal.raw_queue(),
            queue_family: hal.queue_family_index(),
            api_version,
            get_instance_proc_addr: shared.entry().static_fn().get_instance_proc_addr,
            get_device_proc_addr: instance.fp_v1_0().get_device_proc_addr,
        })
    }

    unsafe fn texture_from_vulkan_image(
        &self,
        image: vk::Image,
        desc: &wgpu::TextureDescriptor<'_>,
        on_drop: Option<Box<dyn FnOnce() + Send + Sync>>,
    ) -> Option<wgpu::Texture> {
        let hal_texture = {
            let hal = self.as_hal::<Vulkan>()?;
            hal.texture_from_raw(
                image,
                &wgpu::hal::TextureDescriptor {
                    label: desc.label,
                    size: desc.size,
                    mip_level_count: desc.mip_level_count,
                    sample_count: desc.sample_count,
                    dimension: desc.dimension,
                    format: desc.format,
                    usage: wgpu::TextureUses::RESOURCE | wgpu::TextureUses::COPY_SRC,
                    memory_flags: wgpu::hal::MemoryFlags::empty(),
                    view_formats: desc.view_formats.to_vec(),
                },
                // Never `None`: that would hand the image's ownership to wgpu.
                Some(on_drop.unwrap_or_else(|| Box::new(|| {}))),
                wgpu::hal::vulkan::TextureMemory::External,
            )
        };
        Some(self.create_texture_from_hal::<Vulkan>(hal_texture, desc, wgpu::TextureUses::RESOURCE))
    }

    unsafe fn image_barrier(
        &self,
        image: vk::Image,
        src: Scope,
        dst: Scope,
    ) -> Option<wgpu::CommandBuffer> {
        let raw = self.as_hal::<Vulkan>()?.raw_device().clone();
        let mut enc = self.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("core image barrier"),
        });
        enc.as_hal_mut::<Vulkan, _, _>(|e| {
            let Some(e) = e else {
                return;
            };
            let barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(src.1)
                .dst_access_mask(dst.1)
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
            raw.cmd_pipeline_barrier(
                e.raw_handle(),
                src.0,
                dst.0,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        });
        Some(enc.finish())
    }

    unsafe fn wait_semaphores(&self, semaphores: &[vk::Semaphore]) -> Result<()> {
        let hal = self
            .as_hal::<Vulkan>()
            .ok_or_else(|| anyhow!("not a Vulkan device"))?;
        let stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; semaphores.len()];
        let submit = vk::SubmitInfo::default()
            .wait_semaphores(semaphores)
            .wait_dst_stage_mask(&stages);
        hal.raw_device()
            .queue_submit(hal.raw_queue(), &[submit], vk::Fence::null())
            .map_err(|e| anyhow!("waiting for the core's semaphores: {e}"))
    }
}

/// The wgpu format of a core image's `VkFormat` (raw), for the formats cores hand over.
pub fn wgpu_format(vk_format: u32) -> Option<wgpu::TextureFormat> {
    use wgpu::TextureFormat as T;
    Some(match vk::Format::from_raw(vk_format as i32) {
        vk::Format::R8G8B8A8_UNORM => T::Rgba8Unorm,
        vk::Format::B8G8R8A8_UNORM => T::Bgra8Unorm,
        vk::Format::A2B10G10R10_UNORM_PACK32 => T::Rgb10a2Unorm,
        vk::Format::R8G8B8A8_SRGB => T::Rgba8UnormSrgb,
        vk::Format::B8G8R8A8_SRGB => T::Bgra8UnormSrgb,
        _ => return None,
    })
}

/// A core's image (raw `VkImage`) as a texture of `device` with no copy; see
/// [`VulkanInterop::texture_from_vulkan_image`]. Not owned: the core destroys it.
///
/// # Safety
/// As [`VulkanInterop::texture_from_vulkan_image`].
pub unsafe fn core_image_texture(
    device: &wgpu::Device,
    image: u64,
    desc: &wgpu::TextureDescriptor<'_>,
) -> Option<wgpu::Texture> {
    use vk::Handle as _;
    device.texture_from_vulkan_image(vk::Image::from_raw(image), desc, None)
}

/// The two barriers around a draw that samples a core's image (raw `VkImage`), each in a
/// command buffer of its own: before, the core's colour (or transfer) writes made visible to
/// the fragment shader (its render pass ends with no outgoing dependency, and wgpu sees a
/// read-after-read and inserts nothing); after, every later command (the core's next frame)
/// ordered after the draw's reads, so the core can write its image again. `None` off Vulkan.
///
/// # Safety
/// As [`VulkanInterop::image_barrier`].
pub unsafe fn core_image_barriers(
    device: &wgpu::Device,
    image: u64,
) -> Option<(wgpu::CommandBuffer, wgpu::CommandBuffer)> {
    use vk::Handle as _;
    let image = vk::Image::from_raw(image);
    let before = device.image_barrier(
        image,
        (
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::TRANSFER,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::TRANSFER_WRITE,
        ),
        (
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::SHADER_READ,
        ),
    )?;
    // Write-after-read: an execution dependency is enough (no access to make visible).
    let after = device.image_barrier(
        image,
        (
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::empty(),
        ),
        (
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::AccessFlags::empty(),
        ),
    )?;
    Some((before, after))
}

/// An empty submit waiting for a core's semaphores (raw `VkSemaphore`s); see
/// [`VulkanInterop::wait_semaphores`].
///
/// # Safety
/// As [`VulkanInterop::wait_semaphores`].
pub unsafe fn wait_core_semaphores(device: &wgpu::Device, semaphores: &[u64]) -> Result<()> {
    use vk::Handle as _;
    let raw: Vec<vk::Semaphore> = semaphores
        .iter()
        .map(|s| vk::Semaphore::from_raw(*s))
        .collect();
    device.wait_semaphores(&raw)
}

/// Device extensions a 3DS core (Azahar) uses when the GPU has them, each enabled with its
/// feature struct below. Azahar decides what to use from what the GPU supports, not from what
/// the device enabled (its libretro `Instance::CreateDevice` only detects), so every one it can
/// see must be enabled. From Azahar 2126.2's `libretro_vk.cpp` (`CreateVulkanDevice`) and
/// `vk_instance.cpp` (`Instance::CreateDevice`).
fn core_extensions() -> [&'static CStr; 10] {
    [
        ash::khr::image_format_list::NAME,
        ash::khr::timeline_semaphore::NAME,
        ash::ext::shader_stencil_export::NAME,
        ash::ext::external_memory_host::NAME,
        ash::ext::custom_border_color::NAME,
        ash::ext::extended_dynamic_state::NAME,
        ash::ext::index_type_uint8::NAME,
        ash::ext::pipeline_creation_cache_control::NAME,
        ash::khr::fragment_shader_barycentric::NAME,
        ash::ext::fragment_shader_interlock::NAME,
    ]
}

/// Opens the app's device on `adapter`: wgpu's extensions and features, plus what a 3DS core
/// renders with (core features `geometryShader`, `logicOp`, `samplerAnisotropy`,
/// `fragmentStoresAndAtomics`, `shaderClipDistance`; [`core_extensions`] with their features),
/// each only where the GPU has it. `None` when the adapter is not on Vulkan.
pub fn open_device_for_cores(
    adapter: &wgpu::Adapter,
    desc: &wgpu::DeviceDescriptor<'_>,
) -> Result<Option<(wgpu::Device, wgpu::Queue)>> {
    let open = {
        // SAFETY: the hal adapter is used inside this block only, while `adapter` lives.
        let Some(hal) = (unsafe { adapter.as_hal::<Vulkan>() }) else {
            return Ok(None);
        };
        let caps = hal.physical_device_capabilities();
        let instance = hal.shared_instance().raw_instance();
        let gpu = hal.raw_physical_device();
        let features = desc.required_features;
        let mut extensions = hal.required_device_extensions(features);
        let mut phd = hal.physical_device_features(&extensions, features);
        let wgpu_has = extensions.clone();
        let added: Vec<&'static CStr> = core_extensions()
            .into_iter()
            .filter(|e| caps.supports_extension(e) && !wgpu_has.contains(e))
            .collect();
        let adds = |e: &CStr| added.contains(&e);

        // What the GPU supports of the added extensions' features.
        let mut eds = vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default();
        let mut border = vk::PhysicalDeviceCustomBorderColorFeaturesEXT::default();
        let mut uint8 = vk::PhysicalDeviceIndexTypeUint8FeaturesEXT::default();
        let mut cache = vk::PhysicalDevicePipelineCreationCacheControlFeaturesEXT::default();
        let mut bary = vk::PhysicalDeviceFragmentShaderBarycentricFeaturesKHR::default();
        let mut interlock = vk::PhysicalDeviceFragmentShaderInterlockFeaturesEXT::default();
        {
            let mut query = vk::PhysicalDeviceFeatures2::default();
            if adds(ash::ext::extended_dynamic_state::NAME) {
                query = query.push_next(&mut eds);
            }
            if adds(ash::ext::custom_border_color::NAME) {
                query = query.push_next(&mut border);
            }
            if adds(ash::ext::index_type_uint8::NAME) {
                query = query.push_next(&mut uint8);
            }
            if adds(ash::ext::pipeline_creation_cache_control::NAME) {
                query = query.push_next(&mut cache);
            }
            if adds(ash::khr::fragment_shader_barycentric::NAME) {
                query = query.push_next(&mut bary);
            }
            if adds(ash::ext::fragment_shader_interlock::NAME) {
                query = query.push_next(&mut interlock);
            }
            // SAFETY: a query on this adapter's GPU, with structs of extensions it supports.
            unsafe { instance.get_physical_device_features2(gpu, &mut query) };
        }
        // Out of the query's chain, into the device's below (each one alone: `push_next`
        // follows the pushed struct's own `p_next`).
        eds.p_next = std::ptr::null_mut();
        border.p_next = std::ptr::null_mut();
        uint8.p_next = std::ptr::null_mut();
        cache.p_next = std::ptr::null_mut();
        bary.p_next = std::ptr::null_mut();
        interlock.p_next = std::ptr::null_mut();
        // SAFETY: as above.
        let supported = unsafe { instance.get_physical_device_features(gpu) };
        let mut core = phd.get_core();
        core.geometry_shader |= supported.geometry_shader;
        core.logic_op |= supported.logic_op;
        core.sampler_anisotropy |= supported.sampler_anisotropy;
        core.fragment_stores_and_atomics |= supported.fragment_stores_and_atomics;
        core.shader_clip_distance |= supported.shader_clip_distance;

        extensions.extend(added.iter().copied());
        let names: Vec<*const std::ffi::c_char> = extensions.iter().map(|e| e.as_ptr()).collect();
        // Family 0, as wgpu's own `open` (it checks presentation on that family).
        let family = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&[1.0])];
        let pre = vk::DeviceCreateInfo::default()
            .queue_create_infos(&family)
            .enabled_extension_names(&names);
        let mut info = phd.add_to_device_create(pre).enabled_features(&core);
        if adds(ash::ext::extended_dynamic_state::NAME) {
            info = info.push_next(&mut eds);
        }
        if adds(ash::ext::custom_border_color::NAME) {
            info = info.push_next(&mut border);
        }
        if adds(ash::ext::index_type_uint8::NAME) {
            info = info.push_next(&mut uint8);
        }
        if adds(ash::ext::pipeline_creation_cache_control::NAME) {
            info = info.push_next(&mut cache);
        }
        if adds(ash::khr::fragment_shader_barycentric::NAME) {
            info = info.push_next(&mut bary);
        }
        if adds(ash::ext::fragment_shader_interlock::NAME) {
            info = info.push_next(&mut interlock);
        }
        // SAFETY: the create info is made of wgpu's own (extensions, features) plus extensions
        // and features the GPU reported, all alive until the call returns.
        let raw = unsafe { instance.create_device(gpu, &info, None) }
            .map_err(|e| anyhow!("vkCreateDevice: {e}"))?;
        log::info!(
            "GPU device for cores: added {:?}; geometryShader {}, logicOp {}",
            added,
            core.geometry_shader,
            core.logic_op
        );
        // SAFETY: `raw` was made from this adapter with these extensions, wgpu's features (and
        // more) and queue family 0; wgpu owns it from here (no drop callback).
        unsafe {
            hal.device_from_raw(
                raw,
                None,
                &extensions,
                features,
                &desc.required_limits,
                &desc.memory_hints,
                0,
                0,
            )
        }
        .map_err(|e| anyhow!("wgpu-hal device from the raw one: {e:?}"))?
    };
    // SAFETY: `open` was made from this adapter.
    let (device, queue) = unsafe { adapter.create_device_from_hal::<Vulkan>(open, desc) }
        .context("wgpu device from the raw one")?;
    Ok(Some((device, queue)))
}

/// Headless GPU checks (no window): `cargo test -p emudome --lib gpu_interop -- --ignored`.
#[cfg(test)]
pub(crate) mod gpu_tests {
    use ash::vk;

    use super::VulkanInterop;

    /// A Vulkan device opened as the app opens it, or `None` on a machine without one (the
    /// test then says so and passes).
    pub(crate) fn vulkan() -> Option<(wgpu::Device, wgpu::Queue)> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .ok()?;
        super::open_device_for_cores(&adapter, &wgpu::DeviceDescriptor::default())
            .expect("the app's device opens")
    }

    /// An `R8G8B8A8_UNORM` image of `size` as a core hands it over: cleared to `rgba` with raw
    /// commands and left in `SHADER_READ_ONLY_OPTIMAL`. Its raw handle; destroy with the closure.
    pub(crate) fn painted_core_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        rgba: [f32; 4],
    ) -> (u64, impl FnOnce()) {
        use vk::Handle as _;
        let (image, destroy) = core_image(
            device,
            size,
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC,
        );
        let raw = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
            .unwrap()
            .raw_device()
            .clone();
        let range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let mut enc = device.create_command_encoder(&Default::default());
        unsafe {
            enc.as_hal_mut::<wgpu::hal::api::Vulkan, _, _>(|e| {
                let cmd = e.unwrap().raw_handle();
                let to_dst = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(range);
                raw.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[to_dst],
                );
                raw.cmd_clear_color_image(
                    cmd,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &vk::ClearColorValue { float32: rgba },
                    &[range],
                );
                let to_read = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(range);
                raw.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[to_read],
                );
            });
        }
        queue.submit([enc.finish()]);
        (image.as_raw(), destroy)
    }

    /// An image as a core makes it: on `device`, with its own memory. Destroy with the closure.
    pub(crate) fn core_image(
        device: &wgpu::Device,
        (w, h): (u32, u32),
        usage: vk::ImageUsageFlags,
    ) -> (vk::Image, impl FnOnce()) {
        let hal = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }.unwrap();
        let raw = hal.raw_device().clone();
        let instance = hal.shared_instance().raw_instance().clone();
        let gpu = hal.raw_physical_device();
        drop(hal);
        let image = unsafe {
            raw.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .extent(vk::Extent3D {
                        width: w,
                        height: h,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(usage),
                None,
            )
        }
        .unwrap();
        let needs = unsafe { raw.get_image_memory_requirements(image) };
        let props = unsafe { instance.get_physical_device_memory_properties(gpu) };
        let type_index = (0..props.memory_type_count)
            .find(|i| {
                needs.memory_type_bits & (1 << i) != 0
                    && props.memory_types[*i as usize]
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .unwrap();
        let memory = unsafe {
            raw.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(needs.size)
                    .memory_type_index(type_index),
                None,
            )
        }
        .unwrap();
        unsafe { raw.bind_image_memory(image, memory, 0) }.unwrap();
        (image, move || unsafe {
            raw.destroy_image(image, None);
            raw.free_memory(memory, None);
        })
    }

    #[test]
    #[ignore = "needs a Vulkan GPU"]
    fn a_core_image_is_seen_as_a_texture_without_a_copy() {
        let Some((device, queue)) = vulkan() else {
            eprintln!("no Vulkan adapter: skipped");
            return;
        };
        let h = device
            .vulkan_handles()
            .expect("a Vulkan device has handles");
        assert_ne!(h.instance, vk::Instance::null());
        assert_ne!(h.device, vk::Device::null());
        assert_ne!(h.physical_device, vk::PhysicalDevice::null());
        assert_ne!(h.queue, vk::Queue::null());
        assert!(h.api_version >= vk::API_VERSION_1_1);
        // The loader's entry point answers for the instance it made, and the device's.
        assert!(
            unsafe { (h.get_instance_proc_addr)(h.instance, c"vkGetDeviceProcAddr".as_ptr()) }
                .is_some()
        );
        assert!(unsafe { (h.get_device_proc_addr)(h.device, c"vkQueueSubmit".as_ptr()) }.is_some());
        eprintln!("queue family {}, API {:#x}", h.queue_family, h.api_version);

        let (w, h_px) = (64, 32);
        let (image, destroy) = core_image(
            &device,
            (w, h_px),
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC,
        );
        let (dropped_tx, dropped) = std::sync::mpsc::channel();
        let texture = unsafe {
            device.texture_from_vulkan_image(
                image,
                &wgpu::TextureDescriptor {
                    label: Some("core image"),
                    size: wgpu::Extent3d {
                        width: w,
                        height: h_px,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                Some(Box::new(move || {
                    let _ = dropped_tx.send(());
                })),
            )
        }
        .expect("a Vulkan device wraps the image");
        assert_eq!((texture.width(), texture.height()), (w, h_px));
        let _view = texture.create_view(&Default::default());
        drop(_view);
        drop(texture);
        // wgpu lets go after the queue has moved on.
        queue.submit([]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        assert!(dropped
            .recv_timeout(std::time::Duration::from_secs(5))
            .is_ok());
        destroy();
    }
}
