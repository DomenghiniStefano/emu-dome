//! Settings that become core options: entries of the catalogue (`consoles.*`, `cores.*`) that a
//! core reads through `GET_VARIABLE`, turned into that core's keys and values. Pure: the caller
//! resolves each settings key for the game (Default → Global → Console → Game → Session).
//!
//! # How it is declared
//!
//! One [`CoreTable`] per core, in its own module (`core_values/<core>.rs`), listed in
//! [`TABLES`]. A table is a list of [`Rule`]s, each «setting key + value → core option key(s) +
//! value(s)»:
//!
//! - [`Map::Toggle`]: an on / off setting → one option with an «on» and an «off» value;
//! - [`Map::Choice`]: each option id of a choice setting → one or more core options (a choice
//!   sets **every** option it owns, so switching from one choice to another leaves nothing of the
//!   first behind);
//! - [`Map::Percent`]: a 0..=100 slider → `round(percent × max / 100)` as text (`0`..=`max`).
//!
//! A rule also says the consoles it applies to (a GBA setting does not reach a Game Boy game run
//! by the same core) and whether the core takes a change while the game runs ([`Apply::Live`],
//! read at its next `GET_VARIABLE_UPDATE`) or only when the game is opened again
//! ([`Apply::Restart`]).
//!
//! # Adding a core (extension point)
//!
//! 1. Write `core_values/<core>.rs` with `pub(super) static TABLE: CoreTable` (its `core` is the
//!    name `core_for` gives the core in `app/src-tauri/src/lib.rs`: `mgba`, `melondsds`,
//!    `azahar`).
//! 2. Add `mod <core>;` below and `&<core>::TABLE` to [`TABLES`].
//! 3. Option keys and values are the core's own: list them with
//!    `cargo run -p emudome-core --example headless -- <core file> <rom> --list-options`; the
//!    restart flag comes from the core's labels or source (mGBA writes «(Restart)» in the label).
//!
//! The tests in `settings/tests.rs` check every table against the catalogue: every rule's setting
//! exists with a kind its [`Map`] can read, every option id of a mapped choice has a value.
//!
//! What a change means while a game runs is decided by [`plan_update`]: the live options that
//! changed are pushed to the core, the restart ones are reported by their setting keys (the shell
//! sends them as `emu://core-restart`).

mod melondsds;
mod mgba;

use super::{ConsoleId, Value};

/// When a core takes a new value of an option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Apply {
    /// While the game runs (the core re-reads it at `GET_VARIABLE_UPDATE`).
    Live,
    /// Only when the game is opened again.
    Restart,
}

/// A choice id of a setting and the core options (key, value) it sets.
pub type ChoiceValues = (&'static str, &'static [(&'static str, &'static str)]);

/// How a setting's value becomes core options.
#[derive(Clone, Copy, Debug)]
pub enum Map {
    /// On / off → `option` = `on` or `off`.
    Toggle {
        option: &'static str,
        on: &'static str,
        off: &'static str,
    },
    /// Option id of the setting → the core options it sets.
    Choice(&'static [ChoiceValues]),
    /// 0..=100 → `round(percent × max / 100)`, written as text.
    Percent { option: &'static str, max: u32 },
}

/// One setting of the catalogue and the core options it drives.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub setting: &'static str,
    /// The consoles of the core this rule applies to.
    pub consoles: &'static [ConsoleId],
    pub map: Map,
    pub apply: Apply,
}

/// The rules of one core.
#[derive(Debug)]
pub struct CoreTable {
    /// The core's name in the shell (`mgba`, `melondsds`, `azahar`).
    pub core: &'static str,
    pub rules: &'static [Rule],
}

/// Every core's table.
pub static TABLES: &[&CoreTable] = &[&mgba::TABLE, &melondsds::TABLE];

/// One core option a setting asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappedOption {
    /// The setting it comes from.
    pub setting: &'static str,
    /// The core's option key and value.
    pub key: &'static str,
    pub value: String,
    pub apply: Apply,
}

/// The table of `core`, if it has one.
pub fn table(core: &str) -> Option<&'static CoreTable> {
    TABLES.iter().copied().find(|t| t.core == core)
}

/// Whether a change of `setting` can change some core's options.
pub fn is_core_setting(setting: &str) -> bool {
    TABLES
        .iter()
        .flat_map(|t| t.rules.iter())
        .any(|r| r.setting == setting)
}

/// The core options the settings ask for, for a game of `console` run by `core`. `value`
/// resolves a settings key for that game. A setting that cannot be read, holds a value of
/// another kind or an unknown choice asks for nothing: the core's default stays.
pub fn mapped(
    core: &str,
    console: ConsoleId,
    mut value: impl FnMut(&str) -> Option<Value>,
) -> Vec<MappedOption> {
    let rules = table(core).map(|t| t.rules).unwrap_or_default();
    let mut out = Vec::new();
    for rule in rules.iter().filter(|r| r.consoles.contains(&console)) {
        let mut push = |key: &'static str, v: String| {
            out.push(MappedOption {
                setting: rule.setting,
                key,
                value: v,
                apply: rule.apply,
            });
        };
        match (rule.map, value(rule.setting)) {
            (Map::Toggle { option, on, off }, Some(Value::Bool(b))) => {
                push(option, if b { on } else { off }.to_owned());
            }
            (Map::Choice(choices), Some(Value::Text(id))) => {
                match choices.iter().find(|(c, _)| *c == id) {
                    Some((_, options)) => {
                        for (k, v) in *options {
                            push(k, (*v).to_owned());
                        }
                    }
                    None => log::warn!("{}={id} has no core option value", rule.setting),
                }
            }
            (Map::Percent { option, max }, Some(Value::Int(p))) => {
                let level = (p.clamp(0, 100) as f64 * f64::from(max) / 100.0).round() as u32;
                push(option, level.to_string());
            }
            (
                Map::Toggle { .. } | Map::Choice(_) | Map::Percent { .. },
                Some(Value::Bool(_) | Value::Int(_) | Value::Text(_) | Value::List(_)) | None,
            ) => {}
        }
    }
    out
}

/// [`mapped`] as `GET_VARIABLE` key and value pairs, for opening a game.
pub fn core_options(
    core: &str,
    console: ConsoleId,
    value: impl FnMut(&str) -> Option<Value>,
) -> Vec<(String, String)> {
    pairs(&mapped(core, console, value))
}

/// Options as `GET_VARIABLE` key and value pairs.
pub fn pairs(options: &[MappedOption]) -> Vec<(String, String)> {
    options
        .iter()
        .map(|o| (o.key.to_owned(), o.value.clone()))
        .collect()
}

/// What a settings change means for the running core.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Update {
    /// Live options whose value changed: sent to the core now.
    pub push: Vec<(String, String)>,
    /// Settings whose restart options now differ from what the core was opened with, sorted,
    /// no duplicates. Empty: nothing waits for the game to be opened again.
    pub restart: Vec<&'static str>,
}

/// Compares the options the settings ask for now (`now`) with what the core has: `started`
/// (what it was opened with) for the restart options, `previous` (the last options asked for)
/// for the live ones. A live option no longer asked for goes back to the value it was opened
/// with, if it had one (else the core keeps the value until the game is opened again).
pub fn plan_update(
    started: &[MappedOption],
    previous: &[MappedOption],
    now: &[MappedOption],
) -> Update {
    let find = |list: &[MappedOption], key: &str| -> Option<String> {
        list.iter().find(|o| o.key == key).map(|o| o.value.clone())
    };
    let mut update = Update::default();
    for o in now {
        match o.apply {
            Apply::Live => {
                if find(previous, o.key).as_deref() != Some(o.value.as_str()) {
                    update.push.push((o.key.to_owned(), o.value.clone()));
                }
            }
            Apply::Restart => {
                if find(started, o.key).as_deref() != Some(o.value.as_str()) {
                    update.restart.push(o.setting);
                }
            }
        }
    }
    for o in previous {
        if o.apply == Apply::Live && find(now, o.key).is_none() {
            if let Some(v) = find(started, o.key).filter(|v| *v != o.value) {
                update.push.push((o.key.to_owned(), v));
            }
        }
    }
    for o in started {
        if o.apply == Apply::Restart && find(now, o.key).is_none() {
            update.restart.push(o.setting);
        }
    }
    update.restart.sort_unstable();
    update.restart.dedup();
    update
}
