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

/// Wakes whoever draws the frames (the native renderer) when there is something new to show: a
/// frame in [`Shared::frame`], or a message of its own. A notification is kept until the waiter
/// takes it, so one sent before the wait is never lost.
#[derive(Default)]
pub struct Wake {
    pending: parking_lot::Mutex<bool>,
    cv: parking_lot::Condvar,
}

impl Wake {
    pub fn notify(&self) {
        *self.pending.lock() = true;
        self.cv.notify_one();
    }

    /// Waits for a notification, at most `timeout`; true when there was one.
    pub fn wait(&self, timeout: std::time::Duration) -> bool {
        let mut pending = self.pending.lock();
        if !*pending {
            self.cv.wait_for(&mut pending, timeout);
        }
        std::mem::take(&mut *pending)
    }
}

/// Takes the core's frames. Called from the video callback, on the thread the core calls it
/// from, while the session's callback context is held: it must not call into the core.
pub trait VideoSink: Send {
    /// A new frame; `split` is how the core laid its screens out in it.
    fn frame(&mut self, frame: VideoFrame<'_>, split: Split);
}

/// Today's sink: software frames into [`Shared::frame`] as RGBA8.
///
/// The conversion runs into a back buffer of the sink's own, outside the frame's lock; the lock is
/// held only to swap the new pixels in (an `Arc`), so the renderer, the webview protocol and the
/// PNG writers never wait for a conversion, nor the core for them. The buffer swapped out comes
/// back as the next back buffer once no reader holds it (else a new one is made).
pub struct SharedFrameSink {
    shared: Arc<Shared>,
    /// Where the next frame is converted.
    back: Vec<u8>,
    /// A GPU frame was dropped (logged once).
    gpu_dropped: AtomicBool,
    /// A software frame was refused (logged once).
    cpu_refused: AtomicBool,
}

impl SharedFrameSink {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared,
            back: Vec::new(),
            gpu_dropped: AtomicBool::new(false),
            cpu_refused: AtomicBool::new(false),
        }
    }
}

impl VideoSink for SharedFrameSink {
    fn frame(&mut self, frame: VideoFrame<'_>, split: Split) {
        match frame {
            VideoFrame::Cpu(cpu) => {
                if crate::frame::convert_cpu(&cpu, &mut self.back) {
                    let fresh = Arc::new(std::mem::take(&mut self.back));
                    let old = {
                        let mut f = self.shared.frame.lock();
                        f.width = cpu.width;
                        f.height = cpu.height;
                        f.split = split;
                        f.seq = f.seq.wrapping_add(1);
                        std::mem::replace(&mut f.rgba, fresh)
                    };
                    if let Some(wake) = self.shared.frame_wake.lock().as_ref() {
                        wake.notify();
                    }
                    // The previous pixels, when no reader holds them any more.
                    if let Ok(buffer) = Arc::try_unwrap(old) {
                        self.back = buffer;
                    }
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
            assert_eq!(*f.rgba, [255, 255, 255, 255]);
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

    /// A notification sent before the wait is kept; none times out; the drawer registered in
    /// `Shared::frame_wake` is woken by each new frame.
    #[test]
    fn the_wake_keeps_a_notification_and_new_frames_send_one() {
        use std::time::{Duration, Instant};
        let wake = Arc::new(super::Wake::default());
        wake.notify();
        assert!(wake.wait(Duration::from_secs(5)));
        let t = Instant::now();
        assert!(!wake.wait(Duration::from_millis(20)));
        assert!(t.elapsed() >= Duration::from_millis(15));

        let shared = Arc::new(Shared::default());
        *shared.frame_wake.lock() = Some(wake.clone());
        let mut sink = SharedFrameSink::new(shared);
        let px = 0u16.to_ne_bytes();
        sink.frame(
            VideoFrame::Cpu(CpuFrame {
                data: &px,
                width: 1,
                height: 1,
                pitch: 2,
                format: RETRO_PIXEL_FORMAT_RGB565,
            }),
            Split::Whole,
        );
        assert!(wake.wait(Duration::ZERO));
    }

    /// A reader keeps the pixels it took while new frames come in; once nobody holds a buffer it
    /// is converted into again (no allocation per frame).
    #[test]
    fn frames_are_swapped_in_and_buffers_reused() {
        let shared = Arc::new(Shared::default());
        let mut sink = SharedFrameSink::new(shared.clone());
        let pixel = |v: u16| {
            let bytes = v.to_ne_bytes();
            move |sink: &mut SharedFrameSink| {
                sink.frame(
                    VideoFrame::Cpu(CpuFrame {
                        data: &bytes,
                        width: 1,
                        height: 1,
                        pitch: 2,
                        format: RETRO_PIXEL_FORMAT_RGB565,
                    }),
                    Split::Whole,
                )
            }
        };
        pixel(0xffff)(&mut sink);
        let held = shared.frame.lock().clone();
        pixel(0x0000)(&mut sink);
        // The reader's copy did not change under it.
        assert_eq!(*held.rgba, [255, 255, 255, 255]);
        assert_eq!(*shared.frame.lock().rgba, [0, 0, 0, 255]);
        let second = shared.frame.lock().rgba.as_ptr();
        drop(held);
        // Frame 3 needs a new buffer (frame 1's was held when frame 2 came); frame 2's buffer
        // comes back to the sink and frame 4 is converted into it.
        pixel(0xffff)(&mut sink);
        pixel(0x0000)(&mut sink);
        let f = shared.frame.lock();
        assert_eq!(f.seq, 4);
        assert_eq!(*f.rgba, [0, 0, 0, 255]);
        assert_eq!(f.rgba.as_ptr(), second, "frame 2's buffer reused");
    }
}
