//! The one-time copy to the new app identifier's folders, on temporary folders only.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::{run, Folder, Kind, Plan, STAMP};
use crate::settings::test_dir::TestDir;
use crate::ErrorCode;

/// Windows' two pairs: roaming (synced) and local (per device).
fn plan(dir: &TestDir) -> Plan {
    Plan {
        folders: vec![
            Folder {
                old: dir.path("roaming/com.domenghini.emudome"),
                new: dir.path("roaming/it.domehub.emudome"),
            },
            Folder {
                old: dir.path("local/com.domenghini.emudome"),
                new: dir.path("local/it.domehub.emudome"),
            },
        ],
        skip: vec!["EBWebView".into()],
    }
}

const OLD_DATA: &str = "roaming/com.domenghini.emudome";
const OLD_LOCAL: &str = "local/com.domenghini.emudome";
const NEW_DATA: &str = "roaming/it.domehub.emudome";
const NEW_LOCAL: &str = "local/it.domehub.emudome";

fn write_bytes(dir: &TestDir, rel: &str, bytes: &[u8]) {
    let path = dir.path(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// A user's data as the app lays it out: saves and settings in the synced folder; library,
/// covers, per-device settings, system files and the first-run marker in the local one; the
/// webview's profile, and a temporary file of an interrupted write.
fn user_data(dir: &TestDir) {
    let save: Vec<u8> = (0..=255u8).cycle().take(128 * 1024).collect();
    write_bytes(dir, &format!("{OLD_DATA}/saves/gba/1a/game.sav"), &save);
    write_bytes(
        dir,
        &format!("{OLD_DATA}/saves/gba/1a/slot1.state"),
        &[7; 3000],
    );
    write_bytes(
        dir,
        &format!("{OLD_DATA}/saves/gba/1a/slot1.png"),
        &[1, 2, 3],
    );
    dir.write(&format!("{OLD_DATA}/settings/global.json"), r#"{"a":1}"#);
    dir.write(
        &format!("{OLD_DATA}/settings/game/gba-1a.json"),
        r#"{"b":2}"#,
    );
    dir.write(&format!("{OLD_DATA}/migration-70/done"), "70\n");
    write_bytes(dir, &format!("{OLD_DATA}/saves/empty.sav"), &[]);
    dir.write(
        &format!("{OLD_DATA}/settings/.global.json.1-0.emudome-tmp"),
        "half",
    );
    dir.write(&format!("{OLD_LOCAL}/library.json"), r#"{"games":[]}"#);
    write_bytes(dir, &format!("{OLD_LOCAL}/covers/gba-1a.png"), &[9; 500]);
    dir.write(&format!("{OLD_LOCAL}/local-settings/device.json"), "{}");
    dir.write(&format!("{OLD_LOCAL}/first-run/done"), "1\n");
    write_bytes(
        dir,
        &format!("{OLD_LOCAL}/system/melondsds/firmware.bin"),
        &[4; 256],
    );
    dir.write(&format!("{OLD_LOCAL}/EBWebView/Default/Preferences"), "{}");
}

/// Every file under `rel` (path relative to it → bytes).
fn inventory(dir: &TestDir, rel: &str) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = fs::read_dir(at) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let key = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(key, fs::read(&path).unwrap());
            }
        }
    }
    let root = dir.path(rel);
    let mut out = BTreeMap::new();
    walk(&root, &root, &mut out);
    out
}

/// `files` without the names the move does not copy.
fn copied_part(mut files: BTreeMap<String, Vec<u8>>, drop: &[&str]) -> BTreeMap<String, Vec<u8>> {
    files.retain(|k, _| !drop.iter().any(|d| k.starts_with(d) || k.ends_with(d)));
    files
}

#[test]
fn the_data_survives_the_copy_and_the_old_folders_stay_untouched() {
    let dir = TestDir::new("data-move-copy");
    user_data(&dir);
    let old_data = inventory(&dir, OLD_DATA);
    let old_local = inventory(&dir, OLD_LOCAL);

    let outcome = run(&plan(&dir), 1000);

    assert!(!outcome.use_old, "{:?}", outcome.report.errors);
    assert_eq!(outcome.problem, None);
    assert_eq!(outcome.report.kind, Kind::Copied);
    assert!(outcome.report.done);
    // The old folders: every file, byte for byte, as before.
    assert_eq!(inventory(&dir, OLD_DATA), old_data);
    assert_eq!(inventory(&dir, OLD_LOCAL), old_local);
    // The new folders: the same files, without the webview profile and the temporary file;
    // the marker and the report beside them; no stamp left.
    let mut new_data = inventory(&dir, NEW_DATA);
    assert_eq!(new_data.remove("migration-90/done"), Some(b"90\n".to_vec()));
    assert!(new_data.remove("migration-90/report.json").is_some());
    assert_eq!(new_data, copied_part(old_data, &[".emudome-tmp"]));
    assert_eq!(
        inventory(&dir, NEW_LOCAL),
        copied_part(old_local, &["EBWebView/"])
    );
    assert!(!dir.path(&format!("{NEW_DATA}/{STAMP}")).exists());
    assert!(!dir.path(&format!("{NEW_LOCAL}/{STAMP}")).exists());
    assert_eq!(outcome.report.copied.len(), 2);
    assert_eq!(outcome.report.copied[0].files, 7);
    assert_eq!(outcome.report.copied[1].files, 5);
    let report = dir.read(&format!("{NEW_DATA}/migration-90/report.json"));
    assert!(report.contains("\"kind\": \"copied\""), "{report}");
}

#[test]
fn the_marker_stops_a_second_run() {
    let dir = TestDir::new("data-move-marker");
    user_data(&dir);
    assert!(!run(&plan(&dir), 1).use_old);
    // The old build wrote again after the copy: the new folders are never touched again.
    dir.write(&format!("{OLD_DATA}/settings/global.json"), r#"{"a":2}"#);
    let before = inventory(&dir, NEW_DATA);

    let again = run(&plan(&dir), 2);

    assert_eq!(again.report.kind, Kind::AlreadyDone);
    assert!(!again.use_old);
    assert_eq!(again.problem, None);
    assert_eq!(inventory(&dir, NEW_DATA), before);
}

#[test]
fn an_unfinished_copy_is_made_again_from_the_start() {
    let dir = TestDir::new("data-move-retry");
    user_data(&dir);
    // An earlier pass stopped halfway: a stamped folder, a half-written save, a file the old
    // folder no longer has; no marker.
    dir.write(&format!("{NEW_DATA}/{STAMP}"), "old path");
    write_bytes(&dir, &format!("{NEW_DATA}/saves/gba/1a/game.sav"), &[0; 10]);
    dir.write(
        &format!("{NEW_DATA}/saves/gba/1a/slot9.state"),
        "deleted since",
    );
    let old_data = inventory(&dir, OLD_DATA);

    let outcome = run(&plan(&dir), 3);

    assert!(!outcome.use_old, "{:?}", outcome.report.errors);
    assert_eq!(outcome.report.kind, Kind::Copied);
    let mut new_data = inventory(&dir, NEW_DATA);
    new_data.remove("migration-90/done");
    new_data.remove("migration-90/report.json");
    assert_eq!(new_data, copied_part(old_data, &[".emudome-tmp"]));
}

#[test]
fn a_copy_that_fails_keeps_the_old_folders_in_use_and_is_tried_again() {
    let dir = TestDir::new("data-move-fail");
    user_data(&dir);
    // The new local folder cannot be made (a file is in its way).
    dir.write(NEW_LOCAL, "in the way");
    let old_local = inventory(&dir, OLD_LOCAL);

    let failed = run(&plan(&dir), 4);

    assert!(failed.use_old);
    assert_eq!(failed.report.kind, Kind::Failed);
    assert!(!failed.report.done);
    assert!(matches!(
        &failed.problem,
        Some(ErrorCode::DataMoveFailed { old, report })
            if old.ends_with("com.domenghini.emudome") && report.ends_with("report.json")
    ));
    assert!(!dir.path(&format!("{NEW_DATA}/migration-90/done")).exists());
    assert_eq!(inventory(&dir, OLD_LOCAL), old_local);

    // The next start, with the way clear.
    fs::remove_file(dir.path(NEW_LOCAL)).unwrap();
    let retried = run(&plan(&dir), 5);
    assert!(!retried.use_old, "{:?}", retried.report.errors);
    assert_eq!(retried.report.kind, Kind::Copied);
    assert_eq!(
        inventory(&dir, NEW_LOCAL),
        copied_part(old_local, &["EBWebView/"])
    );
}

#[test]
fn a_new_folder_that_already_holds_data_is_never_overwritten() {
    let dir = TestDir::new("data-move-in-use");
    user_data(&dir);
    dir.write(
        &format!("{NEW_DATA}/settings/global.json"),
        r#"{"mine":true}"#,
    );
    dir.write(
        &format!("{NEW_LOCAL}/library.json"),
        r#"{"games":["mine"]}"#,
    );
    let old_data = inventory(&dir, OLD_DATA);

    let outcome = run(&plan(&dir), 6);

    assert!(!outcome.use_old);
    assert_eq!(outcome.report.kind, Kind::NewInUse);
    assert!(outcome.report.done);
    assert!(matches!(
        &outcome.problem,
        Some(ErrorCode::DataMoveConflict { new, .. }) if new.ends_with("it.domehub.emudome")
    ));
    assert_eq!(
        dir.read(&format!("{NEW_DATA}/settings/global.json")),
        r#"{"mine":true}"#
    );
    assert!(!dir.path(&format!("{NEW_DATA}/saves")).exists());
    assert_eq!(
        dir.read(&format!("{NEW_LOCAL}/library.json")),
        r#"{"games":["mine"]}"#
    );
    assert!(!dir.path(&format!("{NEW_LOCAL}/covers")).exists());
    assert_eq!(inventory(&dir, OLD_DATA), old_data);
    // Said once.
    assert_eq!(run(&plan(&dir), 7).problem, None);
}

#[test]
fn one_new_folder_in_use_and_the_other_empty_copies_nothing_and_keeps_the_old_ones() {
    let dir = TestDir::new("data-move-half-in-use");
    user_data(&dir);
    dir.write(
        &format!("{NEW_LOCAL}/library.json"),
        r#"{"games":["mine"]}"#,
    );

    let outcome = run(&plan(&dir), 8);

    // Using the new folders would leave the old saves behind; copying would overwrite.
    assert!(outcome.use_old);
    assert!(matches!(
        outcome.problem,
        Some(ErrorCode::DataMoveFailed { .. })
    ));
    assert!(!dir.path(&format!("{NEW_DATA}/saves")).exists());
    assert!(!dir.path(&format!("{NEW_DATA}/migration-90/done")).exists());
    assert_eq!(
        dir.read(&format!("{NEW_LOCAL}/library.json")),
        r#"{"games":["mine"]}"#
    );
}

#[test]
fn nothing_to_copy_when_the_old_folders_are_missing_or_hold_only_the_webview() {
    let dir = TestDir::new("data-move-nothing");
    dir.write(&format!("{OLD_LOCAL}/EBWebView/Default/Preferences"), "{}");

    let outcome = run(&plan(&dir), 9);

    assert!(!outcome.use_old);
    assert_eq!(outcome.problem, None);
    assert_eq!(outcome.report.kind, Kind::NothingToMove);
    assert!(dir.path(&format!("{NEW_DATA}/migration-90/done")).is_file());
    assert!(!dir.path(&format!("{NEW_LOCAL}/EBWebView")).exists());
}

#[test]
fn one_folder_for_data_and_local_data_is_copied_once() {
    let dir = TestDir::new("data-move-linux");
    dir.write("share/com.domenghini.emudome/settings/global.json", "{}");
    dir.write("share/com.domenghini.emudome/library.json", "{}");
    let one = Folder {
        old: dir.path("share/com.domenghini.emudome"),
        new: dir.path("share/it.domehub.emudome"),
    };
    let plan = Plan {
        folders: vec![one.clone(), one],
        skip: Vec::new(),
    };

    let outcome = run(&plan, 10);

    assert!(!outcome.use_old, "{:?}", outcome.report.errors);
    assert_eq!(outcome.report.copied.len(), 1);
    assert_eq!(outcome.report.copied[0].files, 2);
    assert_eq!(dir.read("share/it.domehub.emudome/library.json"), "{}");
}
