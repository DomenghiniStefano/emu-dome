//! The settings schema: the one description of every setting, from which the interface builds
//! its pages. The entries themselves are in `catalog.rs`.
//!
//! Every entry has a stable key `<page>.<key>` (e.g. `video.scale`), and its texts live in the
//! interface under `settings.<page>.<key>.name|desc` and `settings.<page>.<key>.option.<id>`.

use serde::{Deserialize, Serialize};

/// Where a value comes from, from the weakest to the strongest.
/// Mirror: `SettingLevel` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// The schema's default.
    Default,
    /// «Tutti i giochi».
    Global,
    /// «Giochi <console>».
    Console,
    /// «Solo questo gioco».
    Game,
    /// Changes «in prova» from the quick menu: live, never written to disk until committed.
    Session,
}

/// The levels an entry accepts (Default is always there; Session follows Console / Game).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Levels {
    pub global: bool,
    pub console: bool,
    pub game: bool,
}

impl Levels {
    pub const GLOBAL: Self = Self::new(true, false, false);
    pub const CONSOLE: Self = Self::new(false, true, false);
    pub const GAME: Self = Self::new(false, false, true);
    pub const GLOBAL_CONSOLE: Self = Self::new(true, true, false);
    pub const GLOBAL_GAME: Self = Self::new(true, false, true);
    pub const CONSOLE_GAME: Self = Self::new(false, true, true);
    pub const ALL: Self = Self::new(true, true, true);

    const fn new(global: bool, console: bool, game: bool) -> Self {
        Self {
            global,
            console,
            game,
        }
    }

    /// Whether a value may be stored at `level`. The quick menu's session is open to the
    /// entries that can be saved for a game or a console.
    pub fn accepts(self, level: Level) -> bool {
        match level {
            Level::Default => false,
            Level::Global => self.global,
            Level::Console => self.console,
            Level::Game => self.game,
            Level::Session => self.console || self.game,
        }
    }

    /// The accepted levels, weakest first (Session included when it applies).
    pub fn list(self) -> Vec<Level> {
        [Level::Global, Level::Console, Level::Game, Level::Session]
            .into_iter()
            .filter(|l| self.accepts(*l))
            .collect()
    }
}

/// Which devices an entry exists on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    All,
    /// «solo PC».
    Desktop,
    /// «solo telefono».
    Phone,
}

/// What this device is, from `emudome_platform`. Mirror: `SettingDevice` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Desktop,
    Phone,
}

impl Platform {
    pub fn includes(self, device: Device) -> bool {
        match (self, device) {
            (Platform::All, Device::Desktop | Device::Phone)
            | (Platform::Desktop, Device::Desktop)
            | (Platform::Phone, Device::Phone) => true,
            (Platform::Desktop, Device::Phone) | (Platform::Phone, Device::Desktop) => false,
        }
    }
}

/// How a single choice is shown (design kinds `seg`, `sel`, `cards`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChoiceStyle {
    /// Segments, few short options.
    Segments,
    /// A list.
    List,
    /// Cards with previews.
    Cards,
}

/// How an entry without a value is shown (design kinds `act`, `link`, `info`, `storage`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStyle {
    /// A button that does something.
    Button,
    /// Opens another screen.
    Link,
    /// Shows information.
    Info,
    /// Shows space used, with its own actions.
    Storage,
}

/// Unit of a slider value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Percent,
    Px,
    Seconds,
    Copies,
}

/// The kind of an entry and its constraints. Mirror: `SettingKind` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Kind {
    /// On / off (`sw`). Value: bool.
    Toggle,
    /// One of `options` (stable ids, snake_case English). Value: the option id.
    /// `more_at_runtime`: the device adds options after these (audio outputs, saved layouts).
    Choice {
        style: ChoiceStyle,
        options: &'static [&'static str],
        more_at_runtime: bool,
    },
    /// Any subset of `options` (`chips`). Value: list of option ids.
    Multi { options: &'static [&'static str] },
    /// Whole number in `min..=max`, on the `step` grid from `min`. Value: integer.
    Slider {
        min: i64,
        max: i64,
        step: i64,
        unit: Unit,
    },
    /// A file path, empty for none. Value: text.
    File,
    /// A list of folder paths. Value: list.
    Folders,
    /// A colour `#rrggbb`, usually one of `swatches`. Value: text.
    Colour { swatches: &'static [&'static str] },
    /// Free text. Value: text.
    Text,
    /// No value: a button, a link to another screen, information.
    Action { style: ActionStyle },
}

/// A stored or resolved value. Mirror: `SettingValue` in app/src/emu.ts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Text(String),
    List(Vec<String>),
}

/// A default written in the static catalogue (converted to a [`Value`] when read).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefaultValue {
    /// Actions only.
    None,
    Bool(bool),
    Int(i64),
    Text(&'static str),
    List(&'static [&'static str]),
}

impl DefaultValue {
    pub fn to_value(self) -> Option<Value> {
        match self {
            DefaultValue::None => None,
            DefaultValue::Bool(b) => Some(Value::Bool(b)),
            DefaultValue::Int(i) => Some(Value::Int(i)),
            DefaultValue::Text(t) => Some(Value::Text(t.to_owned())),
            DefaultValue::List(l) => Some(Value::List(l.iter().map(|s| (*s).to_owned()).collect())),
        }
    }
}

/// One setting.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    /// `<page>.<key>`, stable: it is the key in the files and in i18n.
    pub key: &'static str,
    pub kind: Kind,
    pub default: DefaultValue,
    pub levels: Levels,
    /// The value belongs to this device and never goes into the synced files.
    pub per_device: bool,
    pub platform: Platform,
    /// The roadmap phase the entry ships in (1 = now); hidden by the interface until then.
    pub phase: u8,
}

impl Entry {
    /// The page, the part of the key before the dot.
    pub fn page(&self) -> &'static str {
        self.key.split_once('.').map_or(self.key, |(page, _)| page)
    }

    /// Whether the entry holds a value (actions do not).
    pub fn has_value(&self) -> bool {
        !matches!(self.kind, Kind::Action { .. })
    }

    pub fn default_value(&self) -> Option<Value> {
        self.default.to_value()
    }

    /// Whether `value` is valid for this entry's kind and constraints.
    pub fn accepts_value(&self, value: &Value) -> bool {
        match (self.kind, value) {
            (Kind::Toggle, Value::Bool(_)) => true,
            (Kind::Choice { options, .. }, Value::Text(id)) => options.contains(&id.as_str()),
            (Kind::Multi { options }, Value::List(ids)) => {
                ids.iter().all(|id| options.contains(&id.as_str()))
                    && ids.iter().enumerate().all(|(i, id)| !ids[..i].contains(id))
            }
            (Kind::Slider { min, max, step, .. }, Value::Int(v)) => {
                (min..=max).contains(v) && step > 0 && (v - min) % step == 0
            }
            (Kind::File | Kind::Text, Value::Text(_)) | (Kind::Folders, Value::List(_)) => true,
            (Kind::Colour { .. }, Value::Text(c)) => is_colour(c),
            (
                Kind::Toggle
                | Kind::Choice { .. }
                | Kind::Multi { .. }
                | Kind::Slider { .. }
                | Kind::File
                | Kind::Folders
                | Kind::Colour { .. }
                | Kind::Text
                | Kind::Action { .. },
                Value::Bool(_) | Value::Int(_) | Value::Text(_) | Value::List(_),
            ) => false,
        }
    }
}

/// `#rrggbb`, hex digits in any case.
fn is_colour(text: &str) -> bool {
    text.strip_prefix('#')
        .is_some_and(|hex| hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The entry with this key.
pub fn entry(key: &str) -> Option<&'static Entry> {
    super::catalog::ENTRIES.iter().find(|e| e.key == key)
}

/// The entries of a page, in catalogue order.
pub fn page_entries(page: &str) -> impl Iterator<Item = &'static Entry> + '_ {
    super::catalog::ENTRIES
        .iter()
        .filter(move |e| e.page() == page)
}

/// A heading of a settings page and the entries under it, in order.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Section {
    pub page: &'static str,
    /// Stable id; text `settings.sections.<page>.<id>`.
    pub id: &'static str,
    pub keys: &'static [&'static str],
}

/// The schema as the interface reads it. Mirror: `SettingsSchema` in app/src/emu.ts.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDto {
    /// This device; entries whose `platform` does not include it are not shown.
    pub device: Device,
    /// Phase the app is in: entries with a later `phase` are not shown yet.
    pub current_phase: u8,
    /// Page ids in display order.
    pub pages: &'static [&'static str],
    /// The headings of every page, in order, with their entries.
    pub sections: &'static [Section],
    pub entries: Vec<EntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryDto {
    pub key: &'static str,
    pub page: &'static str,
    pub kind: Kind,
    /// `None` for actions.
    pub default: Option<Value>,
    /// Accepted levels, weakest first.
    pub levels: Vec<Level>,
    pub per_device: bool,
    pub platform: Platform,
    pub phase: u8,
}

/// The whole schema for this device.
pub fn schema(device: Device) -> SchemaDto {
    SchemaDto {
        device,
        current_phase: super::catalog::CURRENT_PHASE,
        pages: super::catalog::PAGES,
        sections: super::catalog::SECTIONS,
        entries: super::catalog::ENTRIES
            .iter()
            .map(|e| EntryDto {
                key: e.key,
                page: e.page(),
                kind: e.kind,
                default: e.default_value(),
                levels: e.levels.list(),
                per_device: e.per_device,
                platform: e.platform,
                phase: e.phase,
            })
            .collect(),
    }
}
