//! The settings the Rust code reads, as typed keys checked against the catalogue when the crate
//! compiles (Trello #73). A key is never a string literal outside `catalog.rs` and this file:
//! renaming a catalogue key, or changing its kind, fails the build here.
//!
//! - [`BoolKey`] (toggles), [`IntKey`] (sliders), [`TextKey`] (choices, files, colours, text),
//!   [`ListKey`] (multiple choices, folders, binding lists): each constructor is a `const fn`
//!   that panics, at compile time, when the key is not in [`super::catalog::ENTRIES`] or has
//!   another kind.
//! - [`Reader`]: the values one scope resolves, with the catalogue's default whenever a value is
//!   missing or unreadable. No reader writes a default of its own.
//!
//! Blind spot: a key passed as text to the engine (`Settings::resolve(&str, …)`, the interface's
//! commands) is still checked only at run time (`settingUnknown`); the interface's keys are typed
//! by the generated `SettingKey` union.

use super::catalog::ENTRIES;
use super::schema::{Entry, Kind};
use super::Value;

/// Byte-wise equality, usable in a `const fn`.
const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The value type a kind stores.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stored {
    Bool,
    Int,
    Text,
    List,
    Nothing,
}

const fn stored(kind: &Kind) -> Stored {
    match kind {
        Kind::Toggle => Stored::Bool,
        Kind::Slider { .. } => Stored::Int,
        Kind::Choice { .. } | Kind::File | Kind::Colour { .. } | Kind::Text => Stored::Text,
        Kind::Multi { .. } | Kind::Folders | Kind::Bindings { .. } | Kind::ScreenLayout => {
            Stored::List
        }
        Kind::Action { .. } | Kind::Apply { .. } => Stored::Nothing,
    }
}

/// `key`, after checking at compile time that the catalogue has it with a kind storing `want`.
const fn checked(key: &'static str, want: Stored) -> &'static str {
    let mut i = 0;
    while i < ENTRIES.len() {
        if same(ENTRIES[i].key, key) {
            let got = stored(&ENTRIES[i].kind);
            // `PartialEq` is not const: compare the discriminants.
            assert!(
                got as u8 == want as u8,
                "the setting has another kind in the catalogue"
            );
            return key;
        }
        i += 1;
    }
    panic!("not a setting key of the catalogue")
}

macro_rules! typed_key {
    ($(#[$doc:meta])* $name:ident, $stored:expr) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(&'static str);

        impl $name {
            /// The key, checked against the catalogue when compiled (use it in a `const`).
            pub const fn new(key: &'static str) -> Self {
                Self(checked(key, $stored))
            }

            pub const fn as_str(self) -> &'static str {
                self.0
            }

            /// Its catalogue entry.
            pub fn entry(self) -> &'static Entry {
                match super::entry(self.0) {
                    Some(entry) => entry,
                    // `new` checked it when compiled.
                    None => unreachable!("{} is in the catalogue", self.0),
                }
            }
        }
    };
}

typed_key!(
    /// A toggle.
    BoolKey,
    Stored::Bool
);
typed_key!(
    /// A slider.
    IntKey,
    Stored::Int
);
typed_key!(
    /// A choice, a file, a colour or text.
    TextKey,
    Stored::Text
);
typed_key!(
    /// A multiple choice, a folder list or a binding list.
    ListKey,
    Stored::List
);

// ---- The settings the Rust code reads ----

pub const VIDEO_SURROUND_IMG: TextKey = TextKey::new("video.surroundImg");
pub const FX_PICK: TextKey = TextKey::new("fx.fxpick");
pub const FX_INTENSITY: IntKey = IntKey::new("fx.fxint");
pub const FX_BRIGHTNESS: IntKey = IntKey::new("fx.fxbright");
pub const FX_FILE: TextKey = TextKey::new("fx.fxfile");
pub const SCREENS_CYCLE: ListKey = ListKey::new("screens.cycle");
pub const SCREENS_CUSTOM: ListKey = ListKey::new("screens.custom");
pub const SCREENS_RATIO: IntKey = IntKey::new("screens.ratio");
pub const SCREENS_GAP: IntKey = IntKey::new("screens.gap");
pub const SCREENS_ROTATION: TextKey = TextKey::new("screens.rot");
pub const SCREENS_SWAP: TextKey = TextKey::new("screens.swap");
pub const AUDIO_FF: TextKey = TextKey::new("audio.ff");
pub const SPEED_FF: TextKey = TextKey::new("speed.ffval");
pub const SPEED_SLOW: TextKey = TextKey::new("speed.slow");
pub const SPEED_REWIND: BoolKey = BoolKey::new("speed.rew");
pub const SPEED_REWIND_LENGTH: IntKey = IntKey::new("speed.rewlen");
pub const SAVES_AUTOSAVE: TextKey = TextKey::new("saves.autoq");
pub const SAVES_AUTOSAVE_ON_EXIT: BoolKey = BoolKey::new("saves.autoexit");
pub const SAVES_RESUME: TextKey = TextKey::new("saves.resume");
pub const SAVES_BACKUPS: IntKey = IntKey::new("saves.backups");
pub const PAD_LAYOUT: TextKey = TextKey::new("pad.ab");
pub const PAD_REMAP: ListKey = ListKey::new("pad.remap");
pub const PAD_MODIFIER: TextKey = TextKey::new("pad.mod");
pub const PAD_COMBOS: ListKey = ListKey::new("pad.combos");
pub const PAD_LID: TextKey = TextKey::new("pad.lid");
pub const PAD_DEAD_ZONE: IntKey = IntKey::new("pad.dead");
pub const PAD_STICK_DPAD: IntKey = IntKey::new("pad.stickdpad");
pub const PAD_TRIGGER: IntKey = IntKey::new("pad.trig");
pub const PAD_CONFIRM: TextKey = TextKey::new("pad.confirm");
pub const PAD_REPEAT: TextKey = TextKey::new("pad.repeat");
pub const GENERAL_THEME: TextKey = TextKey::new("general.theme");
pub const LIBRARY_FOLDERS: ListKey = ListKey::new("library.folders");
pub const LIBRARY_SCAN_AT_START: BoolKey = BoolKey::new("library.scanstart");
pub const LIBRARY_WATCH: BoolKey = BoolKey::new("library.watch");
pub const A11Y_HOLD_TOGGLE: BoolKey = BoolKey::new("a11y.toggle");
pub const BIOS_GB: TextKey = TextKey::new("bios.bgb");
pub const BIOS_GBC: TextKey = TextKey::new("bios.bgbc");
pub const BIOS_GBA: TextKey = TextKey::new("bios.bgba");
pub const BIOS_DS_ARM7: TextKey = TextKey::new("bios.bds7");
pub const BIOS_DS_ARM9: TextKey = TextKey::new("bios.bds9");
pub const BIOS_DS_FIRMWARE: TextKey = TextKey::new("bios.bdsfw");
pub const BIOS_DSI_ARM7: TextKey = TextKey::new("bios.bdsi7");
pub const BIOS_DSI_ARM9: TextKey = TextKey::new("bios.bdsi9");
pub const BIOS_DSI_FIRMWARE: TextKey = TextKey::new("bios.bdsifw");
pub const BIOS_DSI: TextKey = TextKey::new("bios.bdsi");
pub const CONSOLES_DS_MODE: TextKey = TextKey::new("consoles.dsmode");
pub const CONSOLES_DS_NAME: TextKey = TextKey::new("consoles.dsname");
pub const CORES_DS_BOOT: TextKey = TextKey::new("cores.mdsboot");

/// The `pad.*` settings the controller thread reads (`pads::settings_changed` follows them).
pub const PAD_KEYS: [&str; 9] = [
    PAD_LAYOUT.as_str(),
    PAD_MODIFIER.as_str(),
    PAD_COMBOS.as_str(),
    PAD_REMAP.as_str(),
    PAD_DEAD_ZONE.as_str(),
    PAD_STICK_DPAD.as_str(),
    PAD_TRIGGER.as_str(),
    PAD_CONFIRM.as_str(),
    PAD_REPEAT.as_str(),
];

/// The values one scope resolves (`read(key)`: the resolved value, `None` when it has none or
/// cannot be read), each with the catalogue's default in place of a missing or mistyped value.
pub struct Reader<F: Fn(&str) -> Option<Value>> {
    read: F,
}

impl<F: Fn(&str) -> Option<Value>> Reader<F> {
    pub fn new(read: F) -> Self {
        Self { read }
    }

    /// The raw resolved value (what `read` gives), for the code that hands it on as it is.
    pub fn value(&self, key: &str) -> Option<Value> {
        (self.read)(key)
    }

    fn or_default<T>(&self, key: &'static str, pick: impl Fn(Value) -> Option<T>) -> T {
        if let Some(v) = (self.read)(key).and_then(&pick) {
            return v;
        }
        match super::entry(key)
            .and_then(Entry::default_value)
            .and_then(pick)
        {
            Some(v) => v,
            // Every typed key holds a value of its kind (checked when compiled) and every
            // default is valid for its kind (`every_default_is_valid_for_its_kind`).
            None => unreachable!("{key}: the catalogue's default has the key's kind"),
        }
    }

    pub fn bool(&self, key: BoolKey) -> bool {
        self.or_default(key.0, |v| match v {
            Value::Bool(b) => Some(b),
            Value::Int(_) | Value::Text(_) | Value::List(_) => None,
        })
    }

    /// A slider's value, held to the slider's range.
    pub fn int(&self, key: IntKey) -> i64 {
        let v = self.or_default(key.0, |v| match v {
            Value::Int(i) => Some(i),
            Value::Bool(_) | Value::Text(_) | Value::List(_) => None,
        });
        match key.entry().kind {
            Kind::Slider { min, max, .. } => v.clamp(min, max),
            Kind::Toggle
            | Kind::Choice { .. }
            | Kind::Multi { .. }
            | Kind::File
            | Kind::Folders
            | Kind::Colour { .. }
            | Kind::Text
            | Kind::Action { .. }
            | Kind::Apply { .. }
            | Kind::Bindings { .. }
            | Kind::ScreenLayout => v,
        }
    }

    pub fn text(&self, key: TextKey) -> String {
        self.or_default(key.0, |v| match v {
            Value::Text(t) => Some(t),
            Value::Bool(_) | Value::Int(_) | Value::List(_) => None,
        })
    }

    pub fn list(&self, key: ListKey) -> Vec<String> {
        self.or_default(key.0, |v| match v {
            Value::List(l) => Some(l),
            Value::Bool(_) | Value::Int(_) | Value::Text(_) => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Reader, PAD_DEAD_ZONE, SAVES_AUTOSAVE, SAVES_RESUME, SPEED_REWIND};
    use crate::settings::Value;

    #[test]
    fn a_missing_or_mistyped_value_reads_as_the_catalogue_default() {
        let nothing = Reader::new(|_| None);
        assert_eq!(nothing.text(SAVES_RESUME), "always");
        assert_eq!(nothing.text(SAVES_AUTOSAVE), "5min");
        assert_eq!(nothing.int(PAD_DEAD_ZONE), 15);
        assert!(nothing.bool(SPEED_REWIND));

        let wrong = Reader::new(|_| Some(Value::Int(3)));
        assert_eq!(wrong.text(SAVES_RESUME), "always");

        let set = Reader::new(|key| (key == "saves.resume").then(|| Value::Text("ask".into())));
        assert_eq!(set.text(SAVES_RESUME), "ask");
    }
}
