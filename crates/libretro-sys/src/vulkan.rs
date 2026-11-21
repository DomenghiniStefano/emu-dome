//! `libretro_vulkan.h`: the Vulkan interfaces of `SET_HW_RENDER`, with the Vulkan types written
//! as their C representation, so this crate (and the host) need no Vulkan binding:
//! dispatchable handles (`VkInstance`, `VkPhysicalDevice`, `VkDevice`, `VkQueue`,
//! `VkCommandBuffer`) are pointers, non-dispatchable ones (`VkImage`, `VkImageView`,
//! `VkSemaphore`) 64-bit integers, enums C `int`s. The shell turns them into its own types
//! (`ash`, in the app's `gpu_interop.rs`).
//!
//! Vulkan's own function pointers (`PFN_vk*`) use the system calling convention
//! (`VKAPI_PTR`); the libretro callbacks the C one (`RETRO_CALLCONV`). Both are the platform's
//! default C convention on every target EmuDome builds (x86-64, AArch64).

use std::os::raw::{c_char, c_uint, c_void};

pub type VkInstance = *mut c_void;
pub type VkPhysicalDevice = *mut c_void;
pub type VkDevice = *mut c_void;
pub type VkQueue = *mut c_void;
pub type VkCommandBuffer = *mut c_void;
pub type VkImage = u64;
pub type VkImageView = u64;
pub type VkSemaphore = u64;
pub type VkSurfaceKHR = u64;

/// `PFN_vkVoidFunction`.
pub type PFN_vkVoidFunction = Option<unsafe extern "system" fn()>;
/// `PFN_vkGetInstanceProcAddr`.
pub type PFN_vkGetInstanceProcAddr =
    Option<unsafe extern "system" fn(VkInstance, *const c_char) -> PFN_vkVoidFunction>;
/// `PFN_vkGetDeviceProcAddr`.
pub type PFN_vkGetDeviceProcAddr =
    Option<unsafe extern "system" fn(VkDevice, *const c_char) -> PFN_vkVoidFunction>;

/// `VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL`: the layout a core hands its image in.
pub const VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL: i32 = 5;
/// `VK_QUEUE_FAMILY_IGNORED`.
pub const VK_QUEUE_FAMILY_IGNORED: u32 = !0;

/// `VkComponentMapping`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VkComponentMapping {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}

/// `VkImageSubresourceRange`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VkImageSubresourceRange {
    pub aspect_mask: u32,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

/// `VkImageViewCreateInfo`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkImageViewCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub flags: u32,
    pub image: VkImage,
    pub view_type: i32,
    pub format: i32,
    pub components: VkComponentMapping,
    pub subresource_range: VkImageSubresourceRange,
}

/// `VkApplicationInfo`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkApplicationInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub p_application_name: *const c_char,
    pub application_version: u32,
    pub p_engine_name: *const c_char,
    pub engine_version: u32,
    pub api_version: u32,
}

/// `RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION`.
pub const RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION: c_uint = 5;

/// `struct retro_vulkan_image`: what the core hands to `set_image`, a view of the image it
/// rendered, the layout the image is in, and the view's create info (which names the
/// `VkImage` and its format).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_vulkan_image {
    pub image_view: VkImageView,
    pub image_layout: i32,
    pub create_info: VkImageViewCreateInfo,
}

pub type retro_vulkan_set_image_t = unsafe extern "C" fn(
    handle: *mut c_void,
    image: *const retro_vulkan_image,
    num_semaphores: u32,
    semaphores: *const VkSemaphore,
    src_queue_family: u32,
);
pub type retro_vulkan_get_sync_index_t = unsafe extern "C" fn(handle: *mut c_void) -> u32;
pub type retro_vulkan_get_sync_index_mask_t = unsafe extern "C" fn(handle: *mut c_void) -> u32;
pub type retro_vulkan_set_command_buffers_t =
    unsafe extern "C" fn(handle: *mut c_void, num_cmd: u32, cmd: *const VkCommandBuffer);
pub type retro_vulkan_wait_sync_index_t = unsafe extern "C" fn(handle: *mut c_void);
pub type retro_vulkan_lock_queue_t = unsafe extern "C" fn(handle: *mut c_void);
pub type retro_vulkan_unlock_queue_t = unsafe extern "C" fn(handle: *mut c_void);
pub type retro_vulkan_set_signal_semaphore_t =
    unsafe extern "C" fn(handle: *mut c_void, semaphore: VkSemaphore);

/// `struct retro_hw_render_interface_vulkan`, the answer to `GET_HW_RENDER_INTERFACE`.
/// `queue_index` is the queue **family** index (the header's name is historical).
#[repr(C)]
#[derive(Debug)]
pub struct retro_hw_render_interface_vulkan {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
    pub handle: *mut c_void,
    pub instance: VkInstance,
    pub gpu: VkPhysicalDevice,
    pub device: VkDevice,
    pub get_device_proc_addr: PFN_vkGetDeviceProcAddr,
    pub get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    pub queue: VkQueue,
    pub queue_index: c_uint,
    pub set_image: retro_vulkan_set_image_t,
    pub get_sync_index: retro_vulkan_get_sync_index_t,
    pub get_sync_index_mask: retro_vulkan_get_sync_index_mask_t,
    pub set_command_buffers: retro_vulkan_set_command_buffers_t,
    pub wait_sync_index: retro_vulkan_wait_sync_index_t,
    pub lock_queue: retro_vulkan_lock_queue_t,
    pub unlock_queue: retro_vulkan_unlock_queue_t,
    pub set_signal_semaphore: retro_vulkan_set_signal_semaphore_t,
}

/// `retro_vulkan_get_application_info_t`.
pub type retro_vulkan_get_application_info_t = unsafe extern "C" fn() -> *const VkApplicationInfo;

/// `struct retro_hw_render_context_negotiation_interface_vulkan`, version 1 part (what the core
/// hands with `SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE`). `create_device` and
/// `destroy_device` are kept as raw pointers: EmuDome never calls them (the core is handed the
/// shell's own device), it reads `get_application_info` only. Version 2 adds `create_instance`
/// and `create_device2` after these fields.
#[repr(C)]
#[derive(Debug)]
pub struct retro_hw_render_context_negotiation_interface_vulkan {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
    pub get_application_info: Option<retro_vulkan_get_application_info_t>,
    pub create_device: *const c_void,
    pub destroy_device: *const c_void,
}

/// `struct retro_hw_render_context_negotiation_interface`: the header every negotiation
/// interface starts with; `GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT` passes it with
/// the type filled and the frontend writes the highest version it supports.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct retro_hw_render_context_negotiation_interface {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
}

/// `VK_MAKE_API_VERSION(0, major, minor, 0)`.
pub const fn vk_api_version(major: u32, minor: u32) -> u32 {
    (major << 22) | (minor << 12)
}

#[cfg(test)]
mod tests {
    use super::{retro_hw_render_interface_vulkan, retro_vulkan_image, VkImageViewCreateInfo};

    /// The C layouts on a 64-bit target (`libretro_vulkan.h`, `vulkan_core.h`).
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn the_layouts_match_the_c_headers() {
        assert_eq!(std::mem::size_of::<VkImageViewCreateInfo>(), 80);
        assert_eq!(std::mem::size_of::<retro_vulkan_image>(), 96);
        // Two unsigned, 7 pointers, one unsigned (padded to 8), 8 function pointers.
        assert_eq!(std::mem::size_of::<retro_hw_render_interface_vulkan>(), 136);
        assert_eq!(super::vk_api_version(1, 1), 0x0040_1000);
    }
}
