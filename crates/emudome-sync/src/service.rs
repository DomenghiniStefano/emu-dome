//! The sync as the app runs it (Trello #31): one [`CloudService`] per run, over a [`DriveApi`],
//! doing what the Cloud screens and the game flow ask, and telling the interface through a
//! [`Notify`] closure. No Tauri here: the app (`app/src-tauri/src/cloud.rs`) hands in the
//! paths, the settings ([`Prefs`]), the library's games and the closure; the tests hand in
//! [`crate::drive::fake::FakeDrive`].
//!
//! - **Before a game starts** ([`CloudService::pull_game`]): Drive's newer saves come down,
//!   one file at a time; «Gioca senza aspettare» ([`CloudService::skip_pull`]) stops the pull
//!   before its next file (the file already moving finishes, so nothing is half written).
//!   Conflicts are listed, never touched: the game starts with this device's files.
//! - **While it runs**: [`CloudService::claim`] writes this device's «in use» marker, again
//!   every [`crate::marker::HEARTBEAT`].
//! - **After it ended** (its exit writes done): [`CloudService::mark_pending`] records the game
//!   in `sync/state.json` first, then [`CloudService::push_game`] sends its changes and takes
//!   the marker off; only a push that went through takes the game off the list, so a push cut
//!   off (the app quitting, no network) is done at the next start ([`CloudService::push_pending`]).
//! - **On request**: «Sincronizza ora» ([`CloudService::sync_all`]: the settings, every game's
//!   saves both ways, the library list), the games on Drive ([`CloudService::games`],
//!   upload / download / remove of a game file), the devices, the conflicts and their answers.
//!
//! Data (engineering rule 1): a sync never deletes a local file; a conflict keeps both sides;
//! every file is written with `write_atomic` (a downloaded game through a temporary file,
//! checked against Drive's MD5, then renamed); `sync/state.json` and the index are never
//! written over while they cannot be read. One identity: every game is its [`GameKey`].
//!
//! The activity list («Attività recenti») lives in memory for the run.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use emudome_core::library::GameRecord;
use emudome_core::settings::ConsoleId;
use emudome_core::{ErrorCode, GameKey, IoCause};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use ts_rs::TS;

use crate::drive::{DriveApi, Properties, Quota, RemoteFile, ROOT};
use crate::index::SyncIndex;
use crate::marker::{InUse, HEARTBEAT};
use crate::sync::{
    excluded, library_snapshot, Area, Conflict, Device, Direction, Keep, LibraryGame, Outcome,
    QuickSync, Syncer, GAMES_FOLDER, LIBRARY_FOLDER, ROOT_FOLDER,
};
use crate::time::now_ms;
use crate::SyncError;

/// How many entries «Attività recenti» keeps.
pub const ACTIVITY_KEPT: usize = 50;
/// The layout version written in `sync/state.json`.
const STATE_SCHEMA: u64 = 1;

// ---- What the interface sees ----

/// Who is signed in. TypeScript: `CloudAccount`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudAccount")]
pub struct AccountDto {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// The Cloud section's whole state, sent again on `cloud://status` at every change.
/// TypeScript: `CloudStatus`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudStatus")]
pub struct CloudStatus {
    /// This build has the Google OAuth client (otherwise sync does not exist in it).
    pub configured: bool,
    /// This system can sign in (the desktop; Android in Phase 4).
    pub supported: bool,
    /// A sign-in is kept on this device.
    pub signed_in: bool,
    /// The account, once Drive answered since the sign-in or the start.
    pub account: Option<AccountDto>,
    /// The account's Drive space, as last measured.
    pub quota: Option<Quota>,
    /// Bytes of EmuDome's own files on Drive (saves, settings, games, lists), as last measured:
    /// the «EmuDome» part of the space bar (the rest of `quota.usage` is «altri file»).
    pub emudome_bytes: Option<u64>,
    /// A sync, a pull, a push, an upload or a download is running.
    pub syncing: bool,
    /// The end of the last sync that went through (Unix ms).
    pub last_sync: Option<u64>,
    /// The games with two versions of a save (G04), as the last sync of each found them.
    pub conflicts: Vec<GameKey>,
    /// The games whose changes have not reached Drive yet (pushed at the next sync or start).
    pub pending: Vec<GameKey>,
    /// The last failure (offline E07, access expired E08, Drive full E09, …), cleared by the
    /// next sync that goes through.
    pub error: Option<ErrorCode>,
}

/// Where a game is, for the list «Giochi su Google Drive». TypeScript: `CloudGameState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudGameState")]
pub enum GameState {
    /// The game file is here and on Drive («Su entrambi»).
    Both,
    /// Only on Drive («Solo su Drive»): «Scarica».
    DriveOnly,
    /// Only here («Solo qui»): «Carica».
    LocalOnly,
    /// A save of it has two versions («Due versioni»): G04.
    Conflict,
}

/// One game of «Giochi su Google Drive». TypeScript: `CloudGame`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudGame")]
pub struct CloudGame {
    pub id: GameKey,
    pub title: String,
    pub console: ConsoleId,
    /// The game file's size in bytes (this device's, else Drive's): shown before a download.
    pub size: u64,
    pub state: GameState,
    /// The game file is in this device's library.
    pub here: bool,
    /// The game file is on Drive (`EmuDome/games/`).
    pub on_drive: bool,
    /// Its saves folder on Drive has files.
    pub saves_on_drive: bool,
    /// The other devices whose library list has it (their names).
    pub devices: Vec<String>,
}

/// One device that published its library list. TypeScript: `CloudDevice`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudDevice")]
pub struct DeviceDto {
    pub id: String,
    pub name: String,
    /// When it last published its list (Unix ms).
    pub updated: u64,
    /// How many games its library has.
    pub games: u32,
    pub this_device: bool,
}

/// What an entry of «Attività recenti» says happened. TypeScript: `CloudActivityKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudActivityKind")]
pub enum ActivityKind {
    /// Saves came down from Drive (`files` of them).
    Pulled,
    /// Saves went up to Drive.
    Pushed,
    /// The settings were synced.
    Settings,
    /// A save was found changed on both sides.
    Conflict,
    /// A conflict was answered.
    Resolved,
    /// A game file went up («Carica»).
    Uploaded,
    /// A game file came down («Scarica»).
    Downloaded,
    /// A game file was moved to Drive's trash («Rimuovi da Drive»).
    Removed,
    /// Something failed (`error` says what).
    Failed,
}

/// One entry of «Attività recenti». TypeScript: `CloudActivity`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudActivity")]
pub struct Activity {
    /// Unix ms.
    pub at: u64,
    pub kind: ActivityKind,
    /// The game, when it was about one.
    pub game: Option<GameKey>,
    /// Files moved (pulled / pushed / settings), else 0.
    pub files: u32,
    pub error: Option<ErrorCode>,
}

/// One side of a game's two versions (G04). TypeScript: `CloudVersion`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudVersion")]
pub struct Version {
    /// The device's name (this one's, or the one that last wrote Drive's copy, if it said).
    pub device: Option<String>,
    /// The newest of its files in conflict (Unix ms).
    pub modified: u64,
    /// The game's play time on that device then, seconds, if known.
    pub play_time: Option<u64>,
    /// Bytes of its files in conflict.
    pub size: u64,
    /// A picture of it: none yet (an in-game save has no thumbnail; reserved for the quick
    /// save's `.png` when one is in conflict).
    pub thumb: Option<String>,
}

/// A game's two versions, for G04. TypeScript: `CloudGameConflict`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudGameConflict")]
pub struct GameConflict {
    pub id: GameKey,
    pub title: String,
    /// Every file of the game changed on both sides.
    pub files: Vec<Conflict>,
    pub local: Version,
    pub remote: Version,
}

/// The answer to G04. TypeScript: `CloudChoice`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudChoice")]
pub enum Choice {
    /// Go on with this device's version; Drive's stays as a copy under `conflicts/` on Drive.
    Local,
    /// Go on with Drive's version; this device's stays as a copy under `conflicts/` here.
    Remote,
    /// «Tieni entrambe»: this device's goes on, and Drive's is kept on both sides.
    Both,
}

/// What a transfer is doing. TypeScript: `CloudPhase`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudPhase")]
pub enum Phase {
    /// Before a game starts (G03).
    Pulling,
    /// After a game closed (G14).
    Pushing,
    /// «Sincronizza ora», or the sync at start.
    Syncing,
    /// A game file going up.
    Uploading,
    /// A game file coming down.
    Downloading,
    /// The transfer ended (whatever its result: `cloud://status` says if it failed).
    Done,
}

/// `cloud://progress`. TypeScript: `CloudProgress`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudProgress")]
pub struct Progress {
    pub phase: Phase,
    pub game: Option<GameKey>,
    /// Bytes moved so far (0 until the first file ended for saves; a game file counts as it
    /// streams down).
    pub done: u64,
    /// Bytes to move in all (0: nothing to move).
    pub total: u64,
}

/// `cloud://in-use`: a game being opened is open on another device (G05). TypeScript:
/// `CloudInUseEvent`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudInUseEvent")]
pub struct InUseEvent {
    pub id: GameKey,
    pub in_use: InUse,
}

/// What the service tells the interface (the app turns each into its event).
#[derive(Clone, Debug)]
pub enum Notice {
    /// The status changed: send `cloud://status` again.
    Status,
    Progress(Progress),
    /// A game needs G04 (`cloud://conflict`).
    Conflict(GameConflict),
    InUse(InUseEvent),
}

/// The closure the notices go to.
pub type Notify = Arc<dyn Fn(Notice) + Send + Sync>;

// ---- What the app hands in ----

/// The `cloud.*` settings a run follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prefs {
    /// `cloud.ssaves`: the games' saves sync.
    pub saves: bool,
    /// `cloud.squick`: which quick saves.
    pub quick: QuickSync,
    /// `cloud.ssettings`: the portable settings sync.
    pub settings: bool,
    /// `cloud.sauto` = `game`: saves sync by themselves before a game starts and after it
    /// closes (`manual`: only «Sincronizza ora»).
    pub auto: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            saves: true,
            quick: QuickSync::All,
            settings: true,
            auto: true,
        }
    }
}

/// The folders the service works in.
#[derive(Clone, Debug)]
pub struct Paths {
    /// `<app data>/saves`: one folder per game (`<console>/<hex>/`).
    pub saves_root: PathBuf,
    /// `<app data>/settings`: the portable settings (never `local-settings/`).
    pub settings: PathBuf,
    /// `<app data>/sync`: the index and `state.json`.
    pub sync: PathBuf,
    /// Where a downloaded game file goes (`<app local data>/games`, one folder per console).
    pub games: PathBuf,
}

/// A game as the interface names it, for the texts of a notice.
#[derive(Clone, Debug)]
pub struct GameRef {
    pub key: GameKey,
    pub title: String,
    /// This device's play time, seconds.
    pub play_time: Option<u64>,
}

// ---- What is kept: sync/state.json ----

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StateData {
    #[serde(rename = "$schema", default)]
    schema: u64,
    #[serde(default)]
    pending: BTreeSet<GameKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_sync: Option<u64>,
    #[serde(default)]
    conflicts: BTreeMap<GameKey, Vec<Conflict>>,
    #[serde(flatten)]
    other: Map<String, Json>,
}

/// `sync/state.json`: the pending pushes, the last sync, the conflicts found. Same rules as the
/// index: not valid JSON → moved aside and started empty (a pending push then waits for the
/// next «Sincronizza ora», which pushes every change all the same); there but unreadable →
/// never written over, every change refused.
struct StateFile {
    path: PathBuf,
    data: StateData,
    unreadable: Option<(IoCause, String)>,
}

impl StateFile {
    fn open(dir: &Path) -> Self {
        let path = dir.join("state.json");
        let (data, unreadable) = Self::read(&path);
        Self {
            path,
            data,
            unreadable,
        }
    }

    fn read(path: &Path) -> (StateData, Option<(IoCause, String)>) {
        let failed = |e: &std::io::Error| Some((IoCause::of(e), e.to_string()));
        match std::fs::read(path) {
            Ok(bytes) => match serde_json::from_slice::<StateData>(&bytes) {
                Ok(data) => (data, None),
                Err(e) => {
                    log::warn!("cloud: {} is not valid ({e}): set aside", path.display());
                    let aside = path.with_extension("json.unreadable");
                    if let Err(e) = std::fs::rename(path, &aside) {
                        return (StateData::default(), failed(&e));
                    }
                    (StateData::default(), None)
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (StateData::default(), None),
            Err(e) => (StateData::default(), failed(&e)),
        }
    }

    fn change(&mut self, f: impl FnOnce(&mut StateData)) -> Result<(), SyncError> {
        if self.unreadable.is_some() {
            let (data, unreadable) = Self::read(&self.path);
            if let Some((cause, e)) = unreadable {
                self.unreadable = Some((cause, e.clone()));
                return Err(SyncError::IndexReadFailed(
                    cause,
                    format!("{}: {e}", self.path.display()),
                ));
            }
            self.data = data;
            self.unreadable = None;
        }
        f(&mut self.data);
        self.data.schema = STATE_SCHEMA;
        let mut bytes = serde_json::to_vec_pretty(&self.data).map_err(SyncError::local_other)?;
        bytes.push(b'\n');
        emudome_core::write_atomic(&self.path, &bytes).map_err(|e| SyncError::local(&self.path, &e))
    }
}

/// What lives only for the run.
#[derive(Default)]
struct Live {
    account: Option<AccountDto>,
    quota: Option<Quota>,
    emudome_bytes: Option<u64>,
    /// Runs in progress.
    busy: u32,
    error: Option<ErrorCode>,
    activity: VecDeque<Activity>,
}

/// Lowers `busy` when a run ends, however it ends.
struct Busy<'a>(&'a Mutex<Live>);

impl Drop for Busy<'_> {
    fn drop(&mut self) {
        let mut live = self.0.lock();
        live.busy = live.busy.saturating_sub(1);
    }
}

/// The sync service of this run.
pub struct CloudService<D> {
    drive: D,
    paths: Paths,
    device_name: String,
    notify: Notify,
    /// Every run holds it: one run at a time, in the order asked.
    index: tokio::sync::Mutex<SyncIndex>,
    state: Mutex<StateFile>,
    live: Mutex<Live>,
    /// The pulls before launch that «Gioca senza aspettare» can stop, by game.
    skips: Mutex<HashMap<GameKey, Arc<AtomicBool>>>,
}

/// The account flags only the app knows (build, system, credential store).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub configured: bool,
    pub supported: bool,
    pub signed_in: bool,
}

impl<D: DriveApi> CloudService<D> {
    pub fn new(drive: D, paths: Paths, device_name: String, notify: Notify) -> Self {
        Self {
            index: tokio::sync::Mutex::new(SyncIndex::open(&paths.sync)),
            state: Mutex::new(StateFile::open(&paths.sync)),
            drive,
            paths,
            device_name,
            notify,
            live: Mutex::new(Live::default()),
            skips: Mutex::new(HashMap::new()),
        }
    }

    pub fn drive(&self) -> &D {
        &self.drive
    }

    /// The status, with the account flags the app knows.
    pub fn status(&self, flags: Flags) -> CloudStatus {
        let live = self.live.lock();
        let state = self.state.lock();
        let signed = flags.configured && flags.supported && flags.signed_in;
        CloudStatus {
            configured: flags.configured,
            supported: flags.supported,
            signed_in: signed,
            account: live.account.clone().filter(|_| signed),
            quota: live.quota.filter(|_| signed),
            emudome_bytes: live.emudome_bytes.filter(|_| signed),
            syncing: live.busy > 0,
            last_sync: state.data.last_sync,
            conflicts: state.data.conflicts.keys().cloned().collect(),
            pending: state.data.pending.iter().cloned().collect(),
            error: live.error.clone(),
        }
    }

    /// «Attività recenti», newest first.
    pub fn activity(&self) -> Vec<Activity> {
        self.live.lock().activity.iter().cloned().collect()
    }

    /// Forgets what belonged to the account (signed out): its name, space and last failure.
    /// The files and the index stay: signing in again goes on from them.
    pub fn signed_out(&self) {
        let mut live = self.live.lock();
        live.account = None;
        live.quota = None;
        live.emudome_bytes = None;
        live.error = None;
        drop(live);
        (self.notify)(Notice::Status);
    }

    fn begin(&self) -> Busy<'_> {
        self.live.lock().busy += 1;
        (self.notify)(Notice::Status);
        Busy(&self.live)
    }

    fn log(
        &self,
        kind: ActivityKind,
        game: Option<&GameKey>,
        files: usize,
        error: Option<ErrorCode>,
    ) {
        let mut live = self.live.lock();
        live.activity.push_front(Activity {
            at: now_ms(),
            kind,
            game: game.cloned(),
            files: u32::try_from(files).unwrap_or(u32::MAX),
            error,
        });
        live.activity.truncate(ACTIVITY_KEPT);
    }

    /// Records the end of a run: its failure (kept for the status and the activity), or a
    /// success (clears the last failure).
    fn finish<T>(
        &self,
        game: Option<&GameKey>,
        result: Result<T, SyncError>,
    ) -> Result<T, SyncError> {
        match &result {
            Ok(_) => self.live.lock().error = None,
            Err(e) => {
                let code = ErrorCode::from(e.clone());
                log::warn!("cloud: {e}");
                self.live.lock().error = Some(code.clone());
                self.log(ActivityKind::Failed, game, 0, Some(code));
            }
        }
        (self.notify)(Notice::Status);
        result
    }

    fn progress(&self, phase: Phase, game: Option<&GameKey>, done: u64, total: u64) {
        (self.notify)(Notice::Progress(Progress {
            phase,
            game: game.cloned(),
            done,
            total,
        }));
    }

    /// The index, waited for; `None` once `cancel` is set while waiting (another run holds it).
    async fn lock_index(
        &self,
        cancel: Option<&AtomicBool>,
    ) -> Option<tokio::sync::MutexGuard<'_, SyncIndex>> {
        loop {
            if cancel.is_some_and(|c| c.load(Ordering::SeqCst)) {
                return None;
            }
            if let Ok(guard) =
                tokio::time::timeout(Duration::from_millis(50), self.index.lock()).await
            {
                return Some(guard);
            }
        }
    }

    fn device(&self, index: &mut SyncIndex) -> Result<Device, SyncError> {
        Ok(Device {
            id: index.device_id()?,
            name: self.device_name.clone(),
        })
    }

    fn game_area(&self, game: &GameKey, prefs: &Prefs, play_time: Option<u64>) -> Area {
        let mut props = Properties::new();
        if let Some(t) = play_time {
            props.insert("playTime".into(), t.to_string());
        }
        Area::game(&self.paths.saves_root, game, props).with_quick(prefs.quick)
    }

    /// Keeps what a run found about `game`'s conflicts, and tells the interface of new ones.
    fn record_conflicts(&self, game: &GameRef, conflicts: Vec<Conflict>) -> Result<(), SyncError> {
        let before = self.state.lock().data.conflicts.get(&game.key).cloned();
        if before.as_ref() == Some(&conflicts) || (before.is_none() && conflicts.is_empty()) {
            return Ok(());
        }
        let found = !conflicts.is_empty();
        let shown = conflicts.clone();
        self.state.lock().change(|d| {
            if conflicts.is_empty() {
                d.conflicts.remove(&game.key);
            } else {
                d.conflicts.insert(game.key.clone(), conflicts);
            }
        })?;
        if found {
            self.log(ActivityKind::Conflict, Some(&game.key), shown.len(), None);
            (self.notify)(Notice::Conflict(game_conflict(
                game,
                shown,
                &self.device_name,
            )));
        }
        Ok(())
    }

    // ---- The account ----

    /// Asks Drive for the account and its space (after a sign-in, at start, at a sync).
    pub async fn refresh_account(&self) -> Result<(), SyncError> {
        let account = self.drive.account().await;
        let account = self.finish(None, account)?;
        let quota = self.drive.quota().await;
        let quota = self.finish(None, quota)?;
        let mut live = self.live.lock();
        live.account = Some(AccountDto {
            name: account.name,
            email: account.email,
        });
        live.quota = Some(quota);
        drop(live);
        (self.notify)(Notice::Status);
        Ok(())
    }

    // ---- The game flow ----

    /// The flag «Gioca senza aspettare» sets for `game`'s next pull.
    fn skip_flag(&self, game: &GameKey) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.skips.lock().insert(game.clone(), flag.clone());
        flag
    }

    /// «Gioca senza aspettare»: the pull before `game` starts stops before its next file, or
    /// at once while it still waits for another run. False: no pull of it is running.
    pub fn skip_pull(&self, game: &GameKey) -> bool {
        match self.skips.lock().get(game) {
            Some(flag) => {
                flag.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    /// Whether `game` is open on another device (G05): asked before it opens.
    pub async fn launch_check(&self, game: &GameKey) -> Result<InUse, SyncError> {
        let Some(mut index) = self.lock_index(None).await else {
            return Ok(InUse::Free);
        };
        let device = self.device(&mut index)?;
        let area = Area::game(&self.paths.saves_root, game, Properties::new());
        let result = Syncer::new(&self.drive, &mut index, device)
            .in_use(&area, now_ms())
            .await;
        drop(index);
        self.finish(Some(game), result)
    }

    /// Before `game` starts: Drive's newer saves come here (G03, progress on
    /// `cloud://progress`), unless the settings say otherwise. Conflicts are kept for G04 and
    /// the game starts with this device's files.
    pub async fn pull_game(&self, prefs: &Prefs, game: &GameRef) -> Result<Outcome, SyncError> {
        if !(prefs.saves && prefs.auto) {
            return Ok(Outcome::default());
        }
        let skip = self.skip_flag(&game.key);
        let _busy = self.begin();
        self.progress(Phase::Pulling, Some(&game.key), 0, 0);
        let result = self.pull_inner(prefs, game, &skip).await;
        self.skips.lock().remove(&game.key);
        self.progress(Phase::Done, Some(&game.key), 0, 0);
        if let Ok(out) = &result {
            if !out.pulled.is_empty() {
                self.log(
                    ActivityKind::Pulled,
                    Some(&game.key),
                    out.pulled.len(),
                    None,
                );
            }
        }
        self.finish(Some(&game.key), result)
    }

    async fn pull_inner(
        &self,
        prefs: &Prefs,
        game: &GameRef,
        skip: &AtomicBool,
    ) -> Result<Outcome, SyncError> {
        let Some(mut index) = self.lock_index(Some(skip)).await else {
            return Ok(Outcome::default());
        };
        let device = self.device(&mut index)?;
        let area = self.game_area(&game.key, prefs, None);
        let key = game.key.clone();
        let progress =
            move |done: u64, total: u64| self.progress(Phase::Pulling, Some(&key), done, total);
        let out = Syncer::new(&self.drive, &mut index, device)
            .with_cancel(skip)
            .with_progress(&progress)
            .pull_before_launch(&area)
            .await?;
        drop(index);
        self.record_conflicts(game, out.conflicts.clone())?;
        Ok(out)
    }

    /// While `game` runs: this device's «in use» marker, written at the start and again every
    /// heartbeat. At the start (`first`), a marker of another device is told on `cloud://in-use`
    /// (the game was opened without the G05 question: startup, «Open with»).
    pub async fn claim(&self, prefs: &Prefs, game: &GameKey, first: bool) -> Result<(), SyncError> {
        if !prefs.saves {
            return Ok(());
        }
        let Some(mut index) = self.lock_index(None).await else {
            return Ok(());
        };
        let device = self.device(&mut index)?;
        let area = Area::game(&self.paths.saves_root, game, Properties::new());
        let now = now_ms();
        let mut syncer = Syncer::new(&self.drive, &mut index, device);
        if first {
            if let in_use @ InUse::Elsewhere { .. } = syncer.in_use(&area, now).await? {
                (self.notify)(Notice::InUse(InUseEvent {
                    id: game.clone(),
                    in_use,
                }));
            }
        }
        syncer.claim(&area, now).await
    }

    /// While `game` runs: claims its marker now, then again every [`HEARTBEAT`], until `stop` is
    /// set (looked at every second). A failed beat is logged and tried at the next one.
    pub async fn heartbeat(&self, prefs: &Prefs, game: &GameKey, stop: &AtomicBool) {
        let mut first = true;
        while !stop.load(Ordering::SeqCst) {
            if let Err(e) = self.claim(prefs, game, first).await {
                log::info!("cloud: heartbeat of {game}: {e}");
            }
            first = false;
            let mut waited = 0;
            while waited < HEARTBEAT && !stop.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(1)).await;
                waited += 1000;
            }
        }
    }

    /// After `game`'s thread ended (its exit writes done), before its push: the game joins the
    /// pending list, so a push that cannot finish is done at the next start. Writes one small
    /// file; no network.
    pub fn mark_pending(&self, game: &GameKey) -> Result<(), SyncError> {
        let result = self.state.lock().change(|d| {
            d.pending.insert(game.clone());
        });
        (self.notify)(Notice::Status);
        result
    }

    /// After `game` closed: its changes go up (G14) and its marker goes. Only a push that went
    /// through takes it off the pending list.
    pub async fn push_game(&self, prefs: &Prefs, game: &GameRef) -> Result<Outcome, SyncError> {
        if !prefs.saves {
            // Nothing of it syncs: it waits for nothing either.
            let result = self.state.lock().change(|d| {
                d.pending.remove(&game.key);
            });
            (self.notify)(Notice::Status);
            return result.map(|()| Outcome::default());
        }
        let _busy = self.begin();
        self.progress(Phase::Pushing, Some(&game.key), 0, 0);
        let result = self.push_inner(prefs, game).await;
        self.progress(Phase::Done, Some(&game.key), 0, 0);
        if let Ok(out) = &result {
            if !out.pushed.is_empty() {
                self.log(
                    ActivityKind::Pushed,
                    Some(&game.key),
                    out.pushed.len(),
                    None,
                );
            }
        }
        self.finish(Some(&game.key), result)
    }

    async fn push_inner(&self, prefs: &Prefs, game: &GameRef) -> Result<Outcome, SyncError> {
        let mut index = self.index.lock().await;
        let device = self.device(&mut index)?;
        let area = self.game_area(&game.key, prefs, game.play_time);
        let key = game.key.clone();
        let progress =
            move |done: u64, total: u64| self.progress(Phase::Pushing, Some(&key), done, total);
        let out = Syncer::new(&self.drive, &mut index, device)
            .with_progress(&progress)
            .push_after_exit(&area)
            .await?;
        drop(index);
        self.record_conflicts(game, out.conflicts.clone())?;
        self.state.lock().change(|d| {
            d.pending.remove(&game.key);
        })?;
        Ok(out)
    }

    /// At start: the pushes a previous run could not finish. `games` names them (the library;
    /// a game it does not know is pushed under its key). `open`: the game running now, left for
    /// its own push.
    pub async fn push_pending(
        &self,
        prefs: &Prefs,
        games: &[GameRef],
        open: Option<&GameKey>,
    ) -> Result<usize, SyncError> {
        let pending: Vec<GameKey> = self.state.lock().data.pending.iter().cloned().collect();
        let mut done = 0;
        for key in pending.into_iter().filter(|k| Some(k) != open) {
            let game = games
                .iter()
                .find(|g| g.key == key)
                .cloned()
                .unwrap_or_else(|| GameRef {
                    title: key.to_string(),
                    key,
                    play_time: None,
                });
            self.push_game(prefs, &game).await?;
            done += 1;
        }
        Ok(done)
    }

    // ---- «Sincronizza ora» ----

    /// The settings, every game's saves both ways (the open one left out: its files are in
    /// use), this device's library list, the account and its space.
    pub async fn sync_all(
        &self,
        prefs: &Prefs,
        library: &[GameRecord],
        open: Option<&GameKey>,
    ) -> Result<(), SyncError> {
        self.sync_run(prefs, library, open, true).await
    }

    /// At start (and after a sign-in): the pushes a previous run left pending, then the
    /// settings and this device's library list. The other games' saves wait for their own
    /// launch or for «Sincronizza ora».
    pub async fn sync_start(
        &self,
        prefs: &Prefs,
        library: &[GameRecord],
        open: Option<&GameKey>,
    ) -> Result<(), SyncError> {
        let games: Vec<GameRef> = library
            .iter()
            .map(|g| GameRef {
                key: g.id.clone(),
                title: g.shown_title().to_owned(),
                play_time: Some(g.play_time_secs),
            })
            .collect();
        self.push_pending(prefs, &games, open).await?;
        self.sync_run(prefs, library, open, false).await
    }

    async fn sync_run(
        &self,
        prefs: &Prefs,
        library: &[GameRecord],
        open: Option<&GameKey>,
        all_games: bool,
    ) -> Result<(), SyncError> {
        let _busy = self.begin();
        self.progress(Phase::Syncing, None, 0, 0);
        let result = self.sync_all_inner(prefs, library, open, all_games).await;
        self.progress(Phase::Done, None, 0, 0);
        let result = self.finish(None, result);
        if result.is_ok() {
            // The space bar and the account after what this sync moved. A failure is already
            // in the status and the activity (`finish`).
            if let Err(e) = self.refresh_account().await {
                log::debug!("cloud: account not refreshed after the sync: {e}");
            }
        }
        result
    }

    async fn sync_all_inner(
        &self,
        prefs: &Prefs,
        library: &[GameRecord],
        open: Option<&GameKey>,
        all_games: bool,
    ) -> Result<(), SyncError> {
        let mut index = self.index.lock().await;
        let device = self.device(&mut index)?;
        let now = now_ms();
        if prefs.settings {
            let out = Syncer::new(&self.drive, &mut index, device.clone())
                .run(&Area::settings(&self.paths.settings), Direction::Both)
                .await?;
            let moved = out.pulled.len() + out.pushed.len();
            if moved > 0 {
                self.log(ActivityKind::Settings, None, moved, None);
            }
        }
        if prefs.saves && all_games {
            let tree = DriveTree::read(&self.drive).await?;
            let mut games: BTreeSet<GameKey> = tree.save_folders().into_iter().collect();
            games.extend(library.iter().map(|g| g.id.clone()));
            for key in games.into_iter().filter(|k| Some(k) != open) {
                let record = library.iter().find(|g| g.id == key);
                // Nothing on either side: nothing to plan (a library game never played).
                if !self.paths.saves_root.join(key.dir()).is_dir() && !tree.has_saves(&key) {
                    continue;
                }
                let game = GameRef {
                    title: record.map_or_else(|| key.to_string(), |g| g.shown_title().to_owned()),
                    play_time: record.map(|g| g.play_time_secs),
                    key: key.clone(),
                };
                let area = self.game_area(&key, prefs, game.play_time);
                let out = Syncer::new(&self.drive, &mut index, device.clone())
                    .run(&area, Direction::Both)
                    .await?;
                if !out.pulled.is_empty() {
                    self.log(ActivityKind::Pulled, Some(&key), out.pulled.len(), None);
                }
                if !out.pushed.is_empty() {
                    self.log(ActivityKind::Pushed, Some(&key), out.pushed.len(), None);
                }
                self.record_conflicts(&game, out.conflicts.clone())?;
                self.state.lock().change(|d| {
                    d.pending.remove(&key);
                })?;
            }
        }
        let snapshot = library_snapshot(&device, library.iter(), now);
        Syncer::new(&self.drive, &mut index, device.clone())
            .publish(
                &[LIBRARY_FOLDER.to_owned()],
                &format!("{}.json", device.id),
                snapshot,
            )
            .await?;
        drop(index);
        let tree = DriveTree::read(&self.drive).await?;
        self.live.lock().emudome_bytes = Some(tree.bytes());
        self.state.lock().change(|d| d.last_sync = Some(now_ms()))?;
        Ok(())
    }

    // ---- Games on Drive ----

    /// «Giochi su Google Drive»: every game of the library and every game file on Drive.
    pub async fn games(&self, library: &[GameRecord]) -> Result<Vec<CloudGame>, SyncError> {
        let result = self.games_inner(library).await;
        match result {
            Ok(games) => Ok(games),
            Err(e) => self.finish(None, Err(e)),
        }
    }

    async fn games_inner(&self, library: &[GameRecord]) -> Result<Vec<CloudGame>, SyncError> {
        let tree = DriveTree::read(&self.drive).await?;
        self.live.lock().emudome_bytes = Some(tree.bytes());
        let my_id = self.index.lock().await.device_id()?;
        let lists = self.device_lists(&tree).await;
        let conflicts: BTreeSet<GameKey> =
            self.state.lock().data.conflicts.keys().cloned().collect();
        let roms = tree.game_files();
        let mut keys: BTreeSet<GameKey> = library
            .iter()
            .filter(|g| g.listed)
            .map(|g| g.id.clone())
            .collect();
        keys.extend(roms.keys().cloned());
        let rows = keys
            .into_iter()
            .map(|key| {
                let record = library.iter().find(|g| g.id == key && g.listed);
                let rom = roms.get(&key);
                let title = record
                    .map(|g| g.shown_title().to_owned())
                    .or_else(|| rom.and_then(|f| f.app_properties.get("title").cloned()))
                    .unwrap_or_else(|| key.to_string());
                let devices = lists
                    .iter()
                    .filter(|(id, list)| *id != &my_id && list.games.iter().any(|g| g.id == key))
                    .map(|(_, list)| list.device_name.clone())
                    .collect();
                // A library game whose file is gone from this device («Togli dal telefono», or
                // a folder unreachable) is «Solo su Drive» when Drive has it: «Scarica» brings
                // it back. Without a copy on Drive it stays «Solo qui», as before.
                let here = record.is_some_and(|g| g.available || rom.is_none());
                let on_drive = rom.is_some();
                CloudGame {
                    console: key.console(),
                    size: record
                        .map(|g| g.size)
                        .or_else(|| rom.and_then(|f| f.size))
                        .unwrap_or(0),
                    state: if conflicts.contains(&key) {
                        GameState::Conflict
                    } else if here && on_drive {
                        GameState::Both
                    } else if here {
                        GameState::LocalOnly
                    } else {
                        GameState::DriveOnly
                    },
                    saves_on_drive: tree.has_saves(&key),
                    here,
                    on_drive,
                    devices,
                    title,
                    id: key,
                }
            })
            .collect();
        (self.notify)(Notice::Status);
        Ok(rows)
    }

    /// «Carica»: the game file of `game` goes to `EmuDome/games/<console>/<hex>.<ext>`,
    /// streamed from the disk, after checking Drive has room for it (E09 otherwise). A file
    /// already there under the same key (the same content) is not sent again.
    pub async fn upload_game(&self, game: &GameRecord) -> Result<(), SyncError> {
        let _busy = self.begin();
        let result = self.upload_inner(game).await;
        self.progress(Phase::Done, Some(&game.id), 0, 0);
        if result.is_ok() {
            self.log(ActivityKind::Uploaded, Some(&game.id), 1, None);
        }
        self.finish(Some(&game.id), result)
    }

    async fn upload_inner(&self, game: &GameRecord) -> Result<(), SyncError> {
        let path = PathBuf::from(&game.path);
        let size = tokio::fs::metadata(&path)
            .await
            .map_err(|e| SyncError::local(&path, &e))?
            .len();
        self.progress(Phase::Uploading, Some(&game.id), 0, size);
        let name = rom_name(&game.id, &path);
        let mut index = self.index.lock().await;
        let device = self.device(&mut index)?;
        let syncer = Syncer::new(&self.drive, &mut index, device.clone());
        let folder = syncer
            .ensure_folder(&[GAMES_FOLDER.to_owned(), game.id.console().id().to_owned()])
            .await?;
        drop(syncer);
        drop(index);
        if self.drive.find(&folder, &name).await?.is_some() {
            return Ok(());
        }
        let quota = self.drive.quota().await?;
        if !quota.fits(size) {
            return Err(SyncError::QuotaFull {
                needed: size,
                available: quota.available().unwrap_or(u64::MAX),
            });
        }
        let mut props = Properties::new();
        props.insert("game".into(), game.id.as_str().to_owned());
        props.insert("title".into(), game.shown_title().to_owned());
        props.insert("device".into(), device.id);
        props.insert("deviceName".into(), device.name);
        self.drive
            .create(
                &folder,
                &name,
                &props,
                crate::drive::Content::File { path, size },
            )
            .await?;
        self.progress(Phase::Uploading, Some(&game.id), size, size);
        let mut live = self.live.lock();
        live.emudome_bytes = live.emudome_bytes.map(|b| b + size);
        Ok(())
    }

    /// «Scarica»: the game file of `game` comes from Drive into
    /// `<games>/<console>/<title>.<ext>` (a free name: nothing is written over), streamed to a
    /// temporary file, checked against Drive's MD5, then renamed. Returns the new file, for the
    /// library.
    pub async fn download_game(&self, game: &GameKey) -> Result<PathBuf, SyncError> {
        let _busy = self.begin();
        let result = self.download_inner(game).await;
        self.progress(Phase::Done, Some(game), 0, 0);
        if result.is_ok() {
            self.log(ActivityKind::Downloaded, Some(game), 1, None);
        }
        self.finish(Some(game), result)
    }

    async fn download_inner(&self, game: &GameKey) -> Result<PathBuf, SyncError> {
        let tree = DriveTree::read(&self.drive).await?;
        let rom = tree
            .game_files()
            .remove(game)
            .ok_or_else(|| SyncError::NotFound(format!("the game file of {game}")))?;
        let total = rom.size.unwrap_or(0);
        self.progress(Phase::Downloading, Some(game), 0, total);
        let ext = rom.name.rsplit_once('.').map_or("", |(_, e)| e).to_owned();
        let title = rom
            .app_properties
            .get("title")
            .cloned()
            .unwrap_or_else(|| game.hex().to_owned());
        let dir = self.paths.games.join(game.console().id());
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| SyncError::local(&dir, &e))?;
        let target = free_name(&dir, &clean_name(&title), &ext);
        let temp = dir.join(format!(
            ".{}.{}-download.emudome-tmp",
            game.hex(),
            std::process::id()
        ));
        let key = game.clone();
        let progress = move |done: u64| self.progress(Phase::Downloading, Some(&key), done, total);
        let written = self.drive.download_to(&rom.id, &temp, &progress).await;
        let written = match written {
            Ok(w) => w,
            Err(e) => {
                let _ = tokio::fs::remove_file(&temp).await;
                return Err(e);
            }
        };
        if rom.md5.as_deref() != Some(written.md5.as_str()) {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(SyncError::Request(format!(
                "{game}: the download does not match Drive's checksum"
            )));
        }
        tokio::fs::rename(&temp, &target)
            .await
            .map_err(|e| SyncError::local(&target, &e))?;
        Ok(target)
    }

    /// «Rimuovi da Drive»: the game file of `game` goes to Drive's trash. Never its saves,
    /// never anything on this device.
    pub async fn remove_game(&self, game: &GameKey) -> Result<(), SyncError> {
        let _busy = self.begin();
        let result = async {
            let tree = DriveTree::read(&self.drive).await?;
            let rom = tree
                .game_files()
                .remove(game)
                .ok_or_else(|| SyncError::NotFound(format!("the game file of {game}")))?;
            self.drive.trash(&rom.id).await?;
            let mut live = self.live.lock();
            live.emudome_bytes = live
                .emudome_bytes
                .map(|b| b.saturating_sub(rom.size.unwrap_or(0)));
            Ok(())
        }
        .await;
        if result.is_ok() {
            self.log(ActivityKind::Removed, Some(game), 1, None);
        }
        self.finish(Some(game), result)
    }

    // ---- Devices ----

    /// The library lists on Drive, by device id (an unreadable one left out, logged).
    async fn device_lists(&self, tree: &DriveTree) -> BTreeMap<String, DeviceList> {
        let mut out = BTreeMap::new();
        for file in tree.library_lists() {
            let bytes = match self.drive.download(&file.id).await {
                Ok(b) => b,
                Err(e) => {
                    log::warn!("cloud: library list {}: {e}", file.name);
                    continue;
                }
            };
            match serde_json::from_slice::<DeviceList>(&bytes) {
                Ok(list) => {
                    out.insert(list.device_id.clone(), list);
                }
                Err(e) => log::warn!("cloud: library list {} not understood: {e}", file.name),
            }
        }
        out
    }

    /// «I tuoi dispositivi»: every device that published its library list, this one first.
    pub async fn devices(&self) -> Result<Vec<DeviceDto>, SyncError> {
        let result = async {
            let tree = DriveTree::read(&self.drive).await?;
            let my_id = self.index.lock().await.device_id()?;
            let mut rows: Vec<DeviceDto> = self
                .device_lists(&tree)
                .await
                .into_values()
                .map(|l| DeviceDto {
                    this_device: l.device_id == my_id,
                    games: u32::try_from(l.games.len()).unwrap_or(u32::MAX),
                    id: l.device_id,
                    name: l.device_name,
                    updated: l.updated,
                })
                .collect();
            if !rows.iter().any(|d| d.this_device) {
                rows.push(DeviceDto {
                    id: my_id,
                    name: self.device_name.clone(),
                    updated: 0,
                    games: 0,
                    this_device: true,
                });
            }
            rows.sort_by(|a, b| {
                b.this_device
                    .cmp(&a.this_device)
                    .then(b.updated.cmp(&a.updated))
            });
            Ok(rows)
        }
        .await;
        match result {
            Ok(rows) => Ok(rows),
            Err(e) => self.finish(None, Err(e)),
        }
    }

    // ---- Conflicts ----

    /// G04's two versions of `game`, as the last sync found them; `None`: no conflict.
    pub fn conflict(&self, game: &GameRef) -> Option<GameConflict> {
        let files = self.state.lock().data.conflicts.get(&game.key).cloned()?;
        Some(game_conflict(game, files, &self.device_name))
    }

    /// The answer to G04 for every file of `game` in conflict. A file that changed since the
    /// question is left for the next sync (asked again then).
    pub async fn resolve(
        &self,
        prefs: &Prefs,
        game: &GameRef,
        choice: Choice,
    ) -> Result<(), SyncError> {
        let _busy = self.begin();
        let result = async {
            let files = self
                .state
                .lock()
                .data
                .conflicts
                .get(&game.key)
                .cloned()
                .unwrap_or_default();
            let mut index = self.index.lock().await;
            let device = self.device(&mut index)?;
            let area = self.game_area(&game.key, prefs, game.play_time);
            let prefix = area.prefix();
            let keep = match choice {
                Choice::Local => Keep::Local,
                Choice::Remote => Keep::Remote,
                Choice::Both => Keep::Both,
            };
            let now = now_ms();
            let mut syncer = Syncer::new(&self.drive, &mut index, device);
            for file in &files {
                let rel = file.path.strip_prefix(&prefix).unwrap_or(&file.path);
                syncer.resolve(&area, rel, keep, now).await?;
            }
            // What is still in conflict now (changed since the question, or a new one).
            let left: Vec<Conflict> = syncer.run(&area, Direction::Pull).await?.conflicts;
            drop(syncer);
            drop(index);
            self.state.lock().change(|d| {
                if left.is_empty() {
                    d.conflicts.remove(&game.key);
                } else {
                    d.conflicts.insert(game.key.clone(), left);
                }
            })?;
            Ok(files.len())
        }
        .await;
        if let Ok(n) = &result {
            self.log(ActivityKind::Resolved, Some(&game.key), *n, None);
        }
        self.finish(Some(&game.key), result).map(|_| ())
    }
}

/// G04's view of `files`.
fn game_conflict(game: &GameRef, files: Vec<Conflict>, device_name: &str) -> GameConflict {
    let local = Version {
        device: Some(device_name.to_owned()),
        modified: files.iter().map(|f| f.local_modified).max().unwrap_or(0),
        play_time: game.play_time,
        size: files.iter().map(|f| f.local_size).sum(),
        thumb: None,
    };
    let remote = Version {
        device: files.iter().find_map(|f| f.remote_device.clone()),
        modified: files.iter().map(|f| f.remote_modified).max().unwrap_or(0),
        play_time: files.iter().filter_map(|f| f.remote_play_time).max(),
        size: files.iter().map(|f| f.remote_size).sum(),
        thumb: None,
    };
    GameConflict {
        id: game.key.clone(),
        title: game.title.clone(),
        files,
        local,
        remote,
    }
}

/// A device's library list as `library_snapshot` writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceList {
    device_id: String,
    #[serde(default)]
    device_name: String,
    #[serde(default)]
    updated: u64,
    #[serde(default)]
    games: Vec<LibraryGame>,
}

/// A game file's name on Drive: `<hex>.<ext>` (its key; never the file's own name).
fn rom_name(game: &GameKey, path: &Path) -> String {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    format!("{}.{ext}", game.hex())
}

/// A title as a file name: the characters no system allows replaced, kept short.
fn clean_name(title: &str) -> String {
    let clean: String = title
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .take(80)
        .collect();
    let clean = clean.trim().trim_end_matches('.').to_owned();
    if clean.is_empty() {
        "game".to_owned()
    } else {
        clean
    }
}

/// `<dir>/<stem>.<ext>`, or `<stem> (2).<ext>`, … : a name no file has.
fn free_name(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let name = |n: u32| {
        let stem = if n == 1 {
            stem.to_owned()
        } else {
            format!("{stem} ({n})")
        };
        if ext.is_empty() {
            dir.join(stem)
        } else {
            dir.join(format!("{stem}.{ext}"))
        }
    };
    (1..)
        .map(name)
        .find(|p| !p.exists())
        .unwrap_or_else(|| dir.join(stem))
}

/// Every file EmuDome has on Drive, read in one listing (`DriveApi::list_all`) and placed under
/// its `EmuDome/` folder.
struct DriveTree {
    /// `/`-joined path under `EmuDome/` → file (folders too), the oldest of two with one name.
    files: BTreeMap<String, RemoteFile>,
}

impl DriveTree {
    async fn read(drive: &impl DriveApi) -> Result<Self, SyncError> {
        let Some(root) = drive.find(ROOT, ROOT_FOLDER).await?.filter(|f| f.folder) else {
            return Ok(Self {
                files: BTreeMap::new(),
            });
        };
        let all = drive.list_all().await?;
        let mut children: HashMap<&str, Vec<&RemoteFile>> = HashMap::new();
        for f in &all {
            for p in &f.parents {
                children.entry(p.as_str()).or_default().push(f);
            }
        }
        let mut files = BTreeMap::new();
        let mut folders = vec![(root.id.clone(), String::new())];
        while let Some((id, prefix)) = folders.pop() {
            for f in children.get(id.as_str()).into_iter().flatten() {
                let rel = format!("{prefix}{}", f.name);
                if f.folder {
                    folders.push((f.id.clone(), format!("{rel}/")));
                }
                files.entry(rel).or_insert_with(|| (*f).clone());
            }
        }
        Ok(Self { files })
    }

    /// Bytes of every file under `EmuDome/`.
    fn bytes(&self) -> u64 {
        self.files.values().filter_map(|f| f.size).sum()
    }

    /// The games with a saves folder holding at least one synced file.
    fn save_folders(&self) -> Vec<GameKey> {
        let mut out: BTreeSet<GameKey> = BTreeSet::new();
        for (rel, f) in &self.files {
            if f.folder {
                continue;
            }
            let mut parts = rel.splitn(3, '/');
            let (Some(console), Some(hex), Some(rest)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if excluded(rest) {
                continue;
            }
            if let Ok(key) = GameKey::parse(&format!("{console}:{hex}")) {
                out.insert(key);
            }
        }
        out.into_iter().collect()
    }

    fn has_saves(&self, game: &GameKey) -> bool {
        let prefix = format!("{}/{}/", game.console().id(), game.hex());
        self.files.iter().any(|(rel, f)| {
            !f.folder && rel.starts_with(&prefix) && !excluded(&rel[prefix.len()..])
        })
    }

    /// The game files under `games/<console>/`, by key.
    fn game_files(&self) -> BTreeMap<GameKey, RemoteFile> {
        let prefix = format!("{GAMES_FOLDER}/");
        let mut out = BTreeMap::new();
        for (rel, f) in &self.files {
            let Some(rest) = rel.strip_prefix(&prefix) else {
                continue;
            };
            let Some((console, name)) = rest.split_once('/') else {
                continue;
            };
            if f.folder || name.contains('/') {
                continue;
            }
            let hex = name.split('.').next().unwrap_or(name);
            if let Ok(key) = GameKey::parse(&format!("{console}:{hex}")) {
                out.entry(key).or_insert_with(|| f.clone());
            }
        }
        out
    }

    /// The files of `library/`.
    fn library_lists(&self) -> Vec<&RemoteFile> {
        let prefix = format!("{LIBRARY_FOLDER}/");
        self.files
            .iter()
            .filter(|(rel, f)| !f.folder && rel.starts_with(&prefix) && rel.ends_with(".json"))
            .map(|(_, f)| f)
            .collect()
    }
}

#[cfg(test)]
mod tests;
