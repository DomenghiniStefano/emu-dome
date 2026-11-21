//! The sync on the in-memory Drive: two devices, every way a run can end, and the data on both
//! sides after each (engineering rule 1).

use std::fs;

use emudome_core::GameKey;

use super::{excluded, library_snapshot, Area, Device, Direction, Keep, Syncer};
use crate::decide::Decision;
use crate::drive::fake::FakeDrive;
use crate::drive::{md5_hex, DriveApi, Properties};
use crate::index::SyncIndex;
use crate::marker::InUse;
use crate::test_http::TestDir;
use crate::SyncError;

/// One device: its data folder (saves, settings, the sync index) and its name.
struct Phone {
    dir: TestDir,
    device: Device,
    index: SyncIndex,
}

impl Phone {
    fn new(name: &str) -> Self {
        let dir = TestDir::new(&format!("sync-{name}"));
        let index = SyncIndex::open(&dir.path("sync"));
        Self {
            dir,
            device: Device {
                id: format!("id-{name}"),
                name: name.to_owned(),
            },
            index,
        }
    }

    fn game(&self) -> Area {
        let mut props = Properties::new();
        props.insert("playTime".into(), "3600".into());
        Area::game(&self.dir.path("saves"), &game(), props)
    }

    fn save(&self, rel: &str, bytes: &[u8]) {
        self.dir.write(&format!("saves/gba/1a2b/{rel}"), bytes);
    }

    fn read(&self, rel: &str) -> Option<Vec<u8>> {
        fs::read(self.dir.path(&format!("saves/gba/1a2b/{rel}"))).ok()
    }

    fn syncer<'a>(&'a mut self, drive: &'a FakeDrive) -> Syncer<'a, FakeDrive> {
        Syncer::new(drive, &mut self.index, self.device.clone())
    }
}

fn game() -> GameKey {
    GameKey::parse("gba:1a2b").unwrap()
}

fn drive_content(drive: &FakeDrive, path: &[&str]) -> Option<Vec<u8>> {
    drive.at(path).and_then(|f| drive.content(&f.id))
}

#[test]
fn backups_conflict_copies_and_temporary_files_never_sync() {
    for rel in [
        "backups/1700000000000.manual.sav",
        "conflicts/1-pc/game.sav",
        "broken-1700000000000.sav",
        ".game.sav.123-4.emudome-tmp",
        "sub/.x.json.1-1.emudome-tmp",
        "library.json.unreadable",
    ] {
        assert!(excluded(rel), "{rel}");
    }
    for rel in [
        "game.sav",
        "slot1.state",
        "melonDS DS/dldi_sd_card.bin",
        "backup.sav",
    ] {
        assert!(!excluded(rel), "{rel}");
    }
}

#[tokio::test]
async fn a_first_push_mirrors_the_game_folder_and_another_device_pulls_it() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    pc.save("game.sav", b"sav-1");
    pc.save("slot1.state", b"state-1");
    pc.save("melonDS DS/dldi_sd_card.bin", b"sd");
    pc.save("backups/1.manual.sav", b"old");
    pc.save("broken-1.sav", b"broken");
    pc.save(".game.sav.1-1.emudome-tmp", b"tmp");
    let area = pc.game();
    let out = pc.syncer(&drive).push_after_exit(&area).await.unwrap();
    assert_eq!(
        out.pushed,
        vec![
            "gba/1a2b/game.sav",
            "gba/1a2b/melonDS DS/dldi_sd_card.bin",
            "gba/1a2b/slot1.state"
        ]
    );
    let paths: Vec<String> = drive.tree().into_iter().map(|(p, _)| p).collect();
    assert_eq!(
        paths,
        vec![
            "EmuDome",
            "EmuDome/gba",
            "EmuDome/gba/1a2b",
            "EmuDome/gba/1a2b/game.sav",
            "EmuDome/gba/1a2b/melonDS DS",
            "EmuDome/gba/1a2b/melonDS DS/dldi_sd_card.bin",
            "EmuDome/gba/1a2b/slot1.state",
        ]
    );
    let sav = drive.at(&["EmuDome", "gba", "1a2b", "game.sav"]).unwrap();
    assert_eq!(sav.app_properties["game"], "gba:1a2b");
    assert_eq!(sav.app_properties["device"], "id-pc");
    assert_eq!(sav.app_properties["deviceName"], "pc");
    assert_eq!(sav.app_properties["playTime"], "3600");
    let entry = pc.index.get("gba/1a2b/game.sav").unwrap();
    assert_eq!(entry.md5, md5_hex(b"sav-1"));
    assert_eq!(entry.id, sav.id);

    let mut phone = Phone::new("phone");
    let area = phone.game();
    let out = phone
        .syncer(&drive)
        .pull_before_launch(&area)
        .await
        .unwrap();
    assert_eq!(out.pulled.len(), 3);
    assert_eq!(phone.read("game.sav").unwrap(), b"sav-1");
    assert_eq!(phone.read("melonDS DS/dldi_sd_card.bin").unwrap(), b"sd");
    assert_eq!(phone.read("backups/1.manual.sav"), None);
    // Nothing left to do on either side.
    for device in [&mut pc, &mut phone] {
        let area = device.game();
        let steps = device.syncer(&drive).plan(&area).await.unwrap();
        assert!(steps.iter().all(|s| s.decision == Decision::InSync));
    }
}

#[tokio::test]
async fn played_on_one_device_then_the_other_each_gets_the_newer_save() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    let mut phone = Phone::new("phone");
    pc.save("game.sav", b"v1");
    let (a, b) = (pc.game(), phone.game());
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();
    phone.syncer(&drive).pull_before_launch(&b).await.unwrap();

    phone.save("game.sav", b"v2-phone");
    let out = phone.syncer(&drive).push_after_exit(&b).await.unwrap();
    assert_eq!(out.pushed, vec!["gba/1a2b/game.sav"]);
    // A pull run leaves this device's changes for later, a push run Drive's.
    pc.save("slot2.state", b"pc-state");
    let out = pc.syncer(&drive).pull_before_launch(&a).await.unwrap();
    assert_eq!(out.pulled, vec!["gba/1a2b/game.sav"]);
    assert_eq!(out.waiting, vec!["gba/1a2b/slot2.state"]);
    assert_eq!(pc.read("game.sav").unwrap(), b"v2-phone");
    assert!(drive
        .at(&["EmuDome", "gba", "1a2b", "slot2.state"])
        .is_none());
}

#[tokio::test]
async fn played_on_both_is_asked_and_both_saves_survive_either_choice() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    let mut phone = Phone::new("phone");
    pc.save("game.sav", b"base");
    let (a, b) = (pc.game(), phone.game());
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();
    phone.syncer(&drive).pull_before_launch(&b).await.unwrap();

    // Both play offline from the same base.
    pc.save("game.sav", b"pc-progress");
    phone.save("game.sav", b"phone-progress");
    phone.syncer(&drive).push_after_exit(&b).await.unwrap();

    // The PC finds a conflict: nothing moves, either way.
    for direction in [Direction::Pull, Direction::Push, Direction::Both] {
        let out = pc.syncer(&drive).run(&a, direction).await.unwrap();
        assert!(out.pulled.is_empty() && out.pushed.is_empty());
        assert_eq!(out.conflicts.len(), 1);
        let c = &out.conflicts[0];
        assert_eq!(c.path, "gba/1a2b/game.sav");
        assert_eq!(c.remote_device.as_deref(), Some("phone"));
        assert_eq!(c.remote_play_time, Some(3600));
        assert_eq!(c.local_size, 11);
        assert_eq!(c.remote_size, 14);
    }
    assert_eq!(pc.read("game.sav").unwrap(), b"pc-progress");
    assert_eq!(
        drive_content(&drive, &["EmuDome", "gba", "1a2b", "game.sav"]).unwrap(),
        b"phone-progress"
    );

    // The PC keeps its own: Drive's copy (the phone's) stays on Drive as a copy.
    assert!(pc
        .syncer(&drive)
        .resolve(&a, "game.sav", Keep::Local, 5)
        .await
        .unwrap());
    assert_eq!(
        drive_content(&drive, &["EmuDome", "gba", "1a2b", "game.sav"]).unwrap(),
        b"pc-progress"
    );
    assert_eq!(
        drive_content(
            &drive,
            &["EmuDome", "gba", "1a2b", "conflicts", "5-phone", "game.sav"]
        )
        .unwrap(),
        b"phone-progress"
    );
    assert_eq!(pc.read("game.sav").unwrap(), b"pc-progress");
    // Settled: a second resolve finds nothing to do.
    assert!(!pc
        .syncer(&drive)
        .resolve(&a, "game.sav", Keep::Local, 6)
        .await
        .unwrap());

    // The phone's own save is the one Drive had (now kept as the copy): it simply gets the
    // PC's.
    let out = phone.syncer(&drive).run(&b, Direction::Pull).await.unwrap();
    assert_eq!(out.pulled, vec!["gba/1a2b/game.sav"]);
    assert_eq!(phone.read("game.sav").unwrap(), b"pc-progress");

    // Played on both again; this time the PC keeps Drive's (the phone's): its own stays on the
    // PC as a copy.
    pc.save("game.sav", b"pc-2");
    phone.save("game.sav", b"phone-2");
    phone.syncer(&drive).push_after_exit(&b).await.unwrap();
    assert!(pc
        .syncer(&drive)
        .resolve(&a, "game.sav", Keep::Remote, 7)
        .await
        .unwrap());
    assert_eq!(pc.read("game.sav").unwrap(), b"phone-2");
    assert_eq!(pc.read("conflicts/7-pc/game.sav").unwrap(), b"pc-2");
    // The kept copies never sync, and both devices are settled.
    for device in [&mut pc, &mut phone] {
        let area = device.game();
        let out = device
            .syncer(&drive)
            .run(&area, Direction::Both)
            .await
            .unwrap();
        assert!(out.pulled.is_empty() && out.pushed.is_empty() && out.conflicts.is_empty());
    }
    assert!(drive
        .at(&["EmuDome", "gba", "1a2b", "conflicts", "7-pc"])
        .is_none());
}

/// Every version of every file, local or on Drive, is one of `allowed`.
fn assert_whole(bytes: Option<Vec<u8>>, allowed: &[&[u8]]) {
    let bytes = bytes.expect("the file is there");
    assert!(
        allowed.contains(&bytes.as_slice()),
        "a file that is neither version: {bytes:?}"
    );
}

#[tokio::test]
async fn a_pull_cut_off_at_any_call_leaves_whole_files_and_finishes_later() {
    let names = ["game.sav", "slot1.state", "slot2.state", "slot3.state"];
    for cut in 0..14 {
        let drive = FakeDrive::new();
        let mut pc = Phone::new("pc");
        let mut phone = Phone::new("phone");
        for n in names {
            pc.save(n, format!("old-{n}").as_bytes());
        }
        let (a, b) = (pc.game(), phone.game());
        pc.syncer(&drive).push_after_exit(&a).await.unwrap();
        phone.syncer(&drive).pull_before_launch(&b).await.unwrap();
        for n in names {
            pc.save(n, format!("new-{n}").as_bytes());
        }
        pc.syncer(&drive).push_after_exit(&a).await.unwrap();

        drive.offline_after(Some(cut));
        let result = phone.syncer(&drive).pull_before_launch(&b).await;
        for n in names {
            let (old, new) = (format!("old-{n}"), format!("new-{n}"));
            assert_whole(phone.read(n), &[old.as_bytes(), new.as_bytes()]);
            // The index says synced only what was.
            let entry = phone.index.get(&format!("gba/1a2b/{n}")).unwrap();
            assert_eq!(entry.md5, md5_hex(&phone.read(n).unwrap()), "cut {cut}");
        }
        if result.is_ok() {
            break;
        }
        assert!(matches!(result, Err(SyncError::Offline(_))));
        drive.offline_after(None);
        let out = phone.syncer(&drive).pull_before_launch(&b).await.unwrap();
        assert!(
            out.conflicts.is_empty(),
            "cut {cut}: an interruption is no conflict"
        );
        for n in names {
            assert_eq!(phone.read(n).unwrap(), format!("new-{n}").as_bytes());
        }
    }
}

#[tokio::test]
async fn a_push_cut_off_at_any_call_leaves_drive_whole_and_finishes_later() {
    let names = ["game.sav", "slot1.state", "melonDS DS/sd.bin"];
    for cut in 0..20 {
        let drive = FakeDrive::new();
        let mut pc = Phone::new("pc");
        for n in names {
            pc.save(n, format!("old-{n}").as_bytes());
        }
        let a = pc.game();
        pc.syncer(&drive).push_after_exit(&a).await.unwrap();
        for n in names {
            pc.save(n, format!("new-{n}").as_bytes());
        }
        pc.save("slot9.state", b"brand new");

        drive.offline_after(Some(cut));
        let result = pc.syncer(&drive).push_after_exit(&a).await;
        for n in names {
            let path: Vec<&str> = ["EmuDome", "gba", "1a2b"]
                .into_iter()
                .chain(n.split('/'))
                .collect();
            let (old, new) = (format!("old-{n}"), format!("new-{n}"));
            assert_whole(
                drive_content(&drive, &path),
                &[old.as_bytes(), new.as_bytes()],
            );
            assert_eq!(
                pc.read(n).unwrap(),
                new.as_bytes(),
                "the local file is never touched"
            );
        }
        if result.is_ok() {
            break;
        }
        drive.offline_after(None);
        let out = pc.syncer(&drive).push_after_exit(&a).await.unwrap();
        assert!(
            out.conflicts.is_empty(),
            "cut {cut}: an interruption is no conflict"
        );
        for n in names {
            let path: Vec<&str> = ["EmuDome", "gba", "1a2b"]
                .into_iter()
                .chain(n.split('/'))
                .collect();
            assert_eq!(
                drive_content(&drive, &path).unwrap(),
                format!("new-{n}").as_bytes()
            );
        }
        let created = drive
            .tree()
            .into_iter()
            .filter(|(p, _)| p.ends_with("slot9.state"))
            .count();
        assert_eq!(created, 1, "cut {cut}: a file is never uploaded twice");
    }
}

#[tokio::test]
async fn a_full_drive_refuses_the_push_and_changes_nothing() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    pc.save("game.sav", b"12345");
    let a = pc.game();
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();
    drive.set_limit(Some(6));
    pc.save("game.sav", b"1234567890");
    let e = pc.syncer(&drive).push_after_exit(&a).await.unwrap_err();
    assert!(
        matches!(e, SyncError::QuotaFull { needed: 10, .. }),
        "{e:?}"
    );
    assert_eq!(
        drive_content(&drive, &["EmuDome", "gba", "1a2b", "game.sav"]).unwrap(),
        b"12345"
    );
    assert_eq!(
        pc.index.get("gba/1a2b/game.sav").unwrap().md5,
        md5_hex(b"12345")
    );
}

#[tokio::test]
async fn a_file_changed_between_plan_and_transfer_waits_for_the_next_sync() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    let mut phone = Phone::new("phone");
    pc.save("game.sav", b"v1");
    let (a, b) = (pc.game(), phone.game());
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();
    phone.syncer(&drive).pull_before_launch(&b).await.unwrap();
    pc.save("game.sav", b"v2");
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();

    // The phone plans a pull, then its own game writes the save before the transfer.
    let steps = phone.syncer(&drive).plan(&b).await.unwrap();
    assert_eq!(steps[0].decision, Decision::Pull);
    phone.save("game.sav", b"phone-wrote");
    assert!(!phone.syncer(&drive).pull(&b, &steps[0]).await.unwrap());
    assert_eq!(phone.read("game.sav").unwrap(), b"phone-wrote");

    // The PC plans a push, then Drive's copy changes before the upload.
    pc.save("game.sav", b"v3");
    let steps = pc.syncer(&drive).plan(&a).await.unwrap();
    assert_eq!(steps[0].decision, Decision::Push);
    let id = drive
        .at(&["EmuDome", "gba", "1a2b", "game.sav"])
        .unwrap()
        .id;
    drive.overwrite(&id, b"from-elsewhere");
    assert!(!pc.syncer(&drive).push(&a, &steps[0]).await.unwrap());
    assert_eq!(drive.content(&id).unwrap(), b"from-elsewhere");
    // Next sync: both changed, so it asks.
    let out = pc.syncer(&drive).run(&a, Direction::Both).await.unwrap();
    assert_eq!(out.conflicts.len(), 1);
}

#[tokio::test]
async fn deleted_on_one_side_comes_back_from_the_other() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    pc.save("game.sav", b"v1");
    pc.save("slot1.state", b"s1");
    let a = pc.game();
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();
    fs::remove_file(pc.dir.path("saves/gba/1a2b/slot1.state")).unwrap();
    let id = drive
        .at(&["EmuDome", "gba", "1a2b", "game.sav"])
        .unwrap()
        .id;
    drive.remove(&id);
    let out = pc.syncer(&drive).run(&a, Direction::Both).await.unwrap();
    assert_eq!(out.pulled, vec!["gba/1a2b/slot1.state"]);
    assert_eq!(out.pushed, vec!["gba/1a2b/game.sav"]);
    assert_eq!(pc.read("slot1.state").unwrap(), b"s1");
    assert_eq!(
        drive_content(&drive, &["EmuDome", "gba", "1a2b", "game.sav"]).unwrap(),
        b"v1"
    );
}

#[tokio::test]
async fn the_in_use_marker_tells_the_other_device_and_goes_at_exit() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    let mut phone = Phone::new("phone");
    let (a, b) = (pc.game(), phone.game());
    assert_eq!(
        phone.syncer(&drive).in_use(&b, 0).await.unwrap(),
        InUse::Free
    );
    pc.syncer(&drive).claim(&a, 1_000).await.unwrap();
    pc.syncer(&drive).claim(&a, 121_000).await.unwrap();
    assert_eq!(
        pc.syncer(&drive).in_use(&a, 121_000).await.unwrap(),
        InUse::Here
    );
    assert_eq!(
        phone.syncer(&drive).in_use(&b, 130_000).await.unwrap(),
        InUse::Elsewhere {
            name: "pc".into(),
            since: 1_000,
            beat: 121_000
        }
    );
    // The phone cannot take the PC's marker away; it can still play (it is only told).
    phone.syncer(&drive).release(&b).await.unwrap();
    assert!(matches!(
        phone.syncer(&drive).in_use(&b, 130_000).await.unwrap(),
        InUse::Elsewhere { .. }
    ));
    pc.save("game.sav", b"x");
    pc.syncer(&drive).push_after_exit(&a).await.unwrap();
    assert_eq!(
        phone.syncer(&drive).in_use(&b, 130_000).await.unwrap(),
        InUse::Free
    );
}

#[tokio::test]
async fn settings_sync_without_the_per_device_values() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    pc.dir.write("settings/global.json", b"{\"$schema\":1}");
    pc.dir.write("settings/console/gba.json", b"{}");
    pc.dir.write("settings/game/gba-1a2b.json", b"{}");
    pc.dir
        .write("local-settings/device.json", b"{\"per\":\"device\"}");
    let area = Area::settings(&pc.dir.path("settings"));
    let out = pc.syncer(&drive).run(&area, Direction::Push).await.unwrap();
    assert_eq!(
        out.pushed,
        vec![
            "settings/console/gba.json",
            "settings/game/gba-1a2b.json",
            "settings/global.json"
        ]
    );
    assert!(drive.tree().iter().all(|(p, _)| !p.contains("device.json")));
}

#[tokio::test]
async fn the_library_list_says_what_the_device_has_never_where() {
    let drive = FakeDrive::new();
    let mut pc = Phone::new("pc");
    let record: emudome_core::library::GameRecord = serde_json::from_value(serde_json::json!({
        "id": "gba:1a2b", "console": "gba", "title": "HEADER", "name": "My Game",
        "path": "D:/secret/roms/My Game.gba", "folder": "D:/secret/roms", "size": 1, "mtime": 1,
        "available": true, "listed": true, "playTimeSecs": 61, "lastPlayed": 9, "addedAt": 1,
        "cover": "generated", "userCover": null, "coverRev": 0, "hasIcon": false,
        "favourite": false, "hidden": false, "code": null, "version": null
    }))
    .unwrap();
    let bytes = library_snapshot(&pc.device, [&record], 42);
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(!text.contains("secret") && !text.contains(".gba"), "{text}");
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        json["games"][0],
        serde_json::json!({ "id": "gba:1a2b", "title": "My Game", "playTimeSecs": 61, "lastPlayed": 9 })
    );
    let folder = vec!["library".to_owned()];
    let name = format!("{}.json", pc.device.id);
    let first = pc
        .syncer(&drive)
        .publish(&folder, &name, bytes.clone())
        .await
        .unwrap();
    let again = pc
        .syncer(&drive)
        .publish(&folder, &name, b"{}".to_vec())
        .await
        .unwrap();
    assert_eq!(first.id, again.id, "the device's own file, replaced");
    assert_eq!(drive.content(&first.id).unwrap(), b"{}");
    assert_eq!(
        drive
            .list(&drive.at(&["EmuDome", "library"]).unwrap().id)
            .await
            .unwrap()
            .len(),
        1
    );
}
