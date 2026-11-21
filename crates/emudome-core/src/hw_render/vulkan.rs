//! Vulkan hardware rendering, the libretro side: [`VulkanHandler`] accepts a core's Vulkan
//! `SET_HW_RENDER`, hands it the shell's own device in `retro_hw_render_interface_vulkan`, and
//! gives the image of each frame (`set_image`) to the session's video sink as a
//! [`GpuFrame`]. Plain FFI on raw handles: the GPU side (the device, the copy of the core's
//! image, the readback) is the shell's (`gpu_interop.rs` and `hw_render.rs` in the app).
//!
//! The device is the shell's long-lived one ([`VulkanDevice`], made once for the whole run with
//! the extensions and features a 3DS core needs): the core never makes its own. libretro lets
//! the frontend create the device itself; the core's negotiation interface
//! (`SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE`) is read for the Vulkan version the core asks
//! for (`get_application_info`), and its `create_device` is never called. One device means the
//! core's image is sampled by the renderer with no copy through the CPU, and Android's background
//! / resume never pulls the device from under a running core.
//!
//! Synchronisation (`docs/feasibility-hw-render.md`):
//! - one sync index (`get_sync_index` 0, mask 1, `wait_sync_index` returns at once): the shell
//!   copies the core's image on the GPU inside `video_refresh`, on the same queue, with a trailing
//!   barrier ordering the core's next frame after the copy, so the core can always reuse its image
//!   (Azahar renders every frame into one image);
//! - the queue lock ([`QueueLock`]) is taken by the core around its own `vkQueueSubmit`
//!   (`lock_queue` / `unlock_queue`) and by the shell around every submit and present on the
//!   shared queue.

use std::ffi::{c_uint, c_void};
use std::sync::Arc;

use libretro_sys::vulkan::{
    retro_hw_render_context_negotiation_interface,
    retro_hw_render_context_negotiation_interface_vulkan, retro_hw_render_interface_vulkan,
    retro_vulkan_image, vk_api_version, VkCommandBuffer, VkSemaphore,
    RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
    VK_QUEUE_FAMILY_IGNORED,
};
/// The Vulkan loader's entry points as [`VulkanDevice`] carries them (C types).
pub use libretro_sys::vulkan::{PFN_vkGetDeviceProcAddr, PFN_vkGetInstanceProcAddr};
use libretro_sys::{
    retro_hw_context_reset_t, retro_hw_render_callback,
    RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT,
    RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE, RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
    RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE,
    RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT, RETRO_HW_CONTEXT_VULKAN,
    RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN, RETRO_HW_RENDER_INTERFACE_VULKAN,
};
use parking_lot::lock_api::RawMutex as _;
use parking_lot::Mutex;

use super::HwRenderHandler;
use crate::video_sink::{GpuFrame, GpuImage, GpuSync};

/// External synchronisation of the `VkQueue` the core and the shell share. A raw mutex: the core
/// locks and unlocks it with two separate C calls.
pub struct QueueLock(parking_lot::RawMutex);

impl Default for QueueLock {
    fn default() -> Self {
        Self(parking_lot::RawMutex::INIT)
    }
}

impl QueueLock {
    /// Holds the queue until the guard drops: wrap every submit and present on the shared queue.
    pub fn lock(&self) -> QueueGuard<'_> {
        self.0.lock();
        QueueGuard(self)
    }
}

/// See [`QueueLock::lock`].
pub struct QueueGuard<'a>(&'a QueueLock);

impl Drop for QueueGuard<'_> {
    fn drop(&mut self) {
        // SAFETY: the guard exists only while this lock is held.
        unsafe { self.0 .0.unlock() };
    }
}

/// The shell's Vulkan device, as the interface table hands it to the core. Dispatchable handles
/// as addresses, so the description moves between threads; they stay valid for as long as the
/// shell keeps the device, which outlives every session.
#[derive(Clone)]
pub struct VulkanDevice {
    pub instance: usize,
    pub gpu: usize,
    pub device: usize,
    pub queue: usize,
    pub queue_family: u32,
    /// The physical device's Vulkan version (`VkPhysicalDeviceProperties::apiVersion`).
    pub api_version: u32,
    pub get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    pub get_device_proc_addr: PFN_vkGetDeviceProcAddr,
    /// Taken around every submit and present on `queue`, by the core and by the shell.
    pub queue_lock: Arc<QueueLock>,
}

/// The image of the core's last frame (`set_image`): valid until the core's next `retro_run`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CoreImage {
    image: u64,
    format: i32,
    layout: i32,
    wait_semaphores: Vec<VkSemaphore>,
    src_queue_family: u32,
}

/// What the interface's callbacks reach through its `handle`: the queue lock and the last image.
struct Bridge {
    queue_lock: Arc<QueueLock>,
    image: Mutex<Option<CoreImage>>,
}

/// The interface table, boxed (the core keeps its address).
struct Interface(Box<retro_hw_render_interface_vulkan>);

// SAFETY: the table holds the shell's handles and a pointer to the handler's own `Bridge`; it is
// only read (by the core) and moves with the handler, behind the session's callback mutex.
unsafe impl Send for Interface {}

/// Why a frame's image cannot be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProblem {
    /// Not in `SHADER_READ_ONLY_OPTIMAL`, the layout libretro_vulkan.h asks for.
    Layout,
    /// Owned by another queue family: the ownership transfer is not implemented.
    QueueFamily,
}

/// Checks an image from `set_image` against what the shell can sample: its layout and the queue
/// family it comes from, against the shared queue's.
pub fn check_image(layout: i32, src_family: u32, queue_family: u32) -> Result<(), ImageProblem> {
    if layout != VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL {
        return Err(ImageProblem::Layout);
    }
    if src_family != queue_family && src_family != VK_QUEUE_FAMILY_IGNORED {
        return Err(ImageProblem::QueueFamily);
    }
    Ok(())
}

/// Whether the device can serve a core's Vulkan request. Cores write the Vulkan version they
/// need in `version_major` (Azahar: `VK_MAKE_VERSION(1, 1, 0)`), or a bare major number.
pub fn version_supported(requested_major: c_uint, requested_minor: c_uint, device: u32) -> bool {
    let wanted = if requested_major >= 1 << 22 {
        requested_major
    } else {
        vk_api_version(requested_major.max(1), requested_minor)
    };
    device >= wanted
}

/// Answers a core's Vulkan hardware rendering with the shell's device.
pub struct VulkanHandler {
    device: VulkanDevice,
    bridge: Box<Bridge>,
    /// Built when the core's request is accepted.
    interface: Option<Interface>,
    /// The core's accepted request (its `context_reset` / `context_destroy`).
    request: Option<retro_hw_render_callback>,
    /// The last image refused, so a core stuck on it is logged once.
    refused: Option<ImageProblem>,
}

// SAFETY: `retro_hw_render_callback` holds only the core's function pointers, valid while it is
// loaded; the handler moves between threads only behind the session's callback mutex.
unsafe impl Send for VulkanHandler {}

impl VulkanHandler {
    pub fn new(device: VulkanDevice) -> Self {
        let bridge = Box::new(Bridge {
            queue_lock: device.queue_lock.clone(),
            image: Mutex::new(None),
        });
        Self {
            device,
            bridge,
            interface: None,
            request: None,
            refused: None,
        }
    }

    fn interface_table(&self) -> retro_hw_render_interface_vulkan {
        let d = &self.device;
        retro_hw_render_interface_vulkan {
            interface_type: RETRO_HW_RENDER_INTERFACE_VULKAN,
            interface_version: RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION,
            handle: (&*self.bridge as *const Bridge).cast_mut().cast(),
            instance: d.instance as *mut c_void,
            gpu: d.gpu as *mut c_void,
            device: d.device as *mut c_void,
            get_device_proc_addr: d.get_device_proc_addr,
            get_instance_proc_addr: d.get_instance_proc_addr,
            queue: d.queue as *mut c_void,
            queue_index: d.queue_family,
            set_image,
            get_sync_index,
            get_sync_index_mask,
            set_command_buffers,
            wait_sync_index,
            lock_queue,
            unlock_queue,
            set_signal_semaphore,
        }
    }
}

impl HwRenderHandler for VulkanHandler {
    unsafe fn set_hw_render(&mut self, request: *mut retro_hw_render_callback) -> bool {
        let Some(r) = request.as_ref() else {
            return false;
        };
        if r.context_type != RETRO_HW_CONTEXT_VULKAN {
            log::info!(
                "core asks SET_HW_RENDER context {}: refused (Vulkan only)",
                r.context_type
            );
            return false;
        }
        if !version_supported(r.version_major, r.version_minor, self.device.api_version) {
            log::warn!(
                "core asks Vulkan {:#x}.{}, the GPU has {:#x}: refused",
                r.version_major,
                r.version_minor,
                self.device.api_version
            );
            return false;
        }
        if r.context_reset.is_none() {
            log::warn!("core asks Vulkan without a context_reset: refused");
            return false;
        }
        self.request = Some(*r);
        self.interface = Some(Interface(Box::new(self.interface_table())));
        log::info!("core asks Vulkan: accepted on the shell's device");
        true
    }

    unsafe fn environment(&mut self, cmd: c_uint, data: *mut c_void) -> bool {
        match cmd {
            RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE => {
                let (Some(Interface(table)), false) = (&self.interface, data.is_null()) else {
                    return false;
                };
                *(data as *mut *const retro_hw_render_interface_vulkan) = &**table;
                true
            }
            RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE => {
                let Some(n) =
                    (data as *const retro_hw_render_context_negotiation_interface_vulkan).as_ref()
                else {
                    return false;
                };
                if n.interface_type != RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN {
                    return false;
                }
                // The Vulkan version the core will use; its `create_device` is never called.
                let api = n
                    .get_application_info
                    .map(|f| f())
                    .and_then(|info| info.as_ref())
                    .map(|info| info.api_version);
                log::info!(
                    "core's Vulkan negotiation v{}: application API {:?}, device API {:#x}; the core gets the shell's device",
                    n.interface_version,
                    api.map(|a| format!("{a:#x}")),
                    self.device.api_version
                );
                true
            }
            RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT => {
                let Some(q) = (data as *mut retro_hw_render_context_negotiation_interface).as_mut()
                else {
                    return false;
                };
                // Version 1 is understood: the interface is read, the device stays the shell's.
                q.interface_version = c_uint::from(
                    q.interface_type == RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN,
                );
                true
            }
            RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER => {
                if data.is_null() {
                    return false;
                }
                *(data as *mut c_uint) = RETRO_HW_CONTEXT_VULKAN;
                true
            }
            // OpenGL only.
            RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT => false,
            other => {
                log::debug!("hardware rendering call {other:#x} not supported");
                false
            }
        }
    }

    fn hw_frame(&mut self, width: u32, height: u32) -> Option<GpuFrame> {
        let image = self.bridge.image.lock().clone()?;
        match check_image(
            image.layout,
            image.src_queue_family,
            self.device.queue_family,
        ) {
            Ok(()) => self.refused = None,
            Err(problem) => {
                if self.refused != Some(problem) {
                    log::error!(
                        "hw render: the core's image cannot be drawn: {problem:?} ({image:?})"
                    );
                }
                self.refused = Some(problem);
                return None;
            }
        }
        Some(GpuFrame {
            width,
            height,
            image: GpuImage {
                handle: image.image,
                format: image.format as u32,
                layout: image.layout as u32,
            },
            sync: GpuSync {
                wait_semaphores: image.wait_semaphores,
                src_queue_family: image.src_queue_family,
            },
        })
    }

    fn context_reset(&mut self) -> Option<retro_hw_context_reset_t> {
        self.request.and_then(|r| r.context_reset)
    }

    fn context_destroy(&mut self) -> Option<retro_hw_context_reset_t> {
        // The image belongs to the core: forgotten before it destroys it.
        *self.bridge.image.lock() = None;
        self.request.and_then(|r| r.context_destroy)
    }
}

/// # Safety
/// `handle` is the [`Bridge`] of the table [`VulkanHandler`] built.
unsafe fn bridge<'a>(handle: *mut c_void) -> &'a Bridge {
    &*(handle as *const Bridge)
}

unsafe extern "C" fn set_image(
    handle: *mut c_void,
    image: *const retro_vulkan_image,
    num_semaphores: u32,
    semaphores: *const VkSemaphore,
    src_queue_family: u32,
) {
    let next = image.as_ref().map(|img| CoreImage {
        image: img.create_info.image,
        format: img.create_info.format,
        layout: img.image_layout,
        wait_semaphores: if semaphores.is_null() || num_semaphores == 0 {
            Vec::new()
        } else {
            std::slice::from_raw_parts(semaphores, num_semaphores as usize).to_vec()
        },
        src_queue_family,
    });
    *bridge(handle).image.lock() = next;
}

unsafe extern "C" fn get_sync_index(_handle: *mut c_void) -> u32 {
    0
}

unsafe extern "C" fn get_sync_index_mask(_handle: *mut c_void) -> u32 {
    1
}

/// The shell's copy of the previous image is ordered before the core's next submit (see the
/// module's notes): nothing to wait for.
unsafe extern "C" fn wait_sync_index(_handle: *mut c_void) {}

unsafe extern "C" fn lock_queue(handle: *mut c_void) {
    bridge(handle).queue_lock.0.lock();
}

unsafe extern "C" fn unlock_queue(handle: *mut c_void) {
    bridge(handle).queue_lock.0.unlock();
}

/// Cores that hand their command buffers to the frontend instead of submitting them: none of
/// EmuDome's (Azahar submits its own). Logged so a new core shows it.
unsafe extern "C" fn set_command_buffers(
    _handle: *mut c_void,
    num: u32,
    _cmd: *const VkCommandBuffer,
) {
    log::error!("hw render: set_command_buffers({num}) is not supported by the host");
}

unsafe extern "C" fn set_signal_semaphore(_handle: *mut c_void, _semaphore: VkSemaphore) {
    log::error!("hw render: set_signal_semaphore is not supported by the host");
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::sync::Arc;

    use libretro_sys::vulkan::{
        retro_hw_render_context_negotiation_interface, retro_hw_render_interface_vulkan,
        retro_vulkan_image, vk_api_version, VkComponentMapping, VkImageSubresourceRange,
        VkImageViewCreateInfo,
    };
    use libretro_sys::{
        retro_hw_render_callback,
        RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT,
        RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE, RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
        RETRO_HW_CONTEXT_OPENGL_CORE, RETRO_HW_CONTEXT_VULKAN,
    };

    use super::{
        check_image, version_supported, ImageProblem, QueueLock, VulkanDevice, VulkanHandler,
    };
    use crate::hw_render::HwRenderHandler;

    unsafe extern "C" fn reset() {}

    fn device() -> VulkanDevice {
        VulkanDevice {
            instance: 0x10,
            gpu: 0x20,
            device: 0x30,
            queue: 0x40,
            queue_family: 2,
            api_version: vk_api_version(1, 3),
            get_instance_proc_addr: None,
            get_device_proc_addr: None,
            queue_lock: Arc::new(QueueLock::default()),
        }
    }

    fn request(context_type: u32) -> retro_hw_render_callback {
        retro_hw_render_callback {
            context_type,
            context_reset: Some(reset),
            get_current_framebuffer: None,
            get_proc_address: None,
            depth: false,
            stencil: false,
            bottom_left_origin: false,
            version_major: vk_api_version(1, 1),
            version_minor: 0,
            cache_context: true,
            context_destroy: Some(reset),
            debug_context: false,
        }
    }

    #[test]
    fn vulkan_is_accepted_and_the_interface_carries_the_shells_device() {
        let mut h = VulkanHandler::new(device());
        let mut gl = request(RETRO_HW_CONTEXT_OPENGL_CORE);
        assert!(!unsafe { h.set_hw_render(&mut gl) });
        let mut ptr: *const retro_hw_render_interface_vulkan = std::ptr::null();
        // No interface before the request is accepted.
        assert!(!unsafe {
            h.environment(
                RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE,
                (&mut ptr as *mut *const retro_hw_render_interface_vulkan).cast(),
            )
        });
        let mut vk = request(RETRO_HW_CONTEXT_VULKAN);
        assert!(unsafe { h.set_hw_render(&mut vk) });
        assert!(h.context_reset().is_some());
        assert!(unsafe {
            h.environment(
                RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE,
                (&mut ptr as *mut *const retro_hw_render_interface_vulkan).cast(),
            )
        });
        let iface = unsafe { &*ptr };
        assert_eq!(iface.device as usize, 0x30);
        assert_eq!(iface.queue as usize, 0x40);
        assert_eq!(iface.queue_index, 2);
        assert_eq!(unsafe { (iface.get_sync_index_mask)(iface.handle) }, 1);

        // A frame before any image: nothing to draw.
        assert!(h.hw_frame(400, 480).is_none());
        let image = retro_vulkan_image {
            image_view: 7,
            image_layout: 5,
            create_info: VkImageViewCreateInfo {
                s_type: 15,
                p_next: std::ptr::null(),
                flags: 0,
                image: 0xABC,
                view_type: 1,
                format: 37,
                components: VkComponentMapping::default(),
                subresource_range: VkImageSubresourceRange::default(),
            },
        };
        unsafe { (iface.set_image)(iface.handle, &image, 0, std::ptr::null(), 2) };
        let frame = h.hw_frame(400, 480).expect("a frame after set_image");
        assert_eq!(frame.image.handle, 0xABC);
        assert_eq!((frame.image.format, frame.image.layout), (37, 5));
        assert_eq!((frame.width, frame.height), (400, 480));
        // The queue lock is the shell's.
        unsafe { (iface.lock_queue)(iface.handle) };
        unsafe { (iface.unlock_queue)(iface.handle) };
        // Destroying the context forgets the core's image.
        assert!(h.context_destroy().is_some());
        assert!(h.hw_frame(400, 480).is_none());
    }

    #[test]
    fn the_negotiation_questions_are_answered() {
        let mut h = VulkanHandler::new(device());
        let mut preferred = 0u32;
        assert!(unsafe {
            h.environment(
                RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
                (&mut preferred as *mut u32).cast::<c_void>(),
            )
        });
        assert_eq!(preferred, RETRO_HW_CONTEXT_VULKAN);
        let mut q = retro_hw_render_context_negotiation_interface {
            interface_type: 0,
            interface_version: 0,
        };
        assert!(unsafe {
            h.environment(
                RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT,
                (&mut q as *mut retro_hw_render_context_negotiation_interface).cast(),
            )
        });
        assert_eq!(q.interface_version, 1);
    }

    #[test]
    fn the_version_and_the_image_are_checked() {
        let v13 = vk_api_version(1, 3);
        assert!(version_supported(vk_api_version(1, 1), 0, v13));
        assert!(version_supported(1, 2, v13));
        assert!(!version_supported(vk_api_version(1, 4), 0, v13));
        assert_eq!(check_image(5, 0, 0), Ok(()));
        assert_eq!(check_image(5, !0, 0), Ok(()));
        assert_eq!(check_image(2, 0, 0), Err(ImageProblem::Layout));
        assert_eq!(check_image(5, 1, 0), Err(ImageProblem::QueueFamily));
    }
}
