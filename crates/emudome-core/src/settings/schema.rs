//! The settings schema: the one description of every setting, from which the interface builds
//! its pages. The entries themselves are in `catalog.rs`.
//!
//! Every entry has a stable key `<prefix>.<key>` (e.g. `video.scale`), and its texts live in the
//! interface under `settings.<key>.name|desc` and `settings.<key>.option.<id>`. The page it is
//! shown on is its own field: usually the key's prefix, but not always (the «Avanzate» page holds
//! the `perf.*`, `cores.*` and `adv.*` keys, which never change).

use serde::{Deserialize, Serialize};

pub use crate::bindings::Table as BindingTable;

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
    /// Only where there is a touch screen. For now that means the phone.
    Touch,
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
            | (Platform::Phone | Platform::Touch, Device::Phone) => true,
            (Platform::Desktop, Device::Phone)
            | (Platform::Phone | Platform::Touch, Device::Desktop) => false,
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
    /// `option_platforms`: options that exist only on some devices (the others are everywhere);
    /// not serialized: the interface gets the result for its device as `hiddenOptions`.
    Choice {
        style: ChoiceStyle,
        options: &'static [&'static str],
        more_at_runtime: bool,
        #[serde(skip)]
        option_platforms: &'static [(&'static str, Platform)],
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
    /// No value: cards that each apply a bundle of other settings, after a confirmation that
    /// lists the changes (`presets.preset`). Nothing is ever «selected».
    Apply { options: &'static [&'static str] },
    /// A binding list edited in the Controls section, shown as a link in the settings pages
    /// (`pad.remap`, `pad.combos`). Value: list of overrides, shapes in `crate::bindings`.
    Bindings { table: BindingTable },
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

/// When a row is shown: the value of another entry of the same page, resolved at the level being
/// edited, is (or with `not`, is not) one of `values`. Mirror: `SettingShowIf` in app/src/emu.ts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ShowIf {
    pub key: &'static str,
    pub values: &'static [&'static str],
    pub not: bool,
}

impl ShowIf {
    /// Whether the row is shown when the other entry's value is `value` (`None`: no value, which
    /// is one of no `values`).
    pub fn holds(&self, value: Option<&Value>) -> bool {
        let listed = match value {
            Some(Value::Text(id)) => self.values.contains(&id.as_str()),
            Some(Value::Bool(_) | Value::Int(_) | Value::List(_)) | None => false,
        };
        listed != self.not
    }
}

/// One setting.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    /// `<prefix>.<key>`, stable: it is the key in the files and in i18n.
    pub key: &'static str,
    /// The page it is shown on (one of `catalog::PAGES`), set in the catalogue.
    pub page: &'static str,
    pub kind: Kind,
    pub default: DefaultValue,
    pub levels: Levels,
    /// The value belongs to this device and never goes into the synced files.
    pub per_device: bool,
    pub platform: Platform,
    /// The roadmap phase the entry ships in (1 = now); hidden by the interface until then.
    pub phase: u8,
    /// One of the options people change most often: listed in «Usate più spesso» (T01) at the
    /// Global level. Chosen in the catalogue, never in the interface.
    pub most_used: bool,
    /// Shown only when another entry has (or has not) some values; `None`: always shown.
    pub show_if: Option<ShowIf>,
    /// Works only on the native (Accelerated) video path: on Compatible the interface shows the
    /// row disabled with the reason (X05). The screen effects (`fx.*`).
    pub needs_native_video: bool,
}

impl Entry {
    /// The page the entry is shown on.
    pub fn page(&self) -> &'static str {
        self.page
    }

    /// Whether the entry holds a value (actions and presets do not).
    pub fn has_value(&self) -> bool {
        !matches!(self.kind, Kind::Action { .. } | Kind::Apply { .. })
    }

    /// Whether option `id` of a choice is offered on `device` (options without a platform of
    /// their own are offered wherever the entry is).
    pub fn offers(&self, id: &str, device: Device) -> bool {
        match self.kind {
            Kind::Choice {
                option_platforms, ..
            } => option_platforms
                .iter()
                .find(|(option, _)| *option == id)
                .is_none_or(|(_, platform)| platform.includes(device)),
            Kind::Toggle
            | Kind::Multi { .. }
            | Kind::Slider { .. }
            | Kind::File
            | Kind::Folders
            | Kind::Colour { .. }
            | Kind::Text
            | Kind::Action { .. }
            | Kind::Apply { .. }
            | Kind::Bindings { .. } => true,
        }
    }

    /// The options of a choice that `device` does not offer, in catalogue order.
    pub fn hidden_options(&self, device: Device) -> Vec<&'static str> {
        match self.kind {
            Kind::Choice { options, .. } => options
                .iter()
                .copied()
                .filter(|id| !self.offers(id, device))
                .collect(),
            Kind::Toggle
            | Kind::Multi { .. }
            | Kind::Slider { .. }
            | Kind::File
            | Kind::Folders
            | Kind::Colour { .. }
            | Kind::Text
            | Kind::Action { .. }
            | Kind::Apply { .. }
            | Kind::Bindings { .. } => Vec::new(),
        }
    }

    /// Whether `value` is valid and, for a choice, offered on `device`. A stored value that
    /// another device offers (synced files) is skipped here, never rewritten.
    pub fn accepts_value_on(&self, value: &Value, device: Device) -> bool {
        self.accepts_value(value)
            && match value {
                Value::Text(id) => self.offers(id, device),
                Value::Bool(_) | Value::Int(_) | Value::List(_) => true,
            }
    }

    pub fn default_value(&self) -> Option<Value> {
        self.default.to_value()
    }

    /// Whether `value` is valid for this entry's kind and constraints.
    pub fn accepts_value(&self, value: &Value) -> bool {
        match (self.kind, value) {
            (Kind::Toggle, Value::Bool(_)) => true,
            // Options the device adds at run time (an audio output, a saved layout) are not in
            // the schema: any non-empty id is taken, the device decides whether it still exists.
            (
                Kind::Choice {
                    options,
                    more_at_runtime,
                    ..
                },
                Value::Text(id),
            ) => options.contains(&id.as_str()) || (more_at_runtime && !id.trim().is_empty()),
            (Kind::Multi { options }, Value::List(ids)) => {
                ids.iter().all(|id| options.contains(&id.as_str()))
                    && ids.iter().enumerate().all(|(i, id)| !ids[..i].contains(id))
            }
            (Kind::Slider { min, max, step, .. }, Value::Int(v)) => {
                (min..=max).contains(v) && step > 0 && (v - min) % step == 0
            }
            (Kind::File | Kind::Text, Value::Text(_)) | (Kind::Folders, Value::List(_)) => true,
            (Kind::Colour { .. }, Value::Text(c)) => is_colour(c),
            (Kind::Bindings { table }, Value::List(items)) => {
                crate::bindings::valid_list(table, items)
            }
            (
                Kind::Toggle
                | Kind::Choice { .. }
                | Kind::Multi { .. }
                | Kind::Slider { .. }
                | Kind::File
                | Kind::Folders
                | Kind::Colour { .. }
                | Kind::Text
                | Kind::Action { .. }
                | Kind::Apply { .. }
                | Kind::Bindings { .. },
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

/// A group of pages in the category list («GIOCO», «CONTROLLI», …).
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PageGroup {
    pub id: &'static str,
    pub pages: &'static [&'static str],
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
    /// The category list's groups, in order, with their pages.
    pub groups: &'static [PageGroup],
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
    /// Options of a choice this device does not offer: the interface does not show them.
    pub hidden_options: Vec<&'static str>,
    /// Extra words the search matches, in any language (`catalog::SYNONYMS`).
    pub synonyms: Vec<&'static str>,
    /// Listed in «Usate più spesso» (T01).
    pub most_used: bool,
    /// When the row is shown; absent: always.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_if: Option<ShowIf>,
    /// Works only on the native (Accelerated) video path.
    pub needs_native_video: bool,
}

/// The search synonyms of `key`: its own and those of a `<page>.*` row of its page.
pub fn synonyms(key: &str) -> Vec<&'static str> {
    super::catalog::SYNONYMS
        .iter()
        .filter(|(pattern, _)| {
            *pattern == key
                || pattern
                    .strip_suffix('*')
                    .is_some_and(|prefix| key.starts_with(prefix))
        })
        .flat_map(|(_, words)| words.iter().copied())
        .collect()
}

/// The whole schema for this device.
pub fn schema(device: Device) -> SchemaDto {
    SchemaDto {
        device,
        current_phase: super::catalog::CURRENT_PHASE,
        pages: super::catalog::PAGES,
        groups: super::catalog::GROUPS,
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
                hidden_options: e.hidden_options(device),
                synonyms: synonyms(e.key),
                most_used: e.most_used,
                show_if: e.show_if,
                needs_native_video: e.needs_native_video,
            })
            .collect(),
    }
}
