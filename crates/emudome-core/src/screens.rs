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
//! The `screens.*` settings, `video.aspect` and `video.scale` choose the [`Layout`]
//! ([`ScreenSettings`]: the layouts to step through, the ratio, the gap, the rotation, the custom
//! layout, the proportions, the size) together with what the user
//! changed while playing (the current [`LayoutId`], the screens swapped); the renderers draw each
//! [`Placement`] of [`compose`] (native `video.rs`, webview `GameCanvas.vue` from the placements
//! the shell sends), and the touch screen maps back through the same placements (Trello #28).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_profile::CoreId;
use crate::settings::keys::{self, Reader};
use crate::settings::{ConsoleId, Value};

/// Which screen of the console. TypeScript: `ScreenId` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ScreenId {
    /// The only screen of a one-screen console (GB, GBC, GBA).
    Main,
    /// NDS / 3DS upper screen.
    Top,
    /// NDS / 3DS lower screen: the touch screen.
    Bottom,
}

impl ScreenId {
    /// The other screen of a two-screen console (the only screen stays itself).
    pub fn other(self) -> ScreenId {
        match self {
            ScreenId::Main => ScreenId::Main,
            ScreenId::Top => ScreenId::Bottom,
            ScreenId::Bottom => ScreenId::Top,
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<ScreenId> {
        match id {
            "top" => Some(ScreenId::Top),
            "bottom" => Some(ScreenId::Bottom),
            "main" => Some(ScreenId::Main),
            _ => None,
        }
    }

    pub(crate) fn id(self) -> &'static str {
        match self {
            ScreenId::Main => "main",
            ScreenId::Top => "top",
            ScreenId::Bottom => "bottom",
        }
    }
}

/// A rectangle of whole pixels in the core's frame. TypeScript: `PxRect` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, TS)]
pub struct PxRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// A rectangle in window pixels (the renderer's unit: physical pixels for the native path, CSS
/// pixels for the webview canvas). TypeScript: `ScreenRect` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, TS)]
#[ts(rename = "ScreenRect")]
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

    /// The frame the core hands over at its own resolution (1×) for this split; `None` for one
    /// screen, whose size depends on the console. What the editor's presets are composed from.
    pub fn frame_at_1x(self) -> Option<(u32, u32)> {
        match self {
            Split::Whole => None,
            Split::DsStacked { gap } => Some((256, 2 * 192 + gap)),
            Split::N3dsStacked => Some((N3DS_TOP.0, 2 * N3DS_TOP.1)),
        }
    }
}

/// How the screens are arranged (from a [`LayoutId`]; `SecondOnly` is `top_only` swapped).
#[derive(Debug, Clone, PartialEq)]
pub enum Arrangement {
    /// One above the other («Uno sopra l'altro»): the default.
    Stacked,
    /// The phone's default (`auto`): [`Arrangement::Stacked`] in a window taller than wide,
    /// [`Arrangement::SideBySide`] otherwise, decided by the window each time it is composed.
    Auto,
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
    /// The phone's own placement (the C06 touch editor, `touch.tscreens`): rectangles as
    /// fractions (0..1) of the whole window. With no inset they are drawn as given; an inset (the
    /// quick menu) shrinks the whole window's picture uniformly into the part left and centres
    /// it there, so the screens keep their proportions and their places relative to each other.
    /// Turned and swapped as [`Arrangement::Custom`]. A screen not listed is not drawn.
    Phone(Vec<(ScreenId, Rect)>),
}

impl Arrangement {
    /// The arrangement of a `screens.cycle` value; `None` for an id this build does not know.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "vertical" => Some(Arrangement::Stacked),
            "auto" => Some(Arrangement::Auto),
            "side" => Some(Arrangement::SideBySide),
            "big_small" => Some(Arrangement::BigSmall),
            "top_only" => Some(Arrangement::FirstOnly),
            "bottom_only" => Some(Arrangement::SecondOnly),
            _ => None,
        }
    }
}

/// The layouts the user picks: the `screens.cycle` options, in the order of the quick menu.
/// `custom` is the user's own layout (`screens.custom`), there only once one is saved.
/// TypeScript: `ScreenLayoutId` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ScreenLayoutId")]
pub enum LayoutId {
    /// «Uno sopra l'altro»: the default on the desktop.
    Vertical,
    /// «Automatica»: one above the other upright, side by side on its side; the phone's default
    /// (`screens.cycle`'s default rule).
    Auto,
    /// «Affiancati».
    Side,
    /// «Grande e piccolo».
    BigSmall,
    /// «Solo superiore» (swapped: the bottom screen only).
    TopOnly,
    /// «Personalizzata»: `screens.custom`.
    Custom,
}

impl LayoutId {
    pub const ALL: [LayoutId; 6] = [
        LayoutId::Vertical,
        LayoutId::Auto,
        LayoutId::Side,
        LayoutId::BigSmall,
        LayoutId::TopOnly,
        LayoutId::Custom,
    ];

    /// The `screens.cycle` option id.
    pub fn id(self) -> &'static str {
        match self {
            LayoutId::Vertical => "vertical",
            LayoutId::Auto => "auto",
            LayoutId::Side => "side",
            LayoutId::BigSmall => "big_small",
            LayoutId::TopOnly => "top_only",
            LayoutId::Custom => "custom",
        }
    }

    pub fn from_id(id: &str) -> Option<LayoutId> {
        LayoutId::ALL.into_iter().find(|l| l.id() == id)
    }
}

/// The whole picture turned (`screens.rot`), book-style games held sideways.
/// TypeScript: `ScreenRotation` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "ScreenRotation")]
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
    /// `video.aspect`: the proportions of the whole picture as shown.
    pub aspect: Aspect,
    /// `video.scale`: how large the picture is drawn in the room it has.
    pub scale: Scale,
}

impl Default for Layout {
    /// «Uno sopra l'altro»: top above bottom, no gap, not turned, the console's proportions,
    /// whole scales.
    fn default() -> Self {
        Self {
            arrangement: Arrangement::Stacked,
            swap: false,
            gap: 0,
            ratio: 70,
            rotation: Rotation::None,
            aspect: Aspect::Original,
            scale: Scale::Integer,
        }
    }
}

/// `video.scale`: how large the game's picture (one screen, or the composed two) is drawn in the
/// part of the window it may use. The default is the device's (catalogue `default_rules`):
/// [`Scale::Integer`] on a computer, [`Scale::Fit`] on a phone (owner, 2026-11-22, Trello #105).
/// A custom layout (`screens.custom`) is drawn as the user placed it, whatever this says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scale {
    /// As large as the room allows at any scale, the proportions kept («Adatta allo schermo»):
    /// a GBA 1.5× across a 360 px wide phone held upright.
    Fit,
    /// A whole scale when the picture fits at 1× or more, every console pixel the same size
    /// («Pixel interi»); a picture smaller than the room at 1× still shrinks as [`Scale::Fit`].
    #[default]
    Integer,
    /// The whole room, stretched: the proportions (and `video.aspect`) not kept («Riempi lo
    /// schermo»).
    Fill,
}

impl Scale {
    /// The setting's option id; `None` for an id this build does not know.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "fit" => Some(Scale::Fit),
            "integer" => Some(Scale::Integer),
            "fill" => Some(Scale::Fill),
            _ => None,
        }
    }
}

/// `video.aspect`: the game's picture keeps the console's proportions (each screen's pixels
/// square, as before the setting), or is stretched to 4:3 or 16:9. With two screens the
/// composed whole (both screens, the gap, turned) is what is stretched; a custom layout
/// (`screens.custom`) is drawn as the user placed it, whatever this says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Aspect {
    #[default]
    Original,
    FourThree,
    SixteenNine,
}

impl Aspect {
    /// The setting's option id; `None` for an id this build does not know.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "original" => Some(Aspect::Original),
            "4_3" => Some(Aspect::FourThree),
            "16_9" => Some(Aspect::SixteenNine),
            _ => None,
        }
    }

    /// Width over height, `None` for the console's own.
    pub fn ratio(self) -> Option<f32> {
        match self {
            Aspect::Original => None,
            Aspect::FourThree => Some(4.0 / 3.0),
            Aspect::SixteenNine => Some(16.0 / 9.0),
        }
    }
}

/// The window the game is drawn in, and the part of it kept clear (`inset`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub width: f32,
    pub height: f32,
    pub inset: Inset,
}

/// Where the game sits inside the part of the window it may use. TypeScript: `VideoAlign`
/// (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "VideoAlign")]
pub enum Align {
    /// In the middle (the desktop's G09 bar, the phone's quick menu, the touch controls on the
    /// sides).
    #[default]
    Centre,
    /// Against the bottom edge, centred across: the phone held upright, the touch controls in
    /// their band under the game with no empty band between the two (#29).
    Bottom,
    /// Against the top edge, centred across: a two-screen game on a phone on its side, the touch
    /// controls in the strip below the screens and over their lower corners.
    Top,
    /// Two screens laid out with a room between them ([`Arrangement::Stacked`] or
    /// [`Arrangement::SideBySide`]): the first against the start of that axis, the second against
    /// its end, what is left over going between them (at least [`Inset::between`]); across that
    /// axis as [`Align::Bottom`]. A two-screen game on a phone held upright: the top screen under
    /// the top edge, the touch screen on the controls' band, L / menu / R in the strip between.
    /// Anything else: as [`Align::Bottom`].
    Apart,
}

/// The edges of the window kept clear of the game, as shares of its width (`left`, `right`) and
/// of its height (`top`, `bottom`): the quick menu's bar, sheet or side panel, the touch
/// controls' band or side columns. Each share is taken in 0..0.9 and the two of one axis together
/// leave at least a tenth of the window. TypeScript: `VideoInset` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(rename = "VideoInset")]
pub struct Inset {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub align: Align,
    /// The room kept between two screens laid out one above the other or side by side, as a
    /// share of the window's height (whatever the axis), on top of `screens.gap`: the touch
    /// controls' strip between the screens on a phone (L, menu, R upright; a thin line on its
    /// side). Taken in 0..0.5; ignored for one screen and the other arrangements.
    pub between: f32,
}

/// The most of an axis an inset may take.
const INSET_MAX: f32 = 0.9;
/// The most of the window's height [`Inset::between`] may take.
const BETWEEN_MAX: f32 = 0.5;

impl Inset {
    /// Nothing kept clear: the whole window.
    pub const NONE: Inset = Inset {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        align: Align::Centre,
        between: 0.0,
    };

    /// The bottom `share` kept clear, the game centred above it.
    pub fn bottom(share: f32) -> Inset {
        Inset {
            bottom: share,
            ..Inset::NONE
        }
    }

    /// The two shares of an axis, each in 0..0.9, scaled down together when they leave less than
    /// a tenth of the window (a NaN counts as 0).
    fn axis(start: f32, end: f32) -> (f32, f32) {
        let clean = |v: f32| {
            if v.is_finite() {
                v.clamp(0.0, INSET_MAX)
            } else {
                0.0
            }
        };
        let (s, e) = (clean(start), clean(end));
        let sum = s + e;
        if sum > INSET_MAX {
            (s * INSET_MAX / sum, e * INSET_MAX / sum)
        } else {
            (s, e)
        }
    }

    /// [`Inset::between`] in window pixels, for a window `height` tall (a NaN counts as 0).
    fn between_px(&self, height: f32) -> f32 {
        if self.between.is_finite() {
            self.between.clamp(0.0, BETWEEN_MAX) * height
        } else {
            0.0
        }
    }

    /// The part of a `width` × `height` window the game may use.
    pub fn usable(&self, width: f32, height: f32) -> Rect {
        let (l, r) = Inset::axis(self.left, self.right);
        let (t, b) = Inset::axis(self.top, self.bottom);
        Rect {
            x: width * l,
            y: height * t,
            w: width * (1.0 - l - r),
            h: height * (1.0 - t - b),
        }
    }
}

/// Where one screen is drawn. TypeScript: `ScreenPlacement` (generated): the webview canvas
/// draws `src` of the frame into `dst`, its picture turned by `rotation`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(rename = "ScreenPlacement")]
pub struct Placement {
    pub id: ScreenId,
    /// Its pixels in the core's frame.
    pub src: PxRect,
    /// Its rectangle in the window.
    pub dst: Rect,
    /// How its picture is turned inside `dst`.
    pub rotation: Rotation,
}

/// A rectangle of `content` size fitted in `area` as [`Scale::Integer`] does: integer scale when
/// it fits, otherwise the largest that keeps the aspect ratio, in the part the inset leaves
/// (centred, against its bottom with [`Align::Bottom`] / [`Align::Apart`], against its top with
/// [`Align::Top`]). `(x, y, w, h)`. Both renderers (native `video.rs`, webview `GameCanvas.vue`)
/// go through [`compose`], which fits by [`Layout::scale`].
pub fn fit(area: Area, content: (f32, f32)) -> Rect {
    fit_with(area, content, (0.0, 0.0), Scale::Integer)
}

/// [`fit`] for `content` (scaled) plus `extra` window pixels across and down that do not scale
/// (the room between two screens), sized by `scale`. The rectangle holds both.
fn fit_with(area: Area, content: (f32, f32), extra: (f32, f32), scale: Scale) -> Rect {
    let free = area.inset.usable(area.width, area.height);
    let (w, h) = content;
    if w <= 0.0 || h <= 0.0 {
        return Rect::default();
    }
    let ex = extra.0.clamp(0.0, free.w);
    let ey = extra.1.clamp(0.0, free.h);
    let fit = ((free.w - ex) / w).min((free.h - ey) / h);
    let (vw, vh) = match scale {
        Scale::Fill => (free.w, free.h),
        Scale::Integer if fit >= 1.0 => (w * fit.floor() + ex, h * fit.floor() + ey),
        Scale::Integer | Scale::Fit => (w * fit + ex, h * fit + ey),
    };
    let y = match area.inset.align {
        Align::Centre => free.y + (free.h - vh) / 2.0,
        Align::Bottom | Align::Apart => free.y + free.h - vh,
        Align::Top => free.y,
    };
    // On whole pixels; a hair under one (the inset's shares times the window are not exact:
    // 780 × (1 − 220 / 780) = 559.99997) is that pixel, or the game leaves a line over the band.
    let whole = |v: f32| (v + PIXEL_SLACK).floor();
    Rect {
        x: whole(free.x + (free.w - vw) / 2.0),
        y: whole(y),
        w: vw,
        h: vh,
    }
}

/// How far under a whole pixel a position still counts as that pixel (float error only).
const PIXEL_SLACK: f32 = 1e-3;

/// Where the room between two screens lies once the whole is turned: across (between a left and
/// a right screen) or down, and which of the two (in the order they were laid out) comes after.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Between {
    across: bool,
    second_after: bool,
}

impl Between {
    /// For two screens laid out down (`stacked`) or across, turned by `rotation` (see `turn`): a
    /// quarter turn moves the room to the other axis, and a turn that mirrors that axis puts the
    /// first screen after it.
    fn of(stacked: bool, rotation: Rotation) -> Between {
        let (across, second_after) = match (stacked, rotation) {
            (true, Rotation::None) => (false, true),
            (true, Rotation::Flip) => (false, false),
            (true, Rotation::Left) => (true, true),
            (true, Rotation::Right) => (true, false),
            (false, Rotation::None) => (true, true),
            (false, Rotation::Flip) => (true, false),
            (false, Rotation::Left) => (false, false),
            (false, Rotation::Right) => (false, true),
        };
        Between {
            across,
            second_after,
        }
    }
}

/// Places `screens` in `area` as `layout` says. The screens are first laid out at their own size
/// (gap in the same pixels), the whole is turned, then fitted with [`fit`] at its own proportions
/// or at `layout.aspect`'s (stretched across); a custom arrangement is placed as given instead.
pub fn compose(layout: &Layout, screens: &[SourceScreen], area: Area) -> Vec<Placement> {
    let mut order: Vec<SourceScreen> = screens.to_vec();
    if layout.swap {
        order.reverse();
    }

    if let Some((rects, frame)) = given_rects(&layout.arrangement, area) {
        return place_given(rects, frame, layout, screens);
    }

    // The phone's default: down in a window taller than wide, across otherwise.
    let arrangement = match &layout.arrangement {
        Arrangement::Auto if area.height > area.width => &Arrangement::Stacked,
        Arrangement::Auto => &Arrangement::SideBySide,
        given @ (Arrangement::Stacked
        | Arrangement::SideBySide
        | Arrangement::BigSmall
        | Arrangement::FirstOnly
        | Arrangement::SecondOnly
        | Arrangement::Custom(_)
        | Arrangement::Phone(_)) => given,
    };

    // Each screen's rectangle at its own size, in a canvas starting at (0, 0).
    let gap = layout.gap as f32;
    let size = |s: &SourceScreen| (s.src.w as f32, s.src.h as f32);
    let mut placed: Vec<(SourceScreen, Rect)> = Vec::new();
    match (arrangement, order.as_slice()) {
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
        (Arrangement::Custom(_) | Arrangement::Phone(_) | Arrangement::Auto, _) => {}
    }
    if placed.is_empty() {
        return Vec::new();
    }
    // The room kept between two screens laid out down or across, in window pixels (it does not
    // scale with them), and where it lies once the whole is turned.
    let between = match (arrangement, placed.len()) {
        (Arrangement::Stacked, 2) => Some(Between::of(true, layout.rotation)),
        (Arrangement::SideBySide, 2) => Some(Between::of(false, layout.rotation)),
        _ => None,
    }
    .map(|b| (b, area.inset.between_px(area.height)));

    let canvas_w = placed.iter().map(|(_, r)| r.x + r.w).fold(0.0, f32::max);
    let canvas_h = placed.iter().map(|(_, r)| r.y + r.h).fold(0.0, f32::max);
    let (turned_w, turned_h) = match layout.rotation {
        Rotation::None | Rotation::Flip => (canvas_w, canvas_h),
        Rotation::Left | Rotation::Right => (canvas_h, canvas_w),
    };
    // `video.aspect`: the whole keeps its height in console pixels and takes the width of the
    // proportions asked for, so the integer scale of `fit` still follows the console's rows.
    let shown = match layout.aspect.ratio() {
        None => (turned_w, turned_h),
        Some(ratio) => (turned_h * ratio, turned_h),
    };
    let free = area.inset.usable(area.width, area.height);
    // The room in pixels along its axis, kept inside the free part (`fit_with` does the same).
    let (extra, mut room) = match between {
        Some((b, px)) if b.across => {
            let px = px.clamp(0.0, free.w);
            ((px, 0.0), px)
        }
        Some((_, px)) => {
            let px = px.clamp(0.0, free.h);
            ((0.0, px), px)
        }
        None => ((0.0, 0.0), 0.0),
    };
    let mut target = fit_with(area, shown, extra, layout.scale);
    let (scale_x, scale_y) = (
        (target.w - extra.0) / turned_w,
        (target.h - extra.1) / turned_h,
    );
    // Apart: the two screens against both ends of their axis, the room between them growing.
    if let (Some((b, _)), Align::Apart) = (between, area.inset.align) {
        if b.across {
            room += (free.w - target.w).max(0.0);
            target.x = free.x;
        } else {
            room += (free.h - target.h).max(0.0);
            target.y = free.y;
        }
    }

    placed
        .into_iter()
        .enumerate()
        .map(|(i, (s, r))| {
            let turned = turn(r, layout.rotation, canvas_w, canvas_h);
            // The screen after the room moves on by it, along the room's axis.
            let (dx, dy) = match between {
                Some((b, _)) if b.second_after == (i == 1) => {
                    if b.across {
                        (room, 0.0)
                    } else {
                        (0.0, room)
                    }
                }
                Some(_) | None => (0.0, 0.0),
            };
            Placement {
                id: s.id,
                src: s.src,
                dst: Rect {
                    x: target.x + turned.x * scale_x + dx,
                    y: target.y + turned.y * scale_y + dy,
                    w: turned.w * scale_x,
                    h: turned.h * scale_y,
                },
                rotation: layout.rotation,
            }
        })
        .collect()
}

/// The rectangles of an arrangement that places each screen where it is told (custom, the
/// phone's placement) and the window rectangle their fractions are of; `None` for the
/// arrangements [`compose`] lays out itself.
fn given_rects(arrangement: &Arrangement, area: Area) -> Option<(&[(ScreenId, Rect)], Rect)> {
    match arrangement {
        Arrangement::Custom(rects) => Some((rects, area.inset.usable(area.width, area.height))),
        Arrangement::Phone(rects) => Some((rects, phone_frame(area))),
        Arrangement::Stacked
        | Arrangement::Auto
        | Arrangement::SideBySide
        | Arrangement::BigSmall
        | Arrangement::FirstOnly
        | Arrangement::SecondOnly => None,
    }
}

/// The whole window scaled uniformly into the part the inset leaves, centred there: what the
/// phone's placement is drawn in (the window itself with no inset).
fn phone_frame(area: Area) -> Rect {
    let usable = area.inset.usable(area.width, area.height);
    let k = (usable.w / area.width).min(usable.h / area.height);
    let k = if k.is_finite() { k.max(0.0) } else { 0.0 };
    let (w, h) = (k * area.width, k * area.height);
    Rect {
        x: usable.x + (usable.w - w) / 2.0,
        y: usable.y + (usable.h - h) / 2.0,
        w,
        h,
    }
}

/// Each of `rects` (fractions of `frame`) drawn in `frame`. The whole picture turns: each
/// rectangle turns inside the frame, its picture with it; swapped, each screen takes the other's
/// rectangle.
fn place_given(
    rects: &[(ScreenId, Rect)],
    frame: Rect,
    layout: &Layout,
    screens: &[SourceScreen],
) -> Vec<Placement> {
    rects
        .iter()
        .filter_map(|(id, r)| {
            let id = if layout.swap { id.other() } else { *id };
            let s = screens.iter().find(|s| s.id == id)?;
            let r = turn(*r, layout.rotation, 1.0, 1.0);
            Some(Placement {
                id: s.id,
                src: s.src,
                dst: Rect {
                    x: frame.x + r.x * frame.w,
                    y: frame.y + r.y * frame.h,
                    w: r.w * frame.w,
                    h: r.h * frame.h,
                },
                rotation: layout.rotation,
            })
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

/// One screen of a custom layout: its rectangle as fractions (0..1) of the game area.
/// TypeScript: `CustomScreen` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
pub struct CustomScreen {
    pub id: ScreenId,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// The user's own layout (`screens.custom`, the C09 editor), saved per console or per game.
/// TypeScript: `CustomLayout` (generated).
///
/// Stored as the setting's list, one item per screen, back to front (the last one is drawn in
/// front): `"<top|bottom>:<x>,<y>,<w>,<h>"`, fractions of the game area with at most 4 decimals,
/// e.g. `["top:0,0,0.5,0.5", "bottom:0.5,0,0.5,1"]`. An empty list: no custom layout.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
pub struct CustomLayout {
    /// Back to front: the last one is in front.
    pub screens: Vec<CustomScreen>,
}

/// How far past the area's edge a rectangle may go from rounding.
const EDGE_SLACK: f32 = 1e-3;

impl CustomLayout {
    /// Each screen once, `top` or `bottom`, inside the area with a size: what the editor may
    /// save. Empty is valid (no custom layout).
    pub fn is_valid(&self) -> bool {
        let fits = |start: f32, size: f32| {
            start.is_finite()
                && size.is_finite()
                && start >= 0.0
                && size > 0.0
                && start + size <= 1.0 + EDGE_SLACK
        };
        self.screens.iter().enumerate().all(|(i, s)| {
            matches!(s.id, ScreenId::Top | ScreenId::Bottom)
                && self.screens[..i].iter().all(|o| o.id != s.id)
                && fits(s.x, s.w)
                && fits(s.y, s.h)
        })
    }

    /// The layout a stored list holds; `None` when an item is malformed or the whole is not
    /// [valid](Self::is_valid).
    pub fn from_items(items: &[String]) -> Option<CustomLayout> {
        let parse = |item: &str| -> Option<CustomScreen> {
            let (id, rest) = item.split_once(':')?;
            let mut n = rest.split(',').map(|v| v.trim().parse::<f32>().ok());
            let screen = CustomScreen {
                id: ScreenId::from_id(id.trim())?,
                x: n.next()??,
                y: n.next()??,
                w: n.next()??,
                h: n.next()??,
            };
            n.next().is_none().then_some(screen)
        };
        let layout = CustomLayout {
            screens: items.iter().map(|i| parse(i)).collect::<Option<Vec<_>>>()?,
        };
        layout.is_valid().then_some(layout)
    }

    /// The stored list (see the type's doc), numbers rounded to 4 decimals.
    pub fn to_items(&self) -> Vec<String> {
        let n = |v: f32| (v * 10_000.0).round() / 10_000.0;
        self.screens
            .iter()
            .map(|s| format!("{}:{},{},{},{}", s.id.id(), n(s.x), n(s.y), n(s.w), n(s.h)))
            .collect()
    }

    /// Whether a stored value is a valid custom layout (the schema's check, `Kind::ScreenLayout`).
    pub fn valid_items(items: &[String]) -> bool {
        Self::from_items(items).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    fn arrangement(&self) -> Arrangement {
        Arrangement::Custom(given(&self.screens))
    }

    /// Where `layout` puts the screens of `split` in an area of `width` × `height` (the whole
    /// window, no inset), as a custom layout: the C09 editor's «Parti da», so the editor never
    /// works the geometry out again. The gap and the ratio are `settings`'; the picture is not
    /// turned (`screens.rot` turns a custom layout as a whole on top); `custom` is the one saved.
    /// Sized by `settings`' `video.scale` ([`Layout::scale`]), as the game shows them on this
    /// device. Empty for a one-screen split or an empty area.
    pub fn preset(
        settings: &ScreenSettings,
        layout: LayoutId,
        split: Split,
        width: f32,
        height: f32,
    ) -> CustomLayout {
        if layout == LayoutId::Custom {
            return settings.custom.clone();
        }
        let Some((frame_w, frame_h)) = split.frame_at_1x() else {
            return CustomLayout::default();
        };
        if !(width > 0.0 && height > 0.0) {
            return CustomLayout::default();
        }
        let arranged = Layout {
            rotation: Rotation::None,
            ..settings.layout(layout, false)
        };
        let area = Area {
            width,
            height,
            inset: Inset::NONE,
        };
        let placed: Vec<Placement> = compose(&arranged, &split.screens(frame_w, frame_h), area)
            .into_iter()
            .filter(|p| p.id != ScreenId::Main)
            .collect();
        CustomLayout {
            screens: fractions(&placed, width, height),
        }
    }
}

/// `screens` as the rectangles an arrangement places them in.
fn given(screens: &[CustomScreen]) -> Vec<(ScreenId, Rect)> {
    screens
        .iter()
        .map(|s| {
            (
                s.id,
                Rect {
                    x: s.x,
                    y: s.y,
                    w: s.w,
                    h: s.h,
                },
            )
        })
        .collect()
}

/// A fraction of `whole` for the span `start`..`start + size`, kept inside 0..1 whatever the
/// rounding: `(start, size)`.
fn fraction(start: f32, size: f32, whole: f32) -> (f32, f32) {
    let s = (start / whole).clamp(0.0, 1.0);
    (s, (size / whole).min(1.0 - s))
}

/// `placements` (window pixels) as fractions of a `width` x `height` window.
fn fractions(placements: &[Placement], width: f32, height: f32) -> Vec<CustomScreen> {
    placements
        .iter()
        .map(|p| {
            let (x, w) = fraction(p.dst.x, p.dst.w, width);
            let (y, h) = fraction(p.dst.y, p.dst.h, height);
            CustomScreen {
                id: p.id,
                x,
                y,
                w,
                h,
            }
        })
        .collect()
}

/// The screens a `console`'s games have, in the order the touch editor lists them.
pub fn console_screens(console: ConsoleId) -> &'static [ScreenId] {
    match console {
        ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba => &[ScreenId::Main],
        ConsoleId::Nds | ConsoleId::N3ds => &[ScreenId::Top, ScreenId::Bottom],
    }
}

/// The smallest screen the touch editor lets a layout hold, as fractions of the game area
/// (`screenEditor.ts`: `MIN_SIZE`, `MIN_SIZE / 2`).
const PLACEMENT_MIN_W: f32 = 0.08;
const PLACEMENT_MIN_H: f32 = 0.04;

/// Whether `screens` is a placement of exactly the screens `ids`, each once, inside the area and
/// not smaller than the editor's minimum. Empty (none saved) is valid.
pub fn placement_fits(screens: &[CustomScreen], ids: &[ScreenId]) -> bool {
    let inside = |start: f32, size: f32, min: f32| {
        start.is_finite()
            && size.is_finite()
            && start >= 0.0
            && size >= min
            && start + size <= 1.0 + EDGE_SLACK
    };
    screens.is_empty()
        || (screens.len() == ids.len()
            && ids
                .iter()
                .all(|id| screens.iter().filter(|s| s.id == *id).count() == 1)
            && screens
                .iter()
                .all(|s| inside(s.x, s.w, PLACEMENT_MIN_W) && inside(s.y, s.h, PLACEMENT_MIN_H)))
}

/// The frame a `console` game hands over at its own resolution (1×): what the editors draw in
/// place of the game when none is open (Trello #111). Two-screen consoles as their core's
/// [`Split`] stacks them.
pub fn console_frame(console: ConsoleId) -> (u32, u32) {
    let split = CoreId::for_console(console).profile().split;
    split.frame_at_1x().unwrap_or(match console {
        ConsoleId::Gb | ConsoleId::Gbc => (160, 144),
        ConsoleId::Gba => (240, 160),
        // Not reached: both have a two-screen split. Their 1× frames, in case one changes.
        ConsoleId::Nds => (256, 384),
        ConsoleId::N3ds => (400, 480),
    })
}

/// Where a `console` game's screens would be drawn in `area` when it opens with `settings`: the
/// first layout of the cycle, not swapped, its frame at 1× ([`console_frame`]). The picture
/// [`phone_start`] starts the touch editor's screens from.
pub fn preview(settings: &ScreenSettings, console: ConsoleId, area: Area) -> Vec<Placement> {
    let split = CoreId::for_console(console).profile().split;
    let (width, height) = console_frame(console);
    let layout = settings.layout(settings.first(), false);
    compose(&layout, &split.screens(width, height), area)
}

/// Where the touch editor starts the screens of a `console` game in `area` (the whole window)
/// when no placement of the phone's own is saved and no game is open: where [`preview`] draws
/// them, as fractions of the window (empty for an empty area). Over an open game the start is
/// [`ScreenSession::phone_start`].
pub fn phone_start(settings: &ScreenSettings, console: ConsoleId, area: Area) -> Vec<CustomScreen> {
    let split = CoreId::for_console(console).profile().split;
    let (width, height) = console_frame(console);
    start_placement(
        settings.layout(settings.first(), false),
        &split.screens(width, height),
        area,
    )
}

/// `layout`'s screens composed in `area` as the touch editor's start: fractions of the whole
/// window, each keyed by the frame (rectangle) it is, as [`place_given`] reads a placement (the
/// other screen's picture when `layout` is swapped). Not turned: `screens.rot` turns the
/// placement on top, as it does a custom layout. When `layout` leaves a screen out (`top_only`,
/// `bottom_only`) every screen is placed as [`Arrangement::Auto`] does, swapped the same way, so
/// the start holds all of the game's screens. Empty for an empty area.
fn start_placement(layout: Layout, sources: &[SourceScreen], area: Area) -> Vec<CustomScreen> {
    if !(area.width > 0.0 && area.height > 0.0) {
        return Vec::new();
    }
    let layout = Layout {
        rotation: Rotation::None,
        ..layout
    };
    let place = |layout: &Layout| compose(layout, sources, area);
    let all = |placed: &Vec<Placement>| sources.iter().all(|s| placed.iter().any(|p| p.id == s.id));
    let placed = Some(place(&layout)).filter(all).unwrap_or_else(|| {
        place(&Layout {
            arrangement: Arrangement::Auto,
            ..layout.clone()
        })
    });
    let frames: Vec<Placement> = placed
        .into_iter()
        .map(|p| Placement {
            id: if layout.swap { p.id.other() } else { p.id },
            ..p
        })
        .collect();
    fractions(&frames, area.width, area.height)
}

/// The `screens.*` settings of the open game (its changes «in prova» included), read once.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenSettings {
    /// `screens.cycle`: the layouts L3 + R3 and the quick menu step through, in order; `custom`
    /// only while a custom layout is saved.
    pub cycle: Vec<LayoutId>,
    /// `screens.ratio`, per cent.
    pub ratio: u8,
    /// `screens.gap`, console pixels.
    pub gap: u32,
    /// `screens.rot`.
    pub rotation: Rotation,
    /// `screens.custom` (empty: none saved).
    pub custom: CustomLayout,
    /// `screens.swap` is `hold`: the screens are swapped only while the shortcut is held.
    pub swap_hold: bool,
    /// `video.aspect` (every console: one screen or the composed two).
    pub aspect: Aspect,
    /// `video.scale` (every console), the device's default included.
    pub scale: Scale,
}

impl Default for ScreenSettings {
    fn default() -> Self {
        Self::from_settings(|_| None)
    }
}

impl ScreenSettings {
    /// From the resolved settings (`get(key)`, `None`: the catalogue's default). A value of the
    /// wrong shape is the default; an unknown layout id in the cycle is skipped.
    pub fn from_settings(get: impl Fn(&str) -> Option<Value>) -> Self {
        let settings = Reader::new(get);
        let custom =
            CustomLayout::from_items(&settings.list(keys::SCREENS_CUSTOM)).unwrap_or_default();
        let cycle = settings
            .list(keys::SCREENS_CYCLE)
            .iter()
            .filter_map(|id| LayoutId::from_id(id))
            .filter(|l| *l != LayoutId::Custom || !custom.is_empty())
            .collect();
        let clamp = |v: i64, max: i64| v.clamp(0, max);
        Self {
            cycle,
            ratio: u8::try_from(clamp(settings.int(keys::SCREENS_RATIO), 100)).unwrap_or(70),
            gap: u32::try_from(clamp(settings.int(keys::SCREENS_GAP), 1000)).unwrap_or(0),
            rotation: Rotation::from_id(&settings.text(keys::SCREENS_ROTATION)).unwrap_or_default(),
            custom,
            swap_hold: settings.text(keys::SCREENS_SWAP) == "hold",
            aspect: Aspect::from_id(&settings.text(keys::VIDEO_ASPECT)).unwrap_or_default(),
            scale: Scale::from_id(&settings.text(keys::VIDEO_SCALE)).unwrap_or_default(),
        }
    }

    /// Whether `layout` can be shown: every one but `custom` with no custom layout saved.
    pub fn available(&self, layout: LayoutId) -> bool {
        layout != LayoutId::Custom || !self.custom.is_empty()
    }

    /// The layout a game opens with: the first of the cycle (vertical with an empty cycle).
    pub fn first(&self) -> LayoutId {
        self.cycle.first().copied().unwrap_or(LayoutId::Vertical)
    }

    /// The layout after `current` in the cycle, wrapping; from a layout outside the cycle, the
    /// first.
    pub fn next(&self, current: LayoutId) -> LayoutId {
        match self.cycle.iter().position(|l| *l == current) {
            Some(i) => self.cycle[(i + 1) % self.cycle.len()],
            None => self.first(),
        }
    }

    /// After a settings change: `current` stays while it can be shown, else the first.
    pub fn keep(&self, current: LayoutId) -> LayoutId {
        if self.available(current) {
            current
        } else {
            self.first()
        }
    }

    /// The [`Layout`] to compose with `current` shown and the screens `swapped` or not. A
    /// layout that cannot be shown (custom with none saved) is the first of the cycle.
    pub fn layout(&self, current: LayoutId, swapped: bool) -> Layout {
        let arrangement = match self.keep(current) {
            LayoutId::Vertical => Arrangement::Stacked,
            LayoutId::Auto => Arrangement::Auto,
            LayoutId::Side => Arrangement::SideBySide,
            LayoutId::BigSmall => Arrangement::BigSmall,
            LayoutId::TopOnly => Arrangement::FirstOnly,
            LayoutId::Custom if !self.custom.is_empty() => self.custom.arrangement(),
            LayoutId::Custom => Arrangement::Stacked,
        };
        Layout {
            arrangement,
            swap: swapped,
            gap: self.gap,
            ratio: self.ratio,
            rotation: self.rotation,
            aspect: self.aspect,
            scale: self.scale,
        }
    }
}

/// The screens of the open game as the interface shows them (quick menu G09, the tools, the G15
/// lid overlay): sent on `emu://screens` at every change and answered by `screens_status`.
/// TypeScript: `ScreenStatus` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ScreenStatus {
    /// A two-screen game (NDS, 3DS) is open: the rest means something only then.
    pub dual_screen: bool,
    /// The layout shown now.
    pub layout: LayoutId,
    /// The layouts L3 + R3 and «next layout» step through, in order (`screens.cycle`, `custom`
    /// only while one is saved).
    pub cycle: Vec<LayoutId>,
    /// The screens are swapped now.
    pub swapped: bool,
    /// `screens.swap` is `hold`: L3 + Y swaps only while held.
    pub swap_hold: bool,
    /// A custom layout is saved for this game or its console (`screens.custom`).
    pub has_custom: bool,
    /// The core can close the lid; otherwise the lid is shown disabled with the reason.
    pub lid_supported: bool,
    /// The lid is closed now (the G15 overlay «Coperchio chiuso»).
    pub lid_closed: bool,
    /// The phone's own placement of the screens (the touch layout's) is drawn in place of the
    /// layout: the layout cycle waits until it is withdrawn.
    pub phone_placement: bool,
}

impl Default for ScreenStatus {
    /// No game open.
    fn default() -> Self {
        ScreenSession::new(ScreenSettings::default(), false, false).status()
    }
}

/// What the user does with the open game's screens, on top of its settings: the layout shown,
/// the screens swapped, the lid. Pure; the shell (`app/src-tauri/src/screens.rs`) carries the
/// changes out (renderer, interface, the core's lid). A one-screen game changes nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenSession {
    settings: ScreenSettings,
    dual_screen: bool,
    lid_supported: bool,
    current: LayoutId,
    swapped: bool,
    lid_closed: bool,
    /// The phone's own placement of the screens while the touch controls are in use (empty:
    /// none, the layout is drawn).
    phone: Vec<CustomScreen>,
}

impl ScreenSession {
    /// A game opens: the first layout of its cycle, not swapped, lid open.
    pub fn new(settings: ScreenSettings, dual_screen: bool, lid_supported: bool) -> Self {
        Self {
            current: settings.first(),
            settings,
            dual_screen,
            lid_supported: dual_screen && lid_supported,
            swapped: false,
            lid_closed: false,
            phone: Vec::new(),
        }
    }

    /// The settings changed (or their session «in prova»): the layout shown stays while it can.
    pub fn settings_changed(&mut self, settings: ScreenSettings) {
        self.current = settings.keep(self.current);
        self.settings = settings;
    }

    pub fn settings(&self) -> &ScreenSettings {
        &self.settings
    }

    /// The next layout of the cycle (L3 + R3, the quick menu). False: nothing changed (also
    /// while the phone's placement is drawn).
    pub fn next_layout(&mut self) -> bool {
        self.phone.is_empty() && self.show(self.settings.next(self.current))
    }

    /// Shows `layout` (the quick menu, the editor's live preview), in the cycle or not. `None`:
    /// it cannot be shown (a custom layout not saved, a one-screen game, the phone's placement
    /// drawn).
    pub fn set_layout(&mut self, layout: LayoutId) -> Option<bool> {
        if !self.dual_screen || !self.settings.available(layout) || !self.phone.is_empty() {
            return None;
        }
        Some(self.show(layout))
    }

    /// The phone's own placement of the screens (the touch layout in use), drawn in place of the
    /// layout; empty withdraws it. `None`: it is not a placement of exactly this game's screens
    /// ([`placement_fits`]); otherwise whether anything changed.
    pub fn set_phone_placement(&mut self, screens: Vec<CustomScreen>) -> Option<bool> {
        let ids: &[ScreenId] = if self.dual_screen {
            &[ScreenId::Top, ScreenId::Bottom]
        } else {
            &[ScreenId::Main]
        };
        if !placement_fits(&screens, ids) {
            return None;
        }
        let changed = self.phone != screens;
        self.phone = screens;
        Some(changed)
    }

    fn show(&mut self, layout: LayoutId) -> bool {
        let changed = self.dual_screen && self.current != layout;
        if changed {
            self.current = layout;
        }
        changed
    }

    /// L3 + Y went down: toggles the screens, or swaps them until released with
    /// `screens.swap` = hold. False: nothing changed.
    pub fn swap_pressed(&mut self) -> bool {
        if self.settings.swap_hold {
            self.set_swapped(true)
        } else {
            self.toggle_swap()
        }
    }

    /// L3 + Y let go: with `screens.swap` = hold, the screens go back.
    pub fn swap_released(&mut self) -> bool {
        self.settings.swap_hold && self.set_swapped(false)
    }

    /// The quick menu's «Scambia gli schermi»: always a toggle.
    pub fn toggle_swap(&mut self) -> bool {
        self.set_swapped(!self.swapped)
    }

    fn set_swapped(&mut self, swapped: bool) -> bool {
        let changed = self.dual_screen && self.swapped != swapped;
        if changed {
            self.swapped = swapped;
        }
        changed
    }

    /// L3 + Select, the quick menu: the lid flips. False when the core cannot close it (then
    /// nothing is pressed and nothing changes).
    pub fn toggle_lid(&mut self) -> bool {
        if self.lid_supported {
            self.lid_closed = !self.lid_closed;
        }
        self.lid_supported
    }

    /// The layout to compose now. Its size is `video.scale`'s, whose default is the device's
    /// (as large as fits on a phone, whole scales on a computer).
    /// The phone's placement, while one is set, in place of the layout's arrangement (turned
    /// and swapped the same way; `video.aspect` / `video.scale` do not apply to it).
    pub fn layout(&self) -> Layout {
        let layout = self.settings.layout(self.current, self.swapped);
        if self.phone.is_empty() {
            layout
        } else {
            Layout {
                arrangement: Arrangement::Phone(given(&self.phone)),
                ..layout
            }
        }
    }

    /// Where the touch editor starts this game's screens in `area` (the whole window, the edges
    /// the controls keep clear as its inset) when no placement of the phone's own is saved: where
    /// the layout shown now puts them (cycled, swapped, at the frame's real size, `video.scale`
    /// applied), as fractions of the window, so the game does not move at the first drag. A
    /// layout showing one screen of two starts every screen as [`Arrangement::Auto`] places them.
    /// `sources`: the open frame's screens.
    pub fn phone_start(&self, sources: &[SourceScreen], area: Area) -> Vec<CustomScreen> {
        start_placement(
            self.settings.layout(self.current, self.swapped),
            sources,
            area,
        )
    }

    pub fn status(&self) -> ScreenStatus {
        ScreenStatus {
            dual_screen: self.dual_screen,
            layout: self.settings.keep(self.current),
            cycle: self.settings.cycle.clone(),
            swapped: self.swapped,
            swap_hold: self.settings.swap_hold,
            has_custom: !self.settings.custom.is_empty(),
            lid_supported: self.lid_supported,
            lid_closed: self.lid_closed,
            phone_placement: !self.phone.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compose, console_frame, fit, phone_start, pointer_at, pointer_value, preview, touch_at,
        Align, Area, Arrangement, Aspect, CustomLayout, CustomScreen, Inset, Layout, LayoutId,
        Placement, PxRect, Rect, Rotation, Scale, ScreenId, ScreenSession, ScreenSettings,
        ScreenStatus, SourceScreen, Split,
    };
    use crate::settings::Value;

    fn settings(values: &[(&str, Value)]) -> ScreenSettings {
        ScreenSettings::from_settings(|key| {
            values
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        })
    }

    fn list(items: &[&str]) -> Value {
        Value::List(items.iter().map(|s| (*s).to_owned()).collect())
    }

    const CUSTOM: &[&str] = &["top:0,0,0.5,0.5", "bottom:0.5,0,0.5,1"];

    #[test]
    fn the_default_settings_are_the_default_layout() {
        let s = ScreenSettings::default();
        assert_eq!(
            s.cycle,
            [
                LayoutId::Vertical,
                LayoutId::Side,
                LayoutId::BigSmall,
                LayoutId::TopOnly
            ]
        );
        assert_eq!(s.layout(s.first(), false), Layout::default());
        assert!(!s.swap_hold);
        assert!(s.custom.is_empty());
    }

    #[test]
    fn the_settings_choose_the_layout() {
        let s = settings(&[
            ("screens.cycle", list(&["big_small", "vertical", "custom"])),
            ("screens.ratio", Value::Int(80)),
            ("screens.gap", Value::Int(90)),
            ("screens.rot", Value::Text("left".into())),
            ("screens.swap", Value::Text("hold".into())),
        ]);
        // No custom layout saved: `custom` is not stepped to.
        assert_eq!(s.cycle, [LayoutId::BigSmall, LayoutId::Vertical]);
        assert_eq!(s.first(), LayoutId::BigSmall);
        assert_eq!(
            s.layout(LayoutId::BigSmall, true),
            Layout {
                arrangement: Arrangement::BigSmall,
                swap: true,
                gap: 90,
                ratio: 80,
                rotation: Rotation::Left,
                aspect: Aspect::Original,
                scale: Scale::Integer,
            }
        );
        assert_eq!(
            s.layout(LayoutId::TopOnly, false).arrangement,
            Arrangement::FirstOnly
        );
        // A custom layout asked for with none saved is the first of the cycle.
        assert_eq!(
            s.layout(LayoutId::Custom, false).arrangement,
            Arrangement::BigSmall
        );
        assert!(s.swap_hold);
        // Values of the wrong shape are the defaults; an empty cycle is vertical.
        let odd = settings(&[
            ("screens.cycle", list(&[])),
            ("screens.rot", Value::Int(3)),
            ("screens.custom", list(&["top:nonsense"])),
        ]);
        assert_eq!(odd.first(), LayoutId::Vertical);
        assert_eq!(odd.rotation, Rotation::None);
        assert!(odd.custom.is_empty());
    }

    #[test]
    fn the_cycle_wraps_and_a_layout_outside_it_steps_to_the_first() {
        let s = settings(&[
            ("screens.cycle", list(&["side", "custom", "top_only"])),
            ("screens.custom", list(CUSTOM)),
        ]);
        assert_eq!(s.next(LayoutId::Side), LayoutId::Custom);
        assert_eq!(s.next(LayoutId::Custom), LayoutId::TopOnly);
        assert_eq!(s.next(LayoutId::TopOnly), LayoutId::Side);
        assert_eq!(s.next(LayoutId::BigSmall), LayoutId::Side);
        assert_eq!(s.keep(LayoutId::BigSmall), LayoutId::BigSmall);
        assert!(matches!(
            s.layout(LayoutId::Custom, false).arrangement,
            Arrangement::Custom(_)
        ));
    }

    #[test]
    fn a_custom_layout_round_trips_and_bad_ones_are_refused() {
        let items: Vec<String> = CUSTOM.iter().map(|s| (*s).to_owned()).collect();
        let layout = CustomLayout::from_items(&items).unwrap();
        assert_eq!(
            layout.screens[1],
            CustomScreen {
                id: ScreenId::Bottom,
                x: 0.5,
                y: 0.0,
                w: 0.5,
                h: 1.0
            }
        );
        assert_eq!(layout.to_items(), items);
        // Rounded to 4 decimals when written.
        let precise = CustomLayout {
            screens: vec![CustomScreen {
                id: ScreenId::Top,
                x: 0.123_456,
                y: 0.0,
                w: 0.5,
                h: 0.333_333,
            }],
        };
        assert_eq!(precise.to_items(), ["top:0.1235,0,0.5,0.3333"]);
        assert!(CustomLayout::valid_items(&[]));
        for bad in [
            &["top:0,0,0.5"][..],
            &["top:0,0,0.5,0.5,1"],
            &["main:0,0,1,1"],
            &["left:0,0,1,1"],
            &["top:0,0,1,1", "top:0,0,1,1"],
            &["top:0.6,0,0.5,0.5"],
            &["top:0,0,0,0.5"],
            &["top:-0.1,0,0.5,0.5"],
            &["top:NaN,0,0.5,0.5"],
            &["top 0,0,1,1"],
        ] {
            let items: Vec<String> = bad.iter().map(|s| (*s).to_owned()).collect();
            assert!(!CustomLayout::valid_items(&items), "{bad:?}");
        }
        // The settings engine refuses them the same way.
        let entry = crate::settings::schema::entry("screens.custom").unwrap();
        assert!(entry.accepts_value(&list(CUSTOM)));
        assert!(!entry.accepts_value(&list(&["top:2,0,1,1"])));
        assert!(!entry.accepts_value(&Value::Text("top:0,0,1,1".into())));
    }

    #[test]
    fn a_preset_is_where_the_layout_draws_the_screens() {
        let s = ScreenSettings::default();
        // Vertical at 1080p: the DS stack at 2x (512 x 768), centred, as `fit` places it.
        let v = CustomLayout::preset(&s, LayoutId::Vertical, DS, 1920.0, 1080.0);
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        let top = v.screens[0];
        assert_eq!(top.id, ScreenId::Top);
        assert!(near(top.x, 704.0 / 1920.0) && near(top.y, 156.0 / 1080.0));
        assert!(near(top.w, 512.0 / 1920.0) && near(top.h, 384.0 / 1080.0));
        assert_eq!(v.screens[1].id, ScreenId::Bottom);
        assert!(near(v.screens[1].y, (156.0 + 384.0) / 1080.0));
        // Every preset of both consoles is a layout the editor may save, back to front.
        for split in [DS, Split::N3dsStacked] {
            for layout in [
                LayoutId::Vertical,
                LayoutId::Side,
                LayoutId::BigSmall,
                LayoutId::TopOnly,
            ] {
                let p = CustomLayout::preset(&s, layout, split, 1280.0, 720.0);
                assert!(p.is_valid() && !p.is_empty(), "{layout:?} {split:?}");
                assert!(CustomLayout::valid_items(&p.to_items()));
            }
        }
        // Only the top screen: one rectangle.
        let only = CustomLayout::preset(&s, LayoutId::TopOnly, DS, 1280.0, 720.0);
        assert_eq!(only.screens.len(), 1);
        // The settings' ratio and gap count; the rotation does not.
        let wide = settings(&[
            ("screens.ratio", Value::Int(50)),
            ("screens.rot", Value::Text("left".into())),
        ]);
        let half = CustomLayout::preset(&wide, LayoutId::BigSmall, DS, 1280.0, 720.0);
        assert!(near(half.screens[0].w, half.screens[1].w));
        assert!(near(half.screens[0].y, half.screens[1].y));
        // Custom: the one saved; one screen or no room: none.
        let items: Vec<String> = CUSTOM.iter().map(|s| (*s).to_owned()).collect();
        let saved = settings(&[("screens.custom", Value::List(items))]);
        assert_eq!(
            CustomLayout::preset(&saved, LayoutId::Custom, DS, 1280.0, 720.0),
            saved.custom
        );
        assert!(CustomLayout::preset(&s, LayoutId::Side, Split::Whole, 1280.0, 720.0).is_empty());
        assert!(CustomLayout::preset(&s, LayoutId::Side, DS, 0.0, 720.0).is_empty());
        assert_eq!(DS.frame_at_1x(), Some((256, 384)));
        assert_eq!(Split::N3dsStacked.frame_at_1x(), Some((400, 480)));
    }

    #[test]
    fn a_custom_layout_swaps_its_screens_and_turns_as_a_whole() {
        let items: Vec<String> = CUSTOM.iter().map(|s| (*s).to_owned()).collect();
        let s = settings(&[("screens.custom", Value::List(items))]);
        let a = area(1000.0, 1000.0);
        let swapped = ds(&s.layout(LayoutId::Custom, true), a);
        // The bottom screen takes the top one's rectangle.
        assert_eq!(
            dst(&swapped, ScreenId::Bottom),
            rect(0.0, 0.0, 500.0, 500.0)
        );
        assert_eq!(
            dst(&swapped, ScreenId::Top),
            rect(500.0, 0.0, 500.0, 1000.0)
        );
        // Turned left: the right half becomes the top half.
        let turned = ScreenSettings {
            rotation: Rotation::Left,
            ..s
        };
        let p = ds(&turned.layout(LayoutId::Custom, false), a);
        assert_eq!(dst(&p, ScreenId::Bottom), rect(0.0, 0.0, 1000.0, 500.0));
        assert_eq!(dst(&p, ScreenId::Top), rect(0.0, 500.0, 500.0, 500.0));
        // Touch follows: the bottom screen's top-left corner in the window is the screen's
        // top-right (anticlockwise).
        assert_eq!(touch_at(&p, 1.0, 1.0), Some((255, 0)));
    }

    #[test]
    fn touch_maps_through_a_custom_layout() {
        let items: Vec<String> = CUSTOM.iter().map(|s| (*s).to_owned()).collect();
        let s = settings(&[("screens.custom", Value::List(items))]);
        let a = area(1000.0, 1000.0);
        let layout = s.layout(LayoutId::Custom, false);
        // The bottom screen fills the right half: 500 x 1000 for 256 x 192.
        let p = ds(&layout, a);
        assert_eq!(touch_at(&p, 500.0, 0.0), Some((0, 0)));
        assert_eq!(touch_at(&p, 750.0, 500.0), Some((128, 96)));
        assert_eq!(touch_at(&p, 250.0, 250.0), None, "the top screen");
        let pointer = pointer_at(&layout, DS, FRAME, a, 750.0, 500.0).unwrap();
        assert_eq!(melonds_reads(pointer, FRAME), (128, 192 + 96));
    }

    #[test]
    fn the_session_steps_swaps_and_closes_the_lid() {
        let s = settings(&[("screens.swap", Value::Text("hold".into()))]);
        let mut session = ScreenSession::new(s, true, true);
        assert_eq!(session.status().layout, LayoutId::Vertical);
        assert!(session.next_layout());
        assert_eq!(session.status().layout, LayoutId::Side);
        assert_eq!(session.set_layout(LayoutId::TopOnly), Some(true));
        assert_eq!(session.set_layout(LayoutId::TopOnly), Some(false));
        assert_eq!(session.set_layout(LayoutId::Custom), None, "none saved");
        // Hold: swapped while held.
        assert!(session.swap_pressed());
        assert!(session.layout().swap);
        assert!(session.swap_released());
        assert!(!session.layout().swap);
        // The quick menu toggles whatever the setting.
        assert!(session.toggle_swap());
        assert!(session.status().swapped);
        assert!(session.toggle_swap());
        // The lid flips.
        assert!(session.toggle_lid());
        assert!(session.status().lid_closed);
        assert!(session.toggle_lid());
        assert!(!session.status().lid_closed);
        // Toggle mode: each press flips, the release does nothing.
        let mut toggle = ScreenSession::new(ScreenSettings::default(), true, false);
        assert!(toggle.swap_pressed());
        assert!(!toggle.swap_released());
        assert!(toggle.status().swapped);
        // A core with no lid: nothing changes.
        assert!(!toggle.toggle_lid());
        assert!(!toggle.status().lid_supported && !toggle.status().lid_closed);
    }

    #[test]
    fn a_settings_change_keeps_the_layout_while_it_can() {
        let items: Vec<String> = CUSTOM.iter().map(|s| (*s).to_owned()).collect();
        let with_custom = settings(&[
            ("screens.cycle", list(&["custom", "side"])),
            ("screens.custom", Value::List(items)),
        ]);
        let mut session = ScreenSession::new(with_custom.clone(), true, true);
        assert_eq!(session.status().layout, LayoutId::Custom);
        assert!(session.status().has_custom);
        // The editor's layout removed: back to the first of the cycle.
        session.settings_changed(settings(&[("screens.cycle", list(&["custom", "side"]))]));
        assert_eq!(session.status().layout, LayoutId::Side);
        assert!(!session.status().has_custom);
        // Another change keeps what is shown.
        session.settings_changed(with_custom);
        assert_eq!(session.status().layout, LayoutId::Side);
    }

    #[test]
    fn a_one_screen_game_changes_nothing() {
        let mut session = ScreenSession::new(ScreenSettings::default(), false, true);
        assert!(!session.next_layout());
        assert_eq!(session.set_layout(LayoutId::Side), None);
        assert!(!session.swap_pressed());
        assert!(!session.toggle_lid());
        assert_eq!(session.layout(), Layout::default());
        assert!(!session.status().dual_screen);
        assert_eq!(ScreenStatus::default(), session.status());
    }

    #[test]
    fn the_interface_reads_the_ids_and_shapes_it_expects() {
        assert_eq!(
            serde_json::to_value(LayoutId::BigSmall).unwrap(),
            serde_json::json!("big_small")
        );
        for l in LayoutId::ALL {
            assert_eq!(serde_json::to_value(l).unwrap(), serde_json::json!(l.id()));
            assert_eq!(LayoutId::from_id(l.id()), Some(l));
        }
        // Every layout id is a `screens.cycle` option.
        let entry = crate::settings::schema::entry("screens.cycle").unwrap();
        assert!(entry.accepts_value(&Value::List(
            LayoutId::ALL.iter().map(|l| l.id().to_owned()).collect()
        )));
        let status = serde_json::to_value(ScreenStatus::default()).unwrap();
        assert_eq!(status["layout"], "vertical");
        assert_eq!(status["lidClosed"], false);
        let custom: CustomLayout = serde_json::from_value(serde_json::json!({
            "screens": [{ "id": "bottom", "x": 0.0, "y": 0.5, "w": 1.0, "h": 0.5 }]
        }))
        .unwrap();
        assert!(custom.is_valid());
        assert_eq!(
            serde_json::to_value(Rotation::Left).unwrap(),
            serde_json::json!("left")
        );
    }

    const DS: Split = Split::DsStacked { gap: 0 };
    const FRAME: (u32, u32) = (256, 384);

    fn area(width: f32, height: f32) -> Area {
        Area {
            width,
            height,
            inset: Inset::NONE,
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
                inset: Inset::bottom(0.3),
            },
            (240.0, 160.0),
        );
        assert!(r.y + r.h <= 630.0);
        assert_eq!(fit(area(10.0, 10.0), (0.0, 5.0)), Rect::default());
    }

    #[test]
    fn fit_keeps_every_edge_of_the_inset_clear() {
        // The phone on its side (#29), 892 x 412 CSS px at 2.625: the touch controls' columns
        // keep 211 px on the left and 204 on the right; GBA fits 5x between them, centred there.
        let (w, h) = (2341.5, 1081.5);
        let r = fit(
            Area {
                width: w,
                height: h,
                inset: Inset {
                    left: 211.0 / 892.0,
                    right: 204.0 / 892.0,
                    ..Inset::NONE
                },
            },
            (240.0, 160.0),
        );
        assert_eq!((r.w, r.h), (1200.0, 800.0));
        let (left, right) = (w * 211.0 / 892.0, w - w * 204.0 / 892.0);
        assert!(r.x >= left.floor() && r.x + r.w <= right.ceil(), "{r:?}");
        // Centred in the band, not in the window.
        assert!(
            ((r.x + r.w / 2.0) - (left + right) / 2.0).abs() <= 1.0,
            "{r:?}"
        );
        assert_eq!(r.y, ((h - 800.0) / 2.0).floor());

        // The quick menu's side panel: the right share only.
        let side = fit(
            Area {
                width: 892.0,
                height: 412.0,
                inset: Inset {
                    right: 440.0 / 892.0,
                    ..Inset::NONE
                },
            },
            (240.0, 160.0),
        );
        assert!(side.x + side.w <= 452.0 + 0.01, "{side:?}");
    }

    #[test]
    fn fit_against_the_bottom_leaves_no_band_above_the_controls() {
        // Upright, 412 x 892: the touch controls keep the bottom 368 px; the game sits on them.
        let r = fit(
            Area {
                width: 412.0,
                height: 892.0,
                inset: Inset {
                    bottom: 368.0 / 892.0,
                    align: Align::Bottom,
                    ..Inset::NONE
                },
            },
            (240.0, 160.0),
        );
        assert_eq!(r.y + r.h, 524.0);
        assert_eq!(r.x, ((412.0 - r.w) / 2.0).floor());
        // Centred, the same band splits above and below.
        let c = fit(
            Area {
                width: 412.0,
                height: 892.0,
                inset: Inset::bottom(368.0 / 892.0),
            },
            (240.0, 160.0),
        );
        assert!(c.y > 0.0 && c.y + c.h < 524.0);
    }

    #[test]
    fn an_inset_always_leaves_a_tenth_of_the_window() {
        let free = Inset {
            left: 0.7,
            right: 0.7,
            top: f32::NAN,
            bottom: 2.0,
            align: Align::Centre,
            between: 0.0,
        }
        .usable(1000.0, 1000.0);
        assert!((free.w - 100.0).abs() < 0.01 && (free.x - 450.0).abs() < 0.01);
        assert!((free.h - 100.0).abs() < 0.01 && free.y == 0.0);
        assert_eq!(
            Inset::NONE.usable(800.0, 600.0),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 800.0,
                h: 600.0
            }
        );
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
                inset: Inset::bottom(0.2),
            },
        );
        assert_eq!(dst(&p, ScreenId::Bottom), rect(500.0, 0.0, 500.0, 800.0));
        assert_eq!(dst(&p, ScreenId::Top), rect(0.0, 0.0, 500.0, 400.0));
        // Side insets: the rectangles are fractions of the band between them.
        let banded = ds(
            &Layout {
                arrangement: Arrangement::Custom(vec![(ScreenId::Top, rect(0.0, 0.0, 0.5, 0.5))]),
                ..Layout::default()
            },
            Area {
                width: 1000.0,
                height: 1000.0,
                inset: Inset {
                    left: 0.2,
                    right: 0.2,
                    ..Inset::NONE
                },
            },
        );
        assert_eq!(dst(&banded, ScreenId::Top), rect(200.0, 0.0, 300.0, 500.0));
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
    fn the_aspect_stretches_a_one_screen_game_across() {
        let gba = Split::Whole.screens(240, 160);
        let original = compose(&Layout::default(), &gba, area(1280.0, 800.0));
        assert_eq!(
            dst(&original, ScreenId::Main),
            rect(40.0, 0.0, 1200.0, 800.0)
        );
        // 4:3: the console's 160 rows, 213⅓ columns wide: 5x fits 800 rows.
        let four_three = compose(
            &Layout {
                aspect: Aspect::FourThree,
                ..Layout::default()
            },
            &gba,
            area(1280.0, 800.0),
        );
        let r = dst(&four_three, ScreenId::Main);
        assert_eq!((r.x, r.y, r.h), (106.0, 0.0, 800.0));
        assert!((r.w / r.h - 4.0 / 3.0).abs() < 1e-4, "{r:?}");
        let wide = compose(
            &Layout {
                aspect: Aspect::SixteenNine,
                ..Layout::default()
            },
            &gba,
            area(1920.0, 1080.0),
        );
        let r = dst(&wide, ScreenId::Main);
        assert!((r.w / r.h - 16.0 / 9.0).abs() < 1e-4, "{r:?}");
        assert!(r.x >= 0.0 && r.x + r.w <= 1920.0 && r.h <= 1080.0);
    }

    #[test]
    fn two_screens_stretch_as_one_picture_and_touch_follows() {
        let layout = Layout {
            aspect: Aspect::FourThree,
            ..Layout::default()
        };
        // 256 x 384 shown as 512 x 384: 2x fits 1080 rows, 1024 x 768 centred.
        let p = ds(&layout, area(1920.0, 1080.0));
        assert_eq!(dst(&p, ScreenId::Top), rect(448.0, 156.0, 1024.0, 384.0));
        assert_eq!(dst(&p, ScreenId::Bottom), rect(448.0, 540.0, 1024.0, 384.0));
        // The middle of the stretched touch screen is the middle of the console's.
        assert_eq!(touch_at(&p, 448.0 + 512.0, 540.0 + 192.0), Some((128, 96)));
        assert_eq!(touch_at(&p, 1471.9, 923.9), Some((255, 191)));
        // A custom layout is drawn as placed, whatever the aspect.
        let custom = settings(&[("screens.custom", list(CUSTOM))])
            .custom
            .arrangement();
        let plain = Layout {
            arrangement: custom,
            ..Layout::default()
        };
        assert_eq!(
            ds(&plain, area(1920.0, 1080.0)),
            ds(
                &Layout {
                    aspect: Aspect::SixteenNine,
                    ..plain.clone()
                },
                area(1920.0, 1080.0)
            )
        );
    }

    #[test]
    fn the_aspect_comes_from_its_setting() {
        assert_eq!(ScreenSettings::default().aspect, Aspect::Original);
        let s = settings(&[("video.aspect", Value::Text("16_9".into()))]);
        assert_eq!(s.layout(s.first(), false).aspect, Aspect::SixteenNine);
        let s = settings(&[("video.aspect", Value::Text("21_9".into()))]);
        assert_eq!(s.aspect, Aspect::Original, "an unknown id is the console's");
        for (id, ratio) in [
            ("original", None),
            ("4_3", Some(4.0 / 3.0)),
            ("16_9", Some(16.0 / 9.0)),
        ] {
            assert_eq!(Aspect::from_id(id).map(Aspect::ratio), Some(ratio), "{id}");
        }
    }

    #[test]
    fn ids_of_the_settings_are_known() {
        for id in ["vertical", "side", "big_small", "top_only"] {
            assert!(Arrangement::from_id(id).is_some(), "{id}");
        }
        assert_eq!(Arrangement::from_id("hybrid"), None);
        assert_eq!(Arrangement::from_id("auto"), Some(Arrangement::Auto));
        for id in ["none", "left", "right", "flip"] {
            assert!(Rotation::from_id(id).is_some(), "{id}");
        }
    }

    // ---- The phone (owner, 2026-11-22): two screens apart and large ----

    /// A phone's window in CSS pixels, its edges kept clear in pixels as the touch controls
    /// measure them (`touchLayout.ts`, `reservedInset`), turned into shares as `inset.ts` does.
    fn phone(
        width: f32,
        height: f32,
        (top, right, bottom, left): (f32, f32, f32, f32),
        align: Align,
        between: f32,
    ) -> Area {
        Area {
            width,
            height,
            inset: Inset {
                left: left / width,
                top: top / height,
                right: right / width,
                bottom: bottom / height,
                align,
                between: between / height,
            },
        }
    }

    /// Upright: the camera's 32 px at the top, the controls' band at the bottom, the 44 px strip
    /// (L, menu, R) between the screens.
    fn upright(width: f32, height: f32, band: f32) -> Area {
        phone(width, height, (32.0, 0.0, band, 0.0), Align::Apart, 44.0)
    }

    /// On its side: the camera's 32 px on the left, 16 px on the right, 8 px between the screens.
    fn on_its_side(width: f32, height: f32) -> Area {
        phone(width, height, (0.0, 16.0, 0.0, 32.0), Align::Top, 8.0)
    }

    /// The phone's layout: `auto`, the screens filling their room (`video.scale` = fit).
    fn auto() -> Layout {
        Layout {
            arrangement: Arrangement::Auto,
            scale: Scale::Fit,
            ..Layout::default()
        }
    }

    fn n3ds(layout: &Layout, a: Area) -> Vec<Placement> {
        compose(layout, &Split::N3dsStacked.screens(400, 480), a)
    }

    fn near(a: Rect, b: Rect) -> bool {
        let close = |x: f32, y: f32| (x - y).abs() < 0.01;
        close(a.x, b.x) && close(a.y, b.y) && close(a.w, b.w) && close(a.h, b.h)
    }

    #[track_caller]
    fn assert_near(a: Rect, b: Rect) {
        assert!(near(a, b), "{a:?} is not {b:?}");
    }

    #[test]
    fn a_ds_upright_on_the_s23_shows_both_screens_full_width_apart() {
        // 412 x 892, the band 198 px: top 32-341, strip 341-385, touch screen 385-694 (1.61x).
        let p = ds(&auto(), upright(412.0, 892.0, 198.0));
        assert_near(dst(&p, ScreenId::Top), rect(0.0, 32.0, 412.0, 309.0));
        assert_near(dst(&p, ScreenId::Bottom), rect(0.0, 385.0, 412.0, 309.0));
    }

    #[test]
    fn a_ds_on_its_side_on_the_s23_shows_the_screens_side_by_side_at_the_top() {
        // 892 x 412: 32 + 418 + 8 + 418 + 16, top-aligned, 1.63x.
        let p = ds(&auto(), on_its_side(892.0, 412.0));
        assert_near(dst(&p, ScreenId::Top), rect(32.0, 0.0, 418.0, 313.5));
        assert_near(dst(&p, ScreenId::Bottom), rect(458.0, 0.0, 418.0, 313.5));
    }

    #[test]
    fn a_small_phone_scales_both_screens_down_alike_and_keeps_the_strip() {
        // 360 x 740 upright, the band 224 px: the height decides, both screens 293.3 x 220,
        // centred, the strip still 44 px and the touch screen on the band.
        let p = ds(&auto(), upright(360.0, 740.0, 224.0));
        let (top, bottom) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
        let w = 220.0 * 4.0 / 3.0;
        assert_near(top, rect(((360.0 - w) / 2.0_f32).floor(), 32.0, w, 220.0));
        assert_near(bottom, rect(top.x, 296.0, w, 220.0));
        assert!((bottom.y + bottom.h - (740.0 - 224.0)).abs() < 0.01);
        // On its side, 740 x 360: the width decides, 342 x 256.5 each.
        let p = ds(&auto(), on_its_side(740.0, 360.0));
        assert_near(dst(&p, ScreenId::Top), rect(32.0, 0.0, 342.0, 256.5));
        assert_near(dst(&p, ScreenId::Bottom), rect(382.0, 0.0, 342.0, 256.5));
    }

    #[test]
    fn a_taller_phone_puts_the_room_left_over_into_the_strip() {
        // 412 x 915: full width as on the S23; the 23 px more go between the screens, so the
        // top screen stays under the camera and the touch screen on the band.
        let p = ds(&auto(), upright(412.0, 915.0, 198.0));
        assert_near(dst(&p, ScreenId::Top), rect(0.0, 32.0, 412.0, 309.0));
        assert_near(dst(&p, ScreenId::Bottom), rect(0.0, 408.0, 412.0, 309.0));
    }

    #[test]
    fn a_3ds_on_the_phone_keeps_both_screens_at_one_scale() {
        // Upright on the S23, the band 318 px: the top screen 412 wide (1.03x), the touch screen
        // 329.6 wide centred under it, the strip 47.6 px (44 and the room left over).
        let p = n3ds(&auto(), upright(412.0, 892.0, 318.0));
        assert_near(dst(&p, ScreenId::Top), rect(0.0, 32.0, 412.0, 247.2));
        assert_near(dst(&p, ScreenId::Bottom), rect(41.2, 326.8, 329.6, 247.2));
        // On its side: 464.4 + 8 + 371.6 from the camera's edge, top-aligned.
        let p = n3ds(&auto(), on_its_side(892.0, 412.0));
        let s = 836.0 / 720.0;
        assert_near(
            dst(&p, ScreenId::Top),
            rect(32.0, 0.0, 400.0 * s, 240.0 * s),
        );
        assert_near(
            dst(&p, ScreenId::Bottom),
            rect(32.0 + 400.0 * s + 8.0, 0.0, 320.0 * s, 240.0 * s),
        );
    }

    #[test]
    fn the_touch_screen_maps_through_the_phone_layout() {
        let a = upright(412.0, 892.0, 198.0);
        let p = ds(&auto(), a);
        // Its middle is the touch screen's middle; the strip is no screen.
        assert_eq!(touch_at(&p, 206.0, 385.0 + 154.5), Some((128, 96)));
        assert_eq!(touch_at(&p, 0.0, 385.0), Some((0, 0)));
        assert_eq!(touch_at(&p, 206.0, 363.0), None, "the strip");
        assert_eq!(touch_at(&p, 206.0, 200.0), None, "the top screen");
        let pointer = pointer_at(&auto(), DS, FRAME, a, 206.0, 385.0 + 154.5).unwrap();
        assert_eq!(melonds_reads(pointer, FRAME), (128, 192 + 96));
        // On its side the touch screen is on the right; the room between them is no screen.
        let a = on_its_side(892.0, 412.0);
        let p = ds(&auto(), a);
        assert_eq!(touch_at(&p, 458.0 + 209.0, 156.75), Some((128, 96)));
        assert_eq!(touch_at(&p, 454.0, 100.0), None, "between the screens");
        let pointer = pointer_at(&auto(), DS, FRAME, a, 458.0 + 209.0, 156.75).unwrap();
        assert_eq!(melonds_reads(pointer, FRAME), (128, 192 + 96));
        // Swapped upright: the touch screen on top, under the camera.
        let swapped = Layout {
            swap: true,
            ..auto()
        };
        let a = upright(412.0, 892.0, 198.0);
        let p = ds(&swapped, a);
        assert_near(dst(&p, ScreenId::Bottom), rect(0.0, 32.0, 412.0, 309.0));
        assert_eq!(touch_at(&p, 206.0, 32.0 + 154.5), Some((128, 96)));
    }

    #[test]
    fn the_room_between_follows_the_screens_when_the_picture_turns() {
        // Every turn of both arrangements: the two screens never overlap and keep the room (50
        // px) between them along the axis it turned to.
        let a = phone(1000.0, 1000.0, (0.0, 0.0, 0.0, 0.0), Align::Centre, 50.0);
        for arrangement in [Arrangement::Stacked, Arrangement::SideBySide] {
            for rotation in [
                Rotation::None,
                Rotation::Left,
                Rotation::Right,
                Rotation::Flip,
            ] {
                let layout = Layout {
                    arrangement: arrangement.clone(),
                    rotation,
                    scale: Scale::Fit,
                    ..Layout::default()
                };
                let p = ds(&layout, a);
                let (t, b) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
                let gap_x = (b.x - (t.x + t.w)).max(t.x - (b.x + b.w));
                let gap_y = (b.y - (t.y + t.h)).max(t.y - (b.y + b.h));
                let gap = gap_x.max(gap_y);
                assert!(
                    (gap - 50.0).abs() < 0.01,
                    "{arrangement:?} {rotation:?}: {gap}"
                );
                // Touch still lands on the touch screen's middle.
                let (cx, cy) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
                assert_eq!(
                    touch_at(&p, cx, cy),
                    Some((128, 96)),
                    "{arrangement:?} {rotation:?}"
                );
            }
        }
    }

    #[test]
    fn auto_follows_the_window_and_the_desktop_keeps_whole_scales() {
        // Auto in a landscape window is side by side, in a portrait one stacked.
        let wide = ds(
            &Layout {
                arrangement: Arrangement::Auto,
                ..Layout::default()
            },
            area(1920.0, 1080.0),
        );
        assert!(dst(&wide, ScreenId::Bottom).x > dst(&wide, ScreenId::Top).x);
        let tall = ds(
            &Layout {
                arrangement: Arrangement::Auto,
                ..Layout::default()
            },
            area(1080.0, 1920.0),
        );
        assert!(dst(&tall, ScreenId::Bottom).y > dst(&tall, ScreenId::Top).y);
        // The desktop: 1920 x 1080 stacked keeps 2x; filling, 1080 / 384 = 2.8125x.
        assert_eq!(
            dst(&ds(&Layout::default(), area(1920.0, 1080.0)), ScreenId::Top).w,
            512.0
        );
        let filled = Layout {
            scale: Scale::Fit,
            ..Layout::default()
        };
        assert_eq!(
            dst(&ds(&filled, area(1920.0, 1080.0)), ScreenId::Top).w,
            720.0
        );
        // No room asked for: nothing between the screens, whatever the alignment.
        let p = ds(
            &filled,
            phone(412.0, 892.0, (0.0, 0.0, 300.0, 0.0), Align::Apart, 0.0),
        );
        let (t, b) = (dst(&p, ScreenId::Top), dst(&p, ScreenId::Bottom));
        assert!(b.y >= t.y + t.h);
        // One screen with Apart sits on the band as with Bottom.
        let one = |align| {
            compose(
                &Layout::default(),
                &Split::Whole.screens(240, 160),
                phone(412.0, 892.0, (0.0, 0.0, 300.0, 0.0), align, 44.0),
            )[0]
            .dst
        };
        assert_eq!(one(Align::Apart), one(Align::Bottom));
    }

    /// The `video.scale` default `device` resolves to, as the settings engine hands it over.
    fn device_settings(device: crate::settings::schema::Device) -> ScreenSettings {
        let scale = crate::settings::schema::entry("video.scale")
            .and_then(|e| e.default_for(device, Some(crate::settings::ConsoleId::Gba)))
            .unwrap();
        settings(&[("video.scale", scale)])
    }

    #[test]
    fn the_session_sizes_the_picture_by_video_scale_and_its_device_default() {
        use crate::settings::schema::Device;
        // The defaults: as large as fits on a phone, whole scales on a computer; one screen or two.
        for dual in [false, true] {
            let phone = ScreenSession::new(device_settings(Device::Phone), dual, dual);
            assert_eq!(
                phone.layout().scale,
                Scale::Fit,
                "phone, two screens: {dual}"
            );
            let desk = ScreenSession::new(device_settings(Device::Desktop), dual, dual);
            assert_eq!(desk.layout().scale, Scale::Integer, "desktop, two: {dual}");
        }
        // The user's choice wins on any device; an unknown id is whole scales.
        for (id, scale) in [
            ("fit", Scale::Fit),
            ("integer", Scale::Integer),
            ("fill", Scale::Fill),
            ("stretch", Scale::Integer),
        ] {
            let s = settings(&[("video.scale", Value::Text(id.into()))]);
            assert_eq!(ScreenSession::new(s, false, false).layout().scale, scale);
        }
        let s = ScreenSettings::default();
        assert_eq!(s.layout(LayoutId::Vertical, false).scale, Scale::Integer);
        assert_eq!(
            s.layout(LayoutId::Auto, false).arrangement,
            Arrangement::Auto
        );
    }

    // ---- One screen on the phone (owner, 2026-11-22, Trello #105): as large as fits ----

    /// A GBA's frame composed in `a` sized by `scale`.
    fn gba(scale: Scale, a: Area) -> Rect {
        let layout = Layout {
            scale,
            ..Layout::default()
        };
        let p = compose(&layout, &Split::Whole.screens(240, 160), a);
        assert_eq!(p.len(), 1);
        p[0].dst
    }

    /// The S23 upright, 360 × 780 CSS px, the GBA controls' band 220 px at the bottom
    /// (`touchLayout.ts`), the game against it.
    fn s23_upright() -> Area {
        phone(360.0, 780.0, (0.0, 0.0, 220.0, 0.0), Align::Bottom, 0.0)
    }

    /// The S23 on its side, 780 × 360, the GBA controls' columns 148 px on the left and 144 px on
    /// the right (`touchLayout.ts`), the game centred between them.
    fn s23_on_its_side() -> Area {
        phone(780.0, 360.0, (0.0, 144.0, 0.0, 148.0), Align::Centre, 0.0)
    }

    #[test]
    fn a_gba_upright_on_the_s23_takes_the_whole_width_by_default() {
        // 360 / 240 = 1.5x: 360 x 240, on the band (560 - 240 = 320).
        assert_near(
            gba(Scale::Fit, s23_upright()),
            rect(0.0, 320.0, 360.0, 240.0),
        );
        // «Pixel interi» still floors: 1x, 240 x 160, centred across, on the band.
        assert_near(
            gba(Scale::Integer, s23_upright()),
            rect(60.0, 400.0, 240.0, 160.0),
        );
        // «Riempi lo schermo»: the whole room above the band, stretched.
        assert_near(
            gba(Scale::Fill, s23_upright()),
            rect(0.0, 0.0, 360.0, 560.0),
        );
    }

    #[test]
    fn a_gba_on_its_side_on_the_s23_fills_the_room_between_the_columns() {
        // 488 x 360 between the columns: 488 / 240 = 2.0333x, 488 x 325.33, centred down.
        let r = gba(Scale::Fit, s23_on_its_side());
        assert_near(r, rect(148.0, 17.0, 488.0, 488.0 * 160.0 / 240.0));
        // Whole scales: 2x, 480 x 320, centred in the room.
        assert_near(
            gba(Scale::Integer, s23_on_its_side()),
            rect(152.0, 20.0, 480.0, 320.0),
        );
        assert_near(
            gba(Scale::Fill, s23_on_its_side()),
            rect(148.0, 0.0, 488.0, 360.0),
        );
    }

    #[test]
    fn touch_and_both_renderers_map_through_the_same_scaled_screen() {
        // A DS at «fit» upright: the touch screen's middle is its middle pixel, as before.
        let p = ds(&auto(), upright(360.0, 780.0, 198.0));
        let b = dst(&p, ScreenId::Bottom);
        assert_eq!(
            touch_at(&p, b.x + b.w / 2.0, b.y + b.h / 2.0),
            Some((128, 96))
        );
        // Stretched: the corners still land on the corners.
        let fill = Layout {
            scale: Scale::Fill,
            ..auto()
        };
        let p = ds(&fill, upright(360.0, 780.0, 198.0));
        let b = dst(&p, ScreenId::Bottom);
        assert_eq!(touch_at(&p, b.x + 0.5, b.y + 0.5), Some((0, 0)));
        assert_eq!(
            touch_at(&p, b.x + b.w - 0.5, b.y + b.h - 0.5),
            Some((255, 191))
        );
    }

    #[test]
    fn fill_ignores_the_aspect_and_fit_keeps_it() {
        let a = area(1000.0, 500.0);
        let sixteen_nine = |scale| Layout {
            scale,
            aspect: Aspect::SixteenNine,
            ..Layout::default()
        };
        let p = compose(
            &sixteen_nine(Scale::Fill),
            &Split::Whole.screens(240, 160),
            a,
        );
        assert_near(p[0].dst, rect(0.0, 0.0, 1000.0, 500.0));
        let p = compose(
            &sixteen_nine(Scale::Fit),
            &Split::Whole.screens(240, 160),
            a,
        );
        // 16:9 at 500 tall: 888.9 x 500, centred.
        assert_near(p[0].dst, rect(55.0, 0.0, 500.0 * 16.0 / 9.0, 500.0));
    }

    #[test]
    fn a_phone_opens_a_two_screen_game_on_auto() {
        use crate::settings::schema::{entry, Device};
        let cycle = entry("screens.cycle").unwrap();
        let on = |device| cycle.default_for(device, Some(crate::settings::ConsoleId::Nds));
        let phone = settings(&[("screens.cycle", on(Device::Phone).unwrap())]);
        assert_eq!(
            phone.cycle,
            [LayoutId::Auto, LayoutId::BigSmall, LayoutId::TopOnly]
        );
        assert_eq!(phone.first(), LayoutId::Auto);
        let desk = settings(&[("screens.cycle", on(Device::Desktop).unwrap())]);
        assert_eq!(desk.first(), LayoutId::Vertical);
        assert_eq!(desk.cycle, ScreenSettings::default().cycle);
    }

    #[test]
    fn a_preset_on_the_phone_is_where_the_phone_draws_the_screens() {
        // «Parti da» upright on a phone: the screens fill the width as the game shows them.
        use crate::settings::schema::Device;
        let phone = device_settings(Device::Phone);
        let p = CustomLayout::preset(&phone, LayoutId::Auto, DS, 412.0, 892.0);
        assert!(p.is_valid());
        assert!((p.screens[0].w - 1.0).abs() < 1e-4);
        let desk = device_settings(Device::Desktop);
        let d = CustomLayout::preset(&desk, LayoutId::Auto, DS, 412.0, 892.0);
        assert!((d.screens[0].w - 256.0 / 412.0).abs() < 1e-4);
    }

    #[test]
    fn each_console_has_its_frame_at_1x() {
        use crate::settings::ConsoleId;
        assert_eq!(console_frame(ConsoleId::Gb), (160, 144));
        assert_eq!(console_frame(ConsoleId::Gbc), (160, 144));
        assert_eq!(console_frame(ConsoleId::Gba), (240, 160));
        assert_eq!(console_frame(ConsoleId::Nds), (256, 384));
        assert_eq!(console_frame(ConsoleId::N3ds), (400, 480));
    }

    #[test]
    fn a_preview_places_the_screens_as_the_game_would_open() {
        // Trello #111: C06 with no game draws where the game would be, the same compose.
        use crate::settings::ConsoleId;
        let gba = preview(
            &ScreenSettings::default(),
            ConsoleId::Gba,
            area(1280.0, 800.0),
        );
        assert_eq!(gba.len(), 1);
        assert_eq!(gba[0].id, ScreenId::Main);
        assert_eq!(
            gba,
            compose(
                &ScreenSettings::default().layout(LayoutId::Vertical, false),
                &Split::Whole.screens(240, 160),
                area(1280.0, 800.0),
            )
        );
        // A phone upright, DS: two screens apart, as many as its cycle's first layout shows.
        use crate::settings::schema::Device;
        let phone = device_settings(Device::Phone);
        let inset = Inset {
            bottom: 0.3,
            align: Align::Apart,
            between: 36.0 / 780.0,
            ..Inset::NONE
        };
        let a = Area {
            width: 360.0,
            height: 780.0,
            inset,
        };
        let ds = preview(&phone, ConsoleId::Nds, a);
        assert_eq!(
            ds.iter().map(|p| p.id).collect::<Vec<_>>(),
            [ScreenId::Top, ScreenId::Bottom]
        );
        assert!(ds.iter().all(|p| p.dst.y + p.dst.h <= 780.0 * 0.7 + 0.5));
        let three = preview(&phone, ConsoleId::N3ds, a);
        assert_eq!(three.len(), 2);
    }

    fn cs(id: ScreenId, x: f32, y: f32, w: f32, h: f32) -> CustomScreen {
        CustomScreen { id, x, y, w, h }
    }

    #[test]
    fn a_phone_placement_is_composed_over_the_whole_window() {
        let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
        assert_eq!(
            s.set_phone_placement(vec![cs(ScreenId::Main, 0.0, 0.1, 1.0, 0.4)]),
            Some(true)
        );
        let p = compose(
            &s.layout(),
            &Split::Whole.screens(240, 160),
            area(360.0, 780.0),
        );
        assert_eq!(p[0].dst, rect(0.0, 78.0, 360.0, 312.0));
        assert!(s.status().phone_placement);
        assert_eq!(s.set_phone_placement(vec![]), Some(true));
        assert!(!s.status().phone_placement);
    }

    #[test]
    fn a_phone_placement_must_be_the_games_screens() {
        let mut one = ScreenSession::new(ScreenSettings::default(), false, false);
        assert_eq!(
            one.set_phone_placement(vec![
                cs(ScreenId::Top, 0.0, 0.0, 1.0, 0.5),
                cs(ScreenId::Bottom, 0.0, 0.5, 1.0, 0.5)
            ]),
            None
        );
        let mut two = ScreenSession::new(ScreenSettings::default(), true, false);
        assert_eq!(
            two.set_phone_placement(vec![cs(ScreenId::Main, 0.0, 0.0, 1.0, 0.5)]),
            None
        );
        assert_eq!(
            two.set_phone_placement(vec![cs(ScreenId::Top, 0.0, 0.0, 1.0, 0.5)]),
            None
        );
        assert!(!two.status().phone_placement);
    }

    #[test]
    fn a_phone_placement_swaps_and_maps_the_touch_screen_from_where_it_is() {
        let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
        s.set_phone_placement(vec![
            cs(ScreenId::Top, 0.0, 0.5, 1.0, 0.25),
            cs(ScreenId::Bottom, 0.0, 0.0, 1.0, 0.25),
        ])
        .unwrap();
        let bottom = PxRect {
            x: 0,
            y: 192,
            w: 256,
            h: 192,
        };
        let want = Some(pointer_value(bottom, FRAME, 128, 96));
        assert_eq!(
            pointer_at(&s.layout(), DS, FRAME, area(360.0, 780.0), 180.0, 97.5),
            want
        );
        assert!(s.toggle_swap());
        assert_eq!(
            pointer_at(&s.layout(), DS, FRAME, area(360.0, 780.0), 180.0, 97.5),
            None
        );
        assert_eq!(
            pointer_at(&s.layout(), DS, FRAME, area(360.0, 780.0), 180.0, 487.5),
            want
        );
    }

    #[test]
    fn a_phone_placement_makes_room_for_the_quick_menu() {
        let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
        s.set_phone_placement(vec![cs(ScreenId::Main, 0.0, 0.0, 1.0, 0.5)])
            .unwrap();
        // 800 × 400 window, the menu on the right half: the placement (800 × 200 at full size)
        // is scaled by 0.5 into the left 400 × 400 and centred vertically, proportions kept.
        let menu = Area {
            width: 800.0,
            height: 400.0,
            inset: Inset {
                right: 0.5,
                ..Inset::NONE
            },
        };
        assert_eq!(
            compose(&s.layout(), &Split::Whole.screens(240, 160), menu)[0].dst,
            rect(0.0, 100.0, 400.0, 100.0)
        );
    }

    #[test]
    fn a_phone_placement_turns_and_swaps_as_a_custom_layout_does() {
        let placement = vec![
            cs(ScreenId::Top, 0.1, 0.0, 0.8, 0.4),
            cs(ScreenId::Bottom, 0.0, 0.5, 0.5, 0.25),
        ];
        let turned = settings(&[("screens.rot", Value::Text("left".into()))]);
        let mut s = ScreenSession::new(turned, true, false);
        s.set_phone_placement(placement.clone()).unwrap();
        assert!(s.toggle_swap());
        let custom = Layout {
            arrangement: Arrangement::Custom(
                placement
                    .iter()
                    .map(|c| (c.id, rect(c.x, c.y, c.w, c.h)))
                    .collect(),
            ),
            swap: true,
            rotation: Rotation::Left,
            ..Layout::default()
        };
        assert_eq!(
            ds(&s.layout(), area(360.0, 780.0)),
            ds(&custom, area(360.0, 780.0))
        );
    }

    #[test]
    fn the_layout_cycle_waits_while_a_phone_placement_is_in_use() {
        let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
        s.set_phone_placement(vec![
            cs(ScreenId::Top, 0.0, 0.0, 1.0, 0.4),
            cs(ScreenId::Bottom, 0.0, 0.5, 1.0, 0.4),
        ])
        .unwrap();
        let shown = s.status().layout;
        assert!(!s.next_layout());
        assert_eq!(s.set_layout(LayoutId::Side), None);
        assert_eq!(s.status().layout, shown);
        s.set_phone_placement(vec![]).unwrap();
        assert!(s.next_layout());
    }

    #[test]
    fn the_phone_start_has_every_screen_of_the_console() {
        use crate::settings::ConsoleId;
        let ids = |v: Vec<CustomScreen>| v.iter().map(|s| s.id).collect::<Vec<_>>();
        let start = phone_start(
            &ScreenSettings::default(),
            ConsoleId::Nds,
            area(360.0, 780.0),
        );
        assert_eq!(ids(start.clone()), [ScreenId::Top, ScreenId::Bottom]);
        assert!(start
            .iter()
            .all(|s| s.x >= 0.0 && s.x + s.w <= 1.0 && s.y + s.h <= 1.0));
        assert!(start[0].y < start[1].y);
        let top_only = settings(&[("screens.cycle", list(&["top_only"]))]);
        assert_eq!(
            ids(phone_start(&top_only, ConsoleId::N3ds, area(360.0, 780.0))),
            [ScreenId::Top, ScreenId::Bottom]
        );
        assert_eq!(
            ids(phone_start(
                &ScreenSettings::default(),
                ConsoleId::Gba,
                area(360.0, 780.0)
            )),
            [ScreenId::Main]
        );
    }

    /// A 3DS game upscaled 2× in a 1280 × 720 window with whole scales, the bottom fifth kept
    /// clear for the controls: the open game's sources and area for the live start.
    fn live_3ds() -> (Vec<SourceScreen>, Area) {
        let a = Area {
            width: 1280.0,
            height: 720.0,
            inset: Inset {
                bottom: 0.2,
                align: Align::Bottom,
                ..Inset::NONE
            },
        };
        (Split::N3dsStacked.screens(800, 960), a)
    }

    fn live_settings() -> ScreenSettings {
        settings(&[("video.scale", Value::Text("integer".into()))])
    }

    /// `placements` as fractions of `a`'s window, the frame ids mapped when `swapped`.
    fn as_start(placements: &[Placement], a: Area, swapped: bool) -> Vec<CustomScreen> {
        placements
            .iter()
            .map(|p| {
                cs(
                    if swapped { p.id.other() } else { p.id },
                    p.dst.x / a.width,
                    p.dst.y / a.height,
                    p.dst.w / a.width,
                    p.dst.h / a.height,
                )
            })
            .collect()
    }

    fn assert_same_screens(got: &[CustomScreen], want: &[CustomScreen]) {
        assert_eq!(got.len(), want.len(), "{got:?} vs {want:?}");
        for w in want {
            let g = got.iter().find(|g| g.id == w.id).unwrap();
            for (a, b) in [(g.x, w.x), (g.y, w.y), (g.w, w.w), (g.h, w.h)] {
                assert!((a - b).abs() < 1e-4, "{got:?} vs {want:?}");
            }
        }
    }

    #[test]
    fn the_live_phone_start_is_where_the_open_game_shows_its_screens() {
        let (sources, a) = live_3ds();
        let mut s = ScreenSession::new(live_settings(), true, false);
        assert!(s.next_layout());
        assert!(s.next_layout());
        assert_eq!(s.status().layout, LayoutId::BigSmall);
        let shown = compose(&s.layout(), &sources, a);
        assert_same_screens(&s.phone_start(&sources, a), &as_start(&shown, a, false));
    }

    #[test]
    fn the_live_phone_start_of_a_swapped_game_keeps_each_picture_where_it_is() {
        let (sources, a) = live_3ds();
        let mut s = ScreenSession::new(live_settings(), true, false);
        assert!(s.next_layout());
        assert!(s.next_layout());
        assert!(s.toggle_swap());
        let shown = compose(&s.layout(), &sources, a);
        let start = s.phone_start(&sources, a);
        assert_same_screens(&start, &as_start(&shown, a, true));
        // Placed as it starts (over the whole window, as the editor sends it), nothing moves.
        s.set_phone_placement(start).unwrap();
        let whole = Area {
            inset: Inset::NONE,
            ..a
        };
        let placed = compose(&s.layout(), &sources, whole);
        for p in &shown {
            let q = placed.iter().find(|q| q.id == p.id).unwrap();
            for (x, y) in [
                (p.dst.x, q.dst.x),
                (p.dst.y, q.dst.y),
                (p.dst.w, q.dst.w),
                (p.dst.h, q.dst.h),
            ] {
                assert!((x - y).abs() < 0.05, "{shown:?} vs {placed:?}");
            }
            assert_eq!(p.src, q.src);
        }
    }

    #[test]
    fn the_live_phone_start_of_one_screen_shown_holds_both() {
        let (sources, a) = live_3ds();
        let mut s = ScreenSession::new(live_settings(), true, false);
        assert_eq!(s.set_layout(LayoutId::TopOnly), Some(true));
        let start = s.phone_start(&sources, a);
        let ids: Vec<_> = start.iter().map(|c| c.id).collect();
        assert_eq!(ids, [ScreenId::Top, ScreenId::Bottom]);
        assert!(s.phone_start(&sources, area(0.0, 720.0)).is_empty());
    }

    #[test]
    fn the_phone_start_is_empty_for_an_empty_area() {
        use crate::settings::ConsoleId;
        for a in [area(0.0, 780.0), area(360.0, 0.0)] {
            assert!(phone_start(&ScreenSettings::default(), ConsoleId::Nds, a).is_empty());
        }
    }
}
