//! A game in progress: emulation thread, frame pacing, commands.
//!
//! The emulation thread runs an [`emu_loop::EmuLoop`] on a [`Core`] (the libretro one,
//! [`libretro`], or a fake in the tests); it serializes, the writes go to the session's I/O
//! worker ([`io`]).

mod emu_loop;
mod io;
mod libretro;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SendError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use libretro_sys::retro_system_av_info;
use parking_lot::Mutex;

pub use emu_loop::{Core, CoreInfo};

use crate::audio::AudioRing;
use crate::core::{GameFiles, StartError};
use crate::error::ErrorCode;
use crate::frame::Frame;
use crate::input::InputState;
use crate::speed_audio::FastAudio;

/// Emulation speed.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum Speed {
    /// Multiplier of the original speed (1.0 = normal).
    Factor(f32),
    /// As fast as possible.
    Max,
}

impl Speed {
    /// A speed option of the settings (`speed.ffval`, `speed.slow`, `speed.spstart`): `max`, or
    /// a positive factor such as `0.5` or `4`. `None` for anything else.
    pub fn from_option(id: &str) -> Option<Self> {
        if id == "max" {
            return Some(Speed::Max);
        }
        id.parse::<f32>()
            .ok()
            .filter(|f| f.is_finite() && *f > 0.0)
            .map(Speed::Factor)
    }

    fn is_normal(self) -> bool {
        matches!(self, Speed::Factor(f) if (f - 1.0).abs() < f32::EPSILON)
    }
}

#[derive(Debug)]
pub enum Command {
    SetSpeed(Speed),
    SetPaused(bool),
    SaveState(u8),
    LoadState(u8),
    /// «Annulla il caricamento» (`saves.undo`): back to the state from before the last load.
    UndoLoad,
    /// «Salvato nello slot N · Annulla»: the slot of the last save gets back what it held.
    UndoSave,
    /// Changes a core option (`GET_VARIABLE` key and value) while the game runs.
    SetCoreOption {
        key: String,
        value: String,
    },
    Reset,
    /// Rewind on (held, or toggled with `a11y.toggle`) or off. With `speed.rew` off, or nothing
    /// recorded yet, the game only stays on its picture until it is released.
    Rewind(bool),
    /// `speed.rew` / `speed.rewlen` changed: seconds to keep, `None` = rewind off.
    SetRewindLength(Option<u32>),
    /// `audio.ff` changed: what the audio does at speeds other than 1×.
    SetFastAudio(FastAudio),
    /// Saves the frame on screen as a PNG next to the saves ([`crate::screenshot`]).
    Screenshot,
    /// Writes what closing would, without closing: the in-game save, and the auto quick save
    /// when `saves.autoexit` is on (Android going to the background, where the process may be
    /// killed without another word).
    Flush,
    Stop,
}

/// Events the emulation thread sends to the shell.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Event {
    Started {
        core: String,
        game: String,
        fps: f64,
        width: u32,
        height: u32,
    },
    StateSaved {
        slot: u8,
    },
    StateLoaded {
        slot: u8,
    },
    /// The last load was undone.
    LoadUndone,
    /// The last save was undone: `slot` holds what it held before.
    SaveUndone {
        slot: u8,
    },
    /// G13 «Schermata salvata»: `path` is the PNG written.
    ScreenshotSaved {
        path: String,
    },
    Error {
        error: ErrorCode,
    },
    Stopped,
}

/// The auto quick save («Salvataggio rapido automatico»): written periodically
/// (`saves.autoq`) and on close (`saves.autoexit`), loaded to resume (`saves.resume`).
pub const AUTO_SLOT: u8 = 0;

/// Statistics read by the interface (about once a second).
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub av_info: retro_system_av_info,
    pub speed: Option<Speed>,
    pub paused: bool,
    /// Frames emulated in the last second.
    pub emu_fps: f64,
    /// Frames emulated since the start: a watchdog sees a core that stopped answering.
    pub frames: u64,
    /// When the auto quick save was last written in this session.
    pub auto_saved_at: Option<std::time::SystemTime>,
    /// Rewind is playing back.
    pub rewinding: bool,
    /// States the rewind buffer holds, and their bytes.
    pub rewind_states: usize,
    pub rewind_bytes: usize,
}

/// State shared by emulation, renderer, audio and interface.
#[derive(Default)]
pub struct Shared {
    pub frame: Mutex<Frame>,
    /// The audio queue: lock-free between the emulation thread and the audio output.
    pub audio: AudioRing,
    pub input: InputState,
    pub stats: Mutex<Stats>,
    /// «Muto» in the quick menu: the audio output plays silence for this game.
    pub muted: std::sync::atomic::AtomicBool,
    /// The options the core declared (empty until it does), sorted by key.
    pub core_options: Mutex<Vec<CoreOptionInfo>>,
    /// What the core said its states cannot do (`SET_SERIALIZATION_QUIRKS`,
    /// `RETRO_SERIALIZATION_QUIRK_*` bits); 0 until it says.
    pub serialization_quirks: std::sync::atomic::AtomicU64,
    /// The emulation thread's work per frame in µs (commands, `retro_run`, rewind, saves; not
    /// the pacing sleep), recorded only while `Some`: the headless example's `--timing`.
    pub frame_times: Mutex<Option<Vec<u32>>>,
    /// Woken at each new frame: the native renderer registers here and presents only when
    /// something changed (Trello #79).
    pub frame_wake: Mutex<Option<Arc<crate::video_sink::Wake>>>,
}

/// One option as the running core declares it (`SET_VARIABLES` / `SET_CORE_OPTIONS*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreOptionInfo {
    pub key: String,
    /// The core's own label, in English (not for the interface: it shows `settings.*` texts).
    pub label: String,
    /// Accepted values, in the core's order (empty = any).
    pub values: Vec<String>,
    pub default: String,
}

pub struct SessionOptions {
    pub core_path: PathBuf,
    pub game_path: PathBuf,
    /// Folder of the BIOS and system files (empty is fine: the cores have replacements).
    pub system_dir: PathBuf,
    /// The saves root: the game's own files are in its folder under it ([`GameFiles`]), which is
    /// also the core's save directory.
    pub save_dir: PathBuf,
    /// The game's one identity: names its saves.
    pub game: crate::GameKey,
    /// Auto quick save this often while the game runs (`saves.autoq`; None = never).
    pub autosave_every: Option<Duration>,
    /// Auto quick save when the game closes (`saves.autoexit`).
    pub autosave_on_exit: bool,
    /// Load the auto quick save right after the game starts (`saves.resume` = always).
    pub resume_auto: bool,
    /// EmuDome's version, recorded beside every quick save with the core's
    /// ([`crate::saves::SlotMeta`]).
    pub app_version: String,
    /// Core options the shell chooses (`GET_VARIABLE` key and value, e.g.
    /// `citra_graphics_api` = `Software`). A value the core does not accept falls back to
    /// the core's default; a key not listed answers the core's default.
    pub core_options: Vec<(String, String)>,
    /// Rewind (`speed.rew`): the seconds to keep (`speed.rewlen`), `None` = off.
    pub rewind_seconds: Option<u32>,
    /// How the core lays its screens out in the frame ([`crate::screens::Split`]): whole for
    /// one-screen consoles, [`crate::nds::SPLIT`] for melonDS DS with its forced options.
    pub split: crate::screens::Split,
    /// The user's name for the console, answered to `GET_USERNAME` (`consoles.dsname`; melonDS
    /// DS writes it in the DS firmware). `None`: unanswered, the core guesses (the OS user).
    pub username: Option<String>,
    /// The audio at speeds other than 1× (`audio.ff`).
    pub fast_audio: FastAudio,
    /// Where the frames go and how hardware rendering is answered; default: today's (RGBA8 in
    /// [`Shared::frame`], hardware rendering refused).
    pub video: VideoSetup,
}

/// The session's video seam ([`crate::video_sink`], [`crate::hw_render`]); `None` = the default.
#[derive(Default)]
pub struct VideoSetup {
    /// Makes the sink once the session's [`Shared`] exists (a GPU sink keeps its frame filled).
    pub sink: Option<SinkFactory>,
    pub hw_render: Option<Box<dyn crate::hw_render::HwRenderHandler>>,
}

/// See [`VideoSetup::sink`].
pub type SinkFactory =
    Box<dyn FnOnce(&Arc<Shared>) -> Box<dyn crate::video_sink::VideoSink> + Send>;

/// One quick save slot of a game, for the quick menu (G07).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotInfo {
    pub slot: u8,
    /// When the slot was written; None = empty.
    pub saved_at: Option<std::time::SystemTime>,
    /// The thumbnail file, when there is one.
    pub thumb: Option<PathBuf>,
}

/// The auto quick save and slots `1..=count` of a game.
pub fn slot_infos(save_dir: &std::path::Path, game: &crate::GameKey, count: u8) -> Vec<SlotInfo> {
    let files = GameFiles::new(save_dir, game);
    (0..=count)
        .map(|slot| {
            let saved_at = std::fs::metadata(files.slot(slot))
                .and_then(|m| m.modified())
                .ok();
            let thumb = files.thumb(slot);
            SlotInfo {
                slot,
                saved_at,
                thumb: (saved_at.is_some() && thumb.is_file()).then_some(thumb),
            }
        })
        .collect()
}

/// Every file the emulation thread writes goes through here, atomically
/// ([`crate::write_atomic`]). Once the session is dead (let go because it stopped answering,
/// [`Session::abandon`]) nothing is written any more: its files may belong to a newer session.
struct Writer<'a> {
    files: &'a GameFiles,
    dead: &'a AtomicBool,
}

impl Writer<'_> {
    fn alive(&self) -> bool {
        !self.dead.load(Ordering::SeqCst)
    }

    /// The in-game save (`.sav`). A failure is logged and returned: the next flush tries again,
    /// and closing the game reports it ([`Session::wait`]).
    fn sram(&self, core: &impl Core) -> Result<(), ErrorCode> {
        match core.sram() {
            Some(ram) => self.sram_bytes(ram),
            None => Ok(()),
        }
    }

    /// [`Self::sram`] from bytes already copied out of the core (the I/O worker).
    fn sram_bytes(&self, ram: &[u8]) -> Result<(), ErrorCode> {
        if !self.alive() {
            return Ok(());
        }
        if let Err(e) = crate::write_atomic(&self.files.sram, ram) {
            log::warn!("game save not written: {e}");
            return Err(ErrorCode::SaveFileFailed {
                detail: format!("{e:#}"),
            });
        }
        Ok(())
    }

    /// The state of the game into `slot`. `Ok(false)`: not written, the session is dead.
    fn state(&self, core: &impl Core, slot: u8) -> Result<bool> {
        let data = core.serialize()?;
        self.state_bytes(slot, &data)
    }

    /// [`Self::state`] from a state already serialized (the I/O worker).
    fn state_bytes(&self, slot: u8, data: &[u8]) -> Result<bool> {
        if !self.alive() {
            return Ok(false);
        }
        crate::write_atomic(&self.files.slot(slot), data).context("cannot write the slot")?;
        Ok(true)
    }

    /// The thumbnail of a slot from the frame on screen (failures only logged: a slot without a
    /// picture still loads).
    fn thumb(&self, slot: u8, shared: &Shared) {
        // Encoded from a clone (the pixels' `Arc`): the lock is not held while encoding.
        let frame = shared.frame.lock().clone();
        self.thumb_of(slot, &frame);
    }

    /// [`Self::thumb`] from a frame taken when the save was asked for.
    fn thumb_of(&self, slot: u8, frame: &Frame) {
        let png = frame.to_png();
        match png {
            Ok(bytes) if self.alive() => {
                if let Err(e) = crate::write_atomic(&self.files.thumb(slot), &bytes) {
                    log::warn!("slot {slot} thumbnail not written: {e}");
                }
            }
            Ok(_) => {}
            Err(e) => log::warn!("slot {slot} thumbnail not encoded: {e}"),
        }
    }
}

/// Closing the game (and [`Command::Flush`]): the in-game save first, the smallest and the one
/// that matters most, then the auto quick save when `saves.autoexit` is on. Both are tried; the
/// first failure is returned.
fn write_on_exit<C: Core>(
    core: &C,
    writer: &Writer<'_>,
    autosave_on_exit: bool,
    save_auto: impl FnOnce(&C) -> Result<(), ErrorCode>,
) -> Result<(), ErrorCode> {
    let sram = writer.sram(core);
    let auto = if autosave_on_exit {
        save_auto(core)
    } else {
        Ok(())
    };
    sram.and(auto)
}

/// The emulation thread's events: dropped once the session is dead, so a game that was let go
/// never speaks over the next one.
#[derive(Clone)]
struct Outbox {
    tx: Sender<Event>,
    dead: Arc<AtomicBool>,
}

impl Outbox {
    fn send(&self, event: Event) -> Result<(), SendError<Event>> {
        if self.dead.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.tx.send(event)
    }
}

/// How long closing a game waits for its thread, exit writes included, before letting it go
/// as stuck (item 5 of Trello #69): enough for a 3DS state of tens of MB on a slow disk, short
/// enough that a stuck core does not hang the window.
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(10);

/// How closing a session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closed {
    /// The thread ended: its exit writes are done.
    Joined,
    /// The thread did not end in time and was let go: the session is dead (it writes nothing
    /// more) and its core is refused until EmuDome restarts ([`core_needs_restart`]).
    Abandoned,
}

/// Cores whose library a thread that was let go may still be inside. The OS hands back the same
/// loaded module for the same file, so a new game on it would share its globals with that thread.
static CORES_NEEDING_RESTART: Mutex<Vec<PathBuf>> = parking_lot::const_mutex(Vec::new());

/// The key of a core file: the OS identifies a loaded module by its full path.
fn core_key(core_path: &Path) -> PathBuf {
    std::fs::canonicalize(core_path).unwrap_or_else(|_| core_path.to_path_buf())
}

/// Whether a game on this core was let go in this run of the app: the core cannot be used again
/// until EmuDome restarts (`coreNeedsRestart`).
pub fn core_needs_restart(core_path: &Path) -> bool {
    CORES_NEEDING_RESTART.lock().contains(&core_key(core_path))
}

fn mark_core_needs_restart(core_path: &Path) {
    let key = core_key(core_path);
    let mut cores = CORES_NEEDING_RESTART.lock();
    if !cores.contains(&key) {
        cores.push(key);
    }
}

/// Whether a game has an auto quick save to resume from.
pub fn has_auto_save(save_dir: &std::path::Path, game: &crate::GameKey) -> bool {
    GameFiles::new(save_dir, game).slot(AUTO_SLOT).is_file()
}

/// What the emulation thread reports when it ends: its exit writes all done, or the first one
/// that failed.
pub type ExitWrites = Result<(), ErrorCode>;

pub struct Session {
    tx: Sender<Command>,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    /// Receives the exit writes' outcome as the emulation thread ends; disconnected without it
    /// when the thread panicked (its sender dies with the thread).
    done: Receiver<ExitWrites>,
    /// Set when the session is let go: its thread writes nothing and says nothing any more.
    dead: Arc<AtomicBool>,
    core_path: PathBuf,
}

impl Session {
    /// Starts the emulation thread and waits for the game to load.
    /// Later events arrive on `events`.
    /// Refused when a game on the same core was let go in this run ([`core_needs_restart`]).
    pub fn start(opts: SessionOptions, events: Sender<Event>) -> Result<Self, StartError> {
        Self::spawn(opts, events, libretro::boot)
    }

    /// [`Session::start`] on a core already made (the tests' [`crate::testing::FakeCore`]):
    /// everything else (the thread, the files, the exit writes) is the real session's.
    pub fn start_with<C: Core + Send + 'static>(
        opts: SessionOptions,
        events: Sender<Event>,
        core: C,
    ) -> Result<Self, StartError> {
        Self::spawn(opts, events, move |_, _, _| Ok(core))
    }

    /// Starts the emulation thread, which makes its core with `boot` and waits for it to load.
    fn spawn<C: Core>(
        opts: SessionOptions,
        events: Sender<Event>,
        boot: impl FnOnce(&mut SessionOptions, &Arc<Shared>, &GameFiles) -> Result<C, StartError>
            + Send
            + 'static,
    ) -> Result<Self, StartError> {
        if core_needs_restart(&opts.core_path) {
            return Err(StartError::CoreNeedsRestart);
        }
        std::fs::create_dir_all(&opts.system_dir).ok();
        std::fs::create_dir_all(GameFiles::new(&opts.save_dir, &opts.game).dir()).ok();

        let shared = Arc::new(Shared::default());
        let dead = Arc::new(AtomicBool::new(false));
        let core_path = opts.core_path.clone();
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), StartError>>(1);
        let (done_tx, done) = mpsc::channel::<ExitWrites>();

        let thread_shared = shared.clone();
        let outbox = Outbox {
            tx: events,
            dead: dead.clone(),
        };
        let thread_dead = dead.clone();
        let thread = std::thread::Builder::new()
            .name("emudome-emu".into())
            .spawn(move || {
                // Sent when the thread ends; a panic drops it unsent and `done` disconnects.
                let written =
                    emu_thread(opts, thread_shared, rx, outbox, ready_tx, thread_dead, boot);
                let _ = done_tx.send(written);
            })
            .map_err(|e| StartError::ThreadFailed {
                detail: e.to_string(),
            })?;

        ready_rx.recv().map_err(|_| StartError::ThreadFailed {
            detail: "the emulation thread exited while starting".into(),
        })??;

        Ok(Self {
            tx,
            shared,
            thread: Some(thread),
            done,
            dead,
            core_path,
        })
    }

    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }

    pub fn shared(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// Closes the game: asks the thread to stop and waits up to `timeout` for it to end (its exit
    /// writes included). A thread still running then is let go as [`Session::abandon`] does.
    pub fn close(mut self, timeout: Duration) -> Closed {
        self.finish(timeout)
    }

    /// Asks the thread to stop: it writes the in-game save and, with `saves.autoexit`, the auto
    /// quick save, then ends. [`Session::wait`] tells when (the close flow, Trello #86).
    pub fn stop(&self) {
        let _ = self.tx.send(Command::Stop);
    }

    /// Waits up to `timeout` for the thread to end after [`Session::stop`]. `Some`: it ended and
    /// was joined, with its exit writes' outcome; `None`: still running, wait again or
    /// [`Session::abandon`]. Never lets the thread go by itself.
    pub fn wait(&mut self, timeout: Duration) -> Option<ExitWrites> {
        let thread = self.thread.as_ref()?;
        let written = match self.done.recv_timeout(timeout) {
            Ok(written) => written,
            Err(RecvTimeoutError::Timeout) => return None,
            Err(RecvTimeoutError::Disconnected) => Err(ErrorCode::SaveFileFailed {
                detail: format!("the emulation thread {:?} panicked", thread.thread().name()),
            }),
        };
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::error!("the emulation thread panicked");
            }
        }
        Some(written)
    }

    /// Lets go of an emulation thread that stopped answering (E11) without waiting for it, since
    /// joining a core stuck inside `retro_run` would block forever. The session is dead from now
    /// on: if the thread ever wakes up it writes no file, sends no event and leaves the core
    /// loaded as it is (no `retro_unload_game` / `retro_deinit`); the core is refused until
    /// EmuDome restarts.
    pub fn abandon(mut self) {
        if let Some(thread) = self.thread.take() {
            self.let_go(thread);
            let _ = self.tx.send(Command::Stop);
        }
    }

    fn finish(&mut self, timeout: Duration) -> Closed {
        let Some(thread) = self.thread.take() else {
            return Closed::Joined;
        };
        let _ = self.tx.send(Command::Stop);
        match self.done.recv_timeout(timeout) {
            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                if thread.join().is_err() {
                    log::error!("the emulation thread panicked");
                }
                Closed::Joined
            }
            Err(RecvTimeoutError::Timeout) => {
                log::warn!("the emulation thread did not stop within {timeout:?}");
                self.let_go(thread);
                Closed::Abandoned
            }
        }
    }

    /// Marks the session dead and its core as needing a restart, then detaches the thread.
    fn let_go(&self, thread: JoinHandle<()>) {
        self.dead.store(true, Ordering::SeqCst);
        mark_core_needs_restart(&self.core_path);
        log::warn!(
            "emulation thread let go: {:?}; {} refused until restart",
            thread.thread().name(),
            self.core_path.display()
        );
    }
}

impl Drop for Session {
    /// A session dropped without [`Session::close`] closes the same way, bounded by
    /// [`CLOSE_TIMEOUT`].
    fn drop(&mut self) {
        self.finish(CLOSE_TIMEOUT);
    }
}

/// The emulation thread: makes its core with `boot`, says whether it loaded, then runs the loop
/// until `Stop` (or until the session is let go). Returns the exit writes' outcome.
fn emu_thread<C: Core>(
    mut opts: SessionOptions,
    shared: Arc<Shared>,
    rx: Receiver<Command>,
    events: Outbox,
    ready: mpsc::SyncSender<Result<(), StartError>>,
    dead: Arc<AtomicBool>,
    boot: impl FnOnce(&mut SessionOptions, &Arc<Shared>, &GameFiles) -> Result<C, StartError>,
) -> ExitWrites {
    let files = GameFiles::new(&opts.save_dir, &opts.game);
    let mut core = match boot(&mut opts, &shared, &files) {
        Ok(core) => core,
        Err(e) => {
            let _ = ready.send(Err(e));
            // Nothing was loaded: nothing to write.
            return Ok(());
        }
    };
    emu_loop::load_sram(&mut core, &files);
    let _ = ready.send(Ok(()));

    let mut lp = emu_loop::EmuLoop::start(core, &opts, shared, events, dead, Instant::now());
    'main: loop {
        // Let go (abandoned or a close that timed out): stop at once, writing nothing.
        if !lp.alive() {
            break;
        }
        // Commands: paused, wait for them; otherwise check without blocking.
        let first = if lp.paused() {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(c) => Some(c),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            rx.try_recv().ok()
        };
        // The frame's work starts here: the commands it carries out count.
        let work_start = Instant::now();
        for cmd in first
            .into_iter()
            .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        {
            if !lp.alive() || lp.command(cmd, Instant::now()) == emu_loop::Flow::Stop {
                break 'main;
            }
        }
        if lp.paused() {
            continue;
        }
        lp.step(Instant::now());
        lp.record_work(work_start.elapsed());
        // Pacing: at a fixed speed wait for the next frame, at Max do not.
        if let Some(sleep) = lp.pace(Instant::now()) {
            std::thread::sleep(sleep);
        }
    }

    if !lp.alive() {
        lp.let_go();
        return Ok(());
    }
    lp.finish()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, Receiver};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::emu_loop::FpsMeter;
    use super::{
        core_needs_restart, has_auto_save, mark_core_needs_restart, slot_infos, write_on_exit,
        Closed, Command, ErrorCode, Event, ExitWrites, GameFiles, Outbox, Session, SessionOptions,
        Shared, Writer, AUTO_SLOT,
    };
    use crate::settings::test_dir::TestDir;
    use crate::testing::FakeCore;

    /// Runs `n` frames `period` apart from `start`, returning the rates the meter gave.
    fn run(meter: &mut FpsMeter, start: Instant, n: u32, period: Duration) -> Vec<f64> {
        (1..=n)
            .filter_map(|i| meter.frame(start + period * i))
            .collect()
    }

    #[test]
    fn the_fps_meter_gives_the_rate_once_a_second() {
        let period = Duration::from_secs_f64(1.0 / 59.7275);
        let t0 = Instant::now();
        let mut meter = FpsMeter::new(t0);
        let rates = run(&mut meter, t0, 600, period);
        assert_eq!(rates.len(), 10);
        assert!(
            rates.iter().all(|r| (r - 59.7275).abs() < 0.01),
            "{rates:?}"
        );
    }

    #[test]
    fn a_pause_neither_freezes_nor_dilutes_the_rate() {
        let period = Duration::from_secs_f64(1.0 / 59.7275);
        let t0 = Instant::now();
        // Half a window, then a 30 s pause; the session restarts the meter on resume.
        let resumed = t0 + period * 30 + Duration::from_secs(30);
        let mut meter = FpsMeter::new(t0);
        run(&mut meter, t0, 30, period);
        meter.restart(resumed);
        let rates = run(&mut meter, resumed, 120, period);
        assert_eq!(rates.len(), 2);
        assert!(
            rates.iter().all(|r| (r - 59.7275).abs() < 0.01),
            "{rates:?}"
        );

        // Without the restart, the first window after the pause would count the pause.
        let mut stale = FpsMeter::new(t0);
        run(&mut stale, t0, 30, period);
        let after = run(&mut stale, resumed, 1, period);
        assert!(after.len() == 1 && after[0] < 2.0, "{after:?}");
    }

    #[test]
    fn a_new_timing_from_the_core_changes_the_rate_only_when_usable() {
        use super::emu_loop::changed_fps;
        assert_eq!(changed_fps(59.7275, 30.0), Some(30.0));
        assert_eq!(changed_fps(59.7275, 59.7275), None);
        assert_eq!(changed_fps(59.7275, 0.0), None);
        assert_eq!(changed_fps(59.7275, f64::NAN), None);
    }

    #[test]
    fn the_auto_quick_save_has_its_own_file() {
        let files = GameFiles::new(Path::new("saves"), &key());
        let dir = Path::new("saves").join("gba").join("9a3e");
        assert_eq!(files.slot(AUTO_SLOT), dir.join("auto.state"));
        assert_eq!(files.slot(1), dir.join("slot1.state"));
        assert_eq!(files.thumb(3), dir.join("slot3.png"));
        assert_eq!(files.sram, dir.join("game.sav"));
        let empty = slot_infos(Path::new("no-such-dir"), &key(), 3);
        assert_eq!(empty.len(), 4);
        assert!(empty
            .iter()
            .all(|s| s.saved_at.is_none() && s.thumb.is_none()));
        assert!(!has_auto_save(Path::new("no-such-dir"), &key()));
    }

    // ---- Exit writes, dead sessions, bounded close (Trello #69) ----

    fn key() -> crate::GameKey {
        crate::GameKey::parse("gba:9a3e").unwrap()
    }

    /// A core at frame 7: its in-game save and its state both hold 7.
    fn stub() -> FakeCore {
        let mut core = FakeCore::new();
        for _ in 0..7 {
            crate::session::Core::run(&mut core);
        }
        core
    }

    /// What the fake core's frame 7 looks like on disk.
    fn seven() -> Vec<u8> {
        7u64.to_le_bytes().to_vec()
    }

    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn closing_a_game_writes_the_in_game_save_and_the_auto_quick_save() {
        let dir = TestDir::new("session-exit");
        let files = GameFiles::new(&dir.path("saves"), &key());
        let dead = AtomicBool::new(false);
        let writer = Writer {
            files: &files,
            dead: &dead,
        };
        let core = stub();
        let written = write_on_exit(&core, &writer, true, |c| {
            assert!(writer.state(c, AUTO_SLOT).unwrap());
            Ok(())
        });
        assert_eq!(written, Ok(()));
        assert_eq!(std::fs::read(&files.sram).unwrap(), seven());
        assert_eq!(std::fs::read(files.slot(AUTO_SLOT)).unwrap(), seven());
        assert_eq!(
            files_in(&dir.path("saves/gba/9a3e")),
            vec!["auto.state".to_owned(), "game.sav".to_owned()]
        );
    }

    #[test]
    fn without_save_on_exit_closing_writes_only_the_in_game_save() {
        let dir = TestDir::new("session-exit-no-auto");
        let files = GameFiles::new(&dir.path("saves"), &key());
        let dead = AtomicBool::new(false);
        let writer = Writer {
            files: &files,
            dead: &dead,
        };
        let written = write_on_exit(&stub(), &writer, false, |_| {
            panic!("the auto quick save is off")
        });
        assert_eq!(written, Ok(()));
        assert_eq!(
            files_in(&dir.path("saves/gba/9a3e")),
            vec!["game.sav".to_owned()]
        );
    }

    #[test]
    fn a_failed_exit_write_is_reported_and_the_other_one_still_tried() {
        let dir = TestDir::new("session-exit-fails");
        // `saves` is a file: the in-game save cannot be written under it.
        dir.write("saves", "not a folder");
        let files = GameFiles::new(&dir.path("saves"), &key());
        let dead = AtomicBool::new(false);
        let writer = Writer {
            files: &files,
            dead: &dead,
        };
        let mut auto_tried = false;
        let written = write_on_exit(&stub(), &writer, true, |_| {
            auto_tried = true;
            Ok(())
        });
        assert!(auto_tried, "the auto quick save was skipped");
        assert!(matches!(written, Err(ErrorCode::SaveFileFailed { .. })));

        // The auto quick save failing alone is reported too.
        let ok_dir = TestDir::new("session-exit-auto-fails");
        let files = GameFiles::new(&ok_dir.path("saves"), &key());
        let writer = Writer {
            files: &files,
            dead: &dead,
        };
        let failed = ErrorCode::StateSaveFailed {
            slot: AUTO_SLOT,
            detail: "disk full".into(),
        };
        let written = write_on_exit(&stub(), &writer, true, |_| Err(failed.clone()));
        assert_eq!(written, Err(failed));
        assert_eq!(std::fs::read(&files.sram).unwrap(), seven());
    }

    #[test]
    fn a_dead_session_writes_nothing() {
        let dir = TestDir::new("session-dead");
        dir.write("saves/gba/9a3e/game.sav", "the newer session's save");
        let files = GameFiles::new(&dir.path("saves"), &key());
        let dead = AtomicBool::new(true);
        let writer = Writer {
            files: &files,
            dead: &dead,
        };
        let core = stub();
        let written = write_on_exit(&core, &writer, true, |c| {
            assert!(!writer.state(c, AUTO_SLOT).unwrap());
            Ok(())
        });
        assert_eq!(written, Ok(()));
        assert!(!writer.state(&core, 1).unwrap());
        assert_eq!(writer.sram(&core), Ok(()));
        let shared = Shared::default();
        shared.frame.lock().width = 1;
        shared.frame.lock().height = 1;
        shared.frame.lock().rgba = Arc::new(vec![0; 4]);
        writer.thumb(1, &shared);
        assert_eq!(
            files_in(&dir.path("saves/gba/9a3e")),
            vec!["game.sav".to_owned()]
        );
        assert_eq!(
            dir.read("saves/gba/9a3e/game.sav"),
            "the newer session's save"
        );
    }

    /// A session around a fake emulation thread running `body` (no core), whose exit writes
    /// succeed.
    fn fake_session(
        core_path: PathBuf,
        body: impl FnOnce(Receiver<Command>, Arc<AtomicBool>) + Send + 'static,
    ) -> Session {
        fake_session_writing(core_path, move |rx, dead| {
            body(rx, dead);
            Ok(())
        })
    }

    /// A session around a fake emulation thread running `body`, which returns its exit writes'
    /// outcome as `emu_thread` does.
    fn fake_session_writing(
        core_path: PathBuf,
        body: impl FnOnce(Receiver<Command>, Arc<AtomicBool>) -> ExitWrites + Send + 'static,
    ) -> Session {
        let (tx, rx) = mpsc::channel();
        let (done_tx, done) = mpsc::channel::<ExitWrites>();
        let dead = Arc::new(AtomicBool::new(false));
        let thread_dead = dead.clone();
        let thread = std::thread::spawn(move || {
            let written = body(rx, thread_dead);
            let _ = done_tx.send(written);
        });
        Session {
            tx,
            shared: Arc::new(Shared::default()),
            thread: Some(thread),
            done,
            dead,
            core_path,
        }
    }

    #[test]
    fn closing_a_thread_that_stops_waits_for_it() {
        let dir = TestDir::new("session-close-joins");
        let core = dir.path("well_behaved_libretro.dll");
        let (wrote_tx, wrote) = mpsc::channel();
        let session = fake_session(core.clone(), move |rx, _| {
            while !matches!(rx.recv(), Ok(Command::Stop) | Err(_)) {}
            // The exit writes take a while: close waits for them.
            std::thread::sleep(Duration::from_millis(100));
            wrote_tx.send(()).unwrap();
        });
        assert_eq!(session.close(Duration::from_secs(10)), Closed::Joined);
        assert!(
            wrote.try_recv().is_ok(),
            "close returned before the thread ended"
        );
        assert!(!core_needs_restart(&core));
    }

    #[test]
    fn waiting_reports_the_exit_writes_once_the_thread_ends() {
        let dir = TestDir::new("session-wait-reports");
        let mut session = fake_session_writing(dir.path("failing_libretro.dll"), |rx, _| {
            while !matches!(rx.recv(), Ok(Command::Stop) | Err(_)) {}
            Err(ErrorCode::SaveFileFailed {
                detail: "disk full".into(),
            })
        });
        session.stop();
        assert_eq!(
            session.wait(Duration::from_secs(10)),
            Some(Err(ErrorCode::SaveFileFailed {
                detail: "disk full".into()
            }))
        );
        // Joined: waiting again has nothing left to wait for.
        assert_eq!(session.wait(Duration::from_secs(10)), None);
    }

    #[test]
    fn waiting_past_the_timeout_keeps_the_thread_and_can_wait_again() {
        let dir = TestDir::new("session-wait-again");
        let core = dir.path("slow_libretro.dll");
        let (release_tx, release) = mpsc::channel::<()>();
        let mut session = fake_session(core.clone(), move |rx, dead| {
            while !matches!(rx.recv(), Ok(Command::Stop) | Err(_)) {}
            // A slow disk: the exit writes outlast the first wait.
            let _ = release.recv();
            assert!(!dead.load(Ordering::SeqCst), "the thread was let go");
        });
        session.stop();
        assert_eq!(session.wait(Duration::from_millis(50)), None);
        assert!(!core_needs_restart(&core), "a wait that ran out let go");
        release_tx.send(()).unwrap();
        assert_eq!(session.wait(Duration::from_secs(10)), Some(Ok(())));
    }

    #[test]
    fn a_thread_that_panics_is_reported_as_a_failed_write() {
        let dir = TestDir::new("session-wait-panic");
        let mut session = fake_session(dir.path("panicking_libretro.dll"), |_, _| {
            panic!("the core crashed while writing")
        });
        session.stop();
        assert!(matches!(
            session.wait(Duration::from_secs(10)),
            Some(Err(ErrorCode::SaveFileFailed { .. }))
        ));
    }

    #[test]
    fn a_close_that_times_out_lets_the_thread_go_and_it_writes_nothing() {
        let dir = TestDir::new("session-close-times-out");
        dir.write("saves/gba/9a3e/game.sav", "the newer session's save");
        let core = dir.path("stuck_libretro.dll");
        let other_core = dir.path("other_libretro.dll");
        let save_dir = dir.path("saves");
        let (release_tx, release) = mpsc::channel::<()>();
        let (finished_tx, finished) = mpsc::channel::<bool>();
        let session = fake_session(core.clone(), move |_rx, dead| {
            // Stuck inside `retro_run` until released, deaf to Stop.
            let _ = release.recv();
            let files = GameFiles::new(&save_dir, &key());
            let writer = Writer {
                files: &files,
                dead: &dead,
            };
            let core = stub();
            let _ = write_on_exit(&core, &writer, true, |c| {
                let _ = writer.state(c, AUTO_SLOT);
                Ok(())
            });

            finished_tx.send(writer.alive()).unwrap();
        });
        let started = Instant::now();
        assert_eq!(session.close(Duration::from_millis(200)), Closed::Abandoned);
        assert!(started.elapsed() < Duration::from_secs(5));
        // The core is refused until restart; other cores are not.
        assert!(core_needs_restart(&core));
        assert!(!core_needs_restart(&other_core));
        // The thread wakes up: it is dead, and writes nothing.
        release_tx.send(()).unwrap();
        assert!(!finished.recv_timeout(Duration::from_secs(5)).unwrap());
        assert_eq!(
            files_in(&dir.path("saves/gba/9a3e")),
            vec!["game.sav".to_owned()]
        );
        assert_eq!(
            dir.read("saves/gba/9a3e/game.sav"),
            "the newer session's save"
        );
    }

    #[test]
    fn abandoning_marks_the_session_dead_and_refuses_its_core() {
        let dir = TestDir::new("session-abandon");
        let core = dir.path("hung_libretro.dll");
        let (release_tx, release) = mpsc::channel::<()>();
        let (alive_tx, alive) = mpsc::channel::<bool>();
        let session = fake_session(core.clone(), move |_rx, dead| {
            let _ = release.recv();
            alive_tx.send(!dead.load(Ordering::SeqCst)).unwrap();
        });
        let started = Instant::now();
        session.abandon();
        assert!(started.elapsed() < Duration::from_secs(1), "abandon waited");
        assert!(core_needs_restart(&core));
        release_tx.send(()).unwrap();
        assert!(!alive.recv_timeout(Duration::from_secs(5)).unwrap());
    }

    #[test]
    fn a_core_that_needs_a_restart_cannot_start_a_session() {
        let dir = TestDir::new("session-refused");
        let core = dir.path("refused_libretro.dll");
        mark_core_needs_restart(&core);
        let (events, _rx) = mpsc::channel();
        let result = Session::start(
            SessionOptions {
                core_path: core,
                game_path: dir.path("Game.gba"),
                system_dir: dir.path("system"),
                save_dir: dir.path("saves"),
                game: key(),
                autosave_every: None,
                autosave_on_exit: true,
                resume_auto: false,
                app_version: "test".into(),
                core_options: Vec::new(),
                rewind_seconds: None,
                split: crate::screens::Split::Whole,
                username: None,
                fast_audio: crate::speed_audio::FastAudio::Mute,
                video: Default::default(),
            },
            events,
        );
        assert!(result.is_err());
        // Refused before anything else: not even the folders were made.
        assert!(!dir.path("saves").exists());
    }

    #[test]
    fn a_session_on_the_fake_core_runs_and_writes_its_saves_when_closed() {
        let dir = TestDir::new("session-fake-core");
        let (events, rx) = mpsc::channel();
        let opts = SessionOptions {
            core_path: dir.path("fake_libretro.dll"),
            game_path: dir.path("Game.gba"),
            system_dir: dir.path("system"),
            save_dir: dir.path("saves"),
            game: key(),
            autosave_every: None,
            autosave_on_exit: true,
            resume_auto: false,
            app_version: "test".into(),
            core_options: Vec::new(),
            rewind_seconds: None,
            split: crate::screens::Split::Whole,
            username: None,
            fast_audio: crate::speed_audio::FastAudio::Mute,
            video: Default::default(),
        };
        let session = Session::start_with(opts, events, FakeCore::new()).unwrap();
        let started = Instant::now();
        while session.shared().stats.lock().frames < 3 {
            assert!(started.elapsed() < Duration::from_secs(10), "no frames");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(session.close(Duration::from_secs(10)), Closed::Joined);
        let files = GameFiles::new(&dir.path("saves"), &key());
        let sram = FakeCore::counter(&std::fs::read(&files.sram).unwrap()).unwrap();
        let auto = FakeCore::counter(&std::fs::read(files.slot(AUTO_SLOT)).unwrap()).unwrap();
        assert!(sram >= 3 && sram == auto, "{sram} {auto}");
        let events: Vec<Event> = rx.try_iter().collect();
        assert!(matches!(events.first(), Some(Event::Started { .. })));
        assert!(matches!(events.last(), Some(Event::Stopped)));
    }

    #[test]
    fn a_dead_sessions_events_are_dropped() {
        let (tx, rx) = mpsc::channel();
        let dead = Arc::new(AtomicBool::new(false));
        let outbox = Outbox {
            tx,
            dead: dead.clone(),
        };
        outbox.send(Event::Stopped).unwrap();
        dead.store(true, Ordering::SeqCst);
        outbox.send(Event::StateSaved { slot: 1 }).unwrap();
        assert!(matches!(rx.try_recv(), Ok(Event::Stopped)));
        assert!(rx.try_recv().is_err());
    }
}
