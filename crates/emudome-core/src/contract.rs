//! The Rust ↔ TypeScript contract (Trello #73, engineering rule 5): what the interface knows about
//! the backend is generated from the Rust types and the settings catalogue into
//! `app/src/generated/`, never typed again by hand.
//!
//! - TypeScript types of the DTOs, through ts-rs (`#[derive(TS)]`): one file per type.
//! - `error-codes.json`: every [`crate::ErrorCode`] code, read from its generated type. A Vitest
//!   test (`app/src/i18n/parity.test.ts`) holds it against `errors.<code>` in both message files.
//! - `settings-catalog.json`: pages, groups, sections, keys and option ids of the catalogue, which
//!   the same Vitest test holds against the `settings.*` texts, and the colour entries' swatches
//!   and default (`colours`: the default accent of `app/src/lib/accent.ts`, the swatches' names).
//! - `keyboard-shortcuts.json`: the keyboard's fixed keys ([`crate::keyboard::KEYBOARD_SHORTCUTS`]:
//!   `code`, `action`, `status`), which the interface lists (C01, C03).
//!
//! The generated files are committed. A test of each crate builds them again and fails when they
//! differ from the committed ones ([`check_generated`]): a changed type, a renamed setting key or a
//! new error code fails `cargo test` until the files are generated again, and then the Vitest test
//! fails until the texts exist. To generate them again:
//! `EMUDOME_BLESS=1 cargo test -p emudome-core contract` (core types and fixtures) and
//! `EMUDOME_BLESS=1 cargo test -p emudome contract` (the shell's DTOs, `generated/app/`).
//! In PowerShell: `$env:EMUDOME_BLESS=1; cargo test -p <crate> contract; Remove-Item Env:EMUDOME_BLESS`.
//!
//! Blind spots: ts-rs reads the `#[serde]` attributes, not the serializer; a type whose serde
//! attributes it cannot read (`try_from` / `into`, a hand-written `Serialize`) must carry
//! `#[ts(type = ..)]`, and its warning about them is off (`no-serde-warnings`, Cargo.toml), so
//! the serialization tests of each module are what holds those. Integers wider than 32 bits are `number` (ms timestamps
//! and byte sizes stay far below 2^53).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ts_rs::{Config, TypeVisitor, TS};

/// A setting key of the catalogue (`<prefix>.<key>`), as TypeScript sees it: the union of every
/// key, so the interface cannot name a setting that does not exist. Only a type: fields holding a
/// key are `&'static str` in Rust and carry `#[ts(as = "crate::contract::SettingKey")]`.
pub struct SettingKey;

/// A TypeScript union of string literals, for a type whose values come from a Rust list rather
/// than from its serde attributes.
macro_rules! string_union {
    ($ty:ty, $name:literal, $doc:literal, $values:expr) => {
        impl TS for $ty {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;

            fn docs() -> Option<String> {
                Some(format!(
                    "/**
 * {}
 */
",
                    $doc
                ))
            }

            fn name(_: &Config) -> String {
                $name.to_owned()
            }

            fn inline(_: &Config) -> String {
                let values: Vec<String> = $values;
                values
                    .iter()
                    .map(|v| format!("\"{v}\""))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }

            fn decl(cfg: &Config) -> String {
                format!("type {} = {};", $name, Self::inline(cfg))
            }

            fn decl_concrete(cfg: &Config) -> String {
                Self::decl(cfg)
            }

            fn output_path() -> Option<PathBuf> {
                Some(PathBuf::from(concat!($name, ".ts")))
            }
        }
    };
}

string_union!(
    SettingKey,
    "SettingKey",
    "A setting of the catalogue (crates/emudome-core/src/settings/catalog.rs).",
    crate::settings::catalog::ENTRIES
        .iter()
        .map(|e| e.key.to_owned())
        .collect()
);

string_union!(
    crate::hotkeys::Hotkey,
    "HotkeyId",
    "A controller shortcut, by `Hotkey::id`, in `Hotkey::ALL` order.",
    crate::hotkeys::Hotkey::ALL
        .iter()
        .map(|h| h.id().to_owned())
        .collect()
);

string_union!(
    crate::input::Button,
    "Button",
    "A button of the emulated pad, as serialized (`input::Button`).",
    crate::input::Button::ALL
        .iter()
        .filter_map(|b| serde_json::to_value(b).ok())
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
);

/// A type whose TypeScript comes from a shadow: a copy of its shape with `#[derive(TS)]`, kept
/// here because the type's own file belongs to the core host (`session.rs`). The test
/// `contract_shadows_serialize_like_their_types` converts every variant exhaustively and compares
/// the JSON, so a shadow cannot drift from its type.
macro_rules! shadow_ts {
    ($ty:ty => $shadow:ty) => {
        impl TS for $ty {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;

            fn docs() -> Option<String> {
                <$shadow as TS>::docs()
            }

            fn name(cfg: &Config) -> String {
                <$shadow as TS>::name(cfg)
            }

            fn inline(cfg: &Config) -> String {
                <$shadow as TS>::inline(cfg)
            }

            fn decl(cfg: &Config) -> String {
                <$shadow as TS>::decl(cfg)
            }

            fn decl_concrete(cfg: &Config) -> String {
                <$shadow as TS>::decl_concrete(cfg)
            }

            fn visit_dependencies(v: &mut impl TypeVisitor)
            where
                Self: 'static,
            {
                <$shadow as TS>::visit_dependencies(v)
            }

            fn output_path() -> Option<PathBuf> {
                <$shadow as TS>::output_path()
            }
        }
    };
}

/// Emulation speed (`session::Speed`).
#[derive(serde::Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
#[ts(rename = "Speed")]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "a shape for TypeScript, built by tests only")
)]
enum SpeedShape {
    /// Multiplier of the original speed (1.0 = normal).
    Factor(f32),
    /// As fast as possible.
    Max,
}
shadow_ts!(crate::Speed => SpeedShape);

/// What the emulation thread tells the interface (`session::Event`, `emu://session`).
#[derive(serde::Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(rename = "EmuEvent")]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "a shape for TypeScript, built by tests only")
)]
enum EventShape {
    Started {
        core: String,
        game: String,
        fps: f64,
        width: u32,
        height: u32,
    },
    StateSaved {
        slot: u8,
    },
    StateLoaded {
        slot: u8,
    },
    /// The last load was undone.
    LoadUndone,
    /// «Salvato nello slot N · Annulla»: `slot` holds what it held before the last save.
    SaveUndone {
        slot: u8,
    },
    /// G13 «Schermata salvata»: `path` is the PNG written next to the saves.
    ScreenshotSaved {
        path: String,
    },
    Error {
        error: crate::ErrorCode,
    },
    Stopped,
}
shadow_ts!(crate::Event => EventShape);

/// The environment variable that writes the generated files instead of comparing them.
pub const BLESS: &str = "EMUDOME_BLESS";

/// Generated files by their path under `app/src/generated/` (`/`-separated) and their content.
#[derive(Debug, Default)]
pub struct Generated {
    files: BTreeMap<String, String>,
}

impl Generated {
    pub fn new() -> Self {
        Self::default()
    }

    /// `T` and every type it depends on, one TypeScript file each.
    pub fn ts<T: TS + 'static + ?Sized>(&mut self) -> Result<(), String> {
        let mut collect = Collect {
            cfg: config(),
            out: &mut self.files,
            error: None,
        };
        collect.add::<T>();
        collect.error.map_or(Ok(()), Err)
    }

    /// A file written as it is (a JSON fixture).
    pub fn file(&mut self, path: &str, content: String) {
        self.files.insert(path.to_owned(), content);
    }

    /// The content of a generated file.
    pub fn get(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(String::as_str)
    }
}

/// How ts-rs writes the types: integers as `number` (JSON numbers, never `bigint`).
fn config() -> Config {
    Config::new().with_large_int("number")
}

struct Collect<'a> {
    cfg: Config,
    out: &'a mut BTreeMap<String, String>,
    error: Option<String>,
}

impl Collect<'_> {
    fn add<T: TS + 'static + ?Sized>(&mut self) {
        let Some(path) = T::output_path() else {
            return;
        };
        let path = path.to_string_lossy().replace('\\', "/");
        if self.out.contains_key(&path) {
            return;
        }
        match T::export_to_string(&self.cfg) {
            Ok(text) => {
                self.out.insert(path, text);
                T::visit_dependencies(self);
            }
            Err(e) => {
                self.error.get_or_insert(format!("{path}: {e}"));
            }
        }
    }
}

impl TypeVisitor for Collect<'_> {
    fn visit<T: TS + 'static + ?Sized>(&mut self) {
        self.add::<T>();
    }
}

/// `app/src/generated/`, from a crate's manifest directory (`crates/<name>` or `app/src-tauri`).
pub fn generated_dir(manifest_dir: &str) -> PathBuf {
    let manifest = Path::new(manifest_dir);
    let root = manifest
        .ancestors()
        .find(|dir| dir.join("app").join("package.json").is_file())
        .unwrap_or(manifest);
    root.join("app").join("src").join("generated")
}

/// Compares the files `generated` builds and `owns` with the ones under `dir`, or writes them
/// when [`BLESS`] is set (removing the owned files no longer generated). `regenerate` is the
/// command named in the failure. The error lists every file that differs.
pub fn check_generated(
    dir: &Path,
    generated: &Generated,
    owns: impl Fn(&str) -> bool,
    regenerate: &str,
) -> Result<(), String> {
    let mut on_disk = BTreeMap::new();
    list(dir, dir, &mut on_disk).map_err(|e| format!("{}: {e}", dir.display()))?;
    on_disk.retain(|path, _| owns(path));
    let wanted: BTreeMap<_, _> = generated
        .files
        .iter()
        .filter(|(path, _)| owns(path))
        .collect();

    if std::env::var(BLESS).is_ok_and(|v| v == "1") {
        for (path, text) in &wanted {
            let target = dir.join(path.as_str());
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
            }
            fs::write(&target, text.as_bytes())
                .map_err(|e| format!("{}: {e}", target.display()))?;
        }
        for path in on_disk.keys().filter(|p| !wanted.contains_key(*p)) {
            let target = dir.join(path);
            fs::remove_file(&target).map_err(|e| format!("{}: {e}", target.display()))?;
        }
        return Ok(());
    }

    let mut stale = Vec::new();
    for (path, text) in &wanted {
        match on_disk.get(path.as_str()) {
            Some(disk) if disk == *text => {}
            Some(_) => stale.push(format!("changed: {path}")),
            None => stale.push(format!("missing: {path}")),
        }
    }
    for path in on_disk.keys().filter(|p| !wanted.contains_key(*p)) {
        stale.push(format!("no longer generated: {path}"));
    }
    if stale.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "app/src/generated/ is out of date ({}). Generate it again with `{regenerate}` \
             (PowerShell: `$env:{BLESS}=1; …`), then review and commit the files.",
            stale.join(", ")
        ))
    }
}

/// Every file under `dir`, by its `/`-separated path from `root`, with its text (line endings
/// as LF, whatever the checkout did).
fn list(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) -> std::io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            list(root, &path, out)?;
        } else if let Ok(rel) = path.strip_prefix(root) {
            let rel = rel.to_string_lossy().replace('\\', "/");
            out.insert(rel, fs::read_to_string(&path)?.replace("\r\n", "\n"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value as Json};

    use super::{check_generated, generated_dir, EventShape, Generated, SpeedShape, BLESS};
    use crate::settings::catalog::{ENTRIES, GROUPS, PAGES, SECTIONS};
    use crate::settings::{DefaultValue, Kind};
    use crate::{ErrorCode, Event, Speed};

    /// The core's files: those at the top of `generated/` (the shell's are in `app/`).
    fn core_owns(path: &str) -> bool {
        !path.contains('/')
    }

    fn pretty(value: &Json) -> String {
        let mut text = serde_json::to_string_pretty(value).unwrap();
        text.push('\n');
        text
    }

    /// The codes of `ErrorCode`, in declaration order, read from its generated type.
    fn error_codes(ts: &str) -> Vec<String> {
        ts.split("\"code\": \"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .map(str::to_owned)
            .collect()
    }

    fn catalog() -> Json {
        let entries: Vec<Json> = ENTRIES
            .iter()
            .map(|e| {
                let (kind, options): (&str, &[&str]) = match e.kind {
                    Kind::Toggle => ("toggle", &[]),
                    Kind::Choice { options, .. } => ("choice", options),
                    Kind::Multi { options } => ("multi", options),
                    Kind::Slider { .. } => ("slider", &[]),
                    Kind::File => ("file", &[]),
                    Kind::Folders => ("folders", &[]),
                    Kind::Colour { .. } => ("colour", &[]),
                    Kind::Text => ("text", &[]),
                    Kind::Action { .. } => ("action", &[]),
                    Kind::Apply { options } => ("apply", options),
                    Kind::Bindings { .. } => ("bindings", &[]),
                    Kind::ScreenLayout => ("screenLayout", &[]),
                };
                json!({ "key": e.key, "page": e.page(), "kind": kind, "options": options })
            })
            .collect();
        // The colour entries' swatches and default (`general.accent`): the interface needs them
        // before any settings call (accent.ts: which swatch colors.css already holds).
        let colours: serde_json::Map<String, Json> = ENTRIES
            .iter()
            .filter_map(|e| match e.kind {
                Kind::Colour { swatches } => {
                    let default = if let DefaultValue::Text(t) = e.default {
                        Some(t)
                    } else {
                        None
                    };
                    let colour = json!({ "default": default, "swatches": swatches });
                    Some((e.key.to_owned(), colour))
                }
                Kind::Toggle
                | Kind::Choice { .. }
                | Kind::Multi { .. }
                | Kind::Slider { .. }
                | Kind::File
                | Kind::Folders
                | Kind::Text
                | Kind::Action { .. }
                | Kind::Apply { .. }
                | Kind::Bindings { .. }
                | Kind::ScreenLayout => None,
            })
            .collect();
        json!({
            "colours": colours,
            "pages": PAGES,
            "groups": GROUPS.iter().map(|g| g.id).collect::<Vec<_>>(),
            "sections": SECTIONS
                .iter()
                .map(|s| json!({ "page": s.page, "id": s.id }))
                .collect::<Vec<_>>(),
            "entries": entries,
        })
    }

    /// Everything the core generates.
    fn generate() -> Generated {
        let mut g = Generated::new();
        g.ts::<ErrorCode>().unwrap();
        g.ts::<crate::settings::SchemaDto>().unwrap();
        g.ts::<crate::settings::Scope>().unwrap();
        g.ts::<crate::settings::PageValue>().unwrap();
        g.ts::<crate::settings::GameOverride>().unwrap();
        g.ts::<crate::settings::RecentChange>().unwrap();
        g.ts::<crate::settings::CommitTarget>().unwrap();
        g.ts::<crate::close_flow::Progress>().unwrap();
        g.ts::<crate::first_run::Step>().unwrap();
        g.ts::<crate::hotkeys::Nav>().unwrap();
        g.ts::<crate::hotkeys::Status>().unwrap();
        g.ts::<crate::hotkeys::Hotkey>().unwrap();
        g.ts::<crate::input::Button>().unwrap();
        g.ts::<crate::bindings::Family>().unwrap();
        g.ts::<crate::bindings::Pos>().unwrap();
        g.ts::<crate::bios::Status>().unwrap();
        g.ts::<crate::saves::BackupReason>().unwrap();
        g.ts::<crate::saves::ImportFormat>().unwrap();
        g.ts::<crate::saves::FolderImport>().unwrap();
        g.ts::<crate::library::CoverKind>().unwrap();
        g.ts::<crate::library::Region>().unwrap();
        g.ts::<crate::Speed>().unwrap();
        g.ts::<crate::Event>().unwrap();
        g.ts::<crate::screens::Placement>().unwrap();
        g.ts::<crate::screens::LayoutId>().unwrap();
        g.ts::<crate::screens::CustomLayout>().unwrap();
        g.ts::<crate::screens::ScreenStatus>().unwrap();
        g.ts::<crate::notifications::Notification>().unwrap();
        let codes = error_codes(g.get("ErrorCode.ts").unwrap());
        g.file("error-codes.json", pretty(&json!(codes)));
        g.file("settings-catalog.json", pretty(&catalog()));
        let keys: Vec<Json> = crate::keyboard::KEYBOARD_SHORTCUTS
            .iter()
            .map(
                |k| json!({ "code": k.code, "action": k.action.id(), "status": k.action.status() }),
            )
            .collect();
        g.file("keyboard-shortcuts.json", pretty(&json!(keys)));
        g
    }

    #[test]
    fn contract_generated_files_are_up_to_date() {
        let dir = generated_dir(env!("CARGO_MANIFEST_DIR"));
        let regenerate = format!("{BLESS}=1 cargo test -p emudome-core contract");
        if let Err(e) = check_generated(&dir, &generate(), core_owns, &regenerate) {
            panic!("{e}");
        }
    }

    fn speed_shape(s: Speed) -> SpeedShape {
        match s {
            Speed::Factor(f) => SpeedShape::Factor(f),
            Speed::Max => SpeedShape::Max,
        }
    }

    fn event_shape(e: Event) -> EventShape {
        match e {
            Event::Started {
                core,
                game,
                fps,
                width,
                height,
            } => EventShape::Started {
                core,
                game,
                fps,
                width,
                height,
            },
            Event::StateSaved { slot } => EventShape::StateSaved { slot },
            Event::StateLoaded { slot } => EventShape::StateLoaded { slot },
            Event::LoadUndone => EventShape::LoadUndone,
            Event::SaveUndone { slot } => EventShape::SaveUndone { slot },
            Event::ScreenshotSaved { path } => EventShape::ScreenshotSaved { path },
            Event::Error { error } => EventShape::Error { error },
            Event::Stopped => EventShape::Stopped,
        }
    }

    #[test]
    fn contract_shadows_serialize_like_their_types() {
        for speed in [Speed::Factor(0.5), Speed::Max] {
            assert_eq!(
                serde_json::to_value(speed).unwrap(),
                serde_json::to_value(speed_shape(speed)).unwrap()
            );
        }
        let events = [
            Event::Started {
                core: "mgba".into(),
                game: "x".into(),
                fps: 59.7,
                width: 240,
                height: 160,
            },
            Event::StateSaved { slot: 1 },
            Event::StateLoaded { slot: 2 },
            Event::LoadUndone,
            Event::SaveUndone { slot: 3 },
            Event::ScreenshotSaved { path: "p".into() },
            Event::Error {
                error: ErrorCode::NoGame,
            },
            Event::Stopped,
        ];
        for event in events {
            assert_eq!(
                serde_json::to_value(&event).unwrap(),
                serde_json::to_value(event_shape(event.clone())).unwrap()
            );
        }
    }

    #[test]
    fn contract_every_error_code_is_listed_once() {
        let g = generate();
        let codes = error_codes(g.get("ErrorCode.ts").unwrap());
        // Read from the type: one per variant (with and without params).
        assert!(codes.contains(&"noGame".to_owned()));
        assert!(codes.contains(&"slotEmpty".to_owned()));
        let mut unique = codes.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), codes.len(), "{codes:?}");
        let serialized = serde_json::to_value(ErrorCode::SlotEmpty { slot: 1 }).unwrap();
        assert_eq!(serialized["code"], "slotEmpty");
    }
}
