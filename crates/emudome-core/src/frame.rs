//! The latest frame produced by the core, already converted to RGBA8.

use libretro_sys::{RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888};

use crate::screens::{SourceScreen, Split};

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

    /// Converts the core's buffer (with its `pitch`) to RGBA8.
    ///
    /// # Safety
    /// `data` must point to `height * pitch` valid bytes in the given format.
    pub(crate) unsafe fn write_from_core(
        &mut self,
        data: *const u8,
        width: u32,
        height: u32,
        pitch: usize,
        pixel_format: u32,
    ) {
        let (w, h) = (width as usize, height as usize);
        self.width = width;
        self.height = height;
        self.rgba.resize(w * h * 4, 0);
        for y in 0..h {
            let row = data.add(y * pitch);
            let out = &mut self.rgba[y * w * 4..(y + 1) * w * 4];
            match pixel_format {
                RETRO_PIXEL_FORMAT_XRGB8888 => {
                    let src = std::slice::from_raw_parts(row as *const u32, w);
                    for (px, o) in src.iter().zip(out.chunks_exact_mut(4)) {
                        o[0] = (px >> 16) as u8;
                        o[1] = (px >> 8) as u8;
                        o[2] = *px as u8;
                        o[3] = 255;
                    }
                }
                RETRO_PIXEL_FORMAT_RGB565 => {
                    let src = std::slice::from_raw_parts(row as *const u16, w);
                    for (px, o) in src.iter().zip(out.chunks_exact_mut(4)) {
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
                    let src = std::slice::from_raw_parts(row as *const u16, w);
                    for (px, o) in src.iter().zip(out.chunks_exact_mut(4)) {
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
    }
}
