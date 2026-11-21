//! The one module that reaches under wgpu into its Vulkan backend (wgpu-hal and ash).
//!
//! wgpu-hal's interop API (`as_hal`, `device_from_raw`, `texture_from_raw`,
//! `create_device_from_hal`, `create_texture_from_hal`) changes with every wgpu major. Hardware
//! rendering (3DS through Azahar, #46 / #16, `docs/feasibility-hw-render.md`) needs exactly three
//! things from it, behind the traits here:
//! - the raw Vulkan objects of the app's GPU, for the core to render with
//!   ([`VulkanInterop::vulkan_handles`]);
//! - an image the core drew into, seen as a wgpu texture with no copy
//!   ([`VulkanInterop::texture_from_vulkan_image`]);
//! - a device the core made itself through the context negotiation (`create_device` from our
//!   instance and GPU), wrapped as a wgpu device ([`VulkanAdapterInterop::device_from_vulkan`]).
//!
//! A wgpu upgrade changes this file only. Its first user is the 3DS work; until then only its
//! GPU test reads it.
//!
//! Vulkan only, on the platforms that have it (Windows, Linux, Android). macOS (not compiled)
//! would need wgpu's `vulkan-portability`.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the hardware-rendering seam (#78): its first user is the 3DS work (#46 / #16)"
    )
)]

use std::ffi::CStr;

use anyhow::{anyhow, Context, Result};
use ash::vk;
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
    pub get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
}

/// What hardware rendering needs from a wgpu device.
pub trait VulkanInterop {
    /// The device's raw Vulkan objects; `None` when it is not on Vulkan.
    fn vulkan_handles(&self) -> Option<VulkanHandles>;

    /// `image` (made on this device, as `desc` says) as a texture of this device, no copy. It is
    /// not owned: `on_drop` runs when wgpu lets go of it, and the image must live until then.
    /// `None` when the device is not on Vulkan.
    ///
    /// # Safety
    /// `image` is a valid image of this device matching `desc` (size, format, usage).
    unsafe fn texture_from_vulkan_image(
        &self,
        image: vk::Image,
        desc: &wgpu::TextureDescriptor<'_>,
        on_drop: Option<Box<dyn FnOnce() + Send + Sync>>,
    ) -> Option<wgpu::Texture>;
}

impl VulkanInterop for wgpu::Device {
    fn vulkan_handles(&self) -> Option<VulkanHandles> {
        // SAFETY: the hal device is only read, while `self` keeps it alive.
        let hal = unsafe { self.as_hal::<Vulkan>() }?;
        let shared = hal.shared_instance();
        Some(VulkanHandles {
            instance: shared.raw_instance().handle(),
            physical_device: hal.raw_physical_device(),
            device: hal.raw_device().handle(),
            queue: hal.raw_queue(),
            queue_family: hal.queue_family_index(),
            get_instance_proc_addr: shared.entry().static_fn().get_instance_proc_addr,
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
}

/// What the context negotiation needs from a wgpu adapter.
#[cfg_attr(
    test,
    expect(
        dead_code,
        reason = "no headless test makes a raw Vulkan device the way a core would"
    )
)]
pub trait VulkanAdapterInterop {
    /// Wraps `raw`, a device a core created on this adapter's GPU (with `extensions` enabled and
    /// queue `queue_index` of `queue_family`), as a wgpu device and queue.
    ///
    /// # Safety
    /// `raw` was created from this adapter's instance and physical device with those
    /// extensions and that queue; wgpu destroys it when the device is dropped.
    unsafe fn device_from_vulkan(
        &self,
        raw: ash::Device,
        extensions: &[&'static CStr],
        queue_family: u32,
        queue_index: u32,
        desc: &wgpu::DeviceDescriptor<'_>,
    ) -> Result<(wgpu::Device, wgpu::Queue)>;
}

impl VulkanAdapterInterop for wgpu::Adapter {
    unsafe fn device_from_vulkan(
        &self,
        raw: ash::Device,
        extensions: &[&'static CStr],
        queue_family: u32,
        queue_index: u32,
        desc: &wgpu::DeviceDescriptor<'_>,
    ) -> Result<(wgpu::Device, wgpu::Queue)> {
        let open = {
            let hal = self
                .as_hal::<Vulkan>()
                .ok_or_else(|| anyhow!("not a Vulkan adapter"))?;
            hal.device_from_raw(
                raw,
                None,
                extensions,
                desc.required_features,
                &desc.required_limits,
                &desc.memory_hints,
                queue_family,
                queue_index,
            )
            .map_err(|e| anyhow!("wgpu-hal device from the core's: {e:?}"))?
        };
        self.create_device_from_hal::<Vulkan>(open, desc)
            .context("wgpu device from the core's")
    }
}

/// Headless GPU checks (no window): `cargo test -p emudome --lib gpu_interop -- --ignored`.
#[cfg(test)]
mod gpu_tests {
    use ash::vk;

    use super::VulkanInterop;

    /// A Vulkan device, or `None` on a machine without one (the test then says so and passes).
    fn vulkan() -> Option<(wgpu::Device, wgpu::Queue)> {
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
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
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
        // The loader's entry point answers for the instance it made.
        let name = c"vkGetDeviceProcAddr";
        assert!(unsafe { (h.get_instance_proc_addr)(h.instance, name.as_ptr()) }.is_some());
        eprintln!("queue family {}", h.queue_family);

        // An image as a core would make it: on our device, with its own memory.
        let raw = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
            .unwrap()
            .raw_device()
            .clone();
        let instance = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
            .unwrap()
            .shared_instance()
            .raw_instance()
            .clone();
        let (w, h_px) = (64, 32);
        let image = unsafe {
            raw.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .extent(vk::Extent3D {
                        width: w,
                        height: h_px,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC),
                None,
            )
        }
        .unwrap();
        let needs = unsafe { raw.get_image_memory_requirements(image) };
        let props = unsafe { instance.get_physical_device_memory_properties(h.physical_device) };
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
        unsafe {
            raw.destroy_image(image, None);
            raw.free_memory(memory, None);
        }
    }
}
