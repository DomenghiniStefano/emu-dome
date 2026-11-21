//! The Rust ↔ TypeScript contract (Trello #73, engineering rule 5): what the interface knows about
//! the backend is generated from the Rust types and the settings catalogue into
//! `app/src/generated/`, never typed again by hand.
//!
//! - TypeScript types of the DTOs, through ts-rs (`#[derive(TS)]`): one file per type.
//! - `error-codes.json`: every [`crate::ErrorCode`] code, read from its generated type. A Vitest
//!   test (`app/src/i18n/parity.test.ts`) holds it against `errors.<code>` in both message files.
//! - `settings-catalog.json`: pages, groups, sections, keys and option ids of the catalogue, which
//!   the same Vitest test holds against the `settings.*` texts.
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

    use super::{check_generated, generated_dir, Generated, BLESS};
    use crate::settings::catalog::{ENTRIES, GROUPS, PAGES, SECTIONS};
    use crate::settings::Kind;
    use crate::ErrorCode;

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
                };
                json!({ "key": e.key, "page": e.page(), "kind": kind, "options": options })
            })
            .collect();
        json!({
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
        let codes = error_codes(g.get("ErrorCode.ts").unwrap());
        g.file("error-codes.json", pretty(&json!(codes)));
        g.file("settings-catalog.json", pretty(&catalog()));
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
