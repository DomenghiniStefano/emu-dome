//! The one-time move to the game key, on temporary folders only.

use std::path::Path;

use super::{is_done, legacy_backup, legacy_path_id, legacy_save, run, LeftReason, Paths};
use crate::settings::test_dir::TestDir;
use crate::GameKey;

fn paths(dir: &TestDir) -> Paths {
    Paths {
        saves_root: dir.path("data/saves"),
        settings_games: dir.path("data/settings/game"),
        device_file: dir.path("local/local-settings/device.json"),
        work_dir: dir.path("data/migration-70"),
    }
}

fn key(text: &str) -> GameKey {
    GameKey::parse(text).unwrap()
}

/// What the library knows: `C:/roms/Lanterna.gba` and `C:/roms/Volo.nds`.
fn library() -> Vec<(String, GameKey)> {
    vec![
        ("C:/roms/Lanterna.gba".into(), key("gba:1a")),
        ("C:/roms/Volo.nds".into(), key("nds:2b")),
    ]
}

#[test]
fn old_names_are_read_as_the_old_code_wrote_them() {
    let s = |stem: &str, new: &str| Some((stem.to_owned(), new.to_owned()));
    assert_eq!(legacy_save("Lanterna.sav"), s("Lanterna", "game.sav"));
    assert_eq!(
        legacy_save("Pokemon v1.2.sav"),
        s("Pokemon v1.2", "game.sav")
    );
    assert_eq!(
        legacy_save("Lanterna.slot3.state"),
        s("Lanterna", "slot3.state")
    );
    assert_eq!(
        legacy_save("Lanterna.slot3.png"),
        s("Lanterna", "slot3.png")
    );
    assert_eq!(
        legacy_save("Lanterna.slot10.json"),
        s("Lanterna", "slot10.json")
    );
    assert_eq!(
        legacy_save("Lanterna.auto.state"),
        s("Lanterna", "auto.state")
    );
    assert_eq!(legacy_save("Lanterna.auto.png"), s("Lanterna", "auto.png"));
    assert_eq!(
        legacy_save("Lanterna.broken-77.sav"),
        s("Lanterna", "broken-77.sav")
    );
    assert_eq!(legacy_save("note.txt"), None);
    assert_eq!(legacy_save("Lanterna.state"), None);
    assert_eq!(legacy_save("Lanterna.slotx.state"), None);
    assert_eq!(legacy_save(".sav"), None);
    assert_eq!(
        legacy_backup("Lanterna.1700000000000.launch.sav"),
        s("Lanterna", "1700000000000.launch.sav")
    );
    assert_eq!(
        legacy_backup("Pokemon v1.2.5.import.sav"),
        s("Pokemon v1.2", "5.import.sav")
    );
    assert_eq!(legacy_backup("Lanterna.5.other.sav"), None);
}

#[test]
fn the_old_settings_identity_is_the_fnv_hash_of_the_path() {
    // FNV-1a 64 of "abc", as `GameId::from_rom_path` wrote it.
    assert_eq!(legacy_path_id("abc"), "path-e71fa2190541574b");
}

#[test]
fn saves_of_a_library_game_move_once_to_its_key() {
    let dir = TestDir::new("migrate-moves");
    let p = paths(&dir);
    for (name, text) in [
        ("data/saves/Lanterna.sav", "battery"),
        ("data/saves/lanterna.slot2.state", "slot 2"),
        ("data/saves/Lanterna.slot2.png", "thumb 2"),
        ("data/saves/Lanterna.slot2.json", "{}"),
        ("data/saves/Lanterna.auto.state", "auto"),
        ("data/saves/Lanterna.broken-9.sav", "broken"),
        ("data/saves/backups/Lanterna.5.launch.sav", "backup"),
        ("data/saves/Volo.sav", "ds battery"),
        ("data/saves/cores.json", "{}"),
        ("data/saves/screenshots/Lanterna 2026.png", "shot"),
    ] {
        dir.write(name, text);
    }
    let report = run(&p, &library(), 42);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(report.left.is_empty(), "{:?}", report.left);
    assert!(report.done);
    assert!(is_done(&p));
    assert_eq!(report.moved.len(), 8);
    let gba = "data/saves/gba/1a";
    assert_eq!(dir.read(&format!("{gba}/game.sav")), "battery");
    assert_eq!(dir.read(&format!("{gba}/slot2.state")), "slot 2");
    assert_eq!(dir.read(&format!("{gba}/slot2.png")), "thumb 2");
    assert_eq!(dir.read(&format!("{gba}/slot2.json")), "{}");
    assert_eq!(dir.read(&format!("{gba}/auto.state")), "auto");
    assert_eq!(dir.read(&format!("{gba}/broken-9.sav")), "broken");
    assert_eq!(dir.read(&format!("{gba}/backups/5.launch.sav")), "backup");
    assert_eq!(dir.read("data/saves/nds/2b/game.sav"), "ds battery");
    // Nothing old is left; shared files stay; every moved file has its copy under its old name.
    assert!(!dir.path("data/saves/Lanterna.sav").exists());
    assert_eq!(dir.read("data/saves/cores.json"), "{}");
    assert_eq!(dir.read("data/saves/screenshots/Lanterna 2026.png"), "shot");
    assert_eq!(
        dir.read("data/migration-70/backup/saves/Lanterna.sav"),
        "battery"
    );
    assert_eq!(
        dir.read("data/migration-70/backup/saves/backups/Lanterna.5.launch.sav"),
        "backup"
    );
    assert!(dir.read("data/migration-70/report.json").contains("gba:1a"));
    // The new layout is what the saves code reads.
    let files = crate::GameFiles::new(&p.saves_root, &key("gba:1a"));
    assert_eq!(std::fs::read(&files.sram).unwrap(), b"battery");
    // A second pass finds nothing to do and changes nothing.
    let again = run(&p, &library(), 43);
    assert!(again.moved.is_empty() && again.left.is_empty() && again.done);
    assert_eq!(dir.read(&format!("{gba}/game.sav")), "battery");
}

#[test]
fn two_games_with_the_same_file_name_leave_the_save_untouched() {
    let dir = TestDir::new("migrate-ambiguous");
    let p = paths(&dir);
    dir.write("data/saves/Tetris.sav", "whose?");
    dir.write("data/saves/Tetris.slot1.state", "whose state?");
    let known = vec![
        ("C:/roms/Tetris.gb".into(), key("gb:01")),
        ("C:/roms/gba/TETRIS.gba".into(), key("gba:02")),
    ];
    let report = run(&p, &known, 1);
    assert!(report.moved.is_empty());
    assert_eq!(report.left.len(), 2);
    assert!(report
        .left
        .iter()
        .all(|l| l.reason == LeftReason::Ambiguous && l.games == [key("gb:01"), key("gba:02")]));
    assert_eq!(dir.read("data/saves/Tetris.sav"), "whose?");
    assert_eq!(dir.read("data/saves/Tetris.slot1.state"), "whose state?");
    assert!(!dir.path("data/saves/gb").exists() && !dir.path("data/saves/gba").exists());
    assert!(!dir.path("data/migration-70/backup").exists());
    // Ambiguity is a fact that will not change: the move is done, the files stay reported.
    assert!(report.done);
    assert!(dir
        .read("data/migration-70/report.json")
        .contains("ambiguous"));
}

#[test]
fn copies_of_one_game_with_the_same_name_are_not_ambiguous() {
    let dir = TestDir::new("migrate-copies");
    let p = paths(&dir);
    dir.write("data/saves/Lanterna.sav", "battery");
    let known = vec![
        ("C:/roms/Lanterna.gba".into(), key("gba:1a")),
        ("D:/backup/Lanterna.gba".into(), key("gba:1a")),
    ];
    let report = run(&p, &known, 1);
    assert_eq!(report.moved.len(), 1);
    assert_eq!(dir.read("data/saves/gba/1a/game.sav"), "battery");
}

#[test]
fn files_of_no_library_game_and_unknown_names_stay_and_are_reported() {
    let dir = TestDir::new("migrate-orphans");
    let p = paths(&dir);
    dir.write("data/saves/Sconosciuto.sav", "orphan");
    dir.write("data/saves/note.txt", "user's note");
    dir.write("data/saves/melonDS DS/dldi_sd_card.bin", "sd card");
    let report = run(&p, &library(), 1);
    assert!(report.moved.is_empty());
    let reasons: Vec<LeftReason> = report.left.iter().map(|l| l.reason).collect();
    assert_eq!(reasons, [LeftReason::NoGame, LeftReason::UnknownName]);
    assert_eq!(report.shared.len(), 1);
    assert!(report.shared[0].ends_with("melonDS DS"));
    assert_eq!(dir.read("data/saves/Sconosciuto.sav"), "orphan");
    assert_eq!(dir.read("data/saves/note.txt"), "user's note");
    assert_eq!(
        dir.read("data/saves/melonDS DS/dldi_sd_card.bin"),
        "sd card"
    );
}

#[test]
fn a_taken_destination_is_never_overwritten() {
    let dir = TestDir::new("migrate-dest");
    let p = paths(&dir);
    dir.write("data/saves/Lanterna.sav", "old");
    dir.write("data/saves/gba/1a/game.sav", "already new");
    let report = run(&p, &library(), 1);
    assert_eq!(report.left.len(), 1);
    assert_eq!(report.left[0].reason, LeftReason::DestinationExists);
    assert_eq!(dir.read("data/saves/Lanterna.sav"), "old");
    assert_eq!(dir.read("data/saves/gba/1a/game.sav"), "already new");
}

#[test]
fn an_interrupted_move_finishes_on_the_next_run() {
    let dir = TestDir::new("migrate-interrupted");
    let p = paths(&dir);
    // The first run stopped after copying Lanterna.sav and moving its slot: the copy is there,
    // the save still at its old place, the slot at its new place with its copy.
    dir.write("data/saves/Lanterna.sav", "battery");
    dir.write("data/migration-70/backup/saves/Lanterna.sav", "battery");
    dir.write("data/saves/gba/1a/slot1.state", "slot");
    dir.write(
        "data/migration-70/backup/saves/Lanterna.slot1.state",
        "slot",
    );
    let report = run(&p, &library(), 2);
    assert!(report.errors.is_empty() && report.left.is_empty());
    assert_eq!(report.moved.len(), 1);
    assert_eq!(dir.read("data/saves/gba/1a/game.sav"), "battery");
    assert_eq!(dir.read("data/saves/gba/1a/slot1.state"), "slot");
    // The copy was not made twice.
    assert!(!dir
        .path("data/migration-70/backup/saves/Lanterna.sav.1")
        .exists());
}

#[test]
fn a_different_copy_already_kept_is_not_replaced() {
    let dir = TestDir::new("migrate-copy-kept");
    let p = paths(&dir);
    dir.write("data/saves/Lanterna.sav", "new bytes");
    dir.write("data/migration-70/backup/saves/Lanterna.sav", "older bytes");
    run(&p, &library(), 2);
    assert_eq!(
        dir.read("data/migration-70/backup/saves/Lanterna.sav"),
        "older bytes"
    );
    assert_eq!(
        dir.read("data/migration-70/backup/saves/Lanterna.sav.1"),
        "new bytes"
    );
}

#[test]
fn per_game_settings_named_by_path_move_to_the_key() {
    let dir = TestDir::new("migrate-settings");
    let p = paths(&dir);
    let lanterna = legacy_path_id("C:/roms/Lanterna.gba");
    let gone = legacy_path_id("C:/old/place/Gone.gba");
    dir.write(
        &format!("data/settings/game/{lanterna}.json"),
        r#"{"video.scale":"fill"}"#,
    );
    dir.write(&format!("data/settings/game/{gone}.json"), "{}");
    dir.write("data/settings/game/nds-2b.json", r#"{"kept":true}"#);
    dir.write(
        "local/local-settings/device.json",
        &format!(
            r#"{{"global":{{}},"game":{{"{lanterna}":{{"pad.x":1}},"{gone}":{{"pad.y":2}}}}}}"#
        ),
    );
    let report = run(&p, &library(), 3);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(
        dir.read("data/settings/game/gba-1a.json"),
        r#"{"video.scale":"fill"}"#
    );
    assert!(!dir
        .path(&format!("data/settings/game/{lanterna}.json"))
        .exists());
    // No library file has that path: left, and so is a file that is already a key.
    assert_eq!(dir.read(&format!("data/settings/game/{gone}.json")), "{}");
    assert_eq!(
        dir.read("data/settings/game/nds-2b.json"),
        r#"{"kept":true}"#
    );
    let device: serde_json::Value =
        serde_json::from_str(&dir.read("local/local-settings/device.json")).unwrap();
    assert_eq!(device["game"]["gba-1a"]["pad.x"], 1);
    assert_eq!(device["game"][&gone]["pad.y"], 2);
    assert!(device["game"].get(&lanterna).is_none());
    assert!(dir
        .read("data/migration-70/backup/local-settings/device.json")
        .contains(&lanterna));
    let no_game = report
        .left
        .iter()
        .filter(|l| l.reason == LeftReason::NoGame)
        .count();
    assert_eq!(no_game, 2);
    // The settings engine reads the moved file as the game's.
    let mut settings = crate::settings::Settings::open(
        dir.path("data/settings"),
        p.device_file.clone(),
        crate::settings::Device::Desktop,
    );
    let scope = crate::settings::Scope::Game {
        console: crate::settings::ConsoleId::Gba,
        game: key("gba:1a"),
    };
    assert_eq!(
        settings.resolve("video.scale", &scope).unwrap().source,
        crate::settings::Level::Game
    );
}

#[test]
fn with_no_game_in_the_library_old_files_wait_for_a_later_start() {
    let dir = TestDir::new("migrate-empty-library");
    let p = paths(&dir);
    dir.write("data/saves/Lanterna.sav", "battery");
    let report = run(&p, &[], 1);
    assert!(!report.done);
    assert!(!is_done(&p));
    assert_eq!(dir.read("data/saves/Lanterna.sav"), "battery");
    // Once the library knows the game, the next start moves it.
    let report = run(&p, &library(), 2);
    assert!(report.done);
    assert_eq!(dir.read("data/saves/gba/1a/game.sav"), "battery");
}

#[test]
fn nothing_old_at_all_is_done_at_once() {
    let dir = TestDir::new("migrate-nothing");
    let p = paths(&dir);
    let report = run(&p, &[], 1);
    assert!(report.done && report.moved.is_empty() && report.left.is_empty());
    assert!(is_done(&p));
    assert!(Path::new(&p.report()).is_file());
}
