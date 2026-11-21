//! Engine tests: precedence, refused levels, device axis, session, schema invariants.
//! Option ids are read from the schema, so the tests do not depend on the catalogue's wording.

use std::collections::HashSet;

use serde_json::json;

use super::catalog::{CURRENT_PHASE, ENTRIES, GROUPS, PAGES, SECTIONS};
use super::test_dir::TestDir;
use super::{entry, CommitTarget, ConsoleId, GameKey, Kind, Level, Scope, Settings, Value};
use crate::core_profile::CoreId;
use crate::settings::schema::Device;
use crate::ErrorCode;

fn settings(dir: &TestDir) -> Settings {
    Settings::open(
        dir.path("synced"),
        dir.path("local/device.json"),
        Device::Desktop,
    )
}

fn game() -> GameKey {
    GameKey::parse("gba:7e57").unwrap()
}

fn game_scope() -> Scope {
    Scope::Game {
        console: ConsoleId::Gba,
        game: game(),
    }
}

fn console_scope() -> Scope {
    Scope::Console {
        console: ConsoleId::Gba,
    }
}

/// The `i`-th option id of a choice entry.
fn option(key: &str, i: usize) -> Value {
    match entry(key).map(|e| e.kind) {
        Some(Kind::Choice { options, .. }) => Value::Text(options[i].to_owned()),
        other => panic!("{key} is not a choice: {other:?}"),
    }
}

fn value_of(s: &mut Settings, key: &str, scope: &Scope) -> (Option<Value>, Level) {
    let r = s.resolve(key, scope).unwrap();
    (r.value, r.source)
}

/// All the text written under the synced folder.
fn synced_text(dir: &TestDir) -> String {
    fn walk(path: &std::path::Path, out: &mut String) {
        if path.is_dir() {
            for e in std::fs::read_dir(path).unwrap() {
                walk(&e.unwrap().path(), out);
            }
        } else if path.is_file() {
            out.push_str(&std::fs::read_to_string(path).unwrap());
        }
    }
    let mut out = String::new();
    walk(&dir.path("synced"), &mut out);
    out
}

// ---- Precedence ----

#[test]
fn each_level_overrides_the_weaker_ones() {
    let dir = TestDir::new("precedence");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    let key = "video.scale";
    let default = entry(key).unwrap().default_value();

    assert_eq!(
        value_of(&mut s, key, &Scope::Session),
        (default, Level::Default)
    );

    s.set(key, &Scope::Global, option(key, 1)).unwrap();
    assert_eq!(
        value_of(&mut s, key, &Scope::Session),
        (Some(option(key, 1)), Level::Global)
    );

    s.set(key, &console_scope(), option(key, 2)).unwrap();
    assert_eq!(
        value_of(&mut s, key, &Scope::Session),
        (Some(option(key, 2)), Level::Console)
    );

    s.set(key, &game_scope(), option(key, 0)).unwrap();
    assert_eq!(
        value_of(&mut s, key, &Scope::Session),
        (Some(option(key, 0)), Level::Game)
    );

    s.set(key, &Scope::Session, option(key, 1)).unwrap();
    assert_eq!(
        value_of(&mut s, key, &Scope::Session),
        (Some(option(key, 1)), Level::Session)
    );

    // Weaker scopes do not see the stronger levels.
    assert_eq!(
        value_of(&mut s, key, &game_scope()),
        (Some(option(key, 0)), Level::Game)
    );
    assert_eq!(
        value_of(&mut s, key, &console_scope()),
        (Some(option(key, 2)), Level::Console)
    );
    assert_eq!(
        value_of(&mut s, key, &Scope::Global),
        (Some(option(key, 1)), Level::Global)
    );
    // Another console does not see GBA's value.
    let gb = Scope::Console {
        console: ConsoleId::Gb,
    };
    assert_eq!(
        value_of(&mut s, key, &gb),
        (Some(option(key, 1)), Level::Global)
    );
}

#[test]
fn reset_lets_the_weaker_level_show_through() {
    let dir = TestDir::new("reset");
    let mut s = settings(&dir);
    let key = "video.scale";
    s.set(key, &Scope::Global, option(key, 1)).unwrap();
    s.set(key, &game_scope(), option(key, 2)).unwrap();
    assert_eq!(s.reset(key, &game_scope()), Ok(Level::Game));
    assert_eq!(
        value_of(&mut s, key, &game_scope()),
        (Some(option(key, 1)), Level::Global)
    );
}

#[test]
fn levels_the_entry_does_not_accept_are_skipped_when_resolving() {
    let dir = TestDir::new("skip-levels");
    // `video.fps` is global only: a value in a game file (hand-edited) is ignored.
    dir.write(
        &format!("synced/game/{}.json", game().file_stem()),
        r#"{ "video.fps": true }"#,
    );
    let mut s = settings(&dir);
    assert_eq!(
        value_of(&mut s, "video.fps", &game_scope()),
        (Some(Value::Bool(false)), Level::Default)
    );
}

#[test]
fn an_invalid_stored_value_falls_back_to_the_weaker_levels() {
    let dir = TestDir::new("invalid-stored");
    let key = "video.scale";
    let global = serde_json::to_string(&option(key, 2)).unwrap();
    dir.write("synced/global.json", &format!(r#"{{ "{key}": {global} }}"#));
    dir.write(
        "synced/console/gba.json",
        r#"{ "video.scale": "not_an_option" }"#,
    );
    let mut s = settings(&dir);
    assert_eq!(
        value_of(&mut s, key, &console_scope()),
        (Some(option(key, 2)), Level::Global)
    );
}

// ---- Refused levels and values ----

#[test]
fn a_level_the_entry_does_not_accept_is_refused() {
    let dir = TestDir::new("refused");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    // Global only.
    assert_eq!(
        s.set("video.fps", &game_scope(), Value::Bool(true)),
        Err(ErrorCode::SettingLevelRefused {
            key: "video.fps".into(),
            level: Level::Game,
            levels: vec![Level::Global],
        })
    );
    assert_eq!(
        s.set("video.fps", &Scope::Session, Value::Bool(true)),
        Err(ErrorCode::SettingLevelRefused {
            key: "video.fps".into(),
            level: Level::Session,
            levels: vec![Level::Global],
        })
    );
    // Console and game only.
    assert_eq!(
        s.reset("video.orient", &Scope::Global),
        Err(ErrorCode::SettingLevelRefused {
            key: "video.orient".into(),
            level: Level::Global,
            levels: vec![Level::Console, Level::Game],
        })
    );
    assert!(!dir.path("synced").exists(), "nothing written");
}

#[test]
fn values_that_do_not_fit_are_refused() {
    let dir = TestDir::new("invalid-values");
    let mut s = settings(&dir);
    let invalid = |key: &str| Err(ErrorCode::SettingInvalidValue { key: key.into() });
    assert_eq!(
        s.set("video.fps", &Scope::Global, Value::Int(1)),
        invalid("video.fps")
    );
    assert_eq!(
        s.set("video.scale", &Scope::Global, Value::Text("nope".into())),
        invalid("video.scale")
    );
    assert_eq!(
        s.set("audio.vol", &Scope::Global, Value::Int(1000)),
        invalid("audio.vol")
    );
    assert_eq!(
        s.set(
            "general.accent",
            &Scope::Global,
            Value::Text("purple".into())
        ),
        invalid("general.accent")
    );
    // An action holds no value.
    assert_eq!(
        s.set("about.log", &Scope::Global, Value::Bool(true)),
        invalid("about.log")
    );
    assert_eq!(
        s.set("video.nope", &Scope::Global, Value::Bool(true)),
        Err(ErrorCode::SettingUnknown {
            key: "video.nope".into()
        })
    );
}

#[test]
fn slider_values_must_sit_on_the_step_grid() {
    let dir = TestDir::new("slider-grid");
    let mut s = settings(&dir);
    let Some(Kind::Slider { min, max, step, .. }) = entry("audio.vol").map(|e| e.kind) else {
        panic!("audio.vol is a slider")
    };
    assert_eq!(
        s.set("audio.vol", &Scope::Global, Value::Int(min)),
        Ok(Level::Global)
    );
    assert_eq!(
        s.set("audio.vol", &Scope::Global, Value::Int(max)),
        Ok(Level::Global)
    );
    if step > 1 {
        assert!(s
            .set("audio.vol", &Scope::Global, Value::Int(min + 1))
            .is_err());
    }
}

// ---- Device axis ----

#[test]
fn per_device_values_never_reach_the_synced_files() {
    let dir = TestDir::new("per-device");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    let key = "video.vsync";
    assert!(entry(key).unwrap().per_device);
    s.set(key, &Scope::Global, Value::Bool(false)).unwrap();
    s.set(key, &console_scope(), Value::Bool(true)).unwrap();
    s.set(key, &game_scope(), Value::Bool(false)).unwrap();
    s.set(key, &Scope::Session, Value::Bool(true)).unwrap();
    s.commit_session(CommitTarget::Console).unwrap();
    // A portable value next to them, so the synced files exist.
    s.set("video.scale", &Scope::Global, option("video.scale", 1))
        .unwrap();

    assert!(!synced_text(&dir).contains(key));
    let device: serde_json::Value = serde_json::from_str(&dir.read("local/device.json")).unwrap();
    assert_eq!(device["global"][key], json!(false));
    assert_eq!(device["console"]["gba"][key], json!(true));
    assert_eq!(device["game"][game().file_stem()][key], json!(false));
    // And it reads back from a fresh start.
    let mut fresh = settings(&dir);
    assert_eq!(
        value_of(&mut fresh, key, &game_scope()),
        (Some(Value::Bool(false)), Level::Game)
    );
}

// ---- Sync rules (before Drive sync, #20 / #31) ----
//
// What the synced files carry and who wins, as the engine does it today. Rules not decided yet
// are tested as today's behaviour and named `today_…`: changing one is a decision, made by
// changing its test.

/// A portable entry and a per-device one, for the rules below.
const PORTABLE: &str = "video.scale";
const PER_DEVICE: &str = "video.vsync";

#[test]
fn sync_rule_portable_values_are_in_the_synced_files_at_every_level_and_never_in_device_json() {
    let dir = TestDir::new("sync-portable");
    assert!(!entry(PORTABLE).unwrap().per_device);
    let mut s = settings(&dir);
    s.set(PORTABLE, &Scope::Global, option(PORTABLE, 1))
        .unwrap();
    s.set(PORTABLE, &console_scope(), option(PORTABLE, 2))
        .unwrap();
    s.set(PORTABLE, &game_scope(), option(PORTABLE, 0)).unwrap();
    // A per-device value too, so device.json exists.
    s.set(PER_DEVICE, &Scope::Global, Value::Bool(false))
        .unwrap();

    let global: serde_json::Value = serde_json::from_str(&dir.read("synced/global.json")).unwrap();
    let console: serde_json::Value =
        serde_json::from_str(&dir.read("synced/console/gba.json")).unwrap();
    let game_file: serde_json::Value =
        serde_json::from_str(&dir.read(&format!("synced/game/{}.json", game().file_stem())))
            .unwrap();
    for (file, i) in [(&global, 1), (&console, 2), (&game_file, 0)] {
        assert_eq!(
            serde_json::to_value(option(PORTABLE, i)).unwrap(),
            file[PORTABLE]
        );
        // Each synced file says its layout and when its values changed.
        assert_eq!(file["$schema"], json!(1));
        assert!(file["$changed"][PORTABLE].is_u64());
    }
    assert!(!dir.read("local/device.json").contains(PORTABLE));
}

#[test]
fn sync_rule_the_stronger_level_wins_whichever_file_it_is_in() {
    let dir = TestDir::new("sync-precedence");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    // Per-device: Global and Game in device.json; the Game value wins for the game.
    s.set(PER_DEVICE, &Scope::Global, Value::Bool(false))
        .unwrap();
    s.set(PER_DEVICE, &game_scope(), Value::Bool(true)).unwrap();
    assert_eq!(
        value_of(&mut s, PER_DEVICE, &Scope::Session),
        (Some(Value::Bool(true)), Level::Game)
    );
    // Portable: the console's synced file over the global one.
    s.set(PORTABLE, &Scope::Global, option(PORTABLE, 1))
        .unwrap();
    s.set(PORTABLE, &console_scope(), option(PORTABLE, 2))
        .unwrap();
    assert_eq!(
        value_of(&mut s, PORTABLE, &Scope::Session),
        (Some(option(PORTABLE, 2)), Level::Console)
    );
}

#[test]
fn sync_rule_today_a_per_device_value_found_in_a_synced_file_is_ignored() {
    // A per-device entry is per-device on every OS today (`Entry::per_device` is one flag): a
    // value of it in a synced file (another version, a hand edit) is never read on any device,
    // and kept in the file as an unknown key.
    let dir = TestDir::new("sync-device-in-synced");
    dir.write(
        "synced/global.json",
        &format!(r#"{{ "{PER_DEVICE}": false, "{PORTABLE}": "fill" }}"#),
    );
    let mut s = settings(&dir);
    let (value, level) = value_of(&mut s, PER_DEVICE, &Scope::Global);
    assert_eq!(level, Level::Default);
    assert_eq!(value, Some(Value::Bool(true)));
    s.set(PORTABLE, &Scope::Global, option(PORTABLE, 1))
        .unwrap();
    assert!(dir.read("synced/global.json").contains(PER_DEVICE));
}

#[test]
fn sync_rule_today_a_portable_value_found_in_device_json_is_ignored() {
    let dir = TestDir::new("sync-portable-in-device");
    dir.write(
        "local/device.json",
        &format!(r#"{{ "global": {{ "{PORTABLE}": "fill" }} }}"#),
    );
    let mut s = settings(&dir);
    let (_, level) = value_of(&mut s, PORTABLE, &Scope::Global);
    assert_eq!(level, Level::Default);
}

#[test]
fn sync_rule_today_a_synced_file_changed_elsewhere_wins_whole_no_merge_yet() {
    // Today the last write of a file wins as a whole: a file changed by another device replaces
    // what this one read; a value set here before it, and not written again, is gone. The merge
    // by `$changed` times is the sync's work (#20 / #31), not the engine's yet.
    let dir = TestDir::new("sync-last-write");
    let mut s = settings(&dir);
    s.set(PORTABLE, &Scope::Global, option(PORTABLE, 1))
        .unwrap();
    s.set("audio.vol", &Scope::Global, Value::Int(40)).unwrap();
    // The other device's file: only `audio.vol`, newer.
    dir.write(
        "synced/global.json",
        r#"{ "$schema": 1, "audio.vol": 70, "$changed": { "audio.vol": 99999999999999 } }"#,
    );
    assert_eq!(
        value_of(&mut s, "audio.vol", &Scope::Global),
        (Some(Value::Int(70)), Level::Global)
    );
    assert_eq!(value_of(&mut s, PORTABLE, &Scope::Global).1, Level::Default);
}

#[test]
fn sync_rule_per_device_values_keep_their_change_times_on_this_device() {
    let dir = TestDir::new("sync-device-times");
    let mut s = settings(&dir);
    s.set(PER_DEVICE, &console_scope(), Value::Bool(false))
        .unwrap();
    let device: serde_json::Value = serde_json::from_str(&dir.read("local/device.json")).unwrap();
    assert!(device["console"]["gba"]["$changed"][PER_DEVICE].is_u64());
    assert_eq!(device["$schema"], json!(1));
    assert!(!dir.path("synced/console/gba.json").exists());
}

// ---- Session ----

#[test]
fn session_values_stay_in_memory_until_committed() {
    let dir = TestDir::new("session-memory");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    s.set("video.scale", &Scope::Session, option("video.scale", 2))
        .unwrap();
    s.set("audio.vol", &Scope::Session, Value::Int(50)).unwrap();
    assert!(!dir.path("synced").exists());
    assert!(!dir.path("local").exists());
    assert_eq!(
        value_of(&mut s, "audio.vol", &Scope::Session),
        (Some(Value::Int(50)), Level::Session)
    );
}

#[test]
fn commit_to_the_game_saves_and_empties_the_session() {
    let dir = TestDir::new("session-commit-game");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    s.set("audio.vol", &Scope::Session, Value::Int(50)).unwrap();
    s.set("video.scale", &Scope::Session, option("video.scale", 2))
        .unwrap();
    let (level, keys) = s.commit_session(CommitTarget::Game).unwrap();
    assert_eq!(level, Level::Game);
    assert_eq!(keys, vec!["audio.vol", "video.scale"]);
    assert_eq!(
        value_of(&mut s, "audio.vol", &Scope::Session),
        (Some(Value::Int(50)), Level::Game)
    );
    let mut fresh = settings(&dir);
    assert_eq!(
        value_of(&mut fresh, "video.scale", &game_scope()),
        (Some(option("video.scale", 2)), Level::Game)
    );
}

#[test]
fn commit_to_the_console_saves_for_every_game_of_it() {
    let dir = TestDir::new("session-commit-console");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    s.set("video.scale", &Scope::Session, option("video.scale", 2))
        .unwrap();
    assert_eq!(
        s.commit_session(CommitTarget::Console).map(|(l, _)| l),
        Ok(Level::Console)
    );
    let other = Scope::Game {
        console: ConsoleId::Gba,
        game: GameKey::parse("gba:0de7").unwrap(),
    };
    assert_eq!(
        value_of(&mut s, "video.scale", &other),
        (Some(option("video.scale", 2)), Level::Console)
    );
}

#[test]
fn a_commit_is_all_or_nothing() {
    let dir = TestDir::new("session-commit-refused");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    // `audio.vol` accepts global and game, not console.
    s.set("audio.vol", &Scope::Session, Value::Int(50)).unwrap();
    s.set("video.scale", &Scope::Session, option("video.scale", 2))
        .unwrap();
    assert_eq!(
        s.commit_session(CommitTarget::Console),
        Err(ErrorCode::SettingLevelRefused {
            key: "audio.vol".into(),
            level: Level::Console,
            levels: vec![Level::Global, Level::Game],
        })
    );
    assert!(!dir.path("synced").exists());
    // The session keeps its changes.
    assert_eq!(
        value_of(&mut s, "video.scale", &Scope::Session),
        (Some(option("video.scale", 2)), Level::Session)
    );
}

#[test]
fn discard_and_close_drop_the_changes() {
    let dir = TestDir::new("session-discard");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    s.set("audio.vol", &Scope::Session, Value::Int(50)).unwrap();
    assert_eq!(s.discard_session(), Ok(vec!["audio.vol"]));
    assert_eq!(
        value_of(&mut s, "audio.vol", &Scope::Session).1,
        Level::Default
    );

    s.set("audio.vol", &Scope::Session, Value::Int(40)).unwrap();
    assert_eq!(s.end_session(), vec!["audio.vol"]);
    assert_eq!(
        s.resolve("audio.vol", &Scope::Session),
        Err(ErrorCode::NoGame)
    );
    assert_eq!(
        s.set("audio.vol", &Scope::Session, Value::Int(40)),
        Err(ErrorCode::NoGame)
    );
    assert!(!dir.path("synced").exists());
}

#[test]
fn opening_another_game_starts_a_fresh_session() {
    let dir = TestDir::new("session-fresh");
    let mut s = settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    s.set("audio.vol", &Scope::Session, Value::Int(50)).unwrap();
    s.begin_session(ConsoleId::Gba, GameKey::parse("gba:0de8").unwrap());
    assert_eq!(
        value_of(&mut s, "audio.vol", &Scope::Session).1,
        Level::Default
    );
}

// ---- Pages ----

#[test]
fn a_page_lists_its_entries_with_source_and_changed() {
    let dir = TestDir::new("page");
    let mut s = settings(&dir);
    let rows = s.page("general", &Scope::Global).unwrap();
    let expected: Vec<&str> = ENTRIES
        .iter()
        .filter(|e| e.page() == "general")
        .map(|e| e.key)
        .collect();
    assert_eq!(rows.iter().map(|r| r.key).collect::<Vec<_>>(), expected);
    let theme = rows.iter().find(|r| r.key == "general.theme").unwrap();
    assert_eq!(theme.value, Some(Value::Text("system".into())));
    assert_eq!((theme.source, theme.changed), (Level::Default, false));

    s.set("general.theme", &Scope::Global, Value::Text("black".into()))
        .unwrap();
    let rows = s.page("general", &Scope::Global).unwrap();
    let theme = rows.iter().find(|r| r.key == "general.theme").unwrap();
    assert_eq!((theme.source, theme.changed), (Level::Global, true));

    assert_eq!(
        s.page("nope", &Scope::Global),
        Err(ErrorCode::SettingUnknown { key: "nope".into() })
    );
}

// ---- Defaults by device and console (Trello #89) ----

fn scope_of(console: ConsoleId) -> Scope {
    Scope::Game {
        console,
        game: GameKey::parse(&format!("{}:7e57", console.id())).unwrap(),
    }
}

#[test]
fn rewind_is_on_by_default_on_a_computer_but_for_3ds_and_off_on_a_phone() {
    let dir = TestDir::new("rewind-defaults");
    for (device, console, on) in [
        (Device::Desktop, ConsoleId::Gb, true),
        (Device::Desktop, ConsoleId::Gbc, true),
        (Device::Desktop, ConsoleId::Gba, true),
        (Device::Desktop, ConsoleId::Nds, true),
        (Device::Desktop, ConsoleId::N3ds, false),
        (Device::Phone, ConsoleId::Gb, false),
        (Device::Phone, ConsoleId::Gbc, false),
        (Device::Phone, ConsoleId::Gba, false),
        (Device::Phone, ConsoleId::Nds, false),
        (Device::Phone, ConsoleId::N3ds, false),
    ] {
        let mut s = Settings::open(dir.path("synced"), dir.path("local/device.json"), device);
        let expect = (Some(Value::Bool(on)), Level::Default);
        for scope in [Scope::Console { console }, scope_of(console)] {
            assert_eq!(
                value_of(&mut s, "speed.rew", &scope),
                expect,
                "{device:?} {console:?} {scope:?}"
            );
        }
        s.begin_session(console, GameKey::parse("gba:7e57").unwrap());
        assert_eq!(
            value_of(&mut s, "speed.rew", &Scope::Session),
            expect,
            "{device:?} {console:?} session"
        );
        // The rows of a page: the resolved default is not «changed».
        let row = s
            .page("speed", &Scope::Console { console })
            .unwrap()
            .into_iter()
            .find(|r| r.key == "speed.rew")
            .unwrap();
        assert_eq!((row.value, row.changed), (Some(Value::Bool(on)), false));
    }
    // The Global level has no console: the device's default.
    let mut desktop = settings(&dir);
    assert_eq!(
        value_of(&mut desktop, "speed.rew", &Scope::Global),
        (Some(Value::Bool(true)), Level::Default)
    );
    let mut phone = phone_settings(&dir);
    assert_eq!(
        value_of(&mut phone, "speed.rew", &Scope::Global),
        (Some(Value::Bool(false)), Level::Default)
    );
    // The schema the interface gets says the device's default.
    let default_of = |s: &Settings| {
        s.schema()
            .entries
            .into_iter()
            .find(|e| e.key == "speed.rew")
            .and_then(|e| e.default)
    };
    assert_eq!(default_of(&desktop), Some(Value::Bool(true)));
    assert_eq!(default_of(&phone), Some(Value::Bool(false)));
}

#[test]
fn a_user_value_wins_over_a_default_rule_at_every_level() {
    let dir = TestDir::new("rewind-user");
    let mut s = phone_settings(&dir);
    let n3ds = ConsoleId::N3ds;
    let console = Scope::Console { console: n3ds };
    let game = scope_of(n3ds);

    // Global on: every console of the phone, 3DS included, follows it.
    s.set("speed.rew", &Scope::Global, Value::Bool(true))
        .unwrap();
    assert_eq!(
        value_of(&mut s, "speed.rew", &game),
        (Some(Value::Bool(true)), Level::Global)
    );
    s.reset("speed.rew", &Scope::Global).unwrap();

    // Console on, shown as changed on its page.
    s.set("speed.rew", &console, Value::Bool(true)).unwrap();
    assert_eq!(
        value_of(&mut s, "speed.rew", &game),
        (Some(Value::Bool(true)), Level::Console)
    );
    let row = s
        .page("speed", &console)
        .unwrap()
        .into_iter()
        .find(|r| r.key == "speed.rew")
        .unwrap();
    assert_eq!((row.source, row.changed), (Level::Console, true));
    s.reset("speed.rew", &console).unwrap();

    // Game on.
    s.set("speed.rew", &game, Value::Bool(true)).unwrap();
    assert_eq!(
        value_of(&mut s, "speed.rew", &game),
        (Some(Value::Bool(true)), Level::Game)
    );
    s.reset("speed.rew", &game).unwrap();

    // Session on.
    s.begin_session(n3ds, GameKey::parse("3ds:7e57").unwrap());
    s.set("speed.rew", &Scope::Session, Value::Bool(true))
        .unwrap();
    assert_eq!(
        value_of(&mut s, "speed.rew", &Scope::Session),
        (Some(Value::Bool(true)), Level::Session)
    );

    // And off on a computer, where the default is on.
    let mut desktop = settings(&dir);
    let gba = scope_of(ConsoleId::Gba);
    desktop.set("speed.rew", &gba, Value::Bool(false)).unwrap();
    assert_eq!(
        value_of(&mut desktop, "speed.rew", &gba),
        (Some(Value::Bool(false)), Level::Game)
    );
}

#[test]
fn a_default_rule_holds_by_device_and_console() {
    use super::schema::{DefaultRule, DefaultValue};
    let phone = DefaultRule {
        device: Some(Device::Phone),
        consoles: &[],
        value: DefaultValue::Bool(false),
    };
    assert!(phone.holds(Device::Phone, None));
    assert!(phone.holds(Device::Phone, Some(ConsoleId::Gba)));
    assert!(!phone.holds(Device::Desktop, Some(ConsoleId::Gba)));
    let n3ds = DefaultRule {
        device: None,
        consoles: &[ConsoleId::N3ds],
        value: DefaultValue::Bool(false),
    };
    assert!(n3ds.holds(Device::Desktop, Some(ConsoleId::N3ds)));
    assert!(n3ds.holds(Device::Phone, Some(ConsoleId::N3ds)));
    assert!(!n3ds.holds(Device::Desktop, Some(ConsoleId::Nds)));
    // The Global level has no console: a console rule does not hold there.
    assert!(!n3ds.holds(Device::Desktop, None));
}

// ---- Identity and wire format ----

#[test]
fn scopes_and_errors_have_the_interface_shape() {
    let scope: Scope =
        serde_json::from_value(json!({ "level": "game", "console": "gba", "game": "gba:1" }))
            .unwrap();
    assert_eq!(
        scope,
        Scope::Game {
            console: ConsoleId::Gba,
            game: GameKey::parse("gba:1").unwrap()
        }
    );
    let scope: Scope = serde_json::from_value(json!({ "level": "session" })).unwrap();
    assert_eq!(scope, Scope::Session);
    assert!(serde_json::from_value::<Scope>(
        json!({ "level": "game", "console": "gba", "game": "../x" })
    )
    .is_err());
    assert_eq!(serde_json::to_value(ConsoleId::N3ds).unwrap(), json!("3ds"));
    assert_eq!(
        serde_json::to_value(ErrorCode::SettingLevelRefused {
            key: "video.fps".into(),
            level: Level::Game,
            levels: vec![Level::Global],
        })
        .unwrap(),
        json!({ "code": "settingLevelRefused",
            "params": { "key": "video.fps", "level": "game", "levels": ["global"] } })
    );
}

// ---- Schema invariants ----

/// Cowork's final option ids: lowercase letters, digits, `_` and `.` (speeds like `0.5`),
/// starting and ending with a letter or digit. The interface turns `.` into `_` in i18n keys.
fn is_option_id(id: &str) -> bool {
    let edge = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    id.chars().next().is_some_and(edge)
        && id.chars().last().is_some_and(edge)
        && id.chars().all(|c| edge(c) || c == '_' || c == '.')
}

#[test]
fn the_catalogue_has_the_entries_of_the_design_and_the_bios_rows() {
    // 157 of the design, less `speed.splist` and `general.qmorder` (owner, 18/11), plus the
    // GB, GBC, DSi ARM7, DSi ARM9 and DSi firmware rows (owner, 2026-11-21, Trello #89).
    assert_eq!(ENTRIES.len(), 160);
}

#[test]
fn keys_are_unique_and_belong_to_a_page() {
    let mut seen = HashSet::new();
    for e in ENTRIES {
        assert!(seen.insert(e.key), "duplicate key {}", e.key);
        let (prefix, key) = e.key.split_once('.').expect("key is <prefix>.<key>");
        assert!(PAGES.contains(&e.page()), "{}: unknown page", e.key);
        // The key's prefix is its page, but on «Avanzate», which kept the old keys.
        let kept = e.page() == "advanced" && ["perf", "cores", "adv"].contains(&prefix);
        assert!(prefix == e.page() || kept, "{}: page {}", e.key, e.page());
        assert!(
            !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric()),
            "{}: key part",
            e.key
        );
    }
    for page in PAGES {
        assert!(
            ENTRIES.iter().any(|e| e.page() == *page),
            "empty page {page}"
        );
    }
    let unique_pages: HashSet<_> = PAGES.iter().collect();
    assert_eq!(unique_pages.len(), PAGES.len());
}

#[test]
fn every_default_is_valid_for_its_kind() {
    for e in ENTRIES {
        match e.default_value() {
            Some(v) => assert!(
                e.has_value() && e.accepts_value(&v),
                "{}: default {v:?}",
                e.key
            ),
            None => assert!(!e.has_value(), "{}: no default", e.key),
        }
        // A rule's default has the entry's kind (the typed readers rely on it) and is offered
        // on the rule's devices.
        for rule in e.default_rules {
            let v = rule.value.to_value();
            let devices: &[Device] = match rule.device {
                Some(Device::Desktop) => &[Device::Desktop],
                Some(Device::Phone) => &[Device::Phone],
                None => &[Device::Desktop, Device::Phone],
            };
            for d in devices {
                assert!(
                    e.has_value() && v.as_ref().is_some_and(|v| e.accepts_value_on(v, *d)),
                    "{}: rule default {v:?} on {d:?}",
                    e.key
                );
            }
        }
    }
}

#[test]
fn choices_have_unique_well_formed_options() {
    for e in ENTRIES {
        let options: &[&str] = match e.kind {
            Kind::Choice { options, .. } | Kind::Multi { options } => options,
            Kind::Toggle
            | Kind::Slider { .. }
            | Kind::File
            | Kind::Folders
            | Kind::Colour { .. }
            | Kind::Text
            | Kind::Action { .. }
            | Kind::Bindings { .. }
            | Kind::ScreenLayout => continue,
            Kind::Apply { options } => options,
        };
        assert!(!options.is_empty(), "{}: no options", e.key);
        let unique: HashSet<_> = options.iter().collect();
        assert_eq!(unique.len(), options.len(), "{}: duplicate options", e.key);
        for id in options {
            assert!(is_option_id(id), "{}: option {id:?}", e.key);
            assert!(
                !options.contains(&id.replace('.', "_").as_str()) || !id.contains('.'),
                "{}: {id:?} collides with its i18n key",
                e.key
            );
        }
    }
}

#[test]
fn the_groups_list_every_page_once_in_page_order() {
    let grouped: Vec<_> = GROUPS
        .iter()
        .flat_map(|g| g.pages.iter().copied())
        .collect();
    assert_eq!(grouped, PAGES.to_vec());
}

#[test]
fn every_entry_is_in_one_section_of_its_page_in_catalogue_order() {
    let mut seen = HashSet::new();
    let mut order = Vec::new();
    for s in SECTIONS {
        assert!(PAGES.contains(&s.page), "section {} of unknown page", s.id);
        assert!(!s.keys.is_empty(), "empty section {}.{}", s.page, s.id);
        for key in s.keys {
            let e = entry(key).expect("section key is an entry");
            assert_eq!(e.page(), s.page, "{key} is not on page {}", s.page);
            assert!(seen.insert(*key), "{key} is in two sections");
            order.push(*key);
        }
    }
    let catalogue: Vec<_> = ENTRIES.iter().map(|e| e.key).collect();
    assert_eq!(order, catalogue, "sections follow the catalogue order");
}

#[test]
fn sliders_levels_and_phases_are_sane() {
    for e in ENTRIES {
        if let Kind::Slider { min, max, step, .. } = e.kind {
            assert!(min < max && step > 0, "{}: range", e.key);
        }
        assert!(!e.levels.list().is_empty(), "{}: no level", e.key);
        assert!((1..=5).contains(&e.phase), "{}: phase {}", e.key, e.phase);
    }
    assert_eq!(CURRENT_PHASE, 1);
}

#[test]
fn the_theme_entry_matches_the_interface() {
    let theme = entry("general.theme").unwrap();
    assert!(matches!(
        theme.kind,
        Kind::Choice {
            options: &["system", "dark", "black", "light"],
            ..
        }
    ));
    assert_eq!(theme.default_value(), Some(Value::Text("system".into())));
    assert!(theme.levels.accepts(Level::Global) && !theme.levels.accepts(Level::Game));
}

// ---- Options offered per device, presets, search synonyms ----

fn phone_settings(dir: &TestDir) -> Settings {
    Settings::open(
        dir.path("synced"),
        dir.path("local/device.json"),
        Device::Phone,
    )
}

#[test]
fn option_platforms_name_real_options_and_spare_the_default() {
    for e in ENTRIES {
        let Kind::Choice {
            options,
            option_platforms,
            ..
        } = e.kind
        else {
            continue;
        };
        for (id, _) in option_platforms {
            assert!(options.contains(id), "{}: unknown option {id:?}", e.key);
        }
        for device in [Device::Desktop, Device::Phone] {
            if !e.platform.includes(device) {
                continue;
            }
            let Some(Value::Text(default)) = e.default_value() else {
                panic!("{}: a choice without a text default", e.key)
            };
            assert!(
                e.offers(&default, device),
                "{}: default {default:?} not offered on {device:?}",
                e.key
            );
        }
    }
}

#[test]
fn options_of_another_device_are_hidden_refused_and_skipped() {
    let surround = entry("video.surround").unwrap();
    assert_eq!(surround.hidden_options(Device::Phone), vec!["blur"]);
    assert!(surround.hidden_options(Device::Desktop).is_empty());
    assert_eq!(
        entry("pad.gyro").unwrap().hidden_options(Device::Desktop),
        vec!["phone"]
    );
    assert_eq!(
        entry("pad.pointer")
            .unwrap()
            .hidden_options(Device::Desktop),
        vec!["touch"]
    );
    assert!(entry("pad.pointer")
        .unwrap()
        .hidden_options(Device::Phone)
        .is_empty());
    let schema = super::schema(Device::Phone);
    let dto = schema.entries.iter().find(|e| e.key == "video.surround");
    assert_eq!(dto.map(|e| e.hidden_options.clone()), Some(vec!["blur"]));

    // The desktop stores «blur» in the synced global file; the phone reads the same files.
    let dir = TestDir::new("option-platform");
    let blur = Value::Text("blur".into());
    let mut desktop = settings(&dir);
    desktop
        .set("video.surround", &Scope::Global, blur.clone())
        .unwrap();
    assert_eq!(
        value_of(&mut desktop, "video.surround", &Scope::Global),
        (Some(blur.clone()), Level::Global)
    );
    let mut phone = phone_settings(&dir);
    assert_eq!(
        value_of(&mut phone, "video.surround", &Scope::Global),
        (Some(Value::Text("black".into())), Level::Default)
    );
    // A weaker level that the phone can use shows through.
    phone
        .set(
            "video.surround",
            &Scope::Global,
            Value::Text("cover".into()),
        )
        .unwrap();
    desktop
        .set("video.surround", &console_scope(), blur.clone())
        .unwrap();
    assert_eq!(
        value_of(&mut phone, "video.surround", &console_scope()),
        (Some(Value::Text("cover".into())), Level::Global)
    );
    assert_eq!(
        phone.set("video.surround", &Scope::Global, blur),
        Err(ErrorCode::SettingInvalidValue {
            key: "video.surround".into()
        })
    );
}

#[test]
fn presets_are_actions_without_a_value() {
    let preset = entry("presets.preset").unwrap();
    assert!(matches!(preset.kind, Kind::Apply { options } if !options.is_empty()));
    assert!(!preset.has_value());
    assert_eq!(preset.default_value(), None);
    let dir = TestDir::new("presets");
    let mut s = settings(&dir);
    assert_eq!(
        s.set(
            "presets.preset",
            &Scope::Global,
            Value::Text("faithful".into())
        ),
        Err(ErrorCode::SettingInvalidValue {
            key: "presets.preset".into()
        })
    );
    assert_eq!(
        value_of(&mut s, "presets.preset", &Scope::Global),
        (None, Level::Default)
    );
}

#[test]
fn synonyms_point_at_entries_and_reach_the_schema() {
    for (pattern, words) in super::catalog::SYNONYMS {
        match pattern.strip_suffix(".*") {
            Some(page) => assert!(PAGES.contains(&page), "{pattern}: unknown page"),
            None => assert!(entry(pattern).is_some(), "{pattern}: unknown entry"),
        }
        assert!(!words.is_empty(), "{pattern}: no words");
        for w in *words {
            assert_eq!(w.to_lowercase(), *w, "{pattern}: {w:?} is not lower case");
        }
    }
    assert!(super::synonyms("bios.bds7").contains(&"firmware"));
    assert!(super::synonyms("video.vsync").contains(&"tearing"));
    assert!(super::synonyms("video.fps").is_empty());
    let schema = super::schema(Device::Desktop);
    let vsync = schema.entries.iter().find(|e| e.key == "video.vsync");
    assert!(vsync.is_some_and(|e| e.synonyms.contains(&"strappo")));
}

#[test]
fn avanzate_holds_performance_engines_and_tools_with_their_keys() {
    // T14 (#59 item 10): «Prestazioni», «Opzioni avanzate dell'emulatore» and «Strumenti
    // avanzati» are one page, between «Opzioni per console» and «Online e integrazioni».
    let keys: Vec<&str> = ENTRIES
        .iter()
        .filter(|e| e.page() == "advanced")
        .map(|e| e.key)
        .collect();
    let count = |prefix: &str| keys.iter().filter(|k| k.starts_with(prefix)).count();
    assert_eq!(keys.len(), 20);
    assert_eq!((count("perf."), count("cores."), count("adv.")), (4, 10, 6));
    let sections: Vec<&str> = SECTIONS
        .iter()
        .filter(|s| s.page == "advanced")
        .map(|s| s.id)
        .collect();
    assert_eq!(sections, ["perf", "mgba", "melonds", "azahar", "tools"]);
    let system = GROUPS.iter().find(|g| g.id == "system").map(|g| g.pages);
    assert_eq!(
        system,
        Some(&["bios", "consoles", "advanced", "online", "about"][..])
    );
    // 17 numbered pages (T02–T19 without T15, T20) and Cloud; T01 is the interface's own.
    assert_eq!(PAGES.len(), 18);
    for gone in ["cores", "perf", "adv"] {
        assert!(!PAGES.contains(&gone), "{gone} is still a page");
    }
    // Not a «Tutti i giochi» only page: the scope bar shows there.
    assert!(ENTRIES
        .iter()
        .any(|e| e.page() == "advanced" && e.levels.console));
}

#[test]
fn show_if_names_a_choice_of_the_same_page_and_its_options() {
    let mut conditional = Vec::new();
    for e in ENTRIES {
        let Some(cond) = e.show_if else { continue };
        conditional.push(e.key);
        let other = entry(cond.key).expect("show_if names an entry");
        assert_eq!(other.page(), e.page(), "{}", e.key);
        assert!(!cond.values.is_empty(), "{}", e.key);
        let Kind::Choice { options, .. } = other.kind else {
            panic!("{}: show_if on a {:?}", e.key, other.kind);
        };
        for v in cond.values {
            assert!(
                options.contains(v),
                "{}: {v} is no option of {}",
                e.key,
                cond.key
            );
        }
    }
    assert_eq!(
        conditional,
        [
            "video.surroundSet",
            "video.surroundImg",
            "video.surroundDim",
            "video.surroundAnim"
        ]
    );
}

#[test]
fn rows_show_by_the_value_of_another_row() {
    let shown = |key: &str, surround: Option<&str>| {
        let cond = entry(key).and_then(|e| e.show_if).expect("conditional row");
        cond.holds(surround.map(|s| Value::Text(s.into())).as_ref())
    };
    // «Sfondo pronto» and «Sfondo animato» only with the built-in backgrounds.
    for key in ["video.surroundSet", "video.surroundAnim"] {
        assert!(shown(key, Some("builtin")), "{key}");
        assert!(!shown(key, Some("image")), "{key}");
        assert!(!shown(key, Some("black")), "{key}");
    }
    // «Immagine di sfondo» only with your own image.
    assert!(shown("video.surroundImg", Some("image")));
    assert!(!shown("video.surroundImg", Some("cover")));
    // «Oscura lo sfondo» with anything but black.
    assert!(!shown("video.surroundDim", Some("black")));
    assert!(shown("video.surroundDim", Some("blur")));
    assert!(shown("video.surroundDim", Some("image")));
    // A value of the wrong shape, or none, is none of the listed ones.
    assert!(!shown("video.surroundImg", None));
    assert!(shown("video.surroundDim", None));
    let cond = entry("video.surroundSet").and_then(|e| e.show_if);
    assert_eq!(cond.map(|c| c.holds(Some(&Value::Bool(true)))), Some(false));
}

#[test]
fn show_if_reaches_the_schema_only_where_set() {
    let json = serde_json::to_value(super::schema(Device::Desktop)).expect("schema serializes");
    let rows = json["entries"].as_array().expect("entries");
    let row = |key: &str| rows.iter().find(|r| r["key"] == key).cloned();
    assert_eq!(
        row("video.surroundDim").map(|r| r["showIf"].clone()),
        Some(json!({ "key": "video.surround", "values": ["black"], "not": true }))
    );
    assert_eq!(
        row("video.scale").map(|r| r.get("showIf").is_none()),
        Some(true)
    );
    assert_eq!(
        row("perf.frameskip").map(|r| r["page"].clone()),
        Some(json!("advanced"))
    );
}

// ---- Recent changes («Modificate di recente», T01) ----

thread_local! {
    /// The test clock: each test runs on its own thread.
    static NOW: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn test_clock() -> u64 {
    NOW.with(std::cell::Cell::get)
}

fn at(ms: u64) {
    NOW.with(|now| now.set(ms));
}

fn timed_settings(dir: &TestDir) -> Settings {
    let mut s = settings(dir);
    s.clock = test_clock;
    s
}

fn recent_keys(s: &mut Settings) -> Vec<(&'static str, Level, u64)> {
    s.recent(10)
        .into_iter()
        .map(|r| (r.key, r.level, r.at))
        .collect()
}

#[test]
fn recent_changes_list_the_newest_first_across_levels() {
    let dir = TestDir::new("recent-levels");
    let mut s = timed_settings(&dir);
    at(1_000);
    s.set("video.scale", &Scope::Global, option("video.scale", 2))
        .unwrap();
    at(3_000);
    s.set("fx.fxint", &console_scope(), Value::Int(40)).unwrap();
    at(2_000);
    s.set("video.filter", &game_scope(), option("video.filter", 0))
        .unwrap();

    let recent = s.recent(10);
    assert_eq!(
        recent
            .iter()
            .map(|r| (r.key, r.level, r.at))
            .collect::<Vec<_>>(),
        [
            ("fx.fxint", Level::Console, 3_000),
            ("video.filter", Level::Game, 2_000),
            ("video.scale", Level::Global, 1_000),
        ]
    );
    assert_eq!(recent[0].console, Some(ConsoleId::Gba));
    assert_eq!(recent[0].game, None);
    assert_eq!(recent[1].game, Some(game()));
    assert_eq!(recent[2].console, None);
    // The limit keeps the newest.
    assert_eq!(s.recent(1).len(), 1);
    assert_eq!(s.recent(1)[0].key, "fx.fxint");
}

#[test]
fn a_new_change_moves_the_key_up_and_survives_a_restart() {
    let dir = TestDir::new("recent-update");
    {
        let mut s = timed_settings(&dir);
        at(1_000);
        s.set("video.scale", &Scope::Global, option("video.scale", 1))
            .unwrap();
        at(2_000);
        s.set("audio.vol", &Scope::Global, Value::Int(30)).unwrap();
        at(5_000);
        s.set("video.scale", &Scope::Global, option("video.scale", 2))
            .unwrap();
    }
    let mut fresh = settings(&dir);
    assert_eq!(
        recent_keys(&mut fresh),
        [
            ("video.scale", Level::Global, 5_000),
            ("audio.vol", Level::Global, 2_000),
        ]
    );
}

#[test]
fn a_reset_counts_as_a_change_at_its_level() {
    let dir = TestDir::new("recent-reset");
    let mut s = timed_settings(&dir);
    at(1_000);
    s.set("fx.fxint", &console_scope(), Value::Int(40)).unwrap();
    at(4_000);
    s.reset("fx.fxint", &console_scope()).unwrap();
    assert_eq!(recent_keys(&mut s), [("fx.fxint", Level::Console, 4_000)]);
    // The value itself is gone: the weaker level shows through.
    assert_eq!(
        value_of(&mut s, "fx.fxint", &console_scope()).1,
        Level::Default
    );
}

#[test]
fn the_session_records_nothing_until_committed() {
    let dir = TestDir::new("recent-session");
    let mut s = timed_settings(&dir);
    s.begin_session(ConsoleId::Gba, game());
    at(1_000);
    s.set("fx.fxint", &Scope::Session, Value::Int(50)).unwrap();
    assert!(s.recent(10).is_empty());
    at(7_000);
    s.commit_session(CommitTarget::Console).unwrap();
    assert_eq!(recent_keys(&mut s), [("fx.fxint", Level::Console, 7_000)]);
}

#[test]
fn change_times_follow_the_value_into_its_file() {
    let dir = TestDir::new("recent-files");
    let mut s = timed_settings(&dir);
    at(1_000);
    // Per device: the time stays on this device, like the value.
    s.set("video.vsync", &Scope::Global, Value::Bool(false))
        .unwrap();
    at(2_000);
    s.set("video.scale", &Scope::Global, option("video.scale", 1))
        .unwrap();

    assert!(!synced_text(&dir).contains("video.vsync"));
    let synced: serde_json::Value = serde_json::from_str(&dir.read("synced/global.json")).unwrap();
    assert_eq!(synced["$changed"]["video.scale"], json!(2_000));
    let device: serde_json::Value = serde_json::from_str(&dir.read("local/device.json")).unwrap();
    assert_eq!(device["global"]["$changed"]["video.vsync"], json!(1_000));
    assert_eq!(
        recent_keys(&mut s),
        [
            ("video.scale", Level::Global, 2_000),
            ("video.vsync", Level::Global, 1_000),
        ]
    );
}

#[test]
fn unknown_or_malformed_change_records_are_skipped() {
    let dir = TestDir::new("recent-unknown");
    dir.write(
        "synced/global.json",
        r#"{ "$changed": { "future.thing": 9000, "video.scale": "soon", "audio.vol": 10, "presets.preset": 20 } }"#,
    );
    dir.write("synced/console/gba.json", r#"{ "$changed": [1, 2] }"#);
    let mut s = settings(&dir);
    assert_eq!(recent_keys(&mut s), [("audio.vol", Level::Global, 10)]);
    // And the unknown record survives the next write.
    s.set("audio.vol", &Scope::Global, Value::Int(20)).unwrap();
    let synced: serde_json::Value = serde_json::from_str(&dir.read("synced/global.json")).unwrap();
    assert_eq!(synced["$changed"]["future.thing"], json!(9000));
}

#[test]
fn values_of_removed_entries_are_kept_but_never_listed() {
    // `speed.splist` and `general.qmorder` left the catalogue on 18/11: a stored value stays
    // in the file (unknown keys are kept) and «Modificate di recente» skips it.
    let dir = TestDir::new("recent-removed");
    dir.write(
        "synced/global.json",
        r#"{ "speed.splist": ["1", "2"], "general.qmorder": "x", "$changed": { "speed.splist": 30, "general.qmorder": 40, "audio.vol": 10 } }"#,
    );
    let mut s = settings(&dir);
    assert_eq!(recent_keys(&mut s), [("audio.vol", Level::Global, 10)]);
    s.set("audio.vol", &Scope::Global, Value::Int(20)).unwrap();
    let synced: serde_json::Value = serde_json::from_str(&dir.read("synced/global.json")).unwrap();
    assert_eq!(synced["speed.splist"], json!(["1", "2"]));
    assert_eq!(synced["$changed"]["general.qmorder"], json!(40));
}

#[test]
fn recent_changes_have_the_interface_shape() {
    let dir = TestDir::new("recent-shape");
    let mut s = timed_settings(&dir);
    at(42);
    s.set("fx.fxint", &console_scope(), Value::Int(40)).unwrap();
    let json = serde_json::to_value(s.recent(1)).unwrap();
    assert_eq!(
        json,
        json!([{ "key": "fx.fxint", "level": "console", "console": "gba", "game": null, "at": 42 }])
    );
}

#[test]
fn the_most_used_reach_the_schema() {
    let schema = super::schema(Device::Desktop);
    let most: Vec<&str> = schema
        .entries
        .iter()
        .filter(|e| e.most_used)
        .map(|e| e.key)
        .collect();
    assert_eq!(most.len(), 6);
    let json = serde_json::to_value(&schema).unwrap();
    let scale = json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["key"] == "video.scale")
        .unwrap();
    assert_eq!(scale["mostUsed"], json!(true));
}

#[test]
fn game_overrides_list_the_game_values_with_the_console_value_beside() {
    let dir = TestDir::new("overrides");
    let mut s = settings(&dir);
    let scale = "video.scale";
    let vol = "audio.vol";
    s.set(scale, &Scope::Global, option(scale, 1)).unwrap();
    s.set(scale, &console_scope(), option(scale, 2)).unwrap();
    s.set(scale, &game_scope(), option(scale, 0)).unwrap();
    // «Tutti i giochi» only: audio.vol has no Console level, its games get the Global value.
    s.set(vol, &Scope::Global, Value::Int(50)).unwrap();
    s.set(vol, &game_scope(), Value::Int(30)).unwrap();
    // Changes «in prova» are not the game's yet.
    s.begin_session(ConsoleId::Gba, game());
    s.set("video.filter", &Scope::Session, option("video.filter", 2))
        .unwrap();

    let list = s.game_overrides(ConsoleId::Gba, &game());
    let seen: Vec<(&str, Value, Option<Value>)> = list
        .iter()
        .map(|o| (o.key, o.value.clone(), o.base.clone()))
        .collect();
    assert_eq!(
        seen,
        vec![
            (scale, option(scale, 0), Some(option(scale, 2))),
            (vol, Value::Int(30), Some(Value::Int(50))),
        ]
    );
    assert_eq!(list[0].page, "video");

    // Another game of the same console has none.
    let other = GameKey::parse("gba:0de7").unwrap();
    assert!(s.game_overrides(ConsoleId::Gba, &other).is_empty());

    // Ripristina: once reset, the value leaves the list.
    s.reset(scale, &game_scope()).unwrap();
    let keys: Vec<&str> = s
        .game_overrides(ConsoleId::Gba, &game())
        .iter()
        .map(|o| o.key)
        .collect();
    assert_eq!(keys, vec![vol]);
}

// ---- Game Boy / Game Boy Color core options (core_values.rs) ----

/// The core options a game of `console` gets from these settings.
fn gb_options(s: Settings, console: ConsoleId) -> Vec<(String, String)> {
    let s = std::cell::RefCell::new(s);
    let scope = Scope::Game {
        console,
        game: GameKey::parse("gb:7e57").unwrap(),
    };
    super::core_values::core_options(CoreId::Mgba, console, |key| {
        s.borrow_mut()
            .resolve(key, &scope)
            .ok()
            .and_then(|r| r.value)
    })
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
fn gb_defaults_become_mgba_options() {
    let dir = TestDir::new("gb-core-defaults");
    assert_eq!(
        gb_options(settings(&dir), ConsoleId::Gb),
        pairs(&[
            ("mgba_gb_model", "Autodetect"),
            ("mgba_gb_colors", "DMG Green"),
            ("mgba_gb_colors_preset", "0"),
            ("mgba_sgb_borders", "ON"),
            // perf.frameskip «auto»: mGBA's auto mode, which skips nothing without the audio
            // buffer callback (core_values/mgba.rs).
            ("mgba_frameskip", "auto"),
            ("mgba_frameskip_interval", "0"),
            ("mgba_use_bios", "ON"),
            ("mgba_skip_bios", "OFF"),
            ("mgba_audio_low_pass_filter", "disabled"),
            ("mgba_allow_opposing_directions", "no"),
        ])
    );
}

#[test]
fn gbc_console_level_changes_reach_the_core_and_not_gb() {
    let dir = TestDir::new("gb-core-console");
    let mut s = settings(&dir);
    let gbc = Scope::Console {
        console: ConsoleId::Gbc,
    };
    s.set("consoles.gbmodel", &gbc, Value::Text("gbc".into()))
        .unwrap();
    s.set("consoles.gbpal", &gbc, Value::Text("gbc".into()))
        .unwrap();
    s.set("consoles.sgb", &gbc, Value::Bool(false)).unwrap();
    let out = gb_options(s, ConsoleId::Gbc);
    for pair in pairs(&[
        ("mgba_gb_model", "Game Boy Color"),
        ("mgba_gb_colors", "Grayscale"),
        ("mgba_gb_colors_preset", "1"),
        ("mgba_sgb_borders", "OFF"),
    ]) {
        assert!(out.contains(&pair), "{pair:?} missing from {out:?}");
    }
    // The Game Boy console keeps its own values.
    let s = settings(&dir);
    assert!(gb_options(s, ConsoleId::Gb).contains(&("mgba_gb_model".into(), "Autodetect".into())));
}

#[test]
fn every_gb_choice_maps_to_an_mgba_value() {
    // Every option id of the catalogue's GB entries has a core value: a new id without one
    // would silently leave the core on its default.
    for key in ["consoles.gbmodel", "consoles.gbpal"] {
        let Some(Kind::Choice { options, .. }) = entry(key).map(|e| e.kind) else {
            panic!("{key} is not a choice");
        };
        for id in options {
            let out = super::core_values::core_options(CoreId::Mgba, ConsoleId::Gb, |k| {
                (k == key).then(|| Value::Text((*id).to_owned()))
            });
            assert!(!out.is_empty(), "{key}={id} has no mGBA value");
        }
    }
}

#[test]
fn gba_gets_no_gb_options_and_gb_no_gba_options() {
    let dir = TestDir::new("gb-core-gba");
    let gba = gb_options(settings(&dir), ConsoleId::Gba);
    assert!(gba
        .iter()
        .all(|(k, _)| !k.starts_with("mgba_gb_") && k != "mgba_sgb_borders"));
    let gb = gb_options(settings(&dir), ConsoleId::Gb);
    for k in [
        "mgba_color_correction",
        "mgba_interframe_blending",
        "mgba_solar_sensor_level",
    ] {
        assert!(
            gb.iter().all(|(key, _)| key != k),
            "{k} reached a Game Boy game"
        );
    }
}

// ---- Every core's option table (core_values.rs) ----

#[test]
fn frameskip_drives_mgba_on_every_mgba_console_live() {
    use super::core_values::{mapped, Apply};
    let cases = [
        ("auto", "auto", "0"),
        ("never", "disabled", "0"),
        ("1", "fixed_interval", "1"),
        ("2", "fixed_interval", "2"),
        ("3", "fixed_interval", "3"),
        ("4", "fixed_interval", "4"),
    ];
    let Some(Kind::Choice { options, .. }) = entry("perf.frameskip").map(|e| e.kind) else {
        panic!("perf.frameskip is not a choice");
    };
    assert_eq!(
        options.len(),
        cases.len(),
        "a frameskip option without a case"
    );
    for console in [ConsoleId::Gb, ConsoleId::Gbc, ConsoleId::Gba] {
        for (id, mode, interval) in cases {
            let out: Vec<_> = mapped(CoreId::Mgba, console, |k| {
                (k == "perf.frameskip").then(|| Value::Text(id.to_owned()))
            });
            let got: Vec<(&str, &str)> = out.iter().map(|o| (o.key, o.value.as_str())).collect();
            assert_eq!(
                got,
                [
                    ("mgba_frameskip", mode),
                    ("mgba_frameskip_interval", interval)
                ],
                "{console:?} {id}"
            );
            assert!(out.iter().all(|o| o.apply == Apply::Live));
        }
    }
    // melonDS DS has no equivalent: nothing is asked, and the schema marks the row.
    assert!(mapped(CoreId::MelonDsDs, ConsoleId::Nds, |k| {
        (k == "perf.frameskip").then(|| Value::Text("2".into()))
    })
    .is_empty());
}

#[test]
fn gba_defaults_become_mgba_options() {
    let dir = TestDir::new("gba-core-defaults");
    assert_eq!(
        gb_options(settings(&dir), ConsoleId::Gba),
        pairs(&[
            ("mgba_color_correction", "Auto"),
            ("mgba_interframe_blending", "OFF"),
            ("mgba_solar_sensor_level", "5"),
            ("mgba_frameskip", "auto"),
            ("mgba_frameskip_interval", "0"),
            ("mgba_use_bios", "ON"),
            ("mgba_skip_bios", "OFF"),
            ("mgba_audio_low_pass_filter", "disabled"),
            ("mgba_allow_opposing_directions", "no"),
        ])
    );
}

#[test]
fn gba_game_and_session_levels_reach_the_core() {
    let dir = TestDir::new("gba-core-levels");
    let mut s = settings(&dir);
    let gba = Scope::Console {
        console: ConsoleId::Gba,
    };
    let game = Scope::Game {
        console: ConsoleId::Gba,
        game: GameKey::parse("gb:7e57").unwrap(),
    };
    s.set("consoles.gbablend", &gba, Value::Text("on".into()))
        .unwrap();
    s.set("consoles.gbablend", &game, Value::Text("auto".into()))
        .unwrap();
    s.set("consoles.solar", &game, Value::Int(100)).unwrap();
    s.set("cores.mgbaopp", &gba, Value::Bool(true)).unwrap();
    let out = gb_options(s, ConsoleId::Gba);
    for pair in pairs(&[
        ("mgba_interframe_blending", "mix_smart"),
        ("mgba_solar_sensor_level", "10"),
        ("mgba_allow_opposing_directions", "yes"),
    ]) {
        assert!(out.contains(&pair), "{pair:?} missing from {out:?}");
    }
}

#[test]
fn percent_rounds_to_the_core_levels() {
    use super::core_values::mapped;
    let level = |p: i64| {
        mapped(CoreId::Mgba, ConsoleId::Gba, |k| {
            (k == "consoles.solar").then_some(Value::Int(p))
        })
        .into_iter()
        .find(|o| o.key == "mgba_solar_sensor_level")
        .map(|o| o.value)
    };
    assert_eq!(level(0).as_deref(), Some("0"));
    assert_eq!(level(5).as_deref(), Some("1")); // 0.5 rounds up
    assert_eq!(level(50).as_deref(), Some("5"));
    assert_eq!(level(100).as_deref(), Some("10"));
    assert_eq!(level(250).as_deref(), Some("10"), "out of range is clamped");
}

#[test]
fn azahar_takes_the_3ds_settings_as_its_options_when_the_game_opens() {
    use super::core_values::{mapped, Apply};
    let out = mapped(CoreId::Azahar, ConsoleId::N3ds, |k| match k {
        "consoles.n3dsmodel" => Some(Value::Text("o3ds".into())),
        "consoles.n3dsreg" => Some(Value::Text("eur".into())),
        "video.res3d" => Some(Value::Text("3x".into())),
        "video.texf" => Some(Value::Text("anime4k".into())),
        "perf.cpuclock" => Some(Value::Int(150)),
        "cores.azaudio" => Some(Value::Text("lle".into())),
        _ => None,
    });
    let pairs: Vec<(&str, &str)> = out.iter().map(|o| (o.key, o.value.as_str())).collect();
    assert_eq!(
        pairs,
        [
            ("citra_is_new_3ds", "Old 3DS"),
            ("citra_region_value", "Europe"),
            ("citra_resolution_factor", "3"),
            ("citra_texture_filter", "Anime4K Ultrafast"),
            ("citra_cpu_clock_percentage", "150"),
            ("citra_audio_emulation", "lle"),
        ]
    );
    assert!(out.iter().all(|o| o.apply == Apply::Restart));
    // The 3DS rules reach no other console, and the other cores never see them.
    assert!(mapped(CoreId::Azahar, ConsoleId::Nds, |_| Some(Value::Text(
        "3x".into()
    )))
    .is_empty());
    for (core, console) in [
        (CoreId::Mgba, ConsoleId::Gba),
        (CoreId::MelonDsDs, ConsoleId::Nds),
    ] {
        let out = mapped(core, console, |k| match k {
            "video.res3d" => Some(Value::Text("3x".into())),
            "perf.cpuclock" => Some(Value::Int(150)),
            _ => None,
        });
        assert!(out.is_empty(), "{core}: {out:?}");
    }
}

#[test]
fn a_value_of_the_wrong_kind_or_an_unknown_choice_asks_for_nothing() {
    use super::core_values::mapped;
    let out = mapped(CoreId::Mgba, ConsoleId::Gba, |k| match k {
        "consoles.gbacc" => Some(Value::Text("sepia".into())),
        "cores.mgbalp" => Some(Value::Int(1)),
        _ => None,
    });
    assert!(out.is_empty(), "{out:?}");
    assert!(mapped(CoreId::Azahar, ConsoleId::N3ds, |_| Some(Value::Bool(true))).is_empty());
}

#[test]
fn every_core_table_matches_the_catalogue() {
    use super::core_values::{Map, TABLES};
    let mut seen = HashSet::new();
    for table in TABLES {
        for rule in table.rules {
            assert!(
                seen.insert((table.core, rule.setting)),
                "{} maps {} twice",
                table.core,
                rule.setting
            );
            assert!(
                !rule.consoles.is_empty(),
                "{} reaches no console",
                rule.setting
            );
            let e = entry(rule.setting)
                .unwrap_or_else(|| panic!("{} is not in the catalogue", rule.setting));
            match (rule.map, &e.kind) {
                (Map::Toggle { .. }, Kind::Toggle) => {}
                (Map::Number { .. }, Kind::Slider { .. }) => {}
                (Map::Percent { .. }, Kind::Slider { min, max, .. }) => {
                    assert_eq!(
                        (*min, *max),
                        (0, 100),
                        "{} is not a percentage",
                        rule.setting
                    );
                }
                (Map::Choice(choices), Kind::Choice { options, .. }) => {
                    // Every option id of the catalogue has its core values, and nothing else.
                    let ids: HashSet<&str> = choices.iter().map(|(id, _)| *id).collect();
                    let catalogue: HashSet<&str> = options.iter().copied().collect();
                    assert_eq!(ids, catalogue, "{} choices", rule.setting);
                    // Each choice sets the same options, so a change leaves nothing behind.
                    fn keys<'a>(o: &[(&'a str, &str)]) -> Vec<&'a str> {
                        o.iter().map(|(k, _)| *k).collect()
                    }
                    for (id, o) in choices {
                        assert_eq!(keys(o), keys(choices[0].1), "{}={id}", rule.setting);
                    }
                }
                (map, kind) => panic!("{}: {map:?} cannot read {kind:?}", rule.setting),
            }
        }
    }
}

#[test]
fn every_console_and_core_entry_with_an_mgba_option_is_mapped() {
    // The `consoles.*` and `cores.*` entries of the consoles mGBA runs that have a real mGBA
    // option. `cores.mgbaall` is a link (the full option list), not a value.
    use super::core_values::{is_core_setting, table};
    let mapped: HashSet<&str> = table(CoreId::Mgba)
        .map(|t| t.rules.iter().map(|r| r.setting).collect())
        .unwrap_or_default();
    for key in [
        "consoles.gbmodel",
        "consoles.gbpal",
        "consoles.sgb",
        "consoles.gbacc",
        "consoles.gbablend",
        "consoles.solar",
        "cores.mgbabios",
        "cores.mgbaskip",
        "cores.mgbalp",
        "cores.mgbaopp",
    ] {
        assert!(mapped.contains(key), "{key} is not mapped");
        assert!(is_core_setting(key));
    }
    assert!(!is_core_setting("audio.vol"));
}

mod core_updates {
    use super::super::core_values::{plan_update, Apply, MappedOption, Update};

    fn opt(setting: &'static str, key: &'static str, value: &str, apply: Apply) -> MappedOption {
        MappedOption {
            setting,
            key,
            value: value.to_owned(),
            apply,
        }
    }

    fn live(value: &str) -> MappedOption {
        opt(
            "consoles.gbacc",
            "mgba_color_correction",
            value,
            Apply::Live,
        )
    }

    fn restart(value: &str) -> MappedOption {
        opt("consoles.gbmodel", "mgba_gb_model", value, Apply::Restart)
    }

    #[test]
    fn nothing_changed_nothing_to_do() {
        let started = [live("Auto"), restart("Autodetect")];
        assert_eq!(plan_update(&started, &started, &started), Update::default());
    }

    #[test]
    fn a_live_change_is_pushed_once() {
        let started = [live("Auto")];
        let now = [live("GBA")];
        let update = plan_update(&started, &started, &now);
        assert_eq!(
            update.push,
            vec![("mgba_color_correction".to_owned(), "GBA".to_owned())]
        );
        assert!(update.restart.is_empty());
        // The same values again: already pushed.
        assert_eq!(plan_update(&started, &now, &now), Update::default());
        // Back to the start: pushed again.
        assert_eq!(plan_update(&started, &now, &started).push.len(), 1);
    }

    #[test]
    fn a_restart_change_is_reported_until_it_goes_back() {
        let started = [restart("Autodetect"), live("Auto")];
        let now = [restart("Game Boy"), live("Auto")];
        let update = plan_update(&started, &started, &now);
        assert!(update.push.is_empty(), "a restart option is not pushed");
        assert_eq!(update.restart, vec!["consoles.gbmodel"]);
        // Still pending at the next unrelated change.
        let later = [restart("Game Boy"), live("GBA")];
        assert_eq!(
            plan_update(&started, &now, &later).restart,
            vec!["consoles.gbmodel"]
        );
        // Back to the value the game was opened with: nothing waits any more.
        assert!(plan_update(&started, &later, &started).restart.is_empty());
    }

    #[test]
    fn options_that_disappear_go_back_or_wait() {
        let started = [live("Auto"), restart("Autodetect")];
        let previous = [live("GBA"), restart("Autodetect")];
        let update = plan_update(&started, &previous, &[]);
        assert_eq!(
            update.push,
            vec![("mgba_color_correction".to_owned(), "Auto".to_owned())]
        );
        assert_eq!(update.restart, vec!["consoles.gbmodel"]);
    }

    #[test]
    fn a_choice_with_two_options_is_reported_once() {
        let started = [
            opt(
                "consoles.gbpal",
                "mgba_gb_colors",
                "DMG Green",
                Apply::Restart,
            ),
            opt(
                "consoles.gbpal",
                "mgba_gb_colors_preset",
                "0",
                Apply::Restart,
            ),
        ];
        let now = [
            opt(
                "consoles.gbpal",
                "mgba_gb_colors",
                "Grayscale",
                Apply::Restart,
            ),
            opt(
                "consoles.gbpal",
                "mgba_gb_colors_preset",
                "1",
                Apply::Restart,
            ),
        ];
        assert_eq!(
            plan_update(&started, &started, &now).restart,
            vec!["consoles.gbpal"]
        );
    }
}

#[test]
fn a_choice_with_options_at_run_time_takes_any_non_empty_id() {
    // `audio.out` lists only «system»; the device adds its outputs (Trello #73).
    let out = entry("audio.out").unwrap();
    assert!(matches!(
        out.kind,
        Kind::Choice {
            more_at_runtime: true,
            ..
        }
    ));
    assert!(out.accepts_value(&Value::Text("system".into())));
    assert!(out.accepts_value(&Value::Text("Speakers (Realtek)".into())));
    assert!(!out.accepts_value(&Value::Text(String::new())));
    assert!(!out.accepts_value(&Value::Text("  ".into())));
    assert!(!out.accepts_value(&Value::Int(1)));

    let dir = TestDir::new("more-at-runtime");
    let mut s = settings(&dir);
    let device = Value::Text("Speakers (Realtek)".into());
    s.set("audio.out", &Scope::Global, device.clone()).unwrap();
    assert_eq!(
        value_of(&mut s, "audio.out", &Scope::Global),
        (Some(device), Level::Global)
    );

    // A fixed choice still refuses anything else.
    let fixed = entry("video.scale").unwrap();
    assert!(matches!(
        fixed.kind,
        Kind::Choice {
            more_at_runtime: false,
            ..
        }
    ));
    assert!(!fixed.accepts_value(&Value::Text("Speakers (Realtek)".into())));
}

#[test]
fn the_entries_not_wired_yet_are_these() {
    // A setting the user can change is read by the code or shown disabled with the reason
    // (X05, engineering rule 5). Wiring one (or adding one nothing reads) changes this list on
    // purpose; `.not_wired()` in catalog.rs is the switch.
    let unwired: Vec<&str> = ENTRIES
        .iter()
        .filter(|e| e.has_value() && !e.wired)
        .map(|e| e.key)
        .collect();
    assert_eq!(
        unwired,
        [
            "video.aspect",
            "video.fps",
            "video.window",
            "video.winsize",
            "video.portrait",
            "video.orient",
            "video.api",
            "screens.second",
            "screens.stereo",
            "audio.mic",
            "audio.out",
            "audio.lat",
            "speed.ffmode",
            "speed.runahead",
            "saves.rtc",
            "pad.gyro",
            "touch.tport",
            "touch.tland",
            "touch.thapt",
            "touch.thide",
            "touch.tpad",
            "general.ui",
            "general.start",
            "general.bgpause",
            "general.confirmdel",
            "general.updates",
            "library.group",
            "library.region",
            "library.covers",
            "cloud.ssaves",
            "cloud.squick",
            "cloud.ssettings",
            "cloud.sauto",
            "cloud.swifi",
            "bios.b3ds",
            "perf.jit",
            "perf.shcache",
            "adv.advon",
            "adv.inputs",
            "adv.framecnt",
            "online.discord",
        ]
    );
    // Actions and preset cards are carried out by the interface: none is yet.
    for e in ENTRIES.iter().filter(|e| !e.has_value()) {
        assert!(!e.wired, "{}", e.key);
    }
}
