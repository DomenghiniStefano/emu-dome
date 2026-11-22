//! Controller and keyboard bindings from the settings: which physical button or key gives
//! which game button (`pad.remap`), the shortcuts table (`pad.combos`, `pad.mod`), the A/B
//! layout (`pad.ab`), the menu confirm button and repeat (`pad.confirm`, `pad.repeat`), the
//! sticks and triggers (`pad.dead`, `pad.stickdpad`, `pad.trig`).
//!
//! Pure: [`PadConfig::from_settings`] reads the values through a lookup function, the shell
//! (`app/src-tauri/src/gamepad.rs`) applies them. Unit-tested without SDL.
//!
//! ## Value shapes (kept in the settings schema, `Kind::Bindings`)
//!
//! - `pad.remap`: a list of overrides `"<family>:<game button>=<source>"`, only for the buttons
//!   that differ from the defaults, e.g. `"xbox:a=south"`, `"keyboard:start=Space"`.
//!   `<family>` is a [`Family`] id (one table per controller type, plus the keyboard);
//!   `<game button>` one of [`REMAPPABLE`]; `<source>` a [`Pos`] id for controllers, a
//!   `KeyboardEvent.code` for the keyboard, or `none` (the game button gets nothing).
//! - `pad.combos`: a list of overrides `"<hotkey>=<pos>"` ([`Hotkey::id`], [`Pos`] id or
//!   `none`), e.g. `"save=north"`.
//!
//! Both are stored at Global, Console or Game level like any other setting: the strongest
//! level holding a list wins as a whole.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::hotkeys::{Bindings, Hotkey, DEFAULT_COMBOS, DEFAULT_MODIFIER};
use crate::input::Button;
use crate::session::Speed;
use crate::settings::keys::{self, Reader, TextKey};
use crate::settings::Value;

/// A place on the controller, by position (SDL's names): the same position is the same
/// [`Pos`] on every controller, whatever is printed on it. TypeScript: `Pos` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Pos {
    /// Bottom face button.
    South,
    /// Right face button.
    East,
    /// Left face button.
    West,
    /// Top face button.
    North,
    L1,
    R1,
    /// Left trigger (analogue past `pad.trig`, or digital).
    L2,
    R2,
    /// Left stick click.
    L3,
    R3,
    /// Select / View / Share / −.
    Back,
    /// Start / Menu / Options / +.
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Pos {
    pub const ALL: [Pos; 16] = [
        Pos::South,
        Pos::East,
        Pos::West,
        Pos::North,
        Pos::L1,
        Pos::R1,
        Pos::L2,
        Pos::R2,
        Pos::L3,
        Pos::R3,
        Pos::Back,
        Pos::Start,
        Pos::Up,
        Pos::Down,
        Pos::Left,
        Pos::Right,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Pos::South => "south",
            Pos::East => "east",
            Pos::West => "west",
            Pos::North => "north",
            Pos::L1 => "l1",
            Pos::R1 => "r1",
            Pos::L2 => "l2",
            Pos::R2 => "r2",
            Pos::L3 => "l3",
            Pos::R3 => "r3",
            Pos::Back => "back",
            Pos::Start => "start",
            Pos::Up => "up",
            Pos::Down => "down",
            Pos::Left => "left",
            Pos::Right => "right",
        }
    }

    pub fn from_id(id: &str) -> Option<Pos> {
        Pos::ALL.into_iter().find(|p| p.id() == id)
    }

    /// The RetroPad button at this place on a Nintendo pad: the bit the router uses for it
    /// (A = right, B = bottom, X = top, Y = left).
    pub fn button(self) -> Button {
        match self {
            Pos::South => Button::B,
            Pos::East => Button::A,
            Pos::West => Button::Y,
            Pos::North => Button::X,
            Pos::L1 => Button::L,
            Pos::R1 => Button::R,
            Pos::L2 => Button::L2,
            Pos::R2 => Button::R2,
            Pos::L3 => Button::L3,
            Pos::R3 => Button::R3,
            Pos::Back => Button::Select,
            Pos::Start => Button::Start,
            Pos::Up => Button::Up,
            Pos::Down => Button::Down,
            Pos::Left => Button::Left,
            Pos::Right => Button::Right,
        }
    }

    pub fn bit(self) -> u16 {
        self.button().bit()
    }

    /// The position whose router bit is `button`'s.
    pub fn of_button(button: Button) -> Pos {
        Pos::ALL
            .into_iter()
            .find(|p| p.button() == button)
            .unwrap_or(Pos::Start)
    }

    /// The positions held in a router mask, in [`Pos::ALL`] order.
    pub fn in_mask(mask: u16) -> Vec<Pos> {
        Pos::ALL
            .into_iter()
            .filter(|p| mask & p.bit() != 0)
            .collect()
    }
}

/// A controller type, for the remap tables: one table per family, plus the keyboard.
/// TypeScript: `PadFamily` (generated) (and the button names, `controls.pos.<family>.<pos>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "PadFamily")]
pub enum Family {
    Xbox,
    #[serde(rename = "playstation")]
    PlayStation,
    /// Nintendo layout (Switch Pro, Joy-Con pair, 8BitDo in Switch mode…).
    Switch,
    /// Anything else SDL knows: positions only.
    Generic,
    Keyboard,
}

impl Family {
    pub const ALL: [Family; 5] = [
        Family::Xbox,
        Family::PlayStation,
        Family::Switch,
        Family::Generic,
        Family::Keyboard,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Family::Xbox => "xbox",
            Family::PlayStation => "playstation",
            Family::Switch => "switch",
            Family::Generic => "generic",
            Family::Keyboard => "keyboard",
        }
    }

    pub fn from_id(id: &str) -> Option<Family> {
        Family::ALL.into_iter().find(|f| f.id() == id)
    }

    /// Whether the face buttons carry Nintendo's letters (A on the right): there "as printed"
    /// and "like on Nintendo" are the same layout.
    pub fn nintendo_letters(self) -> bool {
        match self {
            Family::Switch => true,
            Family::Xbox | Family::PlayStation | Family::Generic | Family::Keyboard => false,
        }
    }
}

/// The game buttons the remap table lists (C02, and the 14 buttons of the guided setup).
/// L3 is the shortcut button and R3 is not used by GBA, NDS or 3DS games.
pub const REMAPPABLE: [Button; 14] = [
    Button::A,
    Button::B,
    Button::X,
    Button::Y,
    Button::L,
    Button::R,
    Button::L2,
    Button::R2,
    Button::Start,
    Button::Select,
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
];

/// The keyboard's default keys (`KeyboardEvent.code`), from the design (C02 «Tastiera»),
/// except Select: the design gave it Backspace, which is also rewind's key (Trello #42,
/// docs/combo-tasti.md section 5), so Select moved to the right Shift, RetroArch's default.
pub const KEYBOARD_DEFAULTS: [(Button, &str); 14] = [
    (Button::A, "KeyX"),
    (Button::B, "KeyZ"),
    (Button::X, "KeyS"),
    (Button::Y, "KeyA"),
    (Button::L, "KeyQ"),
    (Button::R, "KeyW"),
    (Button::L2, "Digit1"),
    (Button::R2, "Digit2"),
    (Button::Start, "Enter"),
    (Button::Select, "ShiftRight"),
    (Button::Up, "ArrowUp"),
    (Button::Down, "ArrowDown"),
    (Button::Left, "ArrowLeft"),
    (Button::Right, "ArrowRight"),
];

/// `pad.ab`: which face button is the game's A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbLayout {
    /// Positional (default, docs/decisions.md): A is the right button, B the bottom one.
    Nintendo,
    /// A is the button printed A (on Xbox-style pads the bottom one).
    Printed,
}

/// Where a game button comes from on a controller (`None` = nothing).
pub type PadSource = Option<Pos>;

/// The default source of a game button on a controller of `family` with `layout`.
pub fn default_pad_source(button: Button, family: Family, layout: AbLayout) -> PadSource {
    let printed = layout == AbLayout::Printed && !family.nintendo_letters();
    let pos = match (button, printed) {
        (Button::A, true) => Pos::South,
        (Button::B, true) => Pos::East,
        (Button::X, true) => Pos::West,
        (Button::Y, true) => Pos::North,
        (b, true | false) => Pos::of_button(b),
    };
    Some(pos)
}

/// The default key of a game button (`None`: no key, e.g. L3).
pub fn default_key(button: Button) -> Option<&'static str> {
    KEYBOARD_DEFAULTS
        .iter()
        .find(|(b, _)| *b == button)
        .map(|(_, k)| *k)
}

/// One parsed `pad.remap` item.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RemapItem<'a> {
    family: Family,
    button: Button,
    /// A [`Pos`] id or a key code; `none` for nothing.
    source: &'a str,
}

const NONE: &str = "none";

fn parse_remap(item: &str) -> Option<RemapItem<'_>> {
    let (family, rest) = item.split_once(':')?;
    let (button, source) = rest.split_once('=')?;
    let family = Family::from_id(family)?;
    let button = Button::from_id(button).filter(|b| REMAPPABLE.contains(b))?;
    let valid = source == NONE
        || match family {
            Family::Keyboard => is_key_code(source),
            Family::Xbox | Family::PlayStation | Family::Switch | Family::Generic => {
                Pos::from_id(source).is_some()
            }
        };
    valid.then_some(RemapItem {
        family,
        button,
        source,
    })
}

/// A `KeyboardEvent.code`: letters and digits only (`KeyX`, `Digit1`, `ArrowUp`, `F5`…).
fn is_key_code(code: &str) -> bool {
    !code.is_empty() && code.len() <= 24 && code.chars().all(|c| c.is_ascii_alphanumeric())
}

fn parse_combo(item: &str) -> Option<(Hotkey, Option<Pos>)> {
    let (hotkey, source) = item.split_once('=')?;
    let hotkey = Hotkey::from_id(hotkey)?;
    if source == NONE {
        return Some((hotkey, None));
    }
    Some((hotkey, Some(Pos::from_id(source)?)))
}

/// Which binding list an entry holds. TypeScript: `BindingTable` (generated).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "BindingTable")]
pub enum Table {
    /// `pad.remap`.
    Remap,
    /// `pad.combos`.
    Combos,
}

/// Whether `items` is a valid list for `table`: every item parses, and no target (family +
/// button, or hotkey) appears twice.
pub fn valid_list(table: Table, items: &[String]) -> bool {
    let target = |item: &str| item.split_once('=').map(|(t, _)| t.to_owned());
    let unique = items
        .iter()
        .enumerate()
        .all(|(i, item)| !items[..i].iter().any(|o| target(o) == target(item)));
    unique
        && items.iter().all(|item| match table {
            Table::Remap => parse_remap(item).is_some(),
            Table::Combos => parse_combo(item).is_some(),
        })
}

/// How the game buttons come from one controller's positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PadMap {
    /// `(game button, position)` for every game button that has one.
    pairs: Vec<(Button, Pos)>,
}

impl PadMap {
    /// The game buttons held, from the positions held (a router mask).
    pub fn apply(&self, positions: u16) -> u16 {
        self.pairs
            .iter()
            .filter(|(_, pos)| positions & pos.bit() != 0)
            .fold(0, |mask, (button, _)| mask | button.bit())
    }

    /// The position that gives `button`, if any.
    pub fn source(&self, button: Button) -> Option<Pos> {
        self.pairs
            .iter()
            .find(|(b, _)| *b == button)
            .map(|(_, p)| *p)
    }
}

/// Menu repeat speed (`pad.repeat`).
fn repeat_times(id: &str) -> (Duration, Duration) {
    match id {
        "slow" => (Duration::from_millis(600), Duration::from_millis(180)),
        "fast" => (Duration::from_millis(250), Duration::from_millis(70)),
        _ => (
            crate::hotkeys::REPEAT_DELAY,
            crate::hotkeys::REPEAT_INTERVAL,
        ),
    }
}

/// Everything the controller thread reads from the settings, already validated.
#[derive(Debug, Clone, PartialEq)]
pub struct PadConfig {
    /// `pad.dead`, percent of the stick's travel ignored around the centre.
    pub dead_zone: u8,
    /// `pad.stickdpad`, percent of travel past which the left stick is the d-pad.
    pub stick_dpad: u8,
    /// `pad.trig`, percent of an analogue trigger's travel past which it is pressed.
    pub trigger: u8,
    pub layout: AbLayout,
    /// `pad.confirm` is `bottom`.
    pub confirm_bottom: bool,
    /// `pad.repeat` id.
    pub repeat: String,
    /// `pad.mod`.
    pub modifier: Pos,
    /// `pad.combos` overrides, in order.
    pub combos: Vec<(Hotkey, Option<Pos>)>,
    /// `pad.remap` items (valid ones only).
    pub remap: Vec<String>,
    /// `speed.ffval`: where fast forward goes (L3 + R2 held, L3 + up, Tab).
    pub fast_forward: Speed,
    /// `speed.ffmode` is `toggle`: the keyboard's fast forward key (Tab) turns it on at one press
    /// and off at the next instead of lasting while held (`keyboard::KeyRouter`). The
    /// controller keeps its two combos, one held and one toggled.
    pub fast_forward_toggle: bool,
    /// `speed.slow`: where slow motion goes (L3 + down).
    pub slow_motion: Speed,
    /// `a11y.toggle`: rewind (L3 + L2) goes on at one press and off at the next instead of
    /// lasting while held.
    pub hold_toggle: bool,
    /// `pad.lid` is `l3_select` (the default): the lid has its combo. `menu_only`: no combo,
    /// the quick menu only.
    pub lid_combo: bool,
}

impl Default for PadConfig {
    fn default() -> Self {
        Self::from_settings(|_| None)
    }
}

impl PadConfig {
    /// Builds the config from the resolved settings (`get(key)` → value, `None` for the
    /// schema default). A missing value or one of the wrong shape is the catalogue's default.
    pub fn from_settings(get: impl Fn(&str) -> Option<Value>) -> Self {
        let settings = Reader::new(get);
        // Sliders are percentages (0..=100 in the catalogue).
        let percent = |key| u8::try_from(settings.int(key).clamp(0, 100)).unwrap_or(u8::MAX);
        // Every option of the speed settings is a speed (`every_speed_option_is_a_speed`); the
        // last fallback is never reached.
        let speed = |key: TextKey| {
            let default = || Reader::new(|_| None).text(key);
            Speed::from_option(&settings.text(key))
                .or_else(|| Speed::from_option(&default()))
                .unwrap_or(Speed::Factor(1.0))
        };
        let modifier = match settings.text(keys::PAD_MODIFIER).as_str() {
            "r3" => Pos::R3,
            "select" => Pos::Back,
            // `l3`, the default (`hotkeys::DEFAULT_MODIFIER`).
            _ => Pos::of_button(DEFAULT_MODIFIER),
        };
        Self {
            dead_zone: percent(keys::PAD_DEAD_ZONE),
            stick_dpad: percent(keys::PAD_STICK_DPAD),
            trigger: percent(keys::PAD_TRIGGER),
            layout: if settings.text(keys::PAD_LAYOUT) == "printed" {
                AbLayout::Printed
            } else {
                AbLayout::Nintendo
            },
            confirm_bottom: settings.text(keys::PAD_CONFIRM) == "bottom",
            repeat: settings.text(keys::PAD_REPEAT),
            modifier,
            combos: settings
                .list(keys::PAD_COMBOS)
                .iter()
                .filter_map(|i| parse_combo(i))
                .collect(),
            remap: settings
                .list(keys::PAD_REMAP)
                .into_iter()
                .filter(|i| parse_remap(i).is_some())
                .collect(),
            fast_forward: speed(keys::SPEED_FF),
            fast_forward_toggle: settings.text(keys::SPEED_FF_MODE) == "toggle",
            slow_motion: speed(keys::SPEED_SLOW),
            hold_toggle: settings.bool(keys::A11Y_HOLD_TOGGLE),
            lid_combo: settings.text(keys::PAD_LID) != "menu_only",
        }
    }

    /// The router's bindings: modifier, combos (defaults with the overrides applied; a combo
    /// on the modifier itself is dropped), menu confirm and repeat.
    pub fn bindings(&self) -> Bindings {
        let combos = self
            .combo_table()
            .into_iter()
            .filter_map(|(hotkey, pos)| Some((pos?.button(), hotkey)))
            .collect();
        let (repeat_delay, repeat_interval) = repeat_times(&self.repeat);
        Bindings {
            modifier: self.modifier.button(),
            combos,
            confirm: if self.confirm_bottom {
                Button::B
            } else {
                Button::A
            },
            repeat_delay,
            repeat_interval,
        }
    }

    /// Every hotkey with its position (`None`: no combo), in [`Hotkey::ALL`] order. The lid has
    /// none with `pad.lid` on «quick menu only».
    pub fn combo_table(&self) -> Vec<(Hotkey, Option<Pos>)> {
        Hotkey::ALL
            .into_iter()
            .map(|hotkey| {
                if hotkey == Hotkey::Lid && !self.lid_combo {
                    return (hotkey, None);
                }
                let pos = self
                    .combos
                    .iter()
                    .find(|(h, _)| *h == hotkey)
                    .map_or_else(|| default_combo(hotkey), |(_, p)| *p);
                (hotkey, pos.filter(|p| *p != self.modifier))
            })
            .collect()
    }

    /// The source of each remappable game button for a controller `family` (a [`Pos`] id or
    /// key code, `None` for nothing), and whether it differs from the default.
    pub fn remap_table(&self, family: Family) -> Vec<(Button, Option<String>, bool)> {
        REMAPPABLE
            .into_iter()
            .map(|button| {
                let default = match family {
                    Family::Keyboard => default_key(button).map(str::to_owned),
                    Family::Xbox | Family::PlayStation | Family::Switch | Family::Generic => {
                        default_pad_source(button, family, self.layout).map(|p| p.id().to_owned())
                    }
                };
                let custom = self
                    .remap
                    .iter()
                    .filter_map(|i| parse_remap(i))
                    .find(|i| i.family == family && i.button == button)
                    .map(|i| (i.source != NONE).then(|| i.source.to_owned()));
                match custom {
                    Some(source) => {
                        let changed = source != default;
                        (button, source, changed)
                    }
                    None => (button, default, false),
                }
            })
            .collect()
    }

    /// How a controller of `family` gives the game its buttons. R3 keeps its place; L3 and
    /// the modifier never reach the game (the router keeps them away).
    pub fn pad_map(&self, family: Family) -> PadMap {
        let mut pairs: Vec<(Button, Pos)> = self
            .remap_table(family)
            .into_iter()
            .filter_map(|(button, source, _)| Some((button, Pos::from_id(&source?)?)))
            .collect();
        pairs.push((Button::R3, Pos::R3));
        PadMap { pairs }
    }

    /// The axis value (0..=32767) past which a trigger counts as pressed.
    pub fn trigger_threshold(&self) -> i16 {
        percent_of_axis(self.trigger.max(1))
    }

    /// The d-pad directions given by the left stick at (`x`, `y`) (SDL axes, y positive
    /// downwards): nothing inside the dead zone (round), then each axis past `pad.stickdpad`.
    pub fn stick_dpad_mask(&self, x: i16, y: i16) -> u16 {
        let (fx, fy) = (f32::from(x) / 32767.0, f32::from(y) / 32767.0);
        let dead = f32::from(self.dead_zone) / 100.0;
        if fx.hypot(fy) <= dead {
            return 0;
        }
        let t = f32::from(self.stick_dpad.max(1)) / 100.0;
        let mut mask = 0;
        if fx > t {
            mask |= Button::Right.bit();
        }
        if fx < -t {
            mask |= Button::Left.bit();
        }
        if fy > t {
            mask |= Button::Down.bit();
        }
        if fy < -t {
            mask |= Button::Up.bit();
        }
        mask
    }
}

/// The default position of a hotkey (with the modifier held).
pub fn default_combo(hotkey: Hotkey) -> Option<Pos> {
    DEFAULT_COMBOS
        .iter()
        .find(|(_, h)| *h == hotkey)
        .map(|(b, _)| Pos::of_button(*b))
}

fn percent_of_axis(pct: u8) -> i16 {
    i16::try_from(i32::from(pct.min(100)) * 32767 / 100).unwrap_or(i16::MAX)
}

#[cfg(test)]
mod tests;
