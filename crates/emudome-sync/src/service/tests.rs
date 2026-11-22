//! [`super::CloudService`] on [`FakeDrive`]: the game flow, the pending pushes and the games on
//! Drive, with the data surviving every interruption.

use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use emudome_core::library::GameRecord;
use emudome_core::GameKey;
use parking_lot::Mutex;

use super::{Choice, CloudService, Flags, GameRef, GameState, Notice, Paths, Phase, Prefs};
use crate::drive::fake::FakeDrive;
use crate::index::SyncIndex;
use crate::marker::InUse;
use crate::sync::{Area, Device, QuickSync, Syncer};
use crate::test_http::TestDir;
use crate::SyncError;

/// One device: its folders, its service on the shared Drive, and what it told the interface.
struct Peer {
    dir: TestDir,
    notices: Arc<Mutex<Vec<Notice>>>,
}

impl Peer {
    fn new(name: &str) -> Self {
        Self {
            dir: TestDir::new(&format!("service-{name}")),
            notices: Arc::default(),
        }
    }

    /// The service of one run (a new one = the app started again on the same folders).
    fn service(&self, drive: &FakeDrive, name: &str) -> CloudService<FakeDrive> {
        let notices = self.notices.clone();
        CloudService::new(
            drive.clone(),
            Paths {
                saves_root: self.dir.path("saves"),
                settings: self.dir.path("settings"),
                sync: self.dir.path("sync"),
                games: self.dir.path("games"),
            },
            name.to_owned(),
            Arc::new(move |n| notices.lock().push(n)),
        )
    }

    fn save(&self, rel: &str, bytes: &[u8]) {
        self.dir.write(&format!("saves/gba/1a2b/{rel}"), bytes);
    }

    fn read(&self, rel: &str) -> Option<Vec<u8>> {
        fs::read(self.dir.path(&format!("saves/gba/1a2b/{rel}"))).ok()
    }
}

fn key() -> GameKey {
    GameKey::parse("gba:1a2b").unwrap()
}

fn game() -> GameRef {
    GameRef {
        key: key(),
        title: "My Game".into(),
        play_time: Some(60),
    }
}

fn record(path: &std::path::Path) -> GameRecord {
    serde_json::from_value(serde_json::json!({
        "id": "gba:1a2b", "console": "gba", "title": "MY GAME", "name": "My Game",
        "path": path.to_string_lossy(), "folder": null, "size": fs::metadata(path).map_or(0, |m| m.len()),
        "mtime": 1, "available": true, "listed": true, "playTimeSecs": 60, "lastPlayed": 9,
        "addedAt": 1, "cover": "generated", "userCover": null, "coverRev": 0, "hasIcon": false,
        "favourite": false, "hidden": false, "code": null, "version": null
    }))
    .unwrap()
}

const ON: Flags = Flags {
    configured: true,
    supported: true,
    signed_in: true,
};

#[tokio::test]
async fn played_here_then_there_the_pull_before_launch_brings_the_newer_save() {
    let drive = FakeDrive::new();
    let pc = Peer::new("pull-pc");
    let deck = Peer::new("pull-deck");
    let prefs = Prefs::default();
    pc.save("game.sav", b"pc-1");
    let s = pc.service(&drive, "PC");
    s.mark_pending(&key()).unwrap();
    assert_eq!(s.status(ON).pending, vec![key()]);
    let out = s.push_game(&prefs, &game()).await.unwrap();
    assert_eq!(out.pushed, vec!["gba/1a2b/game.sav".to_owned()]);
    assert!(s.status(ON).pending.is_empty(), "pushed: no longer pending");

    let d = deck.service(&drive, "Deck");
    let out = d.pull_game(&prefs, &game()).await.unwrap();
    assert_eq!(out.pulled.len(), 1);
    assert_eq!(deck.read("game.sav").unwrap(), b"pc-1");
    let phases: Vec<Phase> = deck
        .notices
        .lock()
        .iter()
        .filter_map(|n| match n {
            Notice::Progress(p) => Some(p.phase),
            Notice::Status | Notice::Conflict(_) | Notice::InUse(_) => None,
        })
        .collect();
    assert_eq!(phases.first(), Some(&Phase::Pulling));
    assert_eq!(phases.last(), Some(&Phase::Done));
    assert_eq!(d.activity()[0].kind, super::ActivityKind::Pulled);

    // `manual`: nothing moves by itself.
    pc.save("game.sav", b"pc-2");
    let manual = Prefs {
        auto: false,
        ..prefs
    };
    assert!(d
        .pull_game(&manual, &game())
        .await
        .unwrap()
        .pulled
        .is_empty());
    assert_eq!(deck.read("game.sav").unwrap(), b"pc-1");
}

#[tokio::test]
async fn changed_on_both_is_kept_for_g04_and_every_answer_keeps_both_versions() {
    let drive = FakeDrive::new();
    let pc = Peer::new("conflict-pc");
    let deck = Peer::new("conflict-deck");
    let prefs = Prefs::default();
    pc.save("game.sav", b"base");
    let s = pc.service(&drive, "PC");
    s.push_game(&prefs, &game()).await.unwrap();
    let d = deck.service(&drive, "Deck");
    d.pull_game(&prefs, &game()).await.unwrap();
    // Both play.
    pc.save("game.sav", b"pc");
    s.push_game(&prefs, &game()).await.unwrap();
    deck.save("game.sav", b"deck");
    let out = d.pull_game(&prefs, &game()).await.unwrap();
    assert_eq!(out.conflicts.len(), 1);
    assert_eq!(
        deck.read("game.sav").unwrap(),
        b"deck",
        "nothing overwritten"
    );
    assert_eq!(d.status(ON).conflicts, vec![key()]);
    let shown = deck
        .notices
        .lock()
        .iter()
        .find_map(|n| match n {
            Notice::Conflict(c) => Some(c.clone()),
            Notice::Status | Notice::Progress(_) | Notice::InUse(_) => None,
        })
        .unwrap();
    assert_eq!(shown.title, "My Game");
    assert_eq!(shown.remote.device.as_deref(), Some("PC"));
    assert_eq!(shown.local.device.as_deref(), Some("Deck"));
    assert_eq!(d.conflict(&game()).unwrap().files.len(), 1);

    // The conflict list survives a restart.
    let d = deck.service(&drive, "Deck");
    assert_eq!(d.status(ON).conflicts, vec![key()]);

    // «Tieni entrambe»: this device's goes on, Drive's kept on both sides.
    d.resolve(&prefs, &game(), Choice::Both).await.unwrap();
    assert!(d.status(ON).conflicts.is_empty());
    assert!(d.conflict(&game()).is_none());
    assert_eq!(deck.read("game.sav").unwrap(), b"deck");
    let kept: Vec<Vec<u8>> = walk(&deck.dir.path("saves/gba/1a2b/conflicts"));
    assert_eq!(kept, vec![b"pc".to_vec()], "Drive's version kept here");
    let on_drive: Vec<Vec<u8>> = drive
        .tree()
        .into_iter()
        .filter(|(p, f)| p.contains("/conflicts/") && !f.folder)
        .map(|(_, f)| drive.content(&f.id).unwrap())
        .collect();
    assert_eq!(on_drive, vec![b"pc".to_vec()], "and on Drive");
    let sav = drive.at(&["EmuDome", "gba", "1a2b", "game.sav"]).unwrap();
    assert_eq!(drive.content(&sav.id).unwrap(), b"deck");
}

fn walk(dir: &std::path::Path) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for e in entries.flatten() {
        if e.path().is_dir() {
            out.extend(walk(&e.path()));
        } else {
            out.push(fs::read(e.path()).unwrap());
        }
    }
    out
}

#[tokio::test]
async fn a_push_cut_off_at_any_call_stays_pending_and_the_next_start_finishes_it() {
    // Data survival for the new exit path (engineering rule 1): the game closed, its saves are
    // on the disk; the push after it is cut off at every possible call (no network, the app
    // quitting). The local files never change, Drive never holds half a file, and the next
    // start pushes what is left.
    for cut in 0..12 {
        let drive = FakeDrive::new();
        let pc = Peer::new(&format!("cut-{cut}"));
        let prefs = Prefs::default();
        pc.save("game.sav", b"after the session");
        pc.save("auto.state", &[7u8; 64]);
        let s = pc.service(&drive, "PC");
        s.mark_pending(&key()).unwrap();
        drive.offline_after(Some(cut));
        let first = s.push_game(&prefs, &game()).await;
        drive.offline_after(None);
        assert_eq!(pc.read("game.sav").unwrap(), b"after the session");
        assert_eq!(pc.read("auto.state").unwrap(), vec![7u8; 64]);
        for (path, f) in drive.tree() {
            if !f.folder {
                let content = drive.content(&f.id).unwrap();
                assert!(
                    content == b"after the session" || content == vec![7u8; 64],
                    "{path}: half a file on Drive"
                );
            }
        }
        if first.is_ok() {
            assert!(s.status(ON).pending.is_empty());
            continue;
        }
        assert!(matches!(first, Err(SyncError::Offline(_))));
        assert_eq!(
            s.status(ON).pending,
            vec![key()],
            "cut {cut}: still pending"
        );
        assert!(s.status(ON).error.is_some());
        // The app starts again.
        let again = pc.service(&drive, "PC");
        assert_eq!(again.status(ON).pending, vec![key()]);
        assert_eq!(
            again.push_pending(&prefs, &[game()], None).await.unwrap(),
            1
        );
        assert!(again.status(ON).pending.is_empty());
        let sav = drive.at(&["EmuDome", "gba", "1a2b", "game.sav"]).unwrap();
        assert_eq!(drive.content(&sav.id).unwrap(), b"after the session");
        assert!(drive
            .at(&["EmuDome", "gba", "1a2b", "auto.state"])
            .is_some());
    }
}

#[tokio::test]
async fn the_open_game_is_not_pushed_at_start_and_a_skipped_pull_moves_nothing() {
    let drive = FakeDrive::new();
    let pc = Peer::new("skip-pc");
    let deck = Peer::new("skip-deck");
    let prefs = Prefs::default();
    pc.save("game.sav", b"pc");
    let s = pc.service(&drive, "PC");
    s.mark_pending(&key()).unwrap();
    assert_eq!(s.push_pending(&prefs, &[], Some(&key())).await.unwrap(), 0);
    assert_eq!(s.status(ON).pending, vec![key()]);
    s.push_pending(&prefs, &[], None).await.unwrap();

    // A cancelled run moves nothing and leaves the index true: the next one moves it all.
    let mut index = SyncIndex::open(&deck.dir.path("sync"));
    let device = Device {
        id: "deck".into(),
        name: "Deck".into(),
    };
    let area = Area::game(&deck.dir.path("saves"), &key(), Default::default());
    let cancel = AtomicBool::new(true);
    let out = Syncer::new(&drive, &mut index, device.clone())
        .with_cancel(&cancel)
        .pull_before_launch(&area)
        .await
        .unwrap();
    assert!(out.pulled.is_empty());
    assert_eq!(out.waiting, vec!["gba/1a2b/game.sav".to_owned()]);
    assert!(deck.read("game.sav").is_none());
    let out = Syncer::new(&drive, &mut index, device)
        .pull_before_launch(&area)
        .await
        .unwrap();
    assert_eq!(out.pulled.len(), 1);
    // No pull running: nothing to skip.
    assert!(!deck.service(&drive, "Deck").skip_pull(&key()));
}

#[tokio::test]
async fn quick_saves_follow_the_setting() {
    let drive = FakeDrive::new();
    let pc = Peer::new("quick-pc");
    pc.save("game.sav", b"sav");
    pc.save("slot1.state", b"old");
    pc.save("slot1.png", b"old-png");
    pc.save("auto.state", b"auto");
    let s = pc.service(&drive, "PC");
    let none = Prefs {
        quick: QuickSync::None,
        ..Prefs::default()
    };
    s.push_game(&none, &game()).await.unwrap();
    assert!(drive.at(&["EmuDome", "gba", "1a2b", "game.sav"]).is_some());
    assert!(drive
        .at(&["EmuDome", "gba", "1a2b", "slot1.state"])
        .is_none());
    assert!(drive
        .at(&["EmuDome", "gba", "1a2b", "auto.state"])
        .is_none());
    // `last`: the newest quick save only (auto.state written last, a second later).
    std::thread::sleep(std::time::Duration::from_millis(1100));
    pc.save("auto.state", b"auto-2");
    let last = Prefs {
        quick: QuickSync::Last,
        ..Prefs::default()
    };
    s.push_game(&last, &game()).await.unwrap();
    assert!(drive
        .at(&["EmuDome", "gba", "1a2b", "auto.state"])
        .is_some());
    assert!(drive
        .at(&["EmuDome", "gba", "1a2b", "slot1.state"])
        .is_none());
    assert!(drive.at(&["EmuDome", "gba", "1a2b", "slot1.png"]).is_none());
    // Saves off: nothing moves, and the game is no longer waiting.
    let off = Prefs {
        saves: false,
        ..Prefs::default()
    };
    s.mark_pending(&key()).unwrap();
    s.push_game(&off, &game()).await.unwrap();
    assert!(s.status(ON).pending.is_empty());
}

#[tokio::test]
async fn a_game_file_goes_up_comes_down_elsewhere_and_leaves_drive_only_when_asked() {
    let drive = FakeDrive::new();
    let pc = Peer::new("rom-pc");
    let deck = Peer::new("rom-deck");
    pc.dir.write("roms/My Game.gba", &[3u8; 4096]);
    pc.save("game.sav", b"sav");
    let rec = record(&pc.dir.path("roms/My Game.gba"));
    let s = pc.service(&drive, "PC");

    // Too big for the space left: refused before anything is sent.
    drive.set_limit(Some(1000));
    let full = s.upload_game(&rec).await;
    assert!(matches!(
        full,
        Err(SyncError::QuotaFull { needed: 4096, .. })
    ));
    assert!(drive.at(&["EmuDome", "games", "gba", "1a2b.gba"]).is_none());
    assert!(matches!(
        s.status(ON).error,
        Some(emudome_core::ErrorCode::CloudQuotaFull { .. })
    ));
    drive.set_limit(None);

    let rows = s.games(std::slice::from_ref(&rec)).await.unwrap();
    assert_eq!(rows[0].state, GameState::LocalOnly);
    s.upload_game(&rec).await.unwrap();
    let up = drive.at(&["EmuDome", "games", "gba", "1a2b.gba"]).unwrap();
    assert_eq!(up.app_properties["title"], "My Game");
    s.upload_game(&rec).await.unwrap();
    assert_eq!(
        drive
            .tree()
            .iter()
            .filter(|(p, _)| p.ends_with("1a2b.gba"))
            .count(),
        1,
        "the same key is not sent twice"
    );
    s.push_game(&Prefs::default(), &game()).await.unwrap();
    s.sync_all(&Prefs::default(), std::slice::from_ref(&rec), None)
        .await
        .unwrap();
    let rows = s.games(std::slice::from_ref(&rec)).await.unwrap();
    assert_eq!(rows[0].state, GameState::Both);
    assert!(rows[0].saves_on_drive);
    assert!(s.status(ON).emudome_bytes.unwrap() >= 4096);

    // Another device: only on Drive, with its size and the device that has it.
    let d = deck.service(&drive, "Deck");
    let rows = d.games(&[]).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].state, GameState::DriveOnly);
    assert_eq!(rows[0].size, 4096);
    assert_eq!(rows[0].title, "My Game");
    assert_eq!(rows[0].devices, vec!["PC".to_owned()]);
    // A file already there under that name is never written over.
    deck.dir.write("games/gba/My Game.gba", b"mine");
    let path = d.download_game(&key()).await.unwrap();
    assert_eq!(path, deck.dir.path("games/gba/My Game (2).gba"));
    assert_eq!(fs::read(&path).unwrap(), vec![3u8; 4096]);
    assert_eq!(
        fs::read(deck.dir.path("games/gba/My Game.gba")).unwrap(),
        b"mine"
    );
    let left: Vec<_> = fs::read_dir(deck.dir.path("games/gba"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        left.iter().all(|n| !n.ends_with(".emudome-tmp")),
        "{left:?}"
    );

    let devices = d.devices().await.unwrap();
    assert_eq!(devices[0].name, "Deck");
    assert!(devices[0].this_device);
    assert!(devices.iter().any(|x| x.name == "PC" && x.games == 1));

    // «Rimuovi da Drive»: the game file to the trash; its saves and every local file stay.
    s.remove_game(&key()).await.unwrap();
    assert!(drive.at(&["EmuDome", "games", "gba", "1a2b.gba"]).is_none());
    assert_eq!(drive.trashed().len(), 1);
    assert!(drive.at(&["EmuDome", "gba", "1a2b", "game.sav"]).is_some());
    assert!(pc.dir.path("roms/My Game.gba").is_file());
    let rows = s.games(std::slice::from_ref(&rec)).await.unwrap();
    assert_eq!(rows[0].state, GameState::LocalOnly);
}

#[tokio::test]
async fn the_marker_is_claimed_told_and_taken_off_by_the_push() {
    let drive = FakeDrive::new();
    let pc = Peer::new("marker-pc");
    let deck = Peer::new("marker-deck");
    let prefs = Prefs::default();
    let s = pc.service(&drive, "PC");
    s.claim(&prefs, &key(), true).await.unwrap();
    let d = deck.service(&drive, "Deck");
    assert!(matches!(
        d.launch_check(&key()).await.unwrap(),
        InUse::Elsewhere { ref name, .. } if name == "PC"
    ));
    // Opened anyway (no G05 first): told on `cloud://in-use`.
    d.claim(&prefs, &key(), true).await.unwrap();
    assert!(deck
        .notices
        .lock()
        .iter()
        .any(|n| matches!(n, Notice::InUse(e) if e.id == key())));
    d.push_game(&prefs, &game()).await.unwrap();
    assert_eq!(s.launch_check(&key()).await.unwrap(), InUse::Free);
    // A heartbeat stopped before it starts claims nothing.
    let stop = AtomicBool::new(true);
    s.heartbeat(&prefs, &key(), &stop).await;
    assert_eq!(d.launch_check(&key()).await.unwrap(), InUse::Free);
}

#[tokio::test]
async fn an_unreadable_state_file_is_never_written_over() {
    let drive = FakeDrive::new();
    let pc = Peer::new("state-unreadable");
    // A folder where the file should be: there, but not readable as a file.
    fs::create_dir_all(pc.dir.path("sync/state.json")).unwrap();
    let s = pc.service(&drive, "PC");
    assert!(matches!(
        s.mark_pending(&key()),
        Err(SyncError::IndexReadFailed(_))
    ));
    assert!(pc.dir.path("sync/state.json").is_dir());
    // Not valid JSON: set aside, started empty.
    let other = Peer::new("state-invalid");
    other.dir.write("sync/state.json", b"{ nope");
    let s = other.service(&drive, "PC");
    s.mark_pending(&key()).unwrap();
    assert!(other.dir.path("sync/state.json.unreadable").is_file());
    assert_eq!(s.status(ON).pending, vec![key()]);
    assert!(s.status(Flags::default()).account.is_none());
}
