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
}

impl Layout {
    /// Each control once, valid, and the menu kept (the only way to the quick menu on a phone
    /// without a back button); empty is valid (none saved).
    pub fn is_valid(&self) -> bool {
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
        (layouts.portrait.is_valid() && layouts.landscape.is_valid()).then_some(layouts)
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
            portrait: layouts.portrait,
            landscape: layouts.landscape,
            size_min: SIZE_MIN,
            size_max: SIZE_MAX,
            opacity_min: OPACITY_MIN,
            opacity_max: OPACITY_MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Behaviour, Control, ControlId, Layout, Layouts, Orientation};
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
        };
        let landscape = Layout {
            controls: vec![control(ControlId::Dpad, 0.1), control(ControlId::Menu, 0.5)],
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
                },
            )
            .with(
                Orientation::Portrait,
                Layout {
                    controls: vec![control(ControlId::Dpad, 0.2), control(ControlId::Menu, 0.5)],
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
        };
        assert!(!with_l.fits(ConsoleId::Gb));
        assert!(with_l.fits(ConsoleId::Gba));
        let stick = Layout {
            controls: vec![
                control(ControlId::CirclePad, 0.1),
                control(ControlId::Menu, 0.5),
            ],
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
}
