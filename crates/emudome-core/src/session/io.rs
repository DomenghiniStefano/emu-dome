//! The session's file writes off the emulation thread (Trello #79).
//!
//! The emulation thread still serializes (only it may call into the core) and copies what must be
//! written: a state's bytes, the in-game save's bytes, the frame's pixels (an `Arc`). The rest
//! (reading what a slot held for «Annulla», the atomic writes, the thumbnail's PNG, the
//! screenshot) runs here, one job after the other in the order sent, and the events
//! (`stateSaved`, `saveUndone`, `screenshotSaved`, the errors) are sent from here once the files
//! are written.
//!
//! The emulation thread waits for this worker only where order matters: before reading a slot
//! (a load right after a save reads the new file), before the writes of [`super::Command::Flush`]
//! and of closing (an older write must never land over a newer one), and at the end of the
//! session. A session that was let go stops waiting: the worker sees the session dead and
//! writes nothing more.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Sender, SyncSender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::SystemTime;

use super::{Event, Outbox, Shared, Writer, AUTO_SLOT};
use crate::core::GameFiles;
use crate::error::ErrorCode;
use crate::frame::Frame;
use crate::saves::{SlotContent, SlotMeta};

/// One piece of work for the worker.
pub(super) enum Job {
    /// A quick save (the auto quick save when `slot` is [`AUTO_SLOT`]): the state serialized on
    /// the emulation thread and the frame on screen when it was asked for.
    Slot {
        slot: u8,
        state: Vec<u8>,
        frame: Frame,
    },
    /// The in-game save's bytes (the periodic flush).
    Sram(Vec<u8>),
    /// «Salvato nello slot N · Annulla»: the slot of the last save gets back what it held.
    UndoSave,
    /// A load replaced the save's «Annulla»: what the slot held is let go.
    ForgetUndo,
    /// The game's quick slot, already clamped ([`crate::quick_slot::write`]).
    QuickSlot(u8),
    /// G13: the frame as a PNG in the game's screenshots folder, named after `at`.
    Screenshot {
        frame: Frame,
        dir: PathBuf,
        at: SystemTime,
    },
    /// Answered once every job sent before it is done.
    Sync(SyncSender<()>),
}

/// What the worker needs, and the save's «Annulla» it keeps.
pub(super) struct Worker {
    files: GameFiles,
    dead: Arc<AtomicBool>,
    events: Outbox,
    shared: Arc<Shared>,
    meta: SlotMeta,
    /// What the slot of the last quick save held before it (only after a save that was written).
    undo: Option<(u8, SlotContent)>,
}

impl Worker {
    pub(super) fn new(
        files: GameFiles,
        dead: Arc<AtomicBool>,
        events: Outbox,
        shared: Arc<Shared>,
        meta: SlotMeta,
    ) -> Self {
        Self {
            files,
            dead,
            events,
            shared,
            meta,
            undo: None,
        }
    }

    /// The same worker with nothing kept: the fallback when the thread cannot start.
    fn fresh(&self) -> Self {
        Self::new(
            self.files.clone(),
            self.dead.clone(),
            self.events.clone(),
            self.shared.clone(),
            self.meta.clone(),
        )
    }

    fn writer(&self) -> Writer<'_> {
        Writer {
            files: &self.files,
            dead: &self.dead,
        }
    }

    fn send(&self, event: Event) {
        let _ = self.events.send(event);
    }

    fn run(&mut self, job: Job) {
        match job {
            Job::Slot { slot, state, frame } => self.slot(slot, &state, &frame),
            Job::Sram(bytes) => {
                // A failure was logged; the next flush tries again.
                let _ = self.writer().sram_bytes(&bytes);
            }
            Job::UndoSave => {
                let event = match self.undo.take() {
                    Some((slot, before)) => match before.restore(&self.files, slot) {
                        Ok(()) => Event::SaveUndone { slot },
                        Err(e) => Event::Error {
                            error: ErrorCode::UndoFailed {
                                detail: e.to_string(),
                            },
                        },
                    },
                    None => Event::Error {
                        error: ErrorCode::NothingToUndo,
                    },
                };
                self.send(event);
            }
            Job::ForgetUndo => self.undo = None,
            Job::QuickSlot(slot) => {
                // A session let go writes nothing more.
                if self.writer().alive() {
                    if let Err(e) = crate::quick_slot::write(&self.files, slot) {
                        log::warn!("quick slot {slot} not recorded: {e}");
                        self.send(Event::Error {
                            error: ErrorCode::SaveFileFailed {
                                detail: format!("quick-slot.json: {e}"),
                            },
                        });
                    }
                }
            }
            Job::Screenshot { frame, dir, at } => {
                let res = frame.to_png().map_err(|e| e.to_string()).and_then(|png| {
                    crate::screenshot::write(&dir, &png, at).map_err(|e| e.to_string())
                });
                self.send(match res {
                    Ok(path) => Event::ScreenshotSaved {
                        path: path.to_string_lossy().into_owned(),
                    },
                    Err(detail) => Event::Error {
                        error: ErrorCode::ScreenshotFailed { detail },
                    },
                });
            }
            Job::Sync(done) => {
                let _ = done.send(());
            }
        }
    }

    /// A quick save: what the slot held set aside for «Annulla» (not for the auto quick save),
    /// the state, its thumbnail and its record written, then the event.
    fn slot(&mut self, slot: u8, state: &[u8], frame: &Frame) {
        let auto = slot == AUTO_SLOT;
        let before = if auto {
            None
        } else {
            // A new save: the last one's «Annulla» goes, whatever happens to this one.
            self.undo = None;
            SlotContent::read(&self.files, slot)
                .inspect_err(|e| log::warn!("slot {slot} not set aside for undo: {e}"))
                .ok()
        };
        let writer = self.writer();
        match writer.state_bytes(slot, state) {
            Ok(written) => {
                if written {
                    writer.thumb_of(slot, frame);
                    if let Err(e) = crate::saves::write_slot_meta(&self.files, slot, &self.meta) {
                        log::warn!("slot {slot} record not written: {e}");
                    }
                    if auto {
                        self.shared.stats.lock().auto_saved_at = Some(SystemTime::now());
                    } else {
                        self.undo = before.map(|before| (slot, before));
                    }
                }
                if !auto {
                    self.send(Event::StateSaved { slot });
                }
            }
            Err(e) => self.send(Event::Error {
                error: ErrorCode::StateSaveFailed {
                    slot,
                    detail: format!("{e:#}"),
                },
            }),
        }
    }
}

/// The worker thread, or the worker itself when no thread could start (then every job runs on
/// the emulation thread, as before #79).
pub(super) enum Io {
    Thread {
        tx: Sender<Job>,
        thread: JoinHandle<()>,
    },
    Inline(Box<Worker>),
    /// After [`Io::finish`]: the session's exit writes run on the emulation thread itself.
    Finished,
}

impl Io {
    pub(super) fn start(worker: Worker) -> Self {
        let fallback = worker.fresh();
        let (tx, rx) = mpsc::channel::<Job>();
        let mut worker = worker;
        let spawned = std::thread::Builder::new()
            .name("emudome-io".into())
            .spawn(move || {
                for job in rx {
                    worker.run(job);
                }
            });
        match spawned {
            Ok(thread) => Io::Thread { tx, thread },
            Err(e) => {
                log::warn!("save worker not started, writing on the emulation thread: {e}");
                Io::Inline(Box::new(fallback))
            }
        }
    }

    pub(super) fn send(&mut self, job: Job) {
        match self {
            Io::Thread { tx, .. } => {
                // The worker is gone only if it panicked (a release build aborts instead): the
                // job is lost and said so; closing still writes through `write_on_exit`.
                if tx.send(job).is_err() {
                    log::error!("save worker gone: a write was not carried out");
                }
            }
            Io::Inline(worker) => worker.run(job),
            Io::Finished => log::error!("a write sent after the session's end was dropped"),
        }
    }

    /// Waits until every job sent so far is done.
    pub(super) fn sync(&mut self) {
        if let Io::Thread { tx, .. } = self {
            let (done, wait) = mpsc::sync_channel(1);
            if tx.send(Job::Sync(done)).is_ok() {
                let _ = wait.recv();
            }
        }
    }

    /// The end of the session: every job done, the thread joined.
    pub(super) fn finish(self) {
        if let Io::Thread { tx, thread } = self {
            drop(tx);
            if thread.join().is_err() {
                log::error!("the save worker panicked");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, Receiver};
    use std::sync::Arc;

    use super::{Io, Job, Worker};
    use crate::core::GameFiles;
    use crate::error::ErrorCode;
    use crate::frame::Frame;
    use crate::saves::SlotMeta;
    use crate::session::{Event, Outbox, Shared, AUTO_SLOT};
    use crate::settings::test_dir::TestDir;

    fn worker(dir: &TestDir) -> (Io, Receiver<Event>, GameFiles, Arc<AtomicBool>) {
        let files = GameFiles::new(
            &dir.path("saves"),
            &crate::GameKey::parse("gba:9a3e").unwrap(),
        );
        std::fs::create_dir_all(files.dir()).unwrap();
        let (tx, rx) = mpsc::channel();
        let dead = Arc::new(AtomicBool::new(false));
        let outbox = Outbox {
            tx,
            dead: dead.clone(),
        };
        let meta = SlotMeta {
            core: "mGBA".into(),
            core_version: "0.11".into(),
            app_version: "test".into(),
        };
        let io = Io::start(Worker::new(
            files.clone(),
            dead.clone(),
            outbox,
            Arc::new(Shared::default()),
            meta,
        ));
        (io, rx, files, dead)
    }

    fn frame() -> Frame {
        Frame {
            width: 1,
            height: 1,
            rgba: Arc::new(vec![255, 0, 255, 255]),
            seq: 1,
            split: crate::screens::Split::Whole,
        }
    }

    fn slot(slot: u8, state: &[u8]) -> Job {
        Job::Slot {
            slot,
            state: state.to_vec(),
            frame: frame(),
        }
    }

    #[test]
    fn a_quick_save_is_written_with_its_picture_and_record_then_said() {
        let dir = TestDir::new("io-save");
        let (mut io, rx, files, _) = worker(&dir);
        io.send(slot(2, b"state two"));
        io.sync();
        assert_eq!(std::fs::read(files.slot(2)).unwrap(), b"state two");
        assert!(files.thumb(2).is_file());
        assert!(files.meta(2).is_file());
        assert!(matches!(rx.try_recv(), Ok(Event::StateSaved { slot: 2 })));
        // The auto quick save says nothing (the stats record it).
        io.send(slot(AUTO_SLOT, b"auto"));
        io.finish();
        assert_eq!(std::fs::read(files.slot(AUTO_SLOT)).unwrap(), b"auto");
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn undoing_a_save_puts_back_what_the_slot_held() {
        let dir = TestDir::new("io-undo");
        let (mut io, rx, files, _) = worker(&dir);
        io.send(slot(1, b"old"));
        io.send(slot(1, b"new"));
        io.send(Job::UndoSave);
        io.sync();
        assert_eq!(std::fs::read(files.slot(1)).unwrap(), b"old");
        let events: Vec<Event> = rx.try_iter().collect();
        assert!(matches!(events.last(), Some(Event::SaveUndone { slot: 1 })));
        // Once only; and a load in between forgets it.
        io.send(Job::UndoSave);
        io.send(slot(1, b"newer"));
        io.send(Job::ForgetUndo);
        io.send(Job::UndoSave);
        io.finish();
        let errors = rx
            .try_iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::Error {
                        error: ErrorCode::NothingToUndo
                    }
                )
            })
            .count();
        assert_eq!(errors, 2);
        assert_eq!(std::fs::read(files.slot(1)).unwrap(), b"newer");
    }

    #[test]
    fn a_dead_sessions_worker_writes_nothing() {
        let dir = TestDir::new("io-dead");
        let (mut io, rx, files, dead) = worker(&dir);
        dead.store(true, Ordering::SeqCst);
        io.send(slot(3, b"late"));
        io.send(Job::Sram(b"late save".to_vec()));
        io.finish();
        assert!(!files.slot(3).exists());
        assert!(!files.sram.exists());
        assert!(rx.try_recv().is_err());
    }
}
