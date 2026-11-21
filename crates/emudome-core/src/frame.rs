//! The latest frame produced by the core, already converted to RGBA8.

use libretro_sys::{RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888};

use crate::screens::{SourceScreen, Split};
use crate::video_sink::CpuFrame;

#[derive(Default, Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// RGBA8 pixels, row after row, no padding.
    pub rgba: Vec<u8>,
    /// Grows with every new frame: the renderer uploads the texture only when it changes.
    pub seq: u64,
    /// How the core laid its screens out in this frame (NDS: two, one above the other).
    pub split: Split,
}

impl Frame {
    /// The screens inside this frame, each with its rectangle.
    pub fn screens(&self) -> Vec<SourceScreen> {
        self.split.screens(self.width, self.height)
    }

    /// The frame as a PNG file (slot thumbnails, screenshots). Errors on an empty frame.
    pub fn to_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(&self.rgba)?;
            writer.finish()?;
        }
        Ok(out)
    }

    /// Converts a software frame from the core ([`CpuFrame`]: its bytes with their `pitch`) to
    /// RGBA8. No alignment is assumed: the rows are read as bytes, so a core may hand any address
    /// and any pitch. A buffer shorter than its size says is refused (false) and the frame kept.
    pub fn write_cpu(&mut self, src: &CpuFrame<'_>) -> bool {
        let (w, h) = (src.width as usize, src.height as usize);
        let bpp = bytes_per_pixel(src.format);
        if h > 0 && src.data.len() < src.pitch * (h - 1) + w * bpp {
            return false;
        }
        self.width = src.width;
        self.height = src.height;
        self.rgba.resize(w * h * 4, 0);
        for y in 0..h {
            let row = &src.data[y * src.pitch..y * src.pitch + w * bpp];
            let out = &mut self.rgba[y * w * 4..(y + 1) * w * 4];
            match src.format {
                RETRO_PIXEL_FORMAT_XRGB8888 => {
                    for (px, o) in row.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
                        let px = u32::from_ne_bytes([px[0], px[1], px[2], px[3]]);
                        o[0] = (px >> 16) as u8;
                        o[1] = (px >> 8) as u8;
                        o[2] = px as u8;
                        o[3] = 255;
                    }
                }
                RETRO_PIXEL_FORMAT_RGB565 => {
                    for (px, o) in row.chunks_exact(2).zip(out.chunks_exact_mut(4)) {
                        let px = u16::from_ne_bytes([px[0], px[1]]);
                        let r = (px >> 11) & 0x1f;
                        let g = (px >> 5) & 0x3f;
                        let b = px & 0x1f;
                        o[0] = ((r << 3) | (r >> 2)) as u8;
                        o[1] = ((g << 2) | (g >> 4)) as u8;
                        o[2] = ((b << 3) | (b >> 2)) as u8;
                        o[3] = 255;
                    }
                }
                _ => {
                    // 0RGB1555, libretro's default format.
                    for (px, o) in row.chunks_exact(2).zip(out.chunks_exact_mut(4)) {
                        let px = u16::from_ne_bytes([px[0], px[1]]);
                        let r = (px >> 10) & 0x1f;
                        let g = (px >> 5) & 0x1f;
                        let b = px & 0x1f;
                        o[0] = ((r << 3) | (r >> 2)) as u8;
                        o[1] = ((g << 3) | (g >> 2)) as u8;
                        o[2] = ((b << 3) | (b >> 2)) as u8;
                        o[3] = 255;
                    }
                }
            }
        }
        self.seq = self.seq.wrapping_add(1);
        true
    }
}

/// Bytes per pixel of a libretro pixel format (`RETRO_PIXEL_FORMAT_*`).
pub(crate) fn bytes_per_pixel(format: u32) -> usize {
    if format == RETRO_PIXEL_FORMAT_XRGB8888 {
        4
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use libretro_sys::{RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888};

    use super::Frame;
    use crate::video_sink::CpuFrame;

    #[test]
    fn a_frame_at_an_odd_address_converts_without_assuming_alignment() {
        // Two XRGB8888 pixels starting one byte into the buffer: not 4-byte aligned.
        let mut buf = [0u8; 1 + 8];
        buf[1..5].copy_from_slice(&0x00ff_8000_u32.to_ne_bytes());
        buf[5..9].copy_from_slice(&0x0000_00ff_u32.to_ne_bytes());
        let mut frame = Frame::default();
        let cpu = |data, width, pitch, format| CpuFrame {
            data,
            width,
            height: 1,
            pitch,
            format,
        };
        assert!(frame.write_cpu(&cpu(&buf[1..], 2, 8, RETRO_PIXEL_FORMAT_XRGB8888)));
        assert_eq!(frame.rgba, [255, 128, 0, 255, 0, 0, 255, 255]);

        // One RGB565 white pixel at an odd address.
        let mut buf = [0u8; 3];
        buf[1..3].copy_from_slice(&0xffff_u16.to_ne_bytes());
        assert!(frame.write_cpu(&cpu(&buf[1..], 1, 2, RETRO_PIXEL_FORMAT_RGB565)));
        assert_eq!(frame.rgba, [255, 255, 255, 255]);

        // A buffer shorter than its size: refused, the last frame kept.
        let seq = frame.seq;
        assert!(!frame.write_cpu(&cpu(&buf[1..], 2, 4, RETRO_PIXEL_FORMAT_RGB565)));
        assert_eq!((frame.seq, frame.width), (seq, 1));
    }
}
