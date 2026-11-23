//! The sync itself: a local folder mirrored under Drive's `EmuDome/` folder ([`Area`]), planned
//! file by file with [`crate::decide`] and carried out by [`Syncer`].
//!
//! Drive layout (docs/feasibility.md, «Google Drive sync»):
//!
//! ```text
//! EmuDome/
//!   <console>/<hex>/...        a game's folder: saves/<console>/<hex>/ (its GameKey, rule 2)
//!   settings/...               <app data>/settings/ (global, console, game files; never the
//!                              per-device local-settings/)
//!   library/<device id>.json   each device's list of games (`library_snapshot`)
//! ```
//!
//! What never syncs: `backups/` and `broken-*` (this device's own safety copies), `conflicts/`
//! (the copies a conflict kept), the atomic writer's temporary files, files moved aside as
//! unreadable.
//!
//! Keeping the data (engineering rule 1):
//! - a pulled file is checked against Drive's MD5 before it is written, and written with
//!   `write_atomic`; the local file is checked again just before (a file that changed since the
//!   plan is left for the next sync);
//! - a pushed file replaces Drive's copy only if Drive's copy is still the one planned (checked
//!   again just before); a failed upload leaves Drive's copy as it was;
//! - the index records a file only after its transfer succeeded, and is written after each one;
//! - a conflict is never touched by [`Syncer::run`]; [`Syncer::resolve`] keeps the side not
//!   chosen as a copy under `conflicts/<when>-<device>/` on the side it lived on;
//! - nothing is ever deleted, on either side.
//!
//! Drive has no conditional write on a checksum: between the check and the upload (a second
//! for a save) another device can still write the same file; the next sync then sees both
//! sides changed and asks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use emudome_core::GameKey;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::decide::{decide, Decision};
use crate::drive::{md5_hex, Content, DriveApi, Properties, RemoteFile, ROOT};
use crate::index::{Entry, SyncIndex};
use crate::marker::{self, InUse, Marker};
use crate::SyncError;

/// The visible folder in «My Drive» everything goes under.
pub const ROOT_FOLDER: &str = "EmuDome";
/// Where a conflict's kept copies go, in an area (local and on Drive).
pub const CONFLICTS: &str = "conflicts";
/// Files at most this big are read whole to be uploaded; bigger ones are streamed from the disk.
const READ_WHOLE: u64 = crate::drive::rest::MULTIPART_MAX;

/// Where the game files themselves go when the user uploads one («Carica»):
/// `EmuDome/games/<console>/<hex>.<ext>`, never synced on their own.
pub const GAMES_FOLDER: &str = "games";
/// Where each device's library list goes: `EmuDome/library/<device id>.json`.
pub const LIBRARY_FOLDER: &str = "library";

/// The folder names of a game's saves under `EmuDome/`: `[<console>, <hex>]`.
pub fn game_folder(game: &GameKey) -> Vec<String> {
    vec![game.console().id().to_owned(), game.hex().to_owned()]
}

/// This device, as the other devices see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    /// Random, made once ([`SyncIndex::device_id`]).
    pub id: String,
    /// The computer's name (`emudome_platform::cloud::device_name`).
    pub name: String,
}

/// A local folder and its mirror under `EmuDome/`.
#[derive(Clone, Debug)]
pub struct Area {
    pub local: PathBuf,
    /// Folder names under `EmuDome/`.
    pub remote: Vec<String>,
    /// `appProperties` every file pushed from it carries (the game id, its play time, the core
    /// version), besides the device.
    pub props: Properties,
    /// Which quick saves of a game's folder sync (`cloud.squick`).
    pub quick: QuickSync,
}

/// Which quick saves sync (`cloud.squick`): a quick save is its `.state` with its thumbnail
/// (`.png`) and record (`.json`), `auto` or `slotN`, at the top of the game's folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QuickSync {
    /// Every one (`all`).
    #[default]
    All,
    /// Only the newest, on either side (`last`): the others stay where they are.
    Last,
    /// None (`none`): only the in-game save and what the core keeps in the folder.
    None,
}

impl QuickSync {
    /// The setting's value (`all` / `last` / `none`); anything else is the default.
    pub fn from_id(id: &str) -> Self {
        match id {
            "last" => QuickSync::Last,
            "none" => QuickSync::None,
            _ => QuickSync::All,
        }
    }
}

/// The quick save `rel` belongs to (`auto`, `slot3`), if it is one of a quick save's files.
fn quick_stem(rel: &str) -> Option<&str> {
    if rel.contains('/') {
        return None;
    }
    let (stem, ext) = rel.rsplit_once('.')?;
    let slot = stem == "auto"
        || stem
            .strip_prefix("slot")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    (slot && matches!(ext, "state" | "png" | "json")).then_some(stem)
}

impl Area {
    /// A game's saves folder (`<saves root>/<console>/<hex>/`) ↔ `EmuDome/<console>/<hex>/`.
    pub fn game(saves_root: &Path, game: &GameKey, props: Properties) -> Self {
        let mut props = props;
        props.insert("game".into(), game.as_str().to_owned());
        Self {
            local: saves_root.join(game.dir()),
            remote: game_folder(game),
            props,
            quick: QuickSync::All,
        }
    }

    /// The same area syncing only the quick saves `quick` allows.
    pub fn with_quick(mut self, quick: QuickSync) -> Self {
        self.quick = quick;
        self
    }

    /// The portable settings (`<app data>/settings/`) ↔ `EmuDome/settings/`. The per-device
    /// values (`<app local data>/local-settings/`) are another folder and never sync.
    pub fn settings(settings_dir: &Path) -> Self {
        Self {
            local: settings_dir.to_path_buf(),
            remote: vec!["settings".into()],
            props: Properties::new(),
            quick: QuickSync::All,
        }
    }

    /// Leaves out of both sides the quick saves [`Area::quick`] does not sync.
    fn filter_quick(
        &self,
        local: &mut BTreeMap<String, LocalFile>,
        remote: &mut BTreeMap<String, RemoteFile>,
    ) {
        let keep: Box<dyn Fn(&str) -> bool> = match self.quick {
            QuickSync::All => return,
            QuickSync::None => Box::new(|rel| quick_stem(rel).is_none()),
            QuickSync::Last => {
                let newest = local
                    .iter()
                    .map(|(rel, f)| (rel.as_str(), f.modified))
                    .chain(remote.iter().map(|(rel, f)| (rel.as_str(), f.modified)))
                    .filter(|(rel, _)| rel.ends_with(".state") && quick_stem(rel).is_some())
                    .max_by_key(|(_, modified)| *modified)
                    .and_then(|(rel, _)| quick_stem(rel))
                    .map(str::to_owned);
                Box::new(move |rel| quick_stem(rel).is_none_or(|s| Some(s) == newest.as_deref()))
            }
        };
        local.retain(|rel, _| keep(rel));
        remote.retain(|rel, _| keep(rel));
    }

    /// The prefix of this area's paths in the index (`gba/<hex>/`).
    pub fn prefix(&self) -> String {
        format!("{}/", self.remote.join("/"))
    }

    fn local_path(&self, rel: &str) -> PathBuf {
        rel.split('/').fold(self.local.clone(), |p, s| p.join(s))
    }
}

/// Whether the file at `rel` (`/`-separated, in an area) is left out of the sync.
pub fn excluded(rel: &str) -> bool {
    let first = rel.split('/').next().unwrap_or("");
    let name = rel.rsplit('/').next().unwrap_or("");
    first == "backups"
        || first == CONFLICTS
        || name.starts_with("broken-")
        || name.ends_with(".emudome-tmp")
        || name.ends_with(".unreadable")
}

/// A local file as planned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFile {
    pub md5: String,
    pub size: u64,
    /// Unix ms.
    pub modified: u64,
}

/// One file's decision, with both sides as they were seen.
#[derive(Clone, Debug)]
pub struct Step {
    /// `/`-separated, in the area.
    pub rel: String,
    pub decision: Decision,
    pub local: Option<LocalFile>,
    pub remote: Option<RemoteFile>,
}

/// What a conflict shows the user. TypeScript: `CloudConflict`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudConflict")]
pub struct Conflict {
    /// The file's place under `EmuDome/` (`gba/<hex>/game.sav`).
    pub path: String,
    pub local_size: u64,
    /// Unix ms.
    pub local_modified: u64,
    pub remote_size: u64,
    /// Unix ms.
    pub remote_modified: u64,
    /// The device that last wrote Drive's copy, if it said.
    pub remote_device: Option<String>,
    /// That device's play time of the game then, seconds, if it said.
    pub remote_play_time: Option<u64>,
}

/// Which way a run moves files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Before a game starts: Drive's newer copies come here; local changes wait.
    Pull,
    /// After a game ends: this device's changes go up; Drive's wait.
    Push,
    Both,
}

/// What a run did. TypeScript: `CloudOutcome`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "CloudOutcome")]
pub struct Outcome {
    pub pulled: Vec<String>,
    pub pushed: Vec<String>,
    /// Left for the user.
    pub conflicts: Vec<Conflict>,
    /// Waiting for a run the other way, or changed while this one ran (the next sync takes
    /// them).
    pub waiting: Vec<String>,
}

/// Which side the user keeps in a conflict. The other is kept as a copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    Local,
    Remote,
    /// This device's goes on (as `Local`), and Drive's copy is kept on both sides: under
    /// `conflicts/` on Drive and in this device's game folder («Tieni entrambe»).
    Both,
}

/// The syncing of this device against `drive`, recording into `index`.
pub struct Syncer<'a, D> {
    drive: &'a D,
    index: &'a mut SyncIndex,
    device: Device,
    /// Set: the run stops before its next transfer (the rest waits for the next sync).
    cancel: Option<&'a AtomicBool>,
    /// Told `(bytes done, bytes in all)` of the run's transfers, after each one.
    progress: Option<&'a (dyn Fn(u64, u64) + Send + Sync)>,
}

impl<'a, D: DriveApi> Syncer<'a, D> {
    pub fn new(drive: &'a D, index: &'a mut SyncIndex, device: Device) -> Self {
        Self {
            drive,
            index,
            device,
            cancel: None,
            progress: None,
        }
    }

    /// A run that stops before its next transfer once `flag` is set (a file already moving
    /// finishes: nothing is left half written, the index stays true).
    pub fn with_cancel(mut self, flag: &'a AtomicBool) -> Self {
        self.cancel = Some(flag);
        self
    }

    /// A run that tells `progress(done, total)` (bytes of its transfers) after each file.
    pub fn with_progress(mut self, progress: &'a (dyn Fn(u64, u64) + Send + Sync)) -> Self {
        self.progress = Some(progress);
        self
    }

    fn cancelled(&self) -> bool {
        self.cancel.is_some_and(|c| c.load(Ordering::SeqCst))
    }

    /// The decision for every file of `area`, nothing changed.
    pub async fn plan(&mut self, area: &Area) -> Result<Vec<Step>, SyncError> {
        self.index.readable()?;
        let mut local = scan_local(&area.local).await?;
        let mut remote = match self.find_folder(&area.remote).await? {
            Some(folder) => self.list_tree(&folder).await?,
            None => BTreeMap::new(),
        };
        area.filter_quick(&mut local, &mut remote);
        let prefix = area.prefix();
        let mut rels: Vec<&String> = local.keys().chain(remote.keys()).collect();
        rels.sort();
        rels.dedup();
        Ok(rels
            .into_iter()
            .map(|rel| {
                let l = local.get(rel);
                let r = remote.get(rel);
                let base = self.index.get(&format!("{prefix}{rel}"));
                Step {
                    rel: rel.clone(),
                    decision: decide(
                        l.map(|l| l.md5.as_str()),
                        r.map(|r| r.md5.as_deref().unwrap_or("")),
                        base.map(|b| b.md5.as_str()),
                    ),
                    local: l.cloned(),
                    remote: r.cloned(),
                }
            })
            .collect())
    }

    /// Plans `area` and carries out the steps of `direction`. Conflicts are listed, not touched.
    pub async fn run(&mut self, area: &Area, direction: Direction) -> Result<Outcome, SyncError> {
        let steps = self.plan(area).await?;
        let prefix = area.prefix();
        let mut out = Outcome::default();
        let moves = |s: &Step| match s.decision {
            Decision::Pull => direction != Direction::Push,
            Decision::Push => direction != Direction::Pull,
            Decision::InSync | Decision::Conflict => false,
        };
        let size = |s: &Step| match s.decision {
            Decision::Pull => s.remote.as_ref().and_then(|r| r.size).unwrap_or(0),
            Decision::Push => s.local.as_ref().map_or(0, |l| l.size),
            Decision::InSync | Decision::Conflict => 0,
        };
        let total: u64 = steps.iter().filter(|s| moves(s)).map(size).sum();
        let mut done = 0u64;
        for step in steps {
            let path = format!("{prefix}{}", step.rel);
            if moves(&step) && self.cancelled() {
                out.waiting.push(path);
                continue;
            }
            if moves(&step) {
                done += size(&step);
            }
            match step.decision {
                Decision::InSync => self.record_in_sync(&path, &step)?,
                Decision::Conflict => out.conflicts.push(conflict_of(&path, &step)),
                Decision::Pull if direction != Direction::Push => {
                    if self.pull(area, &step).await? {
                        out.pulled.push(path);
                    } else {
                        out.waiting.push(path);
                    }
                }
                Decision::Push if direction != Direction::Pull => {
                    if self.push(area, &step).await? {
                        out.pushed.push(path);
                    } else {
                        out.waiting.push(path);
                    }
                }
                Decision::Pull | Decision::Push => out.waiting.push(path),
            }
            if moves(&step) {
                if let Some(progress) = self.progress {
                    progress(done, total);
                }
            }
        }
        Ok(out)
    }

    /// Before a game starts: Drive's newer files come down. A conflict is returned for the user
    /// (the game can start with this device's files; nothing was overwritten).
    pub async fn pull_before_launch(&mut self, area: &Area) -> Result<Outcome, SyncError> {
        self.run(area, Direction::Pull).await
    }

    /// After a game ended (its files written): this device's changes go up and its «in use»
    /// marker goes.
    pub async fn push_after_exit(&mut self, area: &Area) -> Result<Outcome, SyncError> {
        let out = self.run(area, Direction::Push).await?;
        self.release(area).await?;
        Ok(out)
    }

    /// Settles the conflict on `rel` keeping `keep`; the other side's content stays as a copy
    /// under `conflicts/<now>-<device>/<rel>` on the side it was on. `Ok(false)`: the file is
    /// no longer in conflict (it changed since the question): nothing was done, plan again.
    pub async fn resolve(
        &mut self,
        area: &Area,
        rel: &str,
        keep: Keep,
        now: u64,
    ) -> Result<bool, SyncError> {
        let steps = self.plan(area).await?;
        let Some(step) = steps.into_iter().find(|s| s.rel == rel) else {
            return Ok(false);
        };
        let (Decision::Conflict, Some(local), Some(remote)) =
            (step.decision, step.local.as_ref(), step.remote.as_ref())
        else {
            return Ok(false);
        };
        let (dir, name) = split_rel(rel);
        match keep {
            Keep::Local | Keep::Both => {
                let who = remote
                    .app_properties
                    .get("deviceName")
                    .map_or("drive", String::as_str);
                if keep == Keep::Both {
                    // The other side's copy comes here too, under `conflicts/`, beside this one.
                    let bytes = self.drive.download(&remote.id).await?;
                    if remote.md5.as_deref() != Some(md5_hex(&bytes).as_str()) {
                        return Err(SyncError::Request(format!(
                            "{rel}: the download does not match Drive's checksum"
                        )));
                    }
                    let to = area.local_path(&format!("{CONFLICTS}/{}/{rel}", stamp(now, who)));
                    write_file(&to, bytes).await?;
                }
                let mut folder = area.remote.clone();
                folder.push(CONFLICTS.into());
                folder.push(stamp(now, who));
                folder.extend(dir.iter().map(|s| (*s).to_owned()));
                let target = self.ensure_folder(&folder).await?;
                self.drive.copy(&remote.id, &target, name).await?;
                // Drive's copy is now the one decided over: push over it.
                let mut over = step.clone();
                over.remote = Some(remote.clone());
                if !self.push(area, &over).await? {
                    return Ok(false);
                }
            }
            Keep::Remote => {
                let from = area.local_path(rel);
                let to = area.local_path(&format!(
                    "{CONFLICTS}/{}/{rel}",
                    stamp(now, &self.device.name)
                ));
                let bytes = read_file(&from).await?;
                if md5_hex(&bytes) != local.md5 {
                    return Ok(false);
                }
                write_file(&to, bytes).await?;
                if !self.pull(area, &step).await? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Whether the game of `area` is open on another device.
    pub async fn in_use(&mut self, area: &Area, now: u64) -> Result<InUse, SyncError> {
        let Some(folder) = self.find_folder(&area.remote).await? else {
            return Ok(InUse::Free);
        };
        let folder = self.drive.get(&folder).await?;
        Ok(marker::in_use(
            Marker::read(&folder.app_properties).as_ref(),
            &self.device.id,
            now,
        ))
    }

    /// Marks the game of `area` as open on this device, or refreshes this device's heartbeat.
    pub async fn claim(&mut self, area: &Area, now: u64) -> Result<(), SyncError> {
        let folder = self.ensure_folder(&area.remote).await?;
        let current = self.drive.get(&folder).await?;
        let since = Marker::read(&current.app_properties)
            .filter(|m| m.device == self.device.id)
            .map_or(now, |m| m.since);
        let mine = Marker {
            device: self.device.id.clone(),
            name: self.device.name.clone(),
            since,
            beat: now,
        };
        self.drive.set_properties(&folder, &mine.set()).await?;
        Ok(())
    }

    /// Takes this device's marker off the game of `area` (another device's is left alone).
    pub async fn release(&mut self, area: &Area) -> Result<(), SyncError> {
        let Some(folder) = self.find_folder(&area.remote).await? else {
            return Ok(());
        };
        let current = self.drive.get(&folder).await?;
        if Marker::read(&current.app_properties).is_some_and(|m| m.device == self.device.id) {
            self.drive.set_properties(&folder, &Marker::clear()).await?;
        }
        Ok(())
    }

    /// Writes this device's own file `name` in `EmuDome/<folder>/` (its library list): only
    /// this device writes it, so it is replaced without a decision.
    pub async fn publish(
        &mut self,
        folder: &[String],
        name: &str,
        bytes: Vec<u8>,
    ) -> Result<RemoteFile, SyncError> {
        let parent = self.ensure_folder(folder).await?;
        let props = self.device_props(&Properties::new());
        match self.drive.find(&parent, name).await? {
            Some(f) => {
                self.drive
                    .update(&f.id, &props, Content::Bytes(bytes))
                    .await
            }
            None => {
                self.drive
                    .create(&parent, name, &props, Content::Bytes(bytes))
                    .await
            }
        }
    }

    fn record_in_sync(&mut self, path: &str, step: &Step) -> Result<(), SyncError> {
        match (&step.local, &step.remote) {
            (Some(l), Some(r)) => {
                let entry = Entry {
                    id: r.id.clone(),
                    md5: l.md5.clone(),
                    modified: r.modified,
                    size: l.size,
                };
                if self.index.get(path) != Some(&entry) {
                    self.index.set(path, entry)?;
                }
                Ok(())
            }
            // Gone from both sides: nothing to remember.
            _ => self.index.remove(path),
        }
    }

    /// Brings Drive's copy of `step` here. `Ok(false)`: the local file changed since the plan
    /// (left for the next sync).
    async fn pull(&mut self, area: &Area, step: &Step) -> Result<bool, SyncError> {
        let remote = step
            .remote
            .as_ref()
            .ok_or_else(|| SyncError::Request("pull without a Drive file".into()))?;
        let bytes = self.drive.download(&remote.id).await?;
        let md5 = md5_hex(&bytes);
        if remote.md5.as_deref() != Some(md5.as_str()) {
            return Err(SyncError::Request(format!(
                "{}: the download does not match Drive's checksum",
                step.rel
            )));
        }
        let path = area.local_path(&step.rel);
        let now_local = local_md5(&path).await?;
        if now_local.as_deref() != step.local.as_ref().map(|l| l.md5.as_str()) {
            return Ok(false);
        }
        let size = bytes.len() as u64;
        write_file(&path, bytes).await?;
        self.index.set(
            &format!("{}{}", area.prefix(), step.rel),
            Entry {
                id: remote.id.clone(),
                md5,
                modified: remote.modified,
                size,
            },
        )?;
        Ok(true)
    }

    /// Sends this device's copy of `step` to Drive. `Ok(false)`: one side changed since the
    /// plan (left for the next sync).
    async fn push(&mut self, area: &Area, step: &Step) -> Result<bool, SyncError> {
        let local = step
            .local
            .as_ref()
            .ok_or_else(|| SyncError::Request("push without a local file".into()))?;
        let path = area.local_path(&step.rel);
        let content = if local.size <= READ_WHOLE {
            let bytes = read_file(&path).await?;
            if md5_hex(&bytes) != local.md5 {
                return Ok(false);
            }
            Content::Bytes(bytes)
        } else {
            Content::File {
                path: path.clone(),
                size: local.size,
            }
        };
        let props = self.device_props(&area.props);
        let existing = match &step.remote {
            Some(planned) => match self.drive.get(&planned.id).await {
                Ok(now) if now.md5 == planned.md5 => Some(now.id),
                Ok(_) => return Ok(false),
                // Gone from Drive since the plan: made again.
                Err(SyncError::NotFound(_)) => None,
                Err(e) => return Err(e),
            },
            None => None,
        };
        let made = match existing {
            Some(id) => self.drive.update(&id, &props, content).await?,
            None => {
                let (dir, name) = split_rel(&step.rel);
                let mut folder = area.remote.clone();
                folder.extend(dir.iter().map(|s| (*s).to_owned()));
                let parent = self.ensure_folder(&folder).await?;
                self.drive.create(&parent, name, &props, content).await?
            }
        };
        let md5 = made
            .md5
            .clone()
            .ok_or_else(|| SyncError::Request("Drive gave no checksum".into()))?;
        self.index.set(
            &format!("{}{}", area.prefix(), step.rel),
            Entry {
                id: made.id,
                md5,
                modified: made.modified,
                size: made.size.unwrap_or(local.size),
            },
        )?;
        Ok(true)
    }

    fn device_props(&self, area_props: &Properties) -> Properties {
        let mut props = area_props.clone();
        props.insert("device".into(), self.device.id.clone());
        props.insert("deviceName".into(), self.device.name.clone());
        props
    }

    /// The id of `EmuDome/<path...>`, if it exists.
    pub async fn find_folder(&self, path: &[String]) -> Result<Option<String>, SyncError> {
        let Some(mut id) = self
            .drive
            .find(ROOT, ROOT_FOLDER)
            .await?
            .filter(|f| f.folder)
            .map(|f| f.id)
        else {
            return Ok(None);
        };
        for name in path {
            match self.drive.find(&id, name).await?.filter(|f| f.folder) {
                Some(f) => id = f.id,
                None => return Ok(None),
            }
        }
        Ok(Some(id))
    }

    /// The id of `EmuDome/<path...>`, made where missing.
    pub async fn ensure_folder(&self, path: &[String]) -> Result<String, SyncError> {
        let mut id = ROOT.to_owned();
        let names = std::iter::once(ROOT_FOLDER).chain(path.iter().map(String::as_str));
        for name in names {
            id = match self.drive.find(&id, name).await?.filter(|f| f.folder) {
                Some(f) => f.id,
                None => {
                    self.drive
                        .create_folder(&id, name, &Properties::new())
                        .await?
                        .id
                }
            };
        }
        Ok(id)
    }

    /// Every file under the folder `id`, by its `/`-separated path (the oldest of two with the
    /// same name), the excluded ones left out.
    async fn list_tree(&self, id: &str) -> Result<BTreeMap<String, RemoteFile>, SyncError> {
        let mut out = BTreeMap::new();
        let mut folders = vec![(id.to_owned(), String::new())];
        while let Some((folder, prefix)) = folders.pop() {
            for f in self.drive.list(&folder).await? {
                let rel = format!("{prefix}{}", f.name);
                if excluded(&rel) {
                    continue;
                }
                if f.folder {
                    folders.push((f.id, format!("{rel}/")));
                } else {
                    out.entry(rel).or_insert(f);
                }
            }
        }
        Ok(out)
    }
}

fn conflict_of(path: &str, step: &Step) -> Conflict {
    let (local_size, local_modified) = step.local.as_ref().map_or((0, 0), |l| (l.size, l.modified));
    let remote = step.remote.as_ref();
    Conflict {
        path: path.to_owned(),
        local_size,
        local_modified,
        remote_size: remote.and_then(|r| r.size).unwrap_or(0),
        remote_modified: remote.map_or(0, |r| r.modified),
        remote_device: remote.and_then(|r| r.app_properties.get("deviceName").cloned()),
        remote_play_time: remote
            .and_then(|r| r.app_properties.get("playTime"))
            .and_then(|t| t.parse().ok()),
    }
}

/// `a/b/c.sav` → (`["a", "b"]`, `c.sav`).
fn split_rel(rel: &str) -> (Vec<&str>, &str) {
    let mut parts: Vec<&str> = rel.split('/').collect();
    let name = parts.pop().unwrap_or(rel);
    (parts, name)
}

/// `<ms>-<device>`, the device's name kept to letters, digits, `-` and `_`.
fn stamp(now: u64, device: &str) -> String {
    let clean: String = device
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect();
    format!("{now}-{clean}")
}

async fn read_file(path: &Path) -> Result<Vec<u8>, SyncError> {
    tokio::fs::read(path)
        .await
        .map_err(|e| SyncError::local(path, &e))
}

async fn write_file(path: &Path, bytes: Vec<u8>) -> Result<(), SyncError> {
    let target = path.to_path_buf();
    tokio::task::spawn_blocking(move || emudome_core::write_atomic(&target, &bytes))
        .await
        .map_err(SyncError::local_other)?
        .map_err(|e| SyncError::local(path, &e))
}

async fn local_md5(path: &Path) -> Result<Option<String>, SyncError> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(Some(md5_hex(&bytes))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(SyncError::local(path, &e)),
    }
}

/// Every file of `dir` (not the excluded ones) with its MD5, by `/`-separated path. A missing
/// folder has no files.
async fn scan_local(dir: &Path) -> Result<BTreeMap<String, LocalFile>, SyncError> {
    let dir = dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut out = BTreeMap::new();
        scan_into(&dir, "", &mut out)?;
        Ok(out)
    })
    .await
    .map_err(SyncError::local_other)?
}

fn scan_into(
    dir: &Path,
    prefix: &str,
    out: &mut BTreeMap<String, LocalFile>,
) -> Result<(), SyncError> {
    let local = |e: std::io::Error| SyncError::local(dir, &e);
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && prefix.is_empty() => return Ok(()),
        Err(e) => return Err(local(e)),
    };
    for entry in entries {
        let entry = entry.map_err(local)?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            log::warn!(
                "cloud: {} has a name that is not UTF-8: not synced",
                entry.path().display()
            );
            continue;
        };
        let rel = format!("{prefix}{name}");
        if excluded(&rel) {
            continue;
        }
        let meta = entry.metadata().map_err(local)?;
        if meta.is_dir() {
            scan_into(&entry.path(), &format!("{rel}/"), out)?;
        } else if meta.is_file() {
            let bytes = std::fs::read(entry.path()).map_err(local)?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
            out.insert(
                rel,
                LocalFile {
                    md5: md5_hex(&bytes),
                    size: bytes.len() as u64,
                    modified,
                },
            );
        }
    }
    Ok(())
}

/// One game of this device's library list (`EmuDome/library/<device id>.json`), so another
/// device can show «on the PC, not on this device».
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryGame {
    pub id: GameKey,
    pub title: String,
    pub play_time_secs: u64,
    /// Unix ms.
    pub last_played: Option<u64>,
}

/// The library list of `device`: what it has, never where (no path, no file name).
pub fn library_snapshot<'g>(
    device: &Device,
    games: impl IntoIterator<Item = &'g emudome_core::library::GameRecord>,
    now: u64,
) -> Vec<u8> {
    let games: Vec<LibraryGame> = games
        .into_iter()
        .map(|g| LibraryGame {
            id: g.id.clone(),
            title: g.shown_title().to_owned(),
            play_time_secs: g.play_time_secs,
            last_played: g.last_played,
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "$schema": 1,
        "deviceId": device.id,
        "deviceName": device.name,
        "updated": now,
        "games": games,
    }))
    .unwrap_or_default();
    bytes.push(b'\n');
    bytes
}

#[cfg(test)]
mod tests;
