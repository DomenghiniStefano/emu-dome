//! The one-time copy of the data folders to the new app identifier (Trello #90).
//!
//! On 2026-11-21 the app identifier changed from `com.domenghini.emudome` to
//! `it.domehub.emudome`. The system names the app's data folders after it, so the user's saves,
//! settings, library, covers and system files stay in the old folders until they are copied.
//! The app's shell finds both pairs of folders (`emudome-platform`: where they are on each OS,
//! and none on Android, where the old package's storage cannot be reached) and runs [`run`] at
//! start, before anything opens a data file.
//!
//! Safety (engineering rule 1, the model is the game key move in [`crate::migration`]):
//! - The old folders are only read, never written, moved or deleted: a later cleanup is the
//!   owner's choice.
//! - Each file is copied, flushed to the disk, and verified: size and MD5 of the copy against the
//!   original. A symbolic link or junction is not followed: the copy fails on it.
//! - The new folders are used only once the copy is complete and verified: `<new data>/migration-90/done`
//!   is written last. Until then the app runs on the old folders ([`Outcome::use_old`]) and says
//!   so with [`ErrorCode::DataMoveFailed`]; the next start tries again. The app never starts on an
//!   empty new folder while the old one holds the user's data.
//! - Before the first file, each new folder gets a stamp ([`STAMP`]): a stamped folder without
//!   the marker is an earlier pass's unfinished copy (the app never used it), removed and copied
//!   again. The stamps go once the marker is written.
//! - A new folder that already holds data without a stamp is someone's data (a run of the new
//!   build that did not copy, files put there by hand): nothing is copied over it or merged into
//!   it. The new folders are used as they are, the old ones are left untouched, and the user is
//!   told with [`ErrorCode::DataMoveConflict`]; the marker is written, so it is said once.
//!   Unless another new folder is empty while its old one holds data (Windows has two pairs):
//!   then neither choice keeps everything, so nothing is copied, the old folders are used and
//!   [`ErrorCode::DataMoveFailed`] asks the user to move the folders by hand.
//! - Every pass writes `<new data>/migration-90/report.json`.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};
use serde::Serialize;

use crate::atomic::{sync_dir, TMP_SUFFIX};
use crate::{write_atomic, ErrorCode};

/// The folder of the marker and the report, inside the new data folder.
pub const WORK_DIR: &str = "migration-90";

/// The stamp of a new folder being copied into; its content is the old folder's path.
pub const STAMP: &str = ".emudome-move-90";

/// One folder to copy: the old identifier's and the new one's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Folder {
    pub old: PathBuf,
    pub new: PathBuf,
}

/// What to copy.
pub struct Plan {
    /// The synced data folder first (it holds [`WORK_DIR`]), then the per-device one. A pair
    /// listed twice (on Linux the data and local data folders are one) is handled once.
    pub folders: Vec<Folder>,
    /// Names at the top of a folder that are not user data and are not copied, nor count as
    /// data: the webview's own profile (`EBWebView` on Windows), which the webview makes again.
    pub skip: Vec<String>,
}

impl Plan {
    fn work_dir(&self) -> Option<PathBuf> {
        self.folders.first().map(|f| f.new.join(WORK_DIR))
    }

    /// `<new data>/migration-90/done`.
    pub fn marker(&self) -> Option<PathBuf> {
        self.work_dir().map(|w| w.join("done"))
    }

    /// `<new data>/migration-90/report.json`.
    pub fn report(&self) -> Option<PathBuf> {
        self.work_dir().map(|w| w.join("report.json"))
    }
}

/// How a pass ended.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// The marker was there: nothing done.
    #[default]
    AlreadyDone,
    /// The old folders hold nothing: nothing to copy.
    NothingToMove,
    /// Everything copied and verified: the new folders are used.
    Copied,
    /// The new folders already held data: nothing copied, the new folders are used.
    NewInUse,
    /// The copy did not finish: the old folders are used, tried again at the next start.
    Failed,
}

/// One folder copied.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Copied {
    pub old: String,
    pub new: String,
    pub files: usize,
    pub bytes: u64,
}

/// What a pass did. Written to `report.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Unix ms of the pass.
    pub ran_at: u64,
    pub kind: Kind,
    pub copied: Vec<Copied>,
    /// New folders that already held data (with [`Kind::NewInUse`]).
    pub in_use: Vec<String>,
    /// What failed (technical text).
    pub errors: Vec<String>,
    /// The marker is written: the copy will not run again.
    pub done: bool,
}

/// What the app does with a pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// This run uses the old folders (the copy did not finish).
    pub use_old: bool,
    /// What the user should know.
    pub problem: Option<ErrorCode>,
    pub report: Report,
}

/// Whether the copy already ended for good.
pub fn is_done(plan: &Plan) -> bool {
    plan.marker().is_some_and(|m| m.is_file())
}

/// Runs one pass (see the module comment).
pub fn run(plan: &Plan, now_ms: u64) -> Outcome {
    let mut report = Report {
        ran_at: now_ms,
        ..Report::default()
    };
    if plan.folders.is_empty() || is_done(plan) {
        return Outcome {
            use_old: false,
            problem: None,
            report,
        };
    }
    let folders = distinct(&plan.folders);
    let skip: BTreeSet<&str> = plan.skip.iter().map(String::as_str).collect();
    let first_old = folders[0].old.to_string_lossy().into_owned();

    let mut old_has_data = Vec::new();
    for f in &folders {
        match holds_data(&f.old, &skip) {
            Ok(has) => old_has_data.push(has),
            Err(e) => {
                report.errors.push(format!("{}: {e}", f.old.display()));
                return failed(plan, report, first_old);
            }
        }
    }
    if !old_has_data.contains(&true) {
        log::info!("data move: the old folders hold nothing, nothing to copy");
        report.kind = Kind::NothingToMove;
        return finish(plan, report, None);
    }

    let mut ours = Vec::new();
    // A new folder that is not in use while its old one holds data: using the new folders
    // would leave that data behind.
    let mut would_lose = Vec::new();
    for (f, old_has) in folders.iter().zip(&old_has_data) {
        match state_of(&f.new, &skip) {
            Ok(NewState::InUse) => report.in_use.push(f.new.to_string_lossy().into_owned()),
            Ok(state) => {
                if matches!(state, NewState::Ours) {
                    ours.push(f.new.clone());
                }
                if *old_has {
                    would_lose.push(f.new.to_string_lossy().into_owned());
                }
            }
            Err(e) => {
                report.errors.push(format!("{}: {e}", f.new.display()));
                return failed(plan, report, first_old);
            }
        }
    }
    if let Some(new) = report.in_use.first().cloned() {
        if !would_lose.is_empty() {
            // Neither a whole copy nor the new folders as they are: the user decides.
            report.errors.push(format!(
                "{new} already holds data while {} has none: nothing copied; move the folders by hand",
                would_lose.join(", ")
            ));
            return failed(plan, report, first_old);
        }
        log::warn!(
            "data move: {new} already holds data: nothing copied, the old folders stay untouched"
        );
        report.kind = Kind::NewInUse;
        let problem = ErrorCode::DataMoveConflict {
            old: first_old,
            new,
            report: report_path(plan),
        };
        return finish(plan, report, Some(problem));
    }

    // An earlier pass's unfinished copy: the app never used it.
    for dir in &ours {
        log::info!(
            "data move: removing the unfinished copy in {}",
            dir.display()
        );
        if let Err(e) = fs::remove_dir_all(dir) {
            report.errors.push(format!("{}: {e}", dir.display()));
            return failed(plan, report, first_old);
        }
    }

    for f in &folders {
        let mut copy = Copy {
            skip: &skip,
            copied: Copied {
                old: f.old.to_string_lossy().into_owned(),
                new: f.new.to_string_lossy().into_owned(),
                ..Copied::default()
            },
            errors: Vec::new(),
        };
        let stamped = write_atomic(&f.new.join(STAMP), f.old.to_string_lossy().as_bytes());
        match stamped {
            Ok(()) => copy.dir(&f.old, &f.new, true),
            Err(e) => copy.errors.push(format!("{}: {e}", f.new.display())),
        }
        log::info!(
            "data move: {} → {}: {} files, {} bytes, {} errors",
            f.old.display(),
            f.new.display(),
            copy.copied.files,
            copy.copied.bytes,
            copy.errors.len()
        );
        report.copied.push(copy.copied);
        report.errors.extend(copy.errors);
    }
    if !report.errors.is_empty() {
        return failed(plan, report, first_old);
    }
    report.kind = Kind::Copied;
    let outcome = finish(plan, report, None);
    if outcome.use_old {
        return outcome;
    }
    for f in &folders {
        if let Err(e) = fs::remove_file(f.new.join(STAMP)) {
            log::warn!("data move: stamp in {} not removed: {e}", f.new.display());
        }
    }
    outcome
}

/// The folders, each pair once.
fn distinct(folders: &[Folder]) -> Vec<Folder> {
    let mut seen = Vec::new();
    for f in folders {
        if !seen.contains(f) {
            seen.push(f.clone());
        }
    }
    seen
}

/// Writes the report and the marker; a marker that cannot be written makes the pass a failure.
fn finish(plan: &Plan, mut report: Report, problem: Option<ErrorCode>) -> Outcome {
    let marked = plan
        .marker()
        .ok_or_else(|| io::Error::other("no folder"))
        .and_then(|m| write_atomic(&m, b"90\n"));
    if let Err(e) = marked {
        report.errors.push(format!("marker not written: {e}"));
        let old = plan
            .folders
            .first()
            .map(|f| f.old.to_string_lossy().into_owned())
            .unwrap_or_default();
        return failed(plan, report, old);
    }
    report.done = true;
    write_report(plan, &report);
    Outcome {
        use_old: false,
        problem,
        report,
    }
}

/// The pass did not finish: the old folders are used.
fn failed(plan: &Plan, mut report: Report, old: String) -> Outcome {
    report.kind = Kind::Failed;
    report.done = false;
    for e in &report.errors {
        log::warn!("data move: {e}");
    }
    write_report(plan, &report);
    // The errors themselves (OS texts with paths in them) are in the log and the report.
    Outcome {
        use_old: true,
        problem: Some(ErrorCode::DataMoveFailed {
            old,
            report: report_path(plan),
        }),
        report,
    }
}

fn write_report(plan: &Plan, report: &Report) {
    let written = plan
        .report()
        .ok_or_else(|| io::Error::other("no folder"))
        .and_then(|path| {
            let bytes = serde_json::to_vec_pretty(report).map_err(io::Error::other)?;
            write_atomic(&path, &bytes)
        });
    if let Err(e) = written {
        log::warn!("data move: report not written: {e}");
    }
}

fn report_path(plan: &Plan) -> String {
    plan.report()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Whether a name at any level is a temporary file of [`write_atomic`] (an interrupted write):
/// not data, not copied.
fn is_tmp(name: &str) -> bool {
    name.ends_with(TMP_SUFFIX)
}

/// Whether `dir` holds anything but the skipped names, the work folder, the stamp and temporary
/// files. A missing folder holds nothing.
fn holds_data(dir: &Path, skip: &BTreeSet<&str>) -> io::Result<bool> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let name = entry?.file_name();
        let name = name.to_string_lossy();
        if !(skip.contains(name.as_ref()) || name == WORK_DIR || name == STAMP || is_tmp(&name)) {
            return Ok(true);
        }
    }
    Ok(false)
}

enum NewState {
    /// Missing, or nothing but what the move ignores.
    Free,
    /// Stamped: an earlier pass's unfinished copy.
    Ours,
    /// Holds data the move did not put there.
    InUse,
}

fn state_of(dir: &Path, skip: &BTreeSet<&str>) -> io::Result<NewState> {
    if dir.join(STAMP).is_file() {
        return Ok(NewState::Ours);
    }
    Ok(if holds_data(dir, skip)? {
        NewState::InUse
    } else {
        NewState::Free
    })
}

struct Copy<'a> {
    skip: &'a BTreeSet<&'a str>,
    copied: Copied,
    errors: Vec<String>,
}

impl Copy<'_> {
    /// Copies the entries of `from` into `to` (created); `top`: the skipped names apply.
    fn dir(&mut self, from: &Path, to: &Path, top: bool) {
        if let Err(e) = fs::create_dir_all(to) {
            return self.errors.push(format!("{}: {e}", to.display()));
        }
        let mut entries = match fs::read_dir(from) {
            Ok(entries) => match entries.collect::<io::Result<Vec<_>>>() {
                Ok(list) => list,
                Err(e) => return self.errors.push(format!("{}: {e}", from.display())),
            },
            Err(e) => return self.errors.push(format!("{}: {e}", from.display())),
        };
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let name = entry.file_name();
            let text = name.to_string_lossy();
            if (top && self.skip.contains(text.as_ref())) || is_tmp(&text) {
                continue;
            }
            let src = entry.path();
            let dst = to.join(&name);
            // `DirEntry::file_type` does not follow links.
            match entry.file_type() {
                Ok(t) if t.is_symlink() => self.errors.push(format!(
                    "{}: a link (symbolic link or junction), not followed: copy what it points to by hand",
                    src.display()
                )),
                Ok(t) if t.is_dir() => self.dir(&src, &dst, false),
                Ok(t) if t.is_file() => match copy_verified(&src, &dst) {
                    Ok(bytes) => {
                        self.copied.files += 1;
                        self.copied.bytes += bytes;
                    }
                    Err(e) => self.errors.push(format!("{}: {e}", src.display())),
                },
                Ok(_) => self
                    .errors
                    .push(format!("{}: not a file or a folder", src.display())),
                Err(e) => self.errors.push(format!("{}: {e}", src.display())),
            }
        }
        sync_dir(to);
    }
}

/// Copies `from` to `to`, flushes the copy to the disk and checks its size and MD5 against the
/// original's. The bytes copied.
fn copy_verified(from: &Path, to: &Path) -> io::Result<u64> {
    let original = fingerprint(from)?;
    fs::copy(from, to)?;
    // Flushing needs write access on Windows.
    OpenOptions::new().write(true).open(to)?.sync_all()?;
    let copy = fingerprint(to)?;
    if copy != original {
        return Err(io::Error::other(format!(
            "the copy differs from the original ({} bytes, {} expected)",
            copy.0, original.0
        )));
    }
    Ok(original.0)
}

/// Size and MD5 of a file, read in blocks (a file may be several GB: an SD card image).
fn fingerprint(path: &Path) -> io::Result<(u64, [u8; 16])> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = vec![0u8; 1 << 16];
    let mut size = 0u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }
    Ok((size, hasher.finalize().into()))
}

#[cfg(test)]
mod tests;
