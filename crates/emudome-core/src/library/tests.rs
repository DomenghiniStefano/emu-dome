//! Library tests, on synthetic ROMs built here byte by byte (never real games).

use std::fs;
use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use super::{
    core_of, cover_colour, identify, read_icon, region, scan, settings_game_id, CoverKind, Library,
    Region, ScanInput,
};
use crate::settings::test_dir::TestDir;
use crate::settings::ConsoleId;
use crate::ErrorCode;

// ---- Synthetic ROMs ----

fn put(rom: &mut [u8], at: usize, bytes: &[u8]) {
    rom[at..at + bytes.len()].copy_from_slice(bytes);
}

fn put_utf16(rom: &mut [u8], at: usize, text: &str) {
    for (i, unit) in text.encode_utf16().enumerate() {
        put(rom, at + i * 2, &unit.to_le_bytes());
    }
}

fn sha1_hex(bytes: &[u8]) -> String {
    Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn gba_rom(title: &str, filler: u8) -> Vec<u8> {
    let mut rom = vec![filler; 0x400];
    put(&mut rom, 0xA0, &[0; 0x20]);
    put(&mut rom, 0xA0, title.as_bytes());
    put(&mut rom, 0xAC, b"ZTST");
    rom[0xB2] = 0x96;
    rom[0xBC] = 1;
    rom
}

fn gb_rom(title: &str, cgb_flag: u8) -> Vec<u8> {
    let mut rom = vec![0; 0x8000];
    put(&mut rom, 0x134, title.as_bytes());
    rom[0x143] = cgb_flag;
    let sum = rom[0x134..0x14D]
        .iter()
        .fold(0u8, |x, b| x.wrapping_sub(*b).wrapping_sub(1));
    rom[0x14D] = sum;
    rom
}

const NDS_BANNER: usize = 0x400;
const NDS_USED: usize = NDS_BANNER + 0x840;

/// An NDS ROM with a banner (English title, a 3-colour icon), `NDS_USED` bytes used.
fn nds_rom() -> Vec<u8> {
    let mut rom = vec![0u8; NDS_USED];
    put(&mut rom, 0x00, b"SYNTHETIC");
    put(&mut rom, 0x0C, b"ZSYN");
    rom[0x1E] = 2;
    put(&mut rom, 0x68, &(NDS_BANNER as u32).to_le_bytes());
    put(&mut rom, 0x80, &(NDS_USED as u32).to_le_bytes());
    let crc = nds_crc(&rom[..0x15E]);
    put(&mut rom, 0x15E, &crc.to_le_bytes());
    let b = NDS_BANNER;
    put(&mut rom, b, &1u16.to_le_bytes());
    // Tile 0: pixel (0,0) = colour 1, (1,0) = colour 2, (2,0) = colour 0 (transparent).
    rom[b + 0x20] = 0x21;
    // Tile 1 (x 8..16): pixel (8,0) = colour 2.
    rom[b + 0x20 + 32] = 0x02;
    put(&mut rom, b + 0x220 + 2, &0x001Fu16.to_le_bytes()); // red
    put(&mut rom, b + 0x220 + 4, &0x7C00u16.to_le_bytes()); // blue
    put_utf16(&mut rom, b + 0x240, "Japanese title");
    put_utf16(&mut rom, b + 0x340, "Synthetic Game\nHomebrew");
    rom
}

fn nds_crc(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for b in bytes {
        crc ^= u16::from(*b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xA001
            } else {
                crc >> 1
            };
        }
    }
    crc
}

/// A decrypted NCCH with an extended header (remaster version 2) and an SMDH in the ExeFS.
fn ncch() -> Vec<u8> {
    let exefs = 0x800;
    let smdh = exefs + 0x200;
    let mut n = vec![0u8; smdh + 0x36C0];
    put(&mut n, 0x100, b"NCCH");
    put(&mut n, 0x118, &0x0004_0000_00AB_CD00u64.to_le_bytes());
    put(&mut n, 0x150, b"CTR-P-ZSYN");
    put(&mut n, 0x180, &0x400u32.to_le_bytes());
    n[0x18F] = 0x04; // NoCrypto
    put(&mut n, 0x1A0, &((exefs / 0x200) as u32).to_le_bytes());
    put(&mut n, 0x1A4, &1u32.to_le_bytes());
    put(&mut n, 0x200 + 0x0E, &2u16.to_le_bytes());
    put(&mut n, exefs, b"icon\0\0\0\0");
    put(&mut n, exefs + 12, &0x36C0u32.to_le_bytes());
    put(&mut n, smdh, b"SMDH");
    put_utf16(&mut n, smdh + 0x08 + 0x200, "Tre De Esse");
    let icon = smdh + 0x24C0;
    put(&mut n, icon, &0xF800u16.to_le_bytes()); // (0,0) red
    put(&mut n, icon + 2, &0x07E0u16.to_le_bytes()); // (1,0) green
    put(&mut n, icon + 4, &0x001Fu16.to_le_bytes()); // (0,1) blue
    n
}

fn ncsd_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x200];
    put(&mut rom, 0x100, b"NCSD");
    put(&mut rom, 0x120, &1u32.to_le_bytes()); // partition 0 at media unit 1
    rom.extend(ncch());
    rom
}

fn cia_rom() -> Vec<u8> {
    let content = ncch();
    let mut rom = vec![0u8; 0x2100];
    put(&mut rom, 0x00, &0x2020u32.to_le_bytes());
    put(&mut rom, 0x08, &0x40u32.to_le_bytes());
    put(&mut rom, 0x0C, &0x40u32.to_le_bytes());
    put(&mut rom, 0x10, &0x40u32.to_le_bytes());
    put(&mut rom, 0x18, &(content.len() as u64).to_le_bytes());
    rom.extend(content);
    rom
}

fn write(dir: &TestDir, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

fn pixel(icon: &super::Icon, x: usize, y: usize) -> [u8; 4] {
    let at = (y * icon.width as usize + x) * 4;
    icon.rgba[at..at + 4].try_into().unwrap()
}

// ---- Identity ----

#[test]
fn gba_identity_is_the_file_hash_and_the_title_is_the_file_name() {
    let dir = TestDir::new("lib-gba");
    // A ROM hack keeps its base game's header: the file name is the better name.
    let rom = gba_rom("POKEMON FIRE", 0);
    let id = identify(&write(&dir, "Pokemon Unbound.gba", &rom)).unwrap();
    assert_eq!(id.id, format!("gba:{}", sha1_hex(&rom)));
    assert_eq!(id.console, ConsoleId::Gba);
    assert_eq!(id.title, "Pokemon Unbound");
    assert_eq!(id.code.as_deref(), Some("ZTST"));
    assert_eq!(id.version, Some(1));
    assert!(!id.has_icon);
}

#[test]
fn the_identity_does_not_depend_on_the_file_name() {
    let dir = TestDir::new("lib-name");
    let rom = gba_rom("EMUTEST", 0);
    let a = identify(&write(&dir, "a.gba", &rom)).unwrap();
    let b = identify(&write(&dir, "renamed copy.GBA", &rom)).unwrap();
    assert_eq!(a.id, b.id);
}

#[test]
fn a_patched_rom_is_another_game() {
    let dir = TestDir::new("lib-hack");
    let a = identify(&write(&dir, "a.gba", &gba_rom("EMUTEST", 0))).unwrap();
    let b = identify(&write(&dir, "b.gba", &gba_rom("EMUTEST", 1))).unwrap();
    assert_ne!(a.id, b.id);
}

#[test]
fn a_gba_file_without_the_fixed_value_is_not_a_game() {
    let dir = TestDir::new("lib-gba-bad");
    let mut rom = gba_rom("EMUTEST", 0);
    rom[0xB2] = 0;
    assert!(identify(&write(&dir, "bad.gba", &rom)).is_err());
    assert!(identify(&write(&dir, "short.gba", &[0; 16])).is_err());
}

#[test]
fn an_empty_header_title_falls_back_to_the_file_name() {
    let dir = TestDir::new("lib-gba-untitled");
    let id = identify(&write(&dir, "My Homebrew.gba", &gba_rom("", 0))).unwrap();
    assert_eq!(id.title, "My Homebrew");
}

#[test]
fn gb_and_gbc_identity() {
    let dir = TestDir::new("lib-gb");
    let rom = gb_rom("HELLO", 0);
    let id = identify(&write(&dir, "Hello World.gb", &rom)).unwrap();
    assert_eq!(id.id, format!("gb:{}", sha1_hex(&rom)));
    assert_eq!(id.title, "Hello World");
    // CGB-only cartridge: GBC whatever the extension.
    let id = identify(&write(&dir, "c.gb", &gb_rom("COLOUR", 0xC0))).unwrap();
    assert_eq!(id.console, ConsoleId::Gbc);
    assert!(id.id.starts_with("gbc:"));
    // A broken header checksum is not a game.
    let mut bad = gb_rom("HELLO", 0);
    bad[0x14D] ^= 0xFF;
    assert!(identify(&write(&dir, "bad.gb", &bad)).is_err());
}

#[test]
fn nds_trimmed_and_untrimmed_dumps_are_the_same_game() {
    let dir = TestDir::new("lib-nds");
    let trimmed = nds_rom();
    let mut full = trimmed.clone();
    full.resize(0x4000, 0xFF);
    let a = identify(&write(&dir, "trimmed.nds", &trimmed)).unwrap();
    let b = identify(&write(&dir, "full.nds", &full)).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(a.id, format!("nds:{}", sha1_hex(&trimmed)));
    assert_eq!(a.console, ConsoleId::Nds);
    // English banner title, first line.
    assert_eq!(a.title, "Synthetic Game");
    assert_eq!(a.code.as_deref(), Some("ZSYN"));
    assert_eq!(a.version, Some(2));
    assert!(a.has_icon);
}

#[test]
fn nds_header_crc_is_checked() {
    let dir = TestDir::new("lib-nds-crc");
    let mut rom = nds_rom();
    rom[0x15E] ^= 0xFF;
    assert!(identify(&write(&dir, "bad.nds", &rom)).is_err());
}

#[test]
fn nds_banner_icon_decodes() {
    let dir = TestDir::new("lib-nds-icon");
    let icon = read_icon(&write(&dir, "a.nds", &nds_rom()))
        .unwrap()
        .unwrap();
    assert_eq!((icon.width, icon.height), (32, 32));
    assert_eq!(pixel(&icon, 0, 0), [0xFF, 0, 0, 0xFF]);
    assert_eq!(pixel(&icon, 1, 0), [0, 0, 0xFF, 0xFF]);
    assert_eq!(pixel(&icon, 2, 0), [0, 0, 0, 0]);
    assert_eq!(pixel(&icon, 8, 0), [0, 0, 0xFF, 0xFF]);
    let png = icon.to_png().unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn three_ds_identity_is_the_program_id_and_version() {
    let dir = TestDir::new("lib-3ds");
    let id = identify(&write(&dir, "a.3ds", &ncsd_rom())).unwrap();
    assert_eq!(id.id, "3ds:0004000000abcd000002");
    assert_eq!(id.console, ConsoleId::N3ds);
    assert_eq!(id.title, "Tre De Esse");
    assert_eq!(id.code.as_deref(), Some("CTR-P-ZSYN"));
    assert_eq!(id.version, Some(2));
    assert!(id.has_icon);
    // The same game as a bare NCCH and as a CIA.
    assert_eq!(identify(&write(&dir, "a.cxi", &ncch())).unwrap().id, id.id);
    assert_eq!(
        identify(&write(&dir, "a.cia", &cia_rom())).unwrap().id,
        id.id
    );
}

#[test]
fn smdh_icon_decodes() {
    let dir = TestDir::new("lib-3ds-icon");
    let icon = read_icon(&write(&dir, "a.3ds", &ncsd_rom()))
        .unwrap()
        .unwrap();
    assert_eq!((icon.width, icon.height), (48, 48));
    assert_eq!(pixel(&icon, 0, 0), [0xFF, 0, 0, 0xFF]);
    assert_eq!(pixel(&icon, 1, 0), [0, 0xFF, 0, 0xFF]);
    assert_eq!(pixel(&icon, 0, 1), [0, 0, 0xFF, 0xFF]);
}

#[test]
fn an_encrypted_ncch_still_has_an_id_but_no_title_or_icon() {
    let dir = TestDir::new("lib-3ds-enc");
    let mut rom = ncsd_rom();
    rom[0x200 + 0x18F] = 0;
    let id = identify(&write(&dir, "Encrypted.3ds", &rom)).unwrap();
    assert_eq!(id.id, "3ds:0004000000abcd000000");
    assert_eq!(id.title, "Encrypted");
    assert!(!id.has_icon);
    assert_eq!(read_icon(&dir.path("Encrypted.3ds")).unwrap(), None);
}

// ---- Scan and store ----

fn input(folders: &[&Path], lib: &Library) -> ScanInput {
    ScanInput {
        folders: folders.iter().map(|f| f.to_path_buf()).collect(),
        files: lib.single_files(),
        cache: lib.cache().clone(),
    }
}

fn run(lib: &mut Library, folders: &[&Path], now: u64) -> super::ScanReport {
    let outcome = scan(input(folders, lib), &mut |_, _| {});
    lib.apply_scan(outcome, now).unwrap()
}

#[test]
fn scan_finds_games_and_skips_the_rest() {
    let dir = TestDir::new("lib-scan");
    write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    write(&dir, "roms/sub/b.nds", &nds_rom());
    write(&dir, "roms/broken.gba", &[1, 2, 3]);
    write(&dir, "roms/notes.txt", b"hello");
    let mut lib = Library::open(&dir.path("db"));
    let mut calls = Vec::new();
    let outcome = scan(input(&[&dir.path("roms")], &lib), &mut |c, f| {
        calls.push((c, f));
    });
    let report = lib.apply_scan(outcome, 1000).unwrap();
    assert_eq!(report.found, 2);
    assert_eq!(report.added, 2);
    assert!(report.unreachable.is_empty());
    // Progress after every ROM file (the broken one counts as checked, not found).
    assert_eq!(calls.last(), Some(&(3, 2)));
    let nds = lib.games().find(|g| g.console == ConsoleId::Nds).unwrap();
    assert_eq!(nds.title, "Synthetic Game");
    assert_eq!(nds.cover, CoverKind::Icon);
    assert_eq!(nds.added_at, 1000);
    assert_eq!(
        nds.folder.as_deref(),
        Some(dir.path("roms").to_string_lossy().as_ref())
    );
    let gba = lib.games().find(|g| g.console == ConsoleId::Gba).unwrap();
    assert_eq!(gba.cover, CoverKind::Generated);
    // A second scan adds nothing.
    assert_eq!(run(&mut lib, &[&dir.path("roms")], 2000).added, 0);
    assert_eq!(lib.games().count(), 2);
}

#[test]
fn rescan_uses_the_cache_while_size_and_mtime_hold() {
    let dir = TestDir::new("lib-cache");
    write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    let mut lib = Library::open(&dir.path("db"));
    run(&mut lib, &[&dir.path("roms")], 0);
    // Keyed by the path as the walk spells it.
    let key = lib.games().next().unwrap().path.clone();
    // Tamper with the cached identity: a rescan that trusts the cache keeps it.
    let mut cache = lib.cache().clone();
    cache.get_mut(&key).unwrap().identity.id = "gba:cached".into();
    let outcome = scan(
        ScanInput {
            folders: vec![dir.path("roms")],
            files: Vec::new(),
            cache: cache.clone(),
        },
        &mut |_, _| {},
    );
    assert_eq!(outcome.found[0].identity.id, "gba:cached");
    // A different size: identified again.
    write(
        &dir,
        "roms/a.gba",
        &[gba_rom("ONE", 0), vec![0; 16]].concat(),
    );
    let outcome = scan(
        ScanInput {
            folders: vec![dir.path("roms")],
            files: Vec::new(),
            cache,
        },
        &mut |_, _| {},
    );
    assert!(outcome.found[0].identity.id.starts_with("gba:"));
    assert_ne!(outcome.found[0].identity.id, "gba:cached");
}

#[test]
fn an_unreachable_folder_keeps_its_games() {
    let dir = TestDir::new("lib-unreachable");
    write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    let roms = dir.path("roms");
    let mut lib = Library::open(&dir.path("db"));
    run(&mut lib, &[&roms], 0);
    let id = lib.games().next().unwrap().id.clone();
    lib.add_play_time(&id, 60).unwrap();

    fs::rename(&roms, dir.path("elsewhere")).unwrap();
    let report = run(&mut lib, &[&roms], 0);
    assert_eq!(
        report.unreachable,
        vec![roms.to_string_lossy().into_owned()]
    );
    let game = lib.game(&id).unwrap();
    assert!(!game.available);
    assert_eq!(game.play_time_secs, 60);
    // The cache of the unreachable folder is kept.
    assert_eq!(lib.cache().len(), 1);

    fs::rename(dir.path("elsewhere"), &roms).unwrap();
    let report = run(&mut lib, &[&roms], 0);
    assert_eq!(report.added, 0);
    assert!(lib.game(&id).unwrap().available);
}

#[test]
fn a_deleted_file_is_kept_unavailable() {
    let dir = TestDir::new("lib-deleted");
    let path = write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    let mut lib = Library::open(&dir.path("db"));
    run(&mut lib, &[&dir.path("roms")], 0);
    fs::remove_file(path).unwrap();
    run(&mut lib, &[&dir.path("roms")], 0);
    let game = lib.games().next().unwrap();
    assert!(!game.available);
    assert!(lib.cache().is_empty());
}

#[test]
fn a_removed_folder_leaves_the_library_and_comes_back_with_its_data() {
    let dir = TestDir::new("lib-removed");
    write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    let roms = dir.path("roms");
    let mut lib = Library::open(&dir.path("db"));
    run(&mut lib, &[&roms], 0);
    let id = lib.games().next().unwrap().id.clone();
    lib.set_flags(&id, Some(true), None).unwrap();

    lib.unlist_folder(&roms.to_string_lossy()).unwrap();
    assert_eq!(lib.games().count(), 0);
    assert!(matches!(
        lib.game(&id),
        Err(ErrorCode::LibraryGameUnknown { .. })
    ));
    // Folder gone from the settings without unlist_folder: a scan unlists it too.
    assert_eq!(run(&mut lib, &[&roms], 0).added, 1);
    assert_eq!(run(&mut lib, &[], 0).found, 0);
    assert_eq!(lib.games().count(), 0);

    let report = run(&mut lib, &[&roms], 0);
    assert_eq!(report.added, 1);
    assert!(lib.game(&id).unwrap().favourite);
}

#[test]
fn single_files_are_rechecked_by_every_scan() {
    let dir = TestDir::new("lib-single");
    let path = write(&dir, "loose/a.gba", &gba_rom("ONE", 0));
    let mut lib = Library::open(&dir.path("db"));
    let outcome = scan(
        ScanInput {
            folders: Vec::new(),
            files: vec![path.clone()],
            cache: Default::default(),
        },
        &mut |_, _| {},
    );
    assert_eq!(lib.apply_scan(outcome, 0).unwrap().added, 1);
    assert_eq!(lib.games().next().unwrap().folder, None);
    assert_eq!(run(&mut lib, &[], 0).added, 0);
    assert!(lib.games().next().unwrap().available);
    fs::remove_file(path).unwrap();
    run(&mut lib, &[], 0);
    assert!(!lib.games().next().unwrap().available);
}

#[test]
fn the_store_round_trips_and_keeps_unknown_keys() {
    let dir = TestDir::new("lib-store");
    write(&dir, "roms/a.nds", &nds_rom());
    dir.write("db/library.json", r#"{ "future": [1, 2] }"#);
    let id;
    {
        let mut lib = Library::open(&dir.path("db"));
        run(&mut lib, &[&dir.path("roms")], 5);
        id = lib.games().next().unwrap().id.clone();
        lib.set_flags(&id, Some(true), Some(true)).unwrap();
        lib.mark_played(&id, 77).unwrap();
        lib.add_play_time(&id, 90).unwrap();
        lib.set_cover(&id, CoverKind::User, Some("x.user.png".into()))
            .unwrap();
    }
    let lib = Library::open(&dir.path("db"));
    let game = lib.game(&id).unwrap();
    assert!(game.favourite && game.hidden);
    assert_eq!(game.last_played, Some(77));
    assert_eq!(game.play_time_secs, 90);
    assert_eq!(game.added_at, 5);
    assert_eq!(game.cover, CoverKind::User);
    assert_eq!(game.user_cover.as_deref(), Some("x.user.png"));
    assert_eq!(game.cover_rev, 1);
    assert_eq!(lib.cache().len(), 1);
    let json: serde_json::Value = serde_json::from_str(&dir.read("db/library.json")).unwrap();
    assert_eq!(json["future"], serde_json::json!([1, 2]));
}

#[test]
fn an_unreadable_database_is_moved_aside() {
    let dir = TestDir::new("lib-unreadable");
    dir.write("db/library.json", "{ nope");
    let lib = Library::open(&dir.path("db"));
    assert_eq!(lib.games().count(), 0);
    assert_eq!(dir.read("db/library.json.unreadable"), "{ nope");
}

#[test]
fn covers_are_checked() {
    let dir = TestDir::new("lib-covers");
    write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    let mut lib = Library::open(&dir.path("db"));
    run(&mut lib, &[&dir.path("roms")], 0);
    let id = lib.games().next().unwrap().id.clone();
    assert!(matches!(
        lib.set_cover(&id, CoverKind::Icon, None),
        Err(ErrorCode::CoverInvalid { .. })
    ));
    assert!(matches!(
        lib.set_cover("gba:nope", CoverKind::Generated, None),
        Err(ErrorCode::LibraryGameUnknown { .. })
    ));
}

#[test]
fn cover_colour_is_stable_and_a_colour() {
    let a = cover_colour("gba:0123");
    assert_eq!(a, cover_colour("gba:0123"));
    assert_eq!(a.len(), 7);
    assert!(a.starts_with('#'));
    assert!(a[1..].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn library_ids_make_valid_settings_ids() {
    let id = settings_game_id("3ds:0004000000abcd000002").unwrap();
    assert_eq!(id.as_str(), "3ds-0004000000abcd000002");
}

#[test]
fn library_errors_serialize_as_the_contract() {
    let json =
        serde_json::to_value(ErrorCode::LibraryGameUnavailable { id: "gba:1".into() }).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "code": "libraryGameUnavailable", "params": { "id": "gba:1" } })
    );
    let json = serde_json::to_value(ErrorCode::LibraryFolderUnreachable {
        path: "D:/x".into(),
    })
    .unwrap();
    assert_eq!(json["code"], "libraryFolderUnreachable");
    assert_eq!(json["params"]["path"], "D:/x");
}

#[test]
fn the_shown_name_survives_rescans_and_blank_goes_back_to_the_rom_title() {
    let dir = TestDir::new("lib-name");
    write(&dir, "roms/a.gba", &gba_rom("ONE", 0));
    let roms = dir.path("roms");
    let mut lib = Library::open(&dir.path("db"));
    run(&mut lib, &[&roms], 0);
    let id = lib.games().next().unwrap().id.clone();
    let rom_title = lib.game(&id).unwrap().title.clone();

    lib.set_name(&id, "  Cronache di Brina  ").unwrap();
    assert_eq!(lib.game(&id).unwrap().shown_title(), "Cronache di Brina");
    run(&mut lib, &[&roms], 0);
    assert_eq!(lib.game(&id).unwrap().shown_title(), "Cronache di Brina");
    assert_eq!(lib.game(&id).unwrap().title, rom_title);
    // Kept in the file.
    let reopened = Library::open(&dir.path("db"));
    assert_eq!(
        reopened.game(&id).unwrap().shown_title(),
        "Cronache di Brina"
    );

    // The ROM's own title, or nothing, is no name of the user's.
    lib.set_name(&id, &rom_title).unwrap();
    assert_eq!(lib.game(&id).unwrap().name, None);
    lib.set_name(&id, "x").unwrap();
    lib.set_name(&id, "   ").unwrap();
    assert_eq!(lib.game(&id).unwrap().name, None);
    assert_eq!(lib.game(&id).unwrap().shown_title(), rom_title);

    // Too long: cut.
    lib.set_name(&id, &"a".repeat(200)).unwrap();
    assert_eq!(
        lib.game(&id).unwrap().shown_title().chars().count(),
        super::NAME_MAX
    );
    assert!(matches!(
        lib.set_name("gba:none", "x"),
        Err(ErrorCode::LibraryGameUnknown { .. })
    ));
}

#[test]
fn the_region_is_the_last_letter_of_the_game_code() {
    assert_eq!(region(Some("AXVE")), Some(Region::Usa));
    assert_eq!(region(Some("ABCP")), Some(Region::Europe));
    assert_eq!(region(Some("ABCJ")), Some(Region::Japan));
    assert_eq!(region(Some("ABCI")), Some(Region::Italy));
    assert_eq!(region(Some("CTR-P-ABCD")), Some(Region::Germany));
    assert_eq!(region(Some("abcf")), Some(Region::France));
    assert_eq!(region(Some("0000")), None);
    assert_eq!(region(Some("")), None);
    assert_eq!(region(None), None);
}

#[test]
fn each_console_names_its_core() {
    assert_eq!(core_of(ConsoleId::Gb), "mgba");
    assert_eq!(core_of(ConsoleId::Gbc), "mgba");
    assert_eq!(core_of(ConsoleId::Gba), "mgba");
    assert_eq!(core_of(ConsoleId::Nds), "melondsds");
    assert_eq!(core_of(ConsoleId::N3ds), "azahar");
}
