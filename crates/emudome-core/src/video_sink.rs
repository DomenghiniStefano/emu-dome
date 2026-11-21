//! Where the core's frames go: the seam between the libretro video callback and the shell.
//!
//! A core hands over a frame in one of two ways ([`VideoFrame`]): bytes in memory (every
//! software-rendered core: mGBA, melonDS DS in software, Azahar in software), or an image on the
//! GPU once a hardware-rendering core is accepted ([`crate::hw_render`]). The session gives every
//! frame to its [`VideoSink`]. Today's sink is [`SharedFrameSink`]: the bytes converted to RGBA8
//! in [`crate::Shared::frame`], which the renderer, the webview path, screenshots and slot
//! thumbnails read. A GPU sink (3DS, #46 / #16) must keep `Shared::frame` filled too (a readback
//! when something asks for pixels), so those readers keep working.
//!
//! The types carry raw Vulkan handles as integers, so this crate never depends on a graphics
//! API binding; the shell turns them back into its own types (`gpu_interop.rs` in the app).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::screens::Split;
use crate::session::Shared;

/// A software frame: `height` rows of `width` pixels in `format` (`RETRO_PIXEL_FORMAT_*`), each
/// row `pitch` bytes from the previous one.
#[derive(Clone, Copy, Debug)]
pub struct CpuFrame<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub format: u32,
}

/// An image a hardware-rendering core drew into, valid until its next `retro_run`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuImage {
    /// The raw handle (`VkImage` as `u64`).
    pub handle: u64,
    /// Its format (`VkFormat`) and the layout it is in (`VkImageLayout`).
    pub format: u32,
    pub layout: u32,
}

/// What the host must respect before it reads a [`GpuImage`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuSync {
    /// Semaphores (`VkSemaphore` as `u64`) the first use must wait for.
    pub wait_semaphores: Vec<u64>,
    /// The queue family that owns the image.
    pub src_queue_family: u32,
}

/// A frame on the GPU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuFrame {
    pub width: u32,
    pub height: u32,
    pub image: GpuImage,
    pub sync: GpuSync,
}

/// One frame from the core.
#[derive(Clone, Debug)]
pub enum VideoFrame<'a> {
    Cpu(CpuFrame<'a>),
    Gpu(GpuFrame),
}

/// Takes the core's frames. Called from the video callback, on the thread the core calls it
/// from, while the session's callback context is held: it must not call into the core.
pub trait VideoSink: Send {
    /// A new frame; `split` is how the core laid its screens out in it.
    fn frame(&mut self, frame: VideoFrame<'_>, split: Split);
}

/// Today's sink: software frames into [`Shared::frame`] as RGBA8.
pub struct SharedFrameSink {
    shared: Arc<Shared>,
    /// A GPU frame was dropped (logged once).
    gpu_dropped: AtomicBool,
    /// A software frame was refused (logged once).
    cpu_refused: AtomicBool,
}

impl SharedFrameSink {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared,
            gpu_dropped: AtomicBool::new(false),
            cpu_refused: AtomicBool::new(false),
        }
    }
}

impl VideoSink for SharedFrameSink {
    fn frame(&mut self, frame: VideoFrame<'_>, split: Split) {
        match frame {
            VideoFrame::Cpu(cpu) => {
                let mut f = self.shared.frame.lock();
                if f.write_cpu(&cpu) {
                    f.split = split;
                } else if !self.cpu_refused.swap(true, Ordering::Relaxed) {
                    log::warn!(
                        "video: a {}×{} frame (pitch {}) larger than its buffer ({} bytes) was dropped",
                        cpu.width,
                        cpu.height,
                        cpu.pitch,
                        cpu.data.len()
                    );
                }
            }
            VideoFrame::Gpu(gpu) => {
                if !self.gpu_dropped.swap(true, Ordering::Relaxed) {
                    log::warn!(
                        "video: a GPU frame ({}×{}) reached a software-only sink: dropped",
                        gpu.width,
                        gpu.height
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use libretro_sys::RETRO_PIXEL_FORMAT_RGB565;

    use super::{CpuFrame, GpuFrame, GpuImage, GpuSync, SharedFrameSink, VideoFrame, VideoSink};
    use crate::screens::Split;
    use crate::session::Shared;

    #[test]
    fn the_shared_sink_keeps_todays_cpu_frames_and_drops_gpu_ones() {
        let shared = Arc::new(Shared::default());
        let mut sink = SharedFrameSink::new(shared.clone());
        let white = 0xffff_u16.to_ne_bytes();
        let split = Split::DsStacked { gap: 0 };
        sink.frame(
            VideoFrame::Cpu(CpuFrame {
                data: &white,
                width: 1,
                height: 1,
                pitch: 2,
                format: RETRO_PIXEL_FORMAT_RGB565,
            }),
            split,
        );
        {
            let f = shared.frame.lock();
            assert_eq!((f.width, f.height, f.seq), (1, 1, 1));
            assert_eq!(f.rgba, [255, 255, 255, 255]);
            assert_eq!(f.split, split);
        }
        sink.frame(
            VideoFrame::Gpu(GpuFrame {
                width: 400,
                height: 480,
                image: GpuImage {
                    handle: 1,
                    format: 37,
                    layout: 5,
                },
                sync: GpuSync::default(),
            }),
            Split::Whole,
        );
        assert_eq!(
            shared.frame.lock().seq,
            1,
            "a GPU frame changed the CPU one"
        );
    }
}
