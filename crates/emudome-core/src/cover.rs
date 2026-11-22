//! Covers made from the user's own image or from a screenshot (Trello #85, L12 «Adatta la
//! copertina»). Never downloaded: the source is a file the user picked or a screenshot taken on
//! this device (docs/decisions.md, «Covers»).
//!
//! Every such cover is one normalized copy: the library's 2:3 portrait, [`COVER_WIDTH`] ×
//! [`COVER_HEIGHT`], written again as a JPEG (quality [`JPEG_QUALITY`]), so nothing of the
//! source file survives but its pixels (no EXIF, no GPS, no colour profile, no comment).
//!
//! The interface decides what the cover shows, as a [`CoverView`]: the rectangle of the source,
//! in source pixels, that the 2:3 frame shows. It may reach past the image (the whole image
//! «Intera», smaller than the frame); there, the frame shows the same image filling it,
//! enlarged, blurred and dimmed ([`backdrop`]), as the interface's preview does.
//!
//! The source is untrusted input: at most [`MAX_SOURCE_BYTES`], [`MAX_SIDE`] pixels a side, and
//! PNG, JPEG or WebP only (the only codecs compiled in). A JPEG or WebP turned by its EXIF
//! orientation is turned here too, as the webview shows it.

use std::io::Cursor;

use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageDecoder, ImageReader, Limits, RgbImage, RgbaImage};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ErrorCode;

/// Width of every cover EmuDome writes.
pub const COVER_WIDTH: u32 = 600;
/// Height of every cover EmuDome writes (2:3 with [`COVER_WIDTH`]).
pub const COVER_HEIGHT: u32 = 900;
/// Largest source file accepted.
pub const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;
/// Largest side of a source image, in pixels.
pub const MAX_SIDE: u32 = 8192;
/// JPEG quality of the written cover.
pub const JPEG_QUALITY: u8 = 90;
/// The backdrop's brightness, 1 = unchanged: the preview's `game-scrim` veil (black at 60%,
/// app/src/assets/theme/colors.css) over the blurred image leaves 40%.
const BACKDROP_BRIGHTNESS: f32 = 0.4;
/// The backdrop is blurred at this size, then enlarged.
const BACKDROP_SMALL: (u32, u32) = (60, 90);
/// The backdrop's blur at [`BACKDROP_SMALL`], in its pixels: about the preview's `blur-dim`
/// (8 px on a 168 px wide cover, `cover-hero`), 5% of the width.
const BACKDROP_SIGMA: f32 = 3.0;
/// How far a view's shape may be from 2:3 (rounding in the interface).
const SHAPE_TOLERANCE: f64 = 0.01;
/// How much larger than the image a view may be (the interface's smallest zoom is far above).
const MAX_VIEW_SCALE: f64 = 16.0;

/// The part of the source the cover shows, in source pixels: `x`, `y` its top left corner,
/// `width` × `height` 2:3. It may start before 0 or end past the image. TypeScript: `CoverView`
/// (generated).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
pub struct CoverView {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn invalid(detail: impl Into<String>) -> ErrorCode {
    ErrorCode::CoverInvalid {
        detail: detail.into(),
    }
}

/// Decodes a source image within the limits, turned by its EXIF orientation.
pub fn decode(bytes: &[u8]) -> Result<RgbaImage, ErrorCode> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_SOURCE_BYTES {
        return Err(invalid(format!(
            "{} bytes, limit {MAX_SOURCE_BYTES}",
            bytes.len()
        )));
    }
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| invalid(e.to_string()))?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(u64::from(MAX_SIDE) * u64::from(MAX_SIDE) * 4 + (32 << 20));
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| invalid(e.to_string()))?;
    let (w, h) = decoder.dimensions();
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(invalid(format!("{w}×{h}, limit {MAX_SIDE} a side")));
    }
    let orientation = decoder.orientation().map_err(|e| invalid(e.to_string()))?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(|e| invalid(e.to_string()))?;
    image.apply_orientation(orientation);
    Ok(image.into_rgba8())
}

/// The size of a source image as shown (after its EXIF orientation).
pub fn dimensions(bytes: &[u8]) -> Result<(u32, u32), ErrorCode> {
    decode(bytes).map(|image| image.dimensions())
}

/// Whether `view` can be drawn from a `width` × `height` image.
fn check_view(view: &CoverView, width: u32, height: u32) -> Result<(), ErrorCode> {
    let values = [view.x, view.y, view.width, view.height];
    if values.iter().any(|v| !v.is_finite()) || view.width <= 0.0 || view.height <= 0.0 {
        return Err(invalid(format!("view {view:?}")));
    }
    let shape = view.width / view.height;
    let wanted = f64::from(COVER_WIDTH) / f64::from(COVER_HEIGHT);
    if (shape - wanted).abs() > wanted * SHAPE_TOLERANCE {
        return Err(invalid(format!("view {view:?} is not 2:3")));
    }
    let largest = f64::from(width.max(height)) * MAX_VIEW_SCALE;
    if view.height > largest || view.x.abs() > largest || view.y.abs() > largest {
        return Err(invalid(format!(
            "view {view:?} far from a {width}×{height} image"
        )));
    }
    Ok(())
}

/// The image filling the 2:3 frame (its middle, cropped), enlarged, blurred and dimmed: what
/// shows where the view reaches past the image.
fn backdrop(source: &RgbaImage) -> RgbaImage {
    let (w, h) = source.dimensions();
    let wanted = f64::from(COVER_WIDTH) / f64::from(COVER_HEIGHT);
    let (cw, ch) = if f64::from(w) / f64::from(h) > wanted {
        (((f64::from(h) * wanted).round() as u32).clamp(1, w), h)
    } else {
        (w, ((f64::from(w) / wanted).round() as u32).clamp(1, h))
    };
    let middle = imageops::crop_imm(source, (w - cw) / 2, (h - ch) / 2, cw, ch).to_image();
    let small = imageops::resize(
        &middle,
        BACKDROP_SMALL.0,
        BACKDROP_SMALL.1,
        FilterType::Triangle,
    );
    let mut small = imageops::blur(&small, BACKDROP_SIGMA);
    for pixel in small.pixels_mut() {
        for c in &mut pixel.0[..3] {
            *c = (f32::from(*c) * BACKDROP_BRIGHTNESS).round() as u8;
        }
        pixel.0[3] = 255;
    }
    imageops::resize(&small, COVER_WIDTH, COVER_HEIGHT, FilterType::Triangle)
}

/// The cover of `source` showing `view`, [`COVER_WIDTH`] × [`COVER_HEIGHT`].
pub fn render(source: &RgbaImage, view: CoverView) -> Result<RgbImage, ErrorCode> {
    let (w, h) = source.dimensions();
    check_view(&view, w, h)?;
    let mut cover = backdrop(source);
    let sx = f64::from(COVER_WIDTH) / view.width;
    let sy = f64::from(COVER_HEIGHT) / view.height;
    // The part of the image inside the view, in source pixels.
    let ix0 = view.x.max(0.0);
    let iy0 = view.y.max(0.0);
    let ix1 = (view.x + view.width).min(f64::from(w));
    let iy1 = (view.y + view.height).min(f64::from(h));
    // Where it lands on the cover.
    let dx0 = ((ix0 - view.x) * sx)
        .round()
        .clamp(0.0, f64::from(COVER_WIDTH));
    let dy0 = ((iy0 - view.y) * sy)
        .round()
        .clamp(0.0, f64::from(COVER_HEIGHT));
    let dx1 = ((ix1 - view.x) * sx)
        .round()
        .clamp(0.0, f64::from(COVER_WIDTH));
    let dy1 = ((iy1 - view.y) * sy)
        .round()
        .clamp(0.0, f64::from(COVER_HEIGHT));
    if ix1 > ix0 && iy1 > iy0 && dx1 > dx0 && dy1 > dy0 {
        let sx0 = ix0.floor() as u32;
        let sy0 = iy0.floor() as u32;
        let sx1 = (ix1.ceil() as u32).clamp(sx0 + 1, w);
        let sy1 = (iy1.ceil() as u32).clamp(sy0 + 1, h);
        let part = imageops::crop_imm(source, sx0, sy0, sx1 - sx0, sy1 - sy0).to_image();
        let part = imageops::resize(
            &part,
            (dx1 - dx0) as u32,
            (dy1 - dy0) as u32,
            FilterType::CatmullRom,
        );
        imageops::overlay(&mut cover, &part, dx0 as i64, dy0 as i64);
    }
    Ok(DynamicImage::ImageRgba8(cover).into_rgb8())
}

/// The cover as a JPEG: no metadata of any kind.
pub fn encode(cover: &RgbImage) -> Result<Vec<u8>, ErrorCode> {
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY)
        .encode_image(cover)
        .map_err(|e| invalid(e.to_string()))?;
    Ok(bytes)
}

/// The normalized cover of the source `bytes` showing `view`, as JPEG bytes.
pub fn make(bytes: &[u8], view: CoverView) -> Result<Vec<u8>, ErrorCode> {
    let source = decode(bytes)?;
    encode(&render(&source, view)?)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{ImageFormat, Rgba, RgbaImage};

    use super::{decode, dimensions, make, CoverView, COVER_HEIGHT, COVER_WIDTH, MAX_SIDE};
    use crate::ErrorCode;

    /// `w` × `h`, red on the left half and blue on the right.
    fn halves(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_fn(w, h, |x, _| {
            if x < w / 2 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        })
    }

    fn encoded(image: &RgbaImage, format: ImageFormat) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, format).unwrap();
        bytes.into_inner()
    }

    fn pixel(jpeg: &[u8], x: u32, y: u32) -> [u8; 3] {
        let image = image::load_from_memory(jpeg).unwrap().into_rgb8();
        assert_eq!(image.dimensions(), (COVER_WIDTH, COVER_HEIGHT));
        image.get_pixel(x, y).0
    }

    fn is_red(p: [u8; 3]) -> bool {
        p[0] > 220 && p[1] < 40 && p[2] < 40
    }

    fn is_blue(p: [u8; 3]) -> bool {
        p[2] > 220 && p[0] < 40 && p[1] < 40
    }

    fn invalid(result: Result<impl std::fmt::Debug, ErrorCode>) -> bool {
        matches!(result, Err(ErrorCode::CoverInvalid { .. }))
    }

    #[test]
    fn the_whole_image_sits_on_its_own_blurred_backdrop() {
        // A 3:2 screenshot, whole: 300 wide, the 2:3 view 450 high around it.
        let png = encoded(&halves(300, 200), ImageFormat::Png);
        let view = CoverView {
            x: 0.0,
            y: -125.0,
            width: 300.0,
            height: 450.0,
        };
        let cover = make(&png, view).unwrap();
        assert!(cover.starts_with(&[0xFF, 0xD8, 0xFF]), "a JPEG");
        // The image fills the middle band, 600 × 400 from y 250.
        assert!(is_red(pixel(&cover, 150, 450)));
        assert!(is_blue(pixel(&cover, 450, 450)));
        // Above it, the backdrop: the same colours, dimmed.
        let above = pixel(&cover, 100, 60);
        assert!(above[0] < 160 && above[0] > 30, "{above:?}");
        assert!(!is_red(above) && !is_blue(above));
    }

    #[test]
    fn a_view_inside_the_image_crops_it() {
        let png = encoded(&halves(300, 200), ImageFormat::Png);
        // The left 2:3 part: only red.
        let view = CoverView {
            x: 0.0,
            y: 0.0,
            width: 400.0 / 3.0,
            height: 200.0,
        };
        let cover = make(&png, view).unwrap();
        for (x, y) in [(5, 5), (300, 450), (594, 894)] {
            assert!(is_red(pixel(&cover, x, y)), "{x},{y}");
        }
    }

    #[test]
    fn jpeg_and_webp_sources_are_read_and_no_metadata_is_written() {
        let image = halves(200, 300);
        let view = CoverView {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 300.0,
        };
        for format in [ImageFormat::Jpeg, ImageFormat::WebP] {
            let source = if format == ImageFormat::Jpeg {
                let rgb = image::DynamicImage::ImageRgba8(image.clone()).into_rgb8();
                let mut bytes = Cursor::new(Vec::new());
                rgb.write_to(&mut bytes, format).unwrap();
                bytes.into_inner()
            } else {
                encoded(&image, format)
            };
            assert_eq!(dimensions(&source).unwrap(), (200, 300), "{format:?}");
            let cover = make(&source, view).unwrap();
            assert!(is_red(pixel(&cover, 100, 450)), "{format:?}");
            assert!(is_blue(pixel(&cover, 500, 450)), "{format:?}");
            for marker in [&b"Exif"[..], b"ICC_PROFILE", b"http://ns.adobe.com"] {
                assert!(
                    !cover.windows(marker.len()).any(|w| w == marker),
                    "{format:?}"
                );
            }
        }
    }

    #[test]
    fn refuses_what_is_not_a_usable_image_or_view() {
        assert!(invalid(decode(b"not an image at all")));
        // A GIF is not one of the codecs compiled in.
        assert!(invalid(decode(b"GIF89a\x01\x00\x01\x00\x00\x00\x00;")));
        let too_wide = encoded(
            &RgbaImage::from_pixel(MAX_SIDE + 1, 1, Rgba([0, 0, 0, 255])),
            ImageFormat::Png,
        );
        assert!(invalid(decode(&too_wide)));
        assert!(invalid(decode(&vec![0u8; 16 * 1024 * 1024 + 1])));
        let png = encoded(&halves(30, 20), ImageFormat::Png);
        let view = |x: f64, y: f64, width: f64, height: f64| CoverView {
            x,
            y,
            width,
            height,
        };
        for bad in [
            view(0.0, 0.0, 20.0, 20.0),
            view(0.0, 0.0, 0.0, 0.0),
            view(f64::NAN, 0.0, 20.0, 30.0),
            view(0.0, 0.0, f64::INFINITY, 30.0),
            view(0.0, 0.0, 2000.0, 3000.0),
            view(1.0e6, 0.0, 20.0, 30.0),
        ] {
            assert!(invalid(make(&png, bad)), "{bad:?}");
        }
        // A view past the image is the backdrop alone, not an error.
        let cover = make(&png, view(40.0, 0.0, 20.0, 30.0)).unwrap();
        assert!(!is_red(pixel(&cover, 300, 450)));
    }
}
