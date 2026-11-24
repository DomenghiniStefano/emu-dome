//! Touch controls (Trello #29): the user's own layout of the on-screen controls, one per console
//! and orientation, kept on this device only (a phone's layout fits that phone). The C06 editor
//! writes it into `touch.tedit` at the console's level (per device, never synced); `touch.tport`
//! and `touch.tland` choose, per orientation, between the default layout and this one.
//!
//! This module owns what the stored value looks like and what a valid layout is, and which
//! controls each console has, so the engine (`Kind::TouchLayout`) and the interface agree. The
//! default layouts, the geometry and the drawing are the interface's
//! (`app/src/components/touch/`).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_profile::CoreId;
use crate::screens::{console_frame, console_screens, placement_fits, CustomScreen, ScreenId};
use crate::ConsoleId;

/// One control of the on-screen pad. TypeScript: `TouchControlId` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "TouchControlId")]
pub enum ControlId {
    /// The cross: up, down, left, right and the diagonals.
    Dpad,
    /// The 3DS Circle Pad: the left analog stick.
    CirclePad,
    A,
    B,
    X,
    Y,
    L,
    R,
    Start,
    Select,
    /// Opens the quick menu (not a game button): every layout keeps it.
    Menu,
}

impl ControlId {
    pub const ALL: [ControlId; 11] = [
        ControlId::Dpad,
        ControlId::CirclePad,
        ControlId::A,
        ControlId::B,
        ControlId::X,
        ControlId::Y,
        ControlId::L,
        ControlId::R,
        ControlId::Start,
        ControlId::Select,
        ControlId::Menu,
    ];

    /// The stable id (the serde name): the stored value uses it.
    pub fn id(self) -> &'static str {
        match self {
            ControlId::Dpad => "dpad",
            ControlId::CirclePad => "circlePad",
            ControlId::A => "a",
            ControlId::B => "b",
            ControlId::X => "x",
            ControlId::Y => "y",
            ControlId::L => "l",
            ControlId::R => "r",
            ControlId::Start => "start",
            ControlId::Select => "select",
            ControlId::Menu => "menu",
        }
    }

    pub fn from_id(id: &str) -> Option<ControlId> {
        Self::ALL.into_iter().find(|c| c.id() == id)
    }

    /// Whether the control is one game button, so it can be held or repeated (tap / hold /
    /// turbo); the cross, the stick and the menu only work while touched.
    pub fn is_button(self) -> bool {
        match self {
            ControlId::A
            | ControlId::B
            | ControlId::X
            | ControlId::Y
            | ControlId::L
            | ControlId::R
            | ControlId::Start
            | ControlId::Select => true,
            ControlId::Dpad | ControlId::CirclePad | ControlId::Menu => false,
        }
    }

    /// The controls a console's games have, in drawing order: GB and GBC no shoulders, the GBA
    /// L / R, the DS X / Y too, the 3DS its Circle Pad as well.
    pub fn of(console: ConsoleId) -> &'static [ControlId] {
        use ControlId::{CirclePad, Dpad, Menu, Select, Start, A, B, L, R, X, Y};
        match console {
            ConsoleId::Gb | ConsoleId::Gbc => &[Dpad, B, A, Select, Start, Menu],
            ConsoleId::Gba => &[L, R, Dpad, B, A, Select, Start, Menu],
            ConsoleId::Nds => &[L, R, Dpad, Y, X, B, A, Select, Start, Menu],
            ConsoleId::N3ds => &[L, R, CirclePad, Dpad, Y, X, B, A, Select, Start, Menu],
        }
    }
}

/// Which way the phone is held. TypeScript: `TouchOrientation` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "TouchOrientation")]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl Orientation {
    pub fn id(self) -> &'static str {
        match self {
            Orientation::Portrait => "portrait",
            Orientation::Landscape => "landscape",
        }
    }

    pub fn from_id(id: &str) -> Option<Orientation> {
        [Orientation::Portrait, Orientation::Landscape]
            .into_iter()
            .find(|o| o.id() == id)
    }
}

/// What a button does under the finger. TypeScript: `TouchBehaviour` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "TouchBehaviour")]
pub enum Behaviour {
    /// «Tocco»: pressed while the finger is on it.
    Tap,
    /// «Tieni premuto»: a tap holds it down, the next tap lets it go.
    Hold,
    /// «Turbo»: pressed and released over and over while the finger is on it.
    Turbo,
}

impl Behaviour {
    pub fn id(self) -> &'static str {
        match self {
            Behaviour::Tap => "tap",
            Behaviour::Hold => "hold",
            Behaviour::Turbo => "turbo",
        }
    }

    pub fn from_id(id: &str) -> Option<Behaviour> {
        [Behaviour::Tap, Behaviour::Hold, Behaviour::Turbo]
            .into_iter()
            .find(|b| b.id() == id)
    }
}

/// A control's size, per cent of its standard size.
pub const SIZE_MIN: u16 = 50;
pub const SIZE_MAX: u16 = 200;
/// A control's own opacity, per cent (multiplied by `touch.topac`).
pub const OPACITY_MIN: u8 = 20;
pub const OPACITY_MAX: u8 = 100;

/// One control of a layout: its centre as fractions (0..1) of the game area, its size and
/// opacity in per cent, what it does under the finger. TypeScript: `TouchControl` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(rename = "TouchControl")]
pub struct Control {
    pub id: ControlId,
    pub x: f32,
    pub y: f32,
    pub size: u16,
    pub opacity: u8,
    pub behaviour: Behaviour,
}

impl Control {
    fn is_valid(&self) -> bool {
        let inside = |v: f32| v.is_finite() && (0.0..=1.0).contains(&v);
        inside(self.x)
            && inside(self.y)
            && (SIZE_MIN..=SIZE_MAX).contains(&self.size)
            && (OPACITY_MIN..=OPACITY_MAX).contains(&self.opacity)
            && (self.id.is_button() || self.behaviour == Behaviour::Tap)
    }
}

/// The controls of one orientation, in drawing order. Empty: none saved (the default layout).
/// TypeScript: `TouchLayout` (generated).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(rename = "TouchLayout")]
pub struct Layout {
    pub controls: Vec<Control>,
    /// Where the game's screens go, back to front (`touch.tscreens`). Empty: the game's own
    /// layout decides.
    pub screens: Vec<CustomScreen>,
}

impl Layout {
    /// Each control once, valid, and the menu kept (the only way to the quick menu on a phone
    /// without a back button); empty is valid (none saved).
    pub fn is_valid(&self) -> bool {
        self.controls_valid() && self.screens_valid()
    }

    /// The screens are none, the one of a one-screen console, or both of a two-screen one.
    fn screens_valid(&self) -> bool {
        [
            &[][..],
            &[ScreenId::Main],
            &[ScreenId::Top, ScreenId::Bottom],
        ]
        .iter()
        .any(|ids| placement_fits(&self.screens, ids))
    }

    fn controls_valid(&self) -> bool {
        self.controls.is_empty()
            || (self.controls.iter().any(|c| c.id == ControlId::Menu)
                && self
                    .controls
                    .iter()
                    .enumerate()
                    .all(|(i, c)| c.is_valid() && self.controls[..i].iter().all(|o| o.id != c.id)))
    }

    /// Valid, and only with controls `console`'s games have.
    pub fn fits(&self, console: ConsoleId) -> bool {
        self.is_valid()
            && self
                .controls
                .iter()
                .all(|c| ControlId::of(console).contains(&c.id))
            && placement_fits(&self.screens, console_screens(console))
    }
}

/// Both orientations of one console, as stored in `touch.tedit`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layouts {
    pub portrait: Layout,
    pub landscape: Layout,
}

impl Layouts {
    pub fn get(&self, orientation: Orientation) -> &Layout {
        match orientation {
            Orientation::Portrait => &self.portrait,
            Orientation::Landscape => &self.landscape,
        }
    }

    /// `self` with `layout` for `orientation` (empty: that orientation goes back to none).
    pub fn with(mut self, orientation: Orientation, layout: Layout) -> Layouts {
        match orientation {
            Orientation::Portrait => self.portrait = layout,
            Orientation::Landscape => self.landscape = layout,
        }
        self
    }

    /// The layouts the two stored lists hold (`touch.tedit`, `touch.tscreens`), each on its own:
    /// a list that does not read leaves its part empty and the other one alone.
    pub fn read(tedit: &[String], tscreens: &[String]) -> Layouts {
        let mut layouts = Layouts::from_items(tedit).unwrap_or_default();
        if let Some(screens) = Self::screens_from_items(tscreens) {
            layouts.portrait.screens = screens.portrait.screens;
            layouts.landscape.screens = screens.landscape.screens;
        }
        layouts
    }

    /// The screens of each orientation a `touch.tscreens` list holds, as a `Layouts` with no
    /// controls; `None` when an item is malformed or a placement invalid.
    ///
    /// One item per screen, back to front:
    /// `"<portrait|landscape>/<main|top|bottom>:<x>,<y>,<w>,<h>"`.
    fn screens_from_items(items: &[String]) -> Option<Layouts> {
        let mut layouts = Layouts::default();
        for item in items {
            let (place, rest) = item.split_once(':')?;
            let (orientation, id) = place.split_once('/')?;
            let mut v = rest.split(',').map(str::trim);
            let screen = CustomScreen {
                id: ScreenId::from_id(id.trim())?,
                x: v.next()?.parse().ok()?,
                y: v.next()?.parse().ok()?,
                w: v.next()?.parse().ok()?,
                h: v.next()?.parse().ok()?,
            };
            if v.next().is_some() {
                return None;
            }
            match Orientation::from_id(orientation.trim())? {
                Orientation::Portrait => layouts.portrait.screens.push(screen),
                Orientation::Landscape => layouts.landscape.screens.push(screen),
            }
        }
        (layouts.portrait.screens_valid() && layouts.landscape.screens_valid()).then_some(layouts)
    }

    /// The stored `touch.tscreens` list, portrait first, each orientation back to front,
    /// numbers rounded to 4 decimals.
    pub fn screen_items(&self) -> Vec<String> {
        let n = |v: f32| (v * 10_000.0).round() / 10_000.0;
        [Orientation::Portrait, Orientation::Landscape]
            .into_iter()
            .flat_map(|o| {
                self.get(o).screens.iter().map(move |s| {
                    format!(
                        "{}/{}:{},{},{},{}",
                        o.id(),
                        s.id.id(),
                        n(s.x),
                        n(s.y),
                        n(s.w),
                        n(s.h)
                    )
                })
            })
            .collect()
    }

    /// Whether a `touch.tscreens` value is valid (the schema's check, `Kind::TouchScreens`).
    pub fn valid_screen_items(items: &[String]) -> bool {
        Self::screens_from_items(items).is_some()
    }

    /// `self` with an orientation's screens dropped where they do not fit `console`'s.
    pub fn fitting(mut self, console: ConsoleId) -> Layouts {
        let ids = console_screens(console);
        for layout in [&mut self.portrait, &mut self.landscape] {
            if !placement_fits(&layout.screens, ids) {
                layout.screens.clear();
            }
        }
        self
    }

    /// The layouts a stored list holds; `None` when an item is malformed or a layout invalid.
    ///
    /// One item per control: `"<portrait|landscape>/<control>:<x>,<y>,<size>,<opacity>,<behaviour>"`,
    /// fractions with at most 4 decimals, e.g. `"landscape/a:0.93,0.6,100,100,tap"`. An empty
    /// list: no layout of the user's (both orientations take the default).
    pub fn from_items(items: &[String]) -> Option<Layouts> {
        let mut layouts = Layouts::default();
        for item in items {
            let (place, rest) = item.split_once(':')?;
            let (orientation, id) = place.split_once('/')?;
            let mut v = rest.split(',').map(str::trim);
            let control = Control {
                id: ControlId::from_id(id.trim())?,
                x: v.next()?.parse().ok()?,
                y: v.next()?.parse().ok()?,
                size: v.next()?.parse().ok()?,
                opacity: v.next()?.parse().ok()?,
                behaviour: Behaviour::from_id(v.next()?)?,
            };
            if v.next().is_some() {
                return None;
            }
            match Orientation::from_id(orientation.trim())? {
                Orientation::Portrait => layouts.portrait.controls.push(control),
                Orientation::Landscape => layouts.landscape.controls.push(control),
            }
        }
        (layouts.portrait.controls_valid() && layouts.landscape.controls_valid()).then_some(layouts)
    }

    /// The stored list (see [`Layouts::from_items`]), portrait first, fractions rounded to 4
    /// decimals.
    pub fn to_items(&self) -> Vec<String> {
        let n = |v: f32| (v * 10_000.0).round() / 10_000.0;
        [Orientation::Portrait, Orientation::Landscape]
            .into_iter()
            .flat_map(|o| {
                self.get(o).controls.iter().map(move |c| {
                    format!(
                        "{}/{}:{},{},{},{},{}",
                        o.id(),
                        c.id.id(),
                        n(c.x),
                        n(c.y),
                        c.size,
                        c.opacity,
                        c.behaviour.id()
                    )
                })
            })
            .collect()
    }

    /// Whether a stored value is valid (the schema's check, `Kind::TouchLayout`).
    pub fn valid_items(items: &[String]) -> bool {
        Self::from_items(items).is_some()
    }
}

/// One screen of a console and its size in the core's frame at 1×. TypeScript:
/// `TouchScreenShape` (generated).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(rename = "TouchScreenShape")]
pub struct ScreenShape {
    pub id: ScreenId,
    pub width: u32,
    pub height: u32,
}

/// What the interface needs to draw and edit one console's touch controls. TypeScript:
/// `TouchSetup` (generated).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "TouchSetup")]
pub struct Setup {
    /// The controls the console's games have, in drawing order (`ControlId::of`).
    pub available: Vec<ControlId>,
    /// Those that are one game button, so they can be held or repeated ([`ControlId::is_button`]).
    pub buttons: Vec<ControlId>,
    /// The console's screens with their size in the core's frame at 1×.
    pub screen_shapes: Vec<ScreenShape>,
    /// The user's layouts (empty: none saved for that orientation).
    pub portrait: Layout,
    pub landscape: Layout,
    /// What a control's size and opacity may be, per cent ([`SIZE_MIN`]…).
    pub size_min: u16,
    pub size_max: u16,
    pub opacity_min: u8,
    pub opacity_max: u8,
}

impl Setup {
    pub fn new(console: ConsoleId, layouts: Layouts) -> Setup {
        Setup {
            available: ControlId::of(console).to_vec(),
            buttons: ControlId::of(console)
                .iter()
                .copied()
                .filter(|c| c.is_button())
                .collect(),
            screen_shapes: screen_shapes(console),
            portrait: layouts.portrait,
            landscape: layouts.landscape,
            size_min: SIZE_MIN,
            size_max: SIZE_MAX,
            opacity_min: OPACITY_MIN,
            opacity_max: OPACITY_MAX,
        }
    }
}

fn screen_shapes(console: ConsoleId) -> Vec<ScreenShape> {
    let (width, height) = console_frame(console);
    CoreId::for_console(console)
        .profile()
        .split
        .screens(width, height)
        .into_iter()
        .map(|s| ScreenShape {
            id: s.id,
            width: s.src.w,
            height: s.src.h,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Behaviour, Control, ControlId, Layout, Layouts, Orientation, ScreenShape, Setup};
    use crate::screens::{CustomScreen, ScreenId};
    use crate::ConsoleId;

    fn control(id: ControlId, x: f32) -> Control {
        Control {
            id,
            x,
            y: 0.5,
            size: 100,
            opacity: 100,
            behaviour: Behaviour::Tap,
        }
    }

    fn items(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn saving_one_orientation_keeps_the_other() {
        let portrait = Layout {
            controls: vec![control(ControlId::Menu, 0.5)],
            ..Layout::default()
        };
        let landscape = Layout {
            controls: vec![control(ControlId::Dpad, 0.1), control(ControlId::Menu, 0.5)],
            ..Layout::default()
        };
        let saved = Layouts::default().with(Orientation::Portrait, portrait.clone());
        let both = saved.with(Orientation::Landscape, landscape.clone());
        assert_eq!(both.portrait, portrait);
        assert_eq!(both.landscape, landscape);
        let read = Layouts::from_items(&both.to_items()).unwrap();
        assert_eq!(read.get(Orientation::Portrait), &portrait);
        assert_eq!(read.get(Orientation::Landscape), &landscape);
    }

    #[test]
    fn a_stored_list_reads_back_as_it_was_written() {
        let layouts = Layouts::default()
            .with(
                Orientation::Landscape,
                Layout {
                    controls: vec![
                        Control {
                            behaviour: Behaviour::Turbo,
                            size: 120,
                            opacity: 60,
                            ..control(ControlId::A, 0.93)
                        },
                        control(ControlId::Menu, 0.5),
                    ],
                    ..Layout::default()
                },
            )
            .with(
                Orientation::Portrait,
                Layout {
                    controls: vec![control(ControlId::Dpad, 0.2), control(ControlId::Menu, 0.5)],
                    ..Layout::default()
                },
            );
        let stored = layouts.to_items();
        assert_eq!(
            stored,
            [
                "portrait/dpad:0.2,0.5,100,100,tap",
                "portrait/menu:0.5,0.5,100,100,tap",
                "landscape/a:0.93,0.5,120,60,turbo",
                "landscape/menu:0.5,0.5,100,100,tap",
            ]
        );
        assert_eq!(Layouts::from_items(&stored), Some(layouts));
        assert_eq!(Layouts::from_items(&[]), Some(Layouts::default()));
    }

    #[test]
    fn malformed_or_invalid_lists_are_refused() {
        for bad in [
            &["landscape/a:0.5,0.5,100,100,tap"][..], // no menu
            &[
                "landscape/menu:0.5,0.5,100,100,tap",
                "landscape/menu:0.4,0.5,100,100,tap",
            ],
            &["landscape/menu:1.5,0.5,100,100,tap"],
            &["landscape/menu:0.5,0.5,300,100,tap"],
            &["landscape/menu:0.5,0.5,100,5,tap"],
            &["landscape/menu:0.5,0.5,100,100,turbo"], // the menu is no game button
            &[
                "landscape/dpad:0.5,0.5,100,100,hold",
                "landscape/menu:0.5,0.5,100,100,tap",
            ],
            &["sideways/menu:0.5,0.5,100,100,tap"],
            &["landscape/zr:0.5,0.5,100,100,tap"],
            &["landscape/menu:0.5,0.5,100,100"],
            &["landscape/menu:0.5,0.5,100,100,tap,more"],
            &["landscape/menu:NaN,0.5,100,100,tap"],
        ] {
            assert!(!Layouts::valid_items(&items(bad)), "{bad:?}");
        }
    }

    #[test]
    fn a_layout_fits_only_the_consoles_whose_games_have_its_controls() {
        let with_l = Layout {
            controls: vec![control(ControlId::L, 0.1), control(ControlId::Menu, 0.5)],
            ..Layout::default()
        };
        assert!(!with_l.fits(ConsoleId::Gb));
        assert!(with_l.fits(ConsoleId::Gba));
        let stick = Layout {
            controls: vec![
                control(ControlId::CirclePad, 0.1),
                control(ControlId::Menu, 0.5),
            ],
            ..Layout::default()
        };
        assert!(!stick.fits(ConsoleId::Nds));
        assert!(stick.fits(ConsoleId::N3ds));
        for console in ConsoleId::ALL {
            let ids = ControlId::of(console);
            assert!(ids.contains(&ControlId::Menu), "{console:?}");
            assert!(ids.contains(&ControlId::Dpad), "{console:?}");
        }
    }

    #[test]
    fn every_id_reads_back() {
        for c in ControlId::ALL {
            assert_eq!(ControlId::from_id(c.id()), Some(c));
            let serde = serde_json::to_value(c).unwrap();
            assert_eq!(serde, c.id(), "serde and the stored id agree");
        }
        for b in [Behaviour::Tap, Behaviour::Hold, Behaviour::Turbo] {
            assert_eq!(Behaviour::from_id(b.id()), Some(b));
            assert_eq!(serde_json::to_value(b).unwrap(), b.id());
        }
        for o in [Orientation::Portrait, Orientation::Landscape] {
            assert_eq!(Orientation::from_id(o.id()), Some(o));
            assert_eq!(serde_json::to_value(o).unwrap(), o.id());
        }
    }

    fn scr(id: ScreenId, x: f32, y: f32, w: f32, h: f32) -> CustomScreen {
        CustomScreen { id, x, y, w, h }
    }

    #[test]
    fn screens_are_stored_beside_the_controls_and_read_back() {
        let layouts = Layouts::default().with(
            Orientation::Portrait,
            Layout {
                controls: vec![control(ControlId::Menu, 0.5)],
                screens: vec![
                    scr(ScreenId::Top, 0.0, 0.05, 1.0, 0.35),
                    scr(ScreenId::Bottom, 0.0, 0.45, 1.0, 0.35),
                ],
            },
        );
        assert_eq!(layouts.to_items(), ["portrait/menu:0.5,0.5,100,100,tap"]);
        assert_eq!(
            layouts.screen_items(),
            [
                "portrait/top:0,0.05,1,0.35",
                "portrait/bottom:0,0.45,1,0.35"
            ]
        );
        assert_eq!(
            Layouts::read(&layouts.to_items(), &layouts.screen_items()),
            layouts
        );
    }

    #[test]
    fn an_old_layout_with_no_screens_reads_as_before() {
        let read = Layouts::read(&items(&["landscape/menu:0.5,0.5,100,100,tap"]), &[]);
        assert_eq!(read.landscape.controls.len(), 1);
        assert!(read.landscape.screens.is_empty() && read.portrait.screens.is_empty());
    }

    #[test]
    fn a_bad_screen_list_drops_only_the_screens() {
        let read = Layouts::read(
            &items(&["portrait/menu:0.5,0.5,100,100,tap"]),
            &items(&["portrait/top:0,0,1,0.5"]),
        );
        assert_eq!(read.portrait.controls.len(), 1);
        assert!(read.portrait.screens.is_empty());
        let only_screens = Layouts::read(&[], &items(&["portrait/main:0,0,1,0.6"]));
        assert_eq!(
            only_screens.portrait.screens,
            [scr(ScreenId::Main, 0.0, 0.0, 1.0, 0.6)]
        );
    }

    #[test]
    fn bad_screen_lists_are_refused() {
        for good in [
            &[][..],
            &["landscape/main:0.25,0,0.5,1"],
            &["portrait/top:0,0,1,0.4", "portrait/bottom:0,0.5,1,0.4"],
        ] {
            assert!(Layouts::valid_screen_items(&items(good)), "{good:?}");
        }
        for bad in [
            &["portrait/main:0,0,1,0.5", "portrait/main:0,0.5,1,0.5"][..],
            &["portrait/main:0,0,1,0.5", "portrait/top:0,0.5,1,0.4"],
            &["portrait/top:0,0,1,0.5"],
            &["portrait/main:0.5,0,0.6,0.5"],
            &["portrait/main:0,0,0.05,0.5"],
            &["portrait/main:0,0,0.5,0.03"],
            &["sideways/main:0,0,1,1"],
            &["portrait/left:0,0,1,1"],
            &["portrait/main:0,0,1"],
            &["portrait/main:0,0,1,1,1"],
            &["portrait/main:NaN,0,1,1"],
        ] {
            assert!(!Layouts::valid_screen_items(&items(bad)), "{bad:?}");
        }
    }

    #[test]
    fn screens_fit_only_their_console() {
        let main = Layout {
            controls: vec![control(ControlId::Menu, 0.5)],
            screens: vec![scr(ScreenId::Main, 0.0, 0.0, 1.0, 0.5)],
        };
        assert!(main.fits(ConsoleId::Gba) && main.fits(ConsoleId::Gb));
        assert!(!main.fits(ConsoleId::Nds));
        let two = Layout {
            screens: vec![
                scr(ScreenId::Top, 0.0, 0.0, 1.0, 0.4),
                scr(ScreenId::Bottom, 0.0, 0.5, 1.0, 0.4),
            ],
            ..main.clone()
        };
        assert!(two.fits(ConsoleId::Nds) && two.fits(ConsoleId::N3ds) && !two.fits(ConsoleId::Gbc));
        let stored = Layouts::default()
            .with(Orientation::Portrait, two)
            .fitting(ConsoleId::Gba);
        assert!(stored.portrait.screens.is_empty());
        assert_eq!(stored.portrait.controls.len(), 1);
    }

    #[test]
    fn the_setup_names_each_screen_with_its_size() {
        let shape = |id, width, height| ScreenShape { id, width, height };
        assert_eq!(
            Setup::new(ConsoleId::Gba, Layouts::default()).screen_shapes,
            [shape(ScreenId::Main, 240, 160)]
        );
        assert_eq!(
            Setup::new(ConsoleId::Nds, Layouts::default()).screen_shapes,
            [
                shape(ScreenId::Top, 256, 192),
                shape(ScreenId::Bottom, 256, 192)
            ]
        );
        assert_eq!(
            Setup::new(ConsoleId::N3ds, Layouts::default()).screen_shapes,
            [
                shape(ScreenId::Top, 400, 240),
                shape(ScreenId::Bottom, 320, 240)
            ]
        );
    }
}
