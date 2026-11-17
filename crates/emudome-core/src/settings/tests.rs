//! Engine tests: precedence, refused levels, device axis, session, schema invariants.
//! Option ids are read from the schema, so the tests do not depend on the catalogue's wording.

use std::collections::HashSet;

use serde_json::json;

use super::catalog::{CURRENT_PHASE, ENTRIES, GROUPS, PAGES, SECTIONS};
use super::test_dir::TestDir;
use super::{entry, CommitTarget, ConsoleId, GameId, Kind, Level, Scope, Settings, Value};
use crate::settings::schema::Device;
use crate::ErrorCode;

fn settings(dir: &TestDir) -> Settings {
    Settings::open(
        dir.path("synced"),
        dir.path("local/device.json"),
        Device::Desktop,
    )
}

fn game() -> GameId {
    GameId::from_rom_path("C:/games/test.gba".as_ref())
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
        &format!("synced/game/{}.json", game()),
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
            level: Level::Game
        })
    );
    assert_eq!(
        s.set("video.fps", &Scope::Session, Value::Bool(true)),
        Err(ErrorCode::SettingLevelRefused {
            key: "video.fps".into(),
            level: Level::Session
        })
    );
    // Console and game only.
    assert_eq!(
        s.reset("video.orient", &Scope::Global),
        Err(ErrorCode::SettingLevelRefused {
            key: "video.orient".into(),
            level: Level::Global
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
    assert_eq!(device["game"][game().as_str()][key], json!(false));
    // And it reads back from a fresh start.
    let mut fresh = settings(&dir);
    assert_eq!(
        value_of(&mut fresh, key, &game_scope()),
        (Some(Value::Bool(false)), Level::Game)
    );
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
        game: GameId::from_rom_path("C:/games/other.gba".as_ref()),
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
            level: Level::Console
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
    s.begin_session(
        ConsoleId::Gba,
        GameId::from_rom_path("C:/other.gba".as_ref()),
    );
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

// ---- Identity and wire format ----

#[test]
fn game_ids_are_stable_and_safe() {
    // FNV-1a 64 of "abc".
    assert_eq!(super::fnv1a(b"abc"), 0xe71f_a219_0541_574b);
    assert_eq!(
        GameId::from_rom_path("abc".as_ref()).as_str(),
        "path-e71fa2190541574b"
    );
    assert!(GameId::try_from("path-0123abcd".to_owned()).is_ok());
    for bad in ["", "../global", "a/b", "A", "x y"] {
        assert!(GameId::try_from(bad.to_owned()).is_err(), "{bad:?}");
    }
}

#[test]
fn scopes_and_errors_have_the_interface_shape() {
    let scope: Scope =
        serde_json::from_value(json!({ "level": "game", "console": "gba", "game": "path-1" }))
            .unwrap();
    assert_eq!(
        scope,
        Scope::Game {
            console: ConsoleId::Gba,
            game: GameId::try_from("path-1".to_owned()).unwrap()
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
            level: Level::Game
        })
        .unwrap(),
        json!({ "code": "settingLevelRefused", "params": { "key": "video.fps", "level": "game" } })
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
fn the_catalogue_has_the_157_entries_of_the_design() {
    assert_eq!(ENTRIES.len(), 157);
}

#[test]
fn keys_are_unique_and_belong_to_a_page() {
    let mut seen = HashSet::new();
    for e in ENTRIES {
        assert!(seen.insert(e.key), "duplicate key {}", e.key);
        let (page, key) = e.key.split_once('.').expect("key is <page>.<key>");
        assert!(PAGES.contains(&page), "{}: unknown page", e.key);
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
            | Kind::Action { .. } => continue,
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
