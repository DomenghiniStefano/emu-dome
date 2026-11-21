//! The game's screens: where each one is in the core's frame, where EmuDome draws it in the
//! window, and how a point in the window goes back to a touch on the console's touch screen.
//!
//! NDS (and later 3DS) screens are composed by EmuDome (`docs/decisions.md`, option B): the core
//! is asked for one fixed arrangement ([`Split`], e.g. melonDS DS's `top-bottom` with no gap,
//! [`crate::nds`]), the shell cuts the frame into its screens and places them itself with a
//! [`Layout`]. Everything here is pure and unit-tested:
//!
//! - [`Split::screens`]: frame size → each screen's source rectangle in the frame;
//! - [`compose`]: layout + screens + window → each screen's rectangle in the window;
//! - [`touch_at`]: window point → touch-screen pixel, through the composed rectangle;
//! - [`pointer_value`]: touch-screen pixel → the `RETRO_DEVICE_POINTER` value the core reads.
//!
//! Today the renderers draw the whole frame with [`fit`], which for the default layout (stacked,
//! no gap, no rotation) puts every screen exactly where [`compose`] does (tested below).
//! TODO(#28): the `screens.*` settings choose the [`Layout`]; the renderers then draw each
//! [`Placement`] instead of the whole frame.

/// Which screen of the console.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenId {
    /// The only screen of a one-screen console (GB, GBC, GBA).
    Main,
    /// NDS / 3DS upper screen.
    Top,
    /// NDS / 3DS lower screen: the touch screen.
    Bottom,
}

/// A rectangle of whole pixels in the core's frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PxRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// A rectangle in window pixels (the renderer's unit: physical pixels for the native path, CSS
/// pixels for the webview canvas).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

/// One screen inside the core's frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceScreen {
    pub id: ScreenId,
    pub src: PxRect,
}

/// How the core lays its screens out in the frame it hands over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Split {
    /// One screen: the whole frame.
    #[default]
    Whole,
    /// NDS: top screen above the bottom one, same width, `gap` frame rows between them.
    DsStacked { gap: u32 },
    /// 3DS (Azahar's `default` layout): the 400 × 240 top screen above the 320 × 240 bottom
    /// one, which is centred under it, no gap; both scaled alike when the core upscales.
    N3dsStacked,
}

/// The 3DS top screen (width, height) and the bottom screen's width (same height), at 1×.
const N3DS_TOP: (u32, u32) = (400, 240);
const N3DS_BOTTOM_WIDTH: u32 = 320;

impl Split {
    /// The screens inside a frame of `width` × `height`. A frame too small for the split (none
    /// yet, or a core that ignored the options) is taken as one screen.
    pub fn screens(self, width: u32, height: u32) -> Vec<SourceScreen> {
        let whole = || {
            vec![SourceScreen {
                id: ScreenId::Main,
                src: PxRect {
                    x: 0,
                    y: 0,
                    w: width,
                    h: height,
                },
            }]
        };
        match self {
            Split::Whole => whole(),
            Split::DsStacked { gap } => {
                // Two 4:3 screens (256 × 192, or a multiple when the core upscales).
                if height <= gap || !(height - gap).is_multiple_of(2) {
                    return whole();
                }
                let h = (height - gap) / 2;
                if width == 0 || h * 4 != width * 3 {
                    return whole();
                }
                let screen = |id, y| SourceScreen {
                    id,
                    src: PxRect {
                        x: 0,
                        y,
                        w: width,
                        h,
                    },
                };
                vec![screen(ScreenId::Top, 0), screen(ScreenId::Bottom, h + gap)]
            }
            Split::N3dsStacked => {
                // 400 × 480 times a whole factor (Azahar's internal resolution).
                let scale = width / N3DS_TOP.0;
                if scale == 0 || width != N3DS_TOP.0 * scale || height != 2 * N3DS_TOP.1 * scale {
                    return whole();
                }
                let (top_w, bottom_w, h) = (
                    N3DS_TOP.0 * scale,
                    N3DS_BOTTOM_WIDTH * scale,
                    N3DS_TOP.1 * scale,
                );
                vec![
                    SourceScreen {
                        id: ScreenId::Top,
                        src: PxRect {
                            x: 0,
                            y: 0,
                            w: top_w,
                            h,
                        },
                    },
                    SourceScreen {
                        id: ScreenId::Bottom,
                        src: PxRect {
                            x: (top_w - bottom_w) / 2,
                            y: h,
                            w: bottom_w,
                            h,
                        },
                    },
                ]
            }
        }
    }
}

/// How the screens are arranged (`screens.cycle` ids: `vertical`, `side`, `big_small`,
/// `top_only`; `bottom_only` and custom come with the screen editor, #28).
#[derive(Debug, Clone, PartialEq)]
pub enum Arrangement {
    /// One above the other («Uno sopra l'altro»): the default.
    Stacked,
    /// Next to each other.
    SideBySide,
    /// The first screen large on the left, the second small on its right, top-aligned; the large
    /// one takes `Layout::ratio` per cent of the width.
    BigSmall,
    /// Only the first screen.
    FirstOnly,
    /// Only the second screen.
    SecondOnly,
    /// Each screen anywhere: rectangles as fractions (0..1) of the game area, drawn as given
    /// (no fitting, no aspect kept). A screen not listed is not drawn.
    Custom(Vec<(ScreenId, Rect)>),
}

impl Arrangement {
    /// The arrangement of a `screens.cycle` value; `None` for an id this build does not know.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "vertical" => Some(Arrangement::Stacked),
            "side" => Some(Arrangement::SideBySide),
            "big_small" => Some(Arrangement::BigSmall),
            "top_only" => Some(Arrangement::FirstOnly),
            "bottom_only" => Some(Arrangement::SecondOnly),
            _ => None,
        }
    }
}

/// The whole picture turned (`screens.rot`), book-style games held sideways.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rotation {
    #[default]
    None,
    /// A quarter turn anticlockwise.
    Left,
    /// A quarter turn clockwise.
    Right,
    /// Upside down.
    Flip,
}

impl Rotation {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "none" => Some(Rotation::None),
            "left" => Some(Rotation::Left),
            "right" => Some(Rotation::Right),
            "flip" => Some(Rotation::Flip),
            _ => None,
        }
    }
}

/// How EmuDome places the screens (the `screens.*` settings, #28).
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub arrangement: Arrangement,
    /// `screens.swap`: the bottom screen comes first (on top, on the left, large).
    pub swap: bool,
    /// `screens.gap`: space between the screens, in the console's pixels (scaled with them).
    pub gap: u32,
    /// `screens.ratio`: the large screen's share of the width in [`Arrangement::BigSmall`], per
    /// cent (clamped to 50..=90).
    pub ratio: u8,
    pub rotation: Rotation,
}

impl Default for Layout {
    /// «Uno sopra l'altro»: top above bottom, no gap, not turned.
    fn default() -> Self {
        Self {
            arrangement: Arrangement::Stacked,
            swap: false,
            gap: 0,
            ratio: 70,
            rotation: Rotation::None,
        }
    }
}

/// The part of the window the game may use: `inset` is the bottom share (0..0.9) kept clear for
/// the quick menu's bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub width: f32,
    pub height: f32,
    pub inset: f32,
}

/// Where one screen is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub id: ScreenId,
    /// Its pixels in the core's frame.
    pub src: PxRect,
    /// Its rectangle in the window.
    pub dst: Rect,
    /// How its picture is turned inside `dst`.
    pub rotation: Rotation,
}

/// A rectangle of `content` size fitted in `area`: integer scale when it fits, otherwise the
/// largest that keeps the aspect ratio, centred in the part above the inset. `(x, y, w, h)`.
/// The one rule of both renderers (native `video.rs`, webview `GameCanvas.vue`).
pub fn fit(area: Area, content: (f32, f32)) -> Rect {
    let (sw, full_h) = (area.width, area.height);
    let (w, h) = content;
    let sh = full_h * (1.0 - area.inset.clamp(0.0, 0.9));
    if w <= 0.0 || h <= 0.0 {
        return Rect::default();
    }
    let fit = (sw / w).min(sh / h);
    let scale = if fit >= 1.0 { fit.floor() } else { fit };
    let (vw, vh) = (w * scale, h * scale);
    Rect {
        x: ((sw - vw) / 2.0).floor(),
        y: ((sh - vh) / 2.0).floor(),
        w: vw,
        h: vh,
    }
}

/// Places `screens` in `area` as `layout` says. The screens are first laid out at their own size
/// (gap in the same pixels), the whole is turned, then fitted with [`fit`]; a custom
/// arrangement is placed as given instead.
pub fn compose(layout: &Layout, screens: &[SourceScreen], area: Area) -> Vec<Placement> {
    let mut order: Vec<SourceScreen> = screens.to_vec();
    if layout.swap {
        order.reverse();
    }

    if let Arrangement::Custom(rects) = &layout.arrangement {
        let usable = Area {
            height: area.height * (1.0 - area.inset.clamp(0.0, 0.9)),
            ..area
        };
        return rects
            .iter()
            .filter_map(|(id, r)| {
                let s = screens.iter().find(|s| s.id == *id)?;
                Some(Placement {
                    id: s.id,
                    src: s.src,
                    dst: Rect {
                        x: r.x * usable.width,
                        y: r.y * usable.height,
                        w: r.w * usable.width,
                        h: r.h * usable.height,
                    },
                    rotation: layout.rotation,
                })
            })
            .collect();
    }

    // Each screen's rectangle at its own size, in a canvas starting at (0, 0).
    let gap = layout.gap as f32;
    let size = |s: &SourceScreen| (s.src.w as f32, s.src.h as f32);
    let mut placed: Vec<(SourceScreen, Rect)> = Vec::new();
    match (&layout.arrangement, order.as_slice()) {
        (_, []) => {}
        (_, [only])
        | (Arrangement::FirstOnly, [only, ..])
        | (Arrangement::SecondOnly, [.., only]) => {
            let (w, h) = size(only);
            placed.push((
                *only,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w,
                    h,
                },
            ));
        }
        (Arrangement::Stacked, [first, second, ..]) => {
            let ((w1, h1), (w2, h2)) = (size(first), size(second));
            let width = w1.max(w2);
            placed.push((
                *first,
                Rect {
                    x: (width - w1) / 2.0,
                    y: 0.0,
                    w: w1,
                    h: h1,
                },
            ));
            placed.push((
                *second,
                Rect {
                    x: (width - w2) / 2.0,
                    y: h1 + gap,
                    w: w2,
                    h: h2,
                },
            ));
        }
        (Arrangement::SideBySide, [first, second, ..]) => {
            let ((w1, h1), (w2, h2)) = (size(first), size(second));
            let height = h1.max(h2);
            placed.push((
                *first,
                Rect {
                    x: 0.0,
                    y: (height - h1) / 2.0,
                    w: w1,
                    h: h1,
                },
            ));
            placed.push((
                *second,
                Rect {
                    x: w1 + gap,
                    y: (height - h2) / 2.0,
                    w: w2,
                    h: h2,
                },
            ));
        }
        (Arrangement::BigSmall, [first, second, ..]) => {
            let ((w1, h1), (w2, h2)) = (size(first), size(second));
            let ratio = f32::from(layout.ratio.clamp(50, 90)) / 100.0;
            // big / (big + small) = ratio, the big one at its own size.
            let small_w = w1 * (1.0 - ratio) / ratio;
            let small_h = h2 * small_w / w2;
            placed.push((
                *first,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: w1,
                    h: h1,
                },
            ));
            placed.push((
                *second,
                Rect {
                    x: w1 + gap,
                    y: 0.0,
                    w: small_w,
                    h: small_h,
                },
            ));
        }
        (Arrangement::Custom(_), _) => {}
    }
    if placed.is_empty() {
        return Vec::new();
    }

    let canvas_w = placed.iter().map(|(_, r)| r.x + r.w).fold(0.0, f32::max);
    let canvas_h = placed.iter().map(|(_, r)| r.y + r.h).fold(0.0, f32::max);
    let (turned_w, turned_h) = match layout.rotation {
        Rotation::None | Rotation::Flip => (canvas_w, canvas_h),
        Rotation::Left | Rotation::Right => (canvas_h, canvas_w),
    };
    let target = fit(area, (turned_w, turned_h));
    let scale = target.w / turned_w;

    placed
        .into_iter()
        .map(|(s, r)| {
            let turned = turn(r, layout.rotation, canvas_w, canvas_h);
            Placement {
                id: s.id,
                src: s.src,
                dst: Rect {
                    x: target.x + turned.x * scale,
                    y: target.y + turned.y * scale,
                    w: turned.w * scale,
                    h: turned.h * scale,
                },
                rotation: layout.rotation,
            }
        })
        .collect()
}

/// `r` inside a canvas of `cw` × `ch`, once the canvas is turned.
fn turn(r: Rect, rotation: Rotation, cw: f32, ch: f32) -> Rect {
    match rotation {
        Rotation::None => r,
        // (X, Y) → (Y, cw − X)
        Rotation::Left => Rect {
            x: r.y,
            y: cw - r.x - r.w,
            w: r.h,
            h: r.w,
        },
        // (X, Y) → (ch − Y, X)
        Rotation::Right => Rect {
            x: ch - r.y - r.h,
            y: r.x,
            w: r.h,
            h: r.w,
        },
        Rotation::Flip => Rect {
            x: cw - r.x - r.w,
            y: ch - r.y - r.h,
            w: r.w,
            h: r.h,
        },
    }
}

/// The touch-screen pixel under the window point `(x, y)`: `None` when the point is not on the
/// bottom screen (a touch that leaves the screen is a release, as on the console).
pub fn touch_at(placements: &[Placement], x: f32, y: f32) -> Option<(u32, u32)> {
    let p = placements.iter().find(|p| p.id == ScreenId::Bottom)?;
    if !p.dst.contains(x, y) {
        return None;
    }
    let u = (x - p.dst.x) / p.dst.w;
    let v = (y - p.dst.y) / p.dst.h;
    // Back from the turned picture to the screen's own axes (see `turn`).
    let (sx, sy) = match p.rotation {
        Rotation::None => (u, v),
        Rotation::Left => (1.0 - v, u),
        Rotation::Right => (v, 1.0 - u),
        Rotation::Flip => (1.0 - u, 1.0 - v),
    };
    let px = ((sx * p.src.w as f32) as u32).min(p.src.w.saturating_sub(1));
    let py = ((sy * p.src.h as f32) as u32).min(p.src.h.saturating_sub(1));
    Some((px, py))
}

/// `RETRO_DEVICE_POINTER`'s range: -0x7FFF (left / top edge) to 0x7FFF (right / bottom edge).
const POINTER_MAX: f32 = 32767.0;

/// The `RETRO_DEVICE_POINTER` value for pixel `(x, y)` of the screen at `src` in a frame of
/// `frame` size. libretro's pointer spans the core's whole frame; the core maps it back to its
/// own layout (melonDS DS: `bufferSize / 2 + p × bufferSize / (2 × 0x7FFF)`, truncated), so the
/// value aims at the pixel's centre to land on that pixel.
pub fn pointer_value(src: PxRect, frame: (u32, u32), x: u32, y: u32) -> (i16, i16) {
    let axis = |pixel: u32, size: u32| -> i16 {
        if size == 0 {
            return 0;
        }
        let centre = pixel as f32 + 0.5;
        let v = (centre * 2.0 / size as f32 - 1.0) * POINTER_MAX;
        v.round().clamp(-POINTER_MAX, POINTER_MAX) as i16
    };
    (axis(src.x + x, frame.0), axis(src.y + y, frame.1))
}

/// The whole path: the pointer value for the window point `(x, y)` with `layout` in `area`, for
/// a frame of `frame` size cut by `split`; `None` off the touch screen (or with no touch screen).
pub fn pointer_at(
    layout: &Layout,
    split: Split,
    frame: (u32, u32),
    area: Area,
    x: f32,
    y: f32,
) -> Option<(i16, i16)> {
    let screens = split.screens(frame.0, frame.1);
    let placements = compose(layout, &screens, area);
    let (tx, ty) = touch_at(&placements, x, y)?;
    let bottom = screens.iter().find(|s| s.id == ScreenId::Bottom)?;
    Some(pointer_value(bottom.src, frame, tx, ty))
}

#[cfg(test)]
mod tests {
    use super::{
        compose, fit, pointer_at, pointer_value, touch_at, Area, Arrangement, Layout, Placement,
        PxRect, Rect, Rotation, ScreenId, Split,
    };

    const DS: Split = Split::DsStacked { gap: 0 };
    const FRAME: (u32, u32) = (256, 384);

    fn area(width: f32, height: f32) -> Area {
        Area {
            width,
            height,
            inset: 0.0,
        }
    }

    fn ds(layout: &Layout, a: Area) -> Vec<Placement> {
        compose(layout, &DS.screens(FRAME.0, FRAME.1), a)
    }

    fn dst(placements: &[Placement], id: ScreenId) -> Rect {
        placements.iter().find(|p| p.id == id).unwrap().dst
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    /// melonDS DS's own reading of a pointer value (screenlayout.cpp, `pointerMatrix`).
    fn melonds_reads(p: (i16, i16), buffer: (u32, u32)) -> (i32, i32) {
        let axis = |v: i16, size: u32| {
            let size = size as f32;
            (size / 2.0 + f32::from(v) * size / (2.0 * 32767.0)) as i32
        };
        (axis(p.0, buffer.0), axis(p.1, buffer.1))
    }

    #[test]
    fn the_ds_frame_holds_two_screens() {
        let s = DS.screens(256, 384);
        assert_eq!(s.len(), 2);
        assert_eq!(
            (s[0].id, s[0].src),
            (
                ScreenId::Top,
                PxRect {
                    x: 0,
                    y: 0,
                    w: 256,
                    h: 192
                }
            )
        );
        assert_eq!(
            (s[1].id, s[1].src),
            (
                ScreenId::Bottom,
                PxRect {
                    x: 0,
                    y: 192,
                    w: 256,
                    h: 192
                }
            )
        );
        // Upscaled by the core (OpenGL renderer, later).
        assert_eq!(
            DS.screens(512, 768)[1].src,
            PxRect {
                x: 0,
                y: 384,
                w: 512,
                h: 384
            }
        );
        let gap = Split::DsStacked { gap: 90 }.screens(256, 474);
        assert_eq!(gap[1].src.y, 282);
    }

    #[test]
    fn the_3ds_frame_holds_a_wide_top_and_a_centred_bottom() {
        let s = Split::N3dsStacked.screens(400, 480);
        assert_eq!(
            s.iter().map(|s| (s.id, s.src)).collect::<Vec<_>>(),
            vec![
                (
                    ScreenId::Top,
                    PxRect {
                        x: 0,
                        y: 0,
                        w: 400,
                        h: 240
                    }
                ),
                (
                    ScreenId::Bottom,
                    PxRect {
                        x: 40,
                        y: 240,
                        w: 320,
                        h: 240
                    }
                ),
            ]
        );
        // Upscaled 3x by the core.
        assert_eq!(
            Split::N3dsStacked.screens(1200, 1440)[1].src,
            PxRect {
                x: 120,
                y: 720,
                w: 960,
                h: 720
            }
        );
        // Anything else (no frame yet, another layout) is one screen.
        for (w, h) in [(0, 0), (400, 240), (800, 480), (401, 480)] {
            assert_eq!(Split::N3dsStacked.screens(w, h).len(), 1, "{w}x{h}");
        }
    }

    #[test]
    fn the_default_3ds_layout_draws_where_the_whole_frame_is_drawn() {
        // The renderers draw the whole frame with `fit`; compose must put each screen at the
        // same place, or touch would miss.
        let a = area(1920.0, 1080.0);
        let whole = fit(a, (400.0, 480.0));
        let placed = compose(&Layout::default(), &Split::N3dsStacked.screens(400, 480), a);
        let scale = whole.w / 400.0;
        assert_eq!(
            dst(&placed, ScreenId::Top),
            rect(whole.x, whole.y, 400.0 * scale, 240.0 * scale)
        );
        assert_eq!(
            dst(&placed, ScreenId::Bottom),
            rect(
                whole.x + 40.0 * scale,
                whole.y + 240.0 * scale,
                320.0 * scale,
                240.0 * scale
            )
        );
    }

    #[test]
    fn a_3ds_touch_lands_on_the_same_bottom_screen_pixel_in_azahar() {
        // Azahar (`input/mouse_tracker.cpp`): x = (p + 0x7fff) / (0x7fff * 2) * width, then minus
        // the bottom screen's left edge.
        let frame = (400u32, 480u32);
        let bottom = Split::N3dsStacked.screens(frame.0, frame.1)[1].src;
        let azahar =
            |v: i16, size: u32| ((f32::from(v) + 32767.0) / (32767.0 * 2.0) * size as f32) as i32;
        for (x, y) in [(0, 0), (319, 239), (160, 120), (7, 200)] {
            let (px, py) = pointer_value(bottom, frame, x, y);
            assert_eq!(
                (
                    azahar(px, frame.0) - bottom.x as i32,
                    azahar(py, frame.1) - bottom.y as i32
                ),
                (x as i32, y as i32)
            );
        }
    }

    #[test]
    fn a_frame_that_does_not_fit_the_split_is_one_screen() {
        for (w, h) in [(0, 0), (256, 383), (240, 160)] {
            let s = DS.screens(w, h);
            assert_eq!(s.len(), 1, "{w}x{h}");
            assert_eq!(s[0].id, ScreenId::Main);
        }
        assert_eq!(Split::Whole.screens(240, 160)[0].src.w, 240);
    }

    #[test]
    fn fit_uses_an_integer_scale_when_it_fits() {
        assert_eq!(
            fit(area(1440.0, 900.0), (240.0, 160.0)),
            rect(120.0, 50.0, 1200.0, 800.0)
        );
        // The DS stack at 1080p: 2x (768 px of 1080), centred.
        assert_eq!(
            fit(area(1920.0, 1080.0), (256.0, 384.0)),
            rect(704.0, 156.0, 512.0, 768.0)
        );
        // Smaller than one console pixel: the largest fractional scale.
        let small = fit(area(200.0, 300.0), (256.0, 384.0));
        assert!((small.w - 200.0).abs() < 0.01 && (small.h - 300.0).abs() < 0.01);
    }

    #[test]
    fn fit_keeps_the_inset_clear() {
        let r = fit(
            Area {
                width: 1440.0,
                height: 900.0,
                inset: 0.3,
            },
            (240.0, 160.0),
        );
        assert!(r.y + r.h <= 630.0);
        assert_eq!(fit(area(10.0, 10.0), (0.0, 5.0)), Rect::default());
    }

    #[test]
    fn the_default_layout_draws_where_the_whole_frame_is_drawn() {
        // The renderers draw the whole frame with `fit`: the default layout must agree.
        for (w, h) in [
            (1920.0, 1080.0),
            (1280.0, 720.0),
            (800.0, 1280.0),
            (300.0, 200.0),
        ] {
            let whole = fit(area(w, h), (256.0, 384.0));
            let p = ds(&Layout::default(), area(w, h));
            let top = dst(&p, ScreenId::Top);
            let bottom = dst(&p, ScreenId::Bottom);
            assert_eq!(
                top,
                rect(whole.x, whole.y, whole.w, whole.h / 2.0),
                "{w}x{h}"
            );
            assert_eq!(
                bottom,
                rect(whole.x, whole.y + whole.h / 2.0, whole.w, whole.h / 2.0),
                "{w}x{h}"
            );
        }
    }

    #[test]
    fn swap_puts_the_bottom_screen_first() {
        let p = ds(
            &Layout {
                swap: true,
                ..Layout::default()
            },
            area(1920.0, 1080.0),
        );
        assert!(dst(&p, ScreenId::Bottom).y < dst(&p, ScreenId::Top).y);
    }

    #[test]
    fn the_gap_scales_with_the_screens() {
        // 256 x (192 + 90 + 192) = 256 x 474 fits 2x in 1080 rows.
        let p = ds(
            &Layout {
                gap: 90,
                ..Layout::default()
            },
            area(1920.0, 1080.0),
        );
        let (top, bottom) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
        assert_eq!(top.h, 384.0);
        assert_eq!(bottom.y - (top.y + top.h), 180.0);
    }

    #[test]
    fn side_by_side_sits_on_one_row() {
        let p = ds(
            &Layout {
                arrangement: Arrangement::SideBySide,
                ..Layout::default()
            },
            area(1920.0, 1080.0),
        );
        let (top, bottom) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
        // 512 x 192 fits 3x: 1536 x 576.
        assert_eq!(top, rect(192.0, 252.0, 768.0, 576.0));
        assert_eq!(bottom, rect(960.0, 252.0, 768.0, 576.0));
    }

    #[test]
    fn big_small_gives_the_large_screen_its_ratio() {
        let p = ds(
            &Layout {
                arrangement: Arrangement::BigSmall,
                ratio: 75,
                ..Layout::default()
            },
            area(1920.0, 1080.0),
        );
        let (top, bottom) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
        let share = top.w / (top.w + bottom.w);
        assert!((share - 0.75).abs() < 0.001, "{share}");
        assert_eq!(top.y, bottom.y, "top-aligned");
        assert!(
            (bottom.w / bottom.h - 256.0 / 192.0).abs() < 0.001,
            "aspect kept"
        );
    }

    #[test]
    fn one_screen_arrangements_draw_one_screen() {
        for (arrangement, swap, want) in [
            (Arrangement::FirstOnly, false, ScreenId::Top),
            (Arrangement::FirstOnly, true, ScreenId::Bottom),
            (Arrangement::SecondOnly, false, ScreenId::Bottom),
        ] {
            let p = ds(
                &Layout {
                    arrangement,
                    swap,
                    ..Layout::default()
                },
                area(1920.0, 1080.0),
            );
            assert_eq!(p.len(), 1);
            assert_eq!(p[0].id, want);
            assert_eq!(p[0].dst, rect(320.0, 60.0, 1280.0, 960.0));
        }
    }

    #[test]
    fn a_one_screen_console_ignores_the_arrangement() {
        let screens = Split::Whole.screens(240, 160);
        let p = compose(
            &Layout {
                arrangement: Arrangement::SideBySide,
                ..Layout::default()
            },
            &screens,
            area(1440.0, 900.0),
        );
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].dst, fit(area(1440.0, 900.0), (240.0, 160.0)));
        assert_eq!(touch_at(&p, 700.0, 400.0), None, "no touch screen");
    }

    #[test]
    fn a_custom_arrangement_is_placed_as_given() {
        let p = ds(
            &Layout {
                arrangement: Arrangement::Custom(vec![
                    (ScreenId::Bottom, rect(0.5, 0.0, 0.5, 1.0)),
                    (ScreenId::Top, rect(0.0, 0.0, 0.5, 0.5)),
                ]),
                ..Layout::default()
            },
            Area {
                width: 1000.0,
                height: 1000.0,
                inset: 0.2,
            },
        );
        assert_eq!(dst(&p, ScreenId::Bottom), rect(500.0, 0.0, 500.0, 800.0));
        assert_eq!(dst(&p, ScreenId::Top), rect(0.0, 0.0, 500.0, 400.0));
    }

    #[test]
    fn rotation_turns_the_whole_picture() {
        let a = area(1920.0, 1080.0);
        for rotation in [Rotation::Left, Rotation::Right] {
            let p = ds(
                &Layout {
                    rotation,
                    ..Layout::default()
                },
                a,
            );
            // The stack turned is 384 x 256 and fits 4x: 1536 x 1024, two 768 x 1024 halves.
            let (top, bottom) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
            assert_eq!((top.w, top.h), (768.0, 1024.0));
            assert_eq!((bottom.w, bottom.h), (768.0, 1024.0));
            match rotation {
                // Anticlockwise: the top screen ends on the left.
                Rotation::Left => assert!(top.x < bottom.x),
                Rotation::Right => assert!(top.x > bottom.x),
                Rotation::None | Rotation::Flip => unreachable!(),
            }
        }
        let flip = ds(
            &Layout {
                rotation: Rotation::Flip,
                ..Layout::default()
            },
            a,
        );
        assert!(dst(&flip, ScreenId::Top).y > dst(&flip, ScreenId::Bottom).y);
    }

    #[test]
    fn touch_maps_the_window_back_to_the_touch_screen() {
        // Default layout at 1080p: the bottom screen is (704, 540) to (1216, 924), 2x.
        let p = ds(&Layout::default(), area(1920.0, 1080.0));
        assert_eq!(touch_at(&p, 704.0, 540.0), Some((0, 0)));
        assert_eq!(touch_at(&p, 705.9, 541.9), Some((0, 0)));
        assert_eq!(touch_at(&p, 706.0, 542.0), Some((1, 1)));
        assert_eq!(
            touch_at(&p, 704.0 + 2.0 * 100.0, 540.0 + 2.0 * 50.0),
            Some((100, 50))
        );
        assert_eq!(touch_at(&p, 1215.9, 923.9), Some((255, 191)));
        // The top screen, the bars beside the game and outside the window are not touches.
        assert_eq!(touch_at(&p, 900.0, 300.0), None);
        assert_eq!(touch_at(&p, 100.0, 700.0), None);
        assert_eq!(touch_at(&p, 1216.0, 700.0), None);
        assert_eq!(touch_at(&p, -1.0, -1.0), None);
    }

    #[test]
    fn touch_follows_the_rotation() {
        let a = area(1920.0, 1080.0);
        let corners = |rotation| {
            let p = ds(
                &Layout {
                    rotation,
                    ..Layout::default()
                },
                a,
            );
            let b = dst(&p, ScreenId::Bottom);
            let at = |u: f32, v: f32| touch_at(&p, b.x + u * b.w, b.y + v * b.h).unwrap();
            // The window's top-left and top-right corners of the bottom screen's rectangle.
            (at(0.001, 0.001), at(0.999, 0.001))
        };
        // Anticlockwise: the screen's top-left corner is now bottom-left, its bottom-left is
        // top-left... so the window's top-left holds the screen's top-right.
        assert_eq!(corners(Rotation::Left), ((255, 0), (255, 191)));
        assert_eq!(corners(Rotation::Right), ((0, 191), (0, 0)));
        assert_eq!(corners(Rotation::Flip), ((255, 191), (0, 191)));
        assert_eq!(corners(Rotation::None), ((0, 0), (255, 0)));
    }

    #[test]
    fn the_pointer_value_lands_on_the_same_pixel_in_melonds() {
        let bottom = DS.screens(FRAME.0, FRAME.1)[1].src;
        for (x, y) in [(0, 0), (1, 1), (128, 96), (255, 191), (17, 190), (254, 3)] {
            let p = pointer_value(bottom, FRAME, x, y);
            let (cx, cy) = melonds_reads(p, FRAME);
            assert_eq!(
                (cx, cy - 192),
                (x as i32, y as i32),
                "pixel {x},{y} → {p:?}"
            );
        }
        // The frame's corners stay inside the libretro range.
        assert_eq!(pointer_value(bottom, FRAME, 255, 191).0, 32639);
        assert_eq!(pointer_value(PxRect::default(), (0, 0), 0, 0), (0, 0));
    }

    #[test]
    fn pointer_at_goes_from_the_window_to_the_core() {
        let a = area(1920.0, 1080.0);
        let p = pointer_at(
            &Layout::default(),
            DS,
            FRAME,
            a,
            704.0 + 200.0,
            540.0 + 100.0,
        )
        .unwrap();
        assert_eq!(melonds_reads(p, FRAME), (100, 192 + 50));
        assert_eq!(
            pointer_at(&Layout::default(), DS, FRAME, a, 900.0, 300.0),
            None
        );
        assert_eq!(
            pointer_at(
                &Layout::default(),
                Split::Whole,
                (240, 160),
                a,
                900.0,
                500.0
            ),
            None
        );
    }

    #[test]
    fn ids_of_the_settings_are_known() {
        for id in ["vertical", "side", "big_small", "top_only"] {
            assert!(Arrangement::from_id(id).is_some(), "{id}");
        }
        assert_eq!(Arrangement::from_id("hybrid"), None);
        for id in ["none", "left", "right", "flip"] {
            assert!(Rotation::from_id(id).is_some(), "{id}");
        }
    }
}
