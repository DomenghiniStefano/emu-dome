//! `libretro_vulkan.h`: the Vulkan interfaces of `SET_HW_RENDER`, with ash's handle types
//! (all `#[repr(transparent)]`, so they match the C layout).

use std::ffi::c_void;
use std::os::raw::{c_char, c_uint};

use ash::vk;

pub const RETRO_HW_RENDER_INTERFACE_VULKAN_VERSION: c_uint = 5;

/// What the core hands to `set_image`: a view of the image it rendered, the layout the image is
/// in, and the view's create info (which names the `VkImage` and its format).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RetroVulkanImage {
    pub image_view: vk::ImageView,
    pub image_layout: vk::ImageLayout,
    pub create_info: vk::ImageViewCreateInfo<'static>,
}

pub type SetImage = unsafe extern "C" fn(
    handle: *mut c_void,
    image: *const RetroVulkanImage,
    num_semaphores: u32,
    semaphores: *const vk::Semaphore,
    src_queue_family: u32,
);
pub type GetSyncIndex = unsafe extern "C" fn(handle: *mut c_void) -> u32;
pub type SetCommandBuffers =
    unsafe extern "C" fn(handle: *mut c_void, num_cmd: u32, cmd: *const vk::CommandBuffer);
pub type HandleFn = unsafe extern "C" fn(handle: *mut c_void);
pub type SetSignalSemaphore = unsafe extern "C" fn(handle: *mut c_void, semaphore: vk::Semaphore);

/// `retro_hw_render_interface_vulkan`, answered to `GET_HW_RENDER_INTERFACE`.
/// `queue_index` is the queue **family** index (the header's name is historical).
#[repr(C)]
pub struct RetroHwRenderInterfaceVulkan {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
    pub handle: *mut c_void,
    pub instance: vk::Instance,
    pub gpu: vk::PhysicalDevice,
    pub device: vk::Device,
    pub get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    pub get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub queue: vk::Queue,
    pub queue_index: c_uint,
    pub set_image: SetImage,
    pub get_sync_index: GetSyncIndex,
    pub get_sync_index_mask: GetSyncIndex,
    pub set_command_buffers: SetCommandBuffers,
    pub wait_sync_index: HandleFn,
    pub lock_queue: HandleFn,
    pub unlock_queue: HandleFn,
    pub set_signal_semaphore: SetSignalSemaphore,
}

/// `retro_vulkan_context`: what the core's `create_device` fills in.
#[repr(C)]
#[derive(Default)]
pub struct RetroVulkanContext {
    pub gpu: vk::PhysicalDevice,
    pub device: vk::Device,
    pub queue: vk::Queue,
    pub queue_family_index: c_uint,
    pub presentation_queue: vk::Queue,
    pub presentation_queue_family_index: c_uint,
}

pub type CreateDevice = unsafe extern "C" fn(
    context: *mut RetroVulkanContext,
    instance: vk::Instance,
    gpu: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    required_device_extensions: *const *const c_char,
    num_required_device_extensions: c_uint,
    required_device_layers: *const *const c_char,
    num_required_device_layers: c_uint,
    required_features: *const vk::PhysicalDeviceFeatures,
) -> bool;

/// `retro_hw_render_context_negotiation_interface_vulkan`, version 1 part. Version 2 adds
/// `create_instance` and `create_device2` after these fields; the spike only reports whether
/// they are there (see `main.rs`).
#[repr(C)]
pub struct RetroNegotiationVulkanV1 {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
    pub get_application_info: Option<unsafe extern "C" fn() -> *const vk::ApplicationInfo<'static>>,
    pub create_device: Option<CreateDevice>,
    pub destroy_device: Option<unsafe extern "C" fn()>,
}

/// The version 2 tail; read only when `interface_version >= 2`.
#[repr(C)]
pub struct RetroNegotiationVulkanV2 {
    pub v1: RetroNegotiationVulkanV1,
    pub create_instance: Option<unsafe extern "C" fn()>,
    pub create_device2: Option<unsafe extern "C" fn()>,
}
