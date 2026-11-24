//! The emulation loop (Trello #81): one [`EmuLoop`] per session, one method per [`Command`], one
//! [`EmuLoop::step`] per frame, every time given from outside. It runs on any [`Core`]: the
//! libretro one ([`super::libretro::Libretro`]) or the fake of the tests
//! ([`crate::testing::FakeCore`]), so undo, rewind and pause, the auto quick save's timing and
//! the pacing resets are tested without a real core.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use libretro_sys::retro_system_av_info;

use super::io;
use super::{
    write_on_exit, Command, Event, ExitWrites, Outbox, SessionOptions, Shared, Speed, Writer,
    AUTO_SLOT,
};
use crate::core::GameFiles;
use crate::core_profile::StateLoadBlock;
use crate::error::ErrorCode;
use crate::rewind::Rewind;
use crate::saves::SlotMeta;
use crate::speed_audio::{FastAudio, SpeedMeter};

/// What the session needs from a core: the libretro calls it makes, and what it tells the core's
/// callbacks (audio, options, input). The callbacks' side does nothing by default.
pub trait Core {
    /// Name, version and timing, read once the game is loaded.
    fn info(&self) -> CoreInfo;
    /// One frame (`retro_run`).
    fn run(&mut self);
    /// `retro_reset`.
    fn reset(&mut self);
    /// The full emulation state (`retro_serialize`).
    fn serialize(&self) -> anyhow::Result<Vec<u8>>;
    /// [`Core::serialize`] into a buffer reused from older states (rewind).
    fn serialize_into(&self, buf: &mut Vec<u8>) -> anyhow::Result<()>;
    /// Back to a state (`retro_unserialize`).
    fn unserialize(&mut self, state: &[u8]) -> anyhow::Result<()>;
    /// The game's save memory (cartridge battery), when it has one.
    fn sram(&self) -> Option<&[u8]>;
    fn sram_mut(&mut self) -> Option<&mut [u8]>;

    /// Freezes the input the next frame reads.
    fn refresh_input(&mut self) {}
    /// The timing the core set since the last call (`SET_SYSTEM_AV_INFO`), if it did.
    fn take_av_change(&mut self) -> Option<retro_system_av_info> {
        None
    }
    /// The emulation speed for the audio (`None`: at Max, not measured yet).
    fn set_audio_speed(&mut self, _speed: Option<f64>) {}
    /// When false, the audio the core makes is dropped (while rewinding).
    fn set_audio_enabled(&mut self, _enabled: bool) {}
    /// What the audio does off 1× (`audio.ff`).
    fn set_fast_audio(&mut self, _policy: FastAudio) {}
    /// A core option changed while the game runs.
    fn set_core_option(&mut self, _key: String, _value: String) {}
    /// The session ended: the core is unloaded, or (`let_go`) left loaded as it is, since a
    /// thread that was let go may still be inside it.
    fn end(self, let_go: bool)
    where
        Self: Sized,
    {
        let _ = let_go;
    }
}

/// What a loaded core says about itself.
#[derive(Debug, Clone, Default)]
pub struct CoreInfo {
    pub name: String,
    pub version: String,
    pub av: retro_system_av_info,
}

/// Whether the loop goes on after a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Flow {
    Continue,
    Stop,
}

/// The game save is flushed this often while the game runs, and on exit.
const SRAM_FLUSH_EVERY: Duration = Duration::from_secs(30);
/// Pacing diagnostics (Trello #57) are logged once per window this long.
const PACE_LOG_EVERY: Duration = Duration::from_secs(10);

/// One session's emulation: the core and everything the commands and frames change.
pub(super) struct EmuLoop<C: Core> {
    pub(super) core: C,
    shared: Arc<Shared>,
    events: Outbox,
    files: GameFiles,
    dead: Arc<AtomicBool>,
    /// The writes while the game runs, off this thread, in the order asked.
    io: io::Io,
    slot_meta: SlotMeta,
    autosave_every: Option<Duration>,
    autosave_on_exit: bool,
    /// The game's screenshots folder, keyed by its `GameKey` ([`crate::screenshot::game_dir`]).
    screenshots: PathBuf,
    base_fps: f64,
    speed: Speed,
    fast_audio: FastAudio,
    /// The speed at Max, measured for the audio.
    meter: SpeedMeter,
    paused: bool,
    rewind: Rewind,
    rewinding: bool,
    next_frame: Instant,
    fps: FpsMeter,
    last_sram_flush: Instant,
    last_autosave: Instant,
    pace: PaceLog,
    /// What the last save or load can go back to («Annulla»): only the newest one, until the
    /// next save or load. Always kept; the interface offers the load's only when `saves.undo` is
    /// on.
    undo: Option<Undo>,
    /// Why this core would crash loading a state (Trello #93): every load and rewind refused.
    load_block: Option<StateLoadBlock>,
}

impl<C: Core> EmuLoop<C> {
    /// The loop for a loaded `core`: its version recorded, the I/O worker started, `Started`
    /// sent, and the auto quick save loaded when `saves.resume` is «always».
    pub(super) fn start(
        core: C,
        opts: &SessionOptions,
        shared: Arc<Shared>,
        events: Outbox,
        dead: Arc<AtomicBool>,
        now: Instant,
    ) -> Self {
        let files = GameFiles::new(&opts.save_dir, &opts.game);
        let info = core.info();
        // The core's version: quick saves made by another one might not load (Saves, V02).
        let slot_meta = SlotMeta {
            core: info.name.clone(),
            core_version: info.version.clone(),
            app_version: opts.app_version.clone(),
        };
        if let Err(e) = crate::saves::note_core_version(
            &opts.save_dir,
            &slot_meta.core,
            &slot_meta.core_version,
        ) {
            log::warn!("core version not recorded: {e}");
        }
        let io = io::Io::start(io::Worker::new(
            files.clone(),
            dead.clone(),
            events.clone(),
            shared.clone(),
            slot_meta.clone(),
        ));
        let av = info.av;
        {
            let mut stats = shared.stats.lock();
            stats.av_info = av;
            stats.speed = Some(Speed::Factor(1.0));
        }
        shared.audio.set_rate(av.timing.sample_rate);
        let rom_name = opts
            .game_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let _ = events.send(Event::Started {
            core: format!("{} {}", info.name, info.version),
            game: rom_name,
            fps: av.timing.fps,
            width: av.geometry.base_width,
            height: av.geometry.base_height,
        });
        let base_fps = if av.timing.fps > 0.0 {
            av.timing.fps
        } else {
            60.0
        };
        let mut lp = Self {
            core,
            shared,
            events,
            files,
            dead,
            io,
            slot_meta,
            autosave_every: opts.autosave_every,
            autosave_on_exit: opts.autosave_on_exit,
            screenshots: crate::screenshot::game_dir(&opts.save_dir, &opts.game),
            base_fps,
            speed: Speed::Factor(1.0),
            fast_audio: opts.fast_audio,
            meter: SpeedMeter::new(base_fps, now),
            paused: false,
            // Rewind loads states: with loads refused it records nothing.
            rewind: Rewind::with_encoder(
                opts.rewind_seconds
                    .filter(|_| opts.state_load_block.is_none()),
                base_fps,
            ),
            rewinding: false,
            next_frame: now,
            fps: FpsMeter::new(now),
            last_sram_flush: now,
            last_autosave: now,
            pace: PaceLog::new(now),
            undo: None,
            load_block: opts.state_load_block,
        };
        if opts.resume_auto && lp.files.slot(AUTO_SLOT).is_file() && !lp.load_refused() {
            let event = match lp.load_undoable(AUTO_SLOT) {
                Ok(()) => Event::StateLoaded { slot: AUTO_SLOT },
                Err(error) => Event::Error { error },
            };
            let _ = lp.events.send(event);
        }
        lp
    }

    fn writer(&self) -> Writer<'_> {
        Writer {
            files: &self.files,
            dead: &self.dead,
        }
    }

    /// False once the session is let go (abandoned, or a close that timed out).
    pub(super) fn alive(&self) -> bool {
        self.writer().alive()
    }

    pub(super) fn paused(&self) -> bool {
        self.paused
    }

    fn send(&self, event: Event) {
        let _ = self.events.send(event);
    }

    /// True, and `stateLoadUnsupported` sent, when this core would crash loading a state: the
    /// load (or rewind) asked for never reaches it.
    fn load_refused(&self) -> bool {
        let Some(reason) = self.load_block else {
            return false;
        };
        self.send(Event::Error {
            error: ErrorCode::StateLoadUnsupported { reason },
        });
        true
    }

    // ---- Commands ----

    /// Carries out one command at `now`.
    pub(super) fn command(&mut self, cmd: Command, now: Instant) -> Flow {
        match cmd {
            Command::SetSpeed(s) => self.set_speed(s, now),
            Command::SetPaused(p) => self.set_paused(p, now),
            Command::SaveState(slot) => self.save_state(slot),
            Command::LoadState(slot) => self.load_state(slot),
            Command::UndoLoad => self.undo_load(),
            Command::UndoSave => self.undo_save(),
            Command::SetQuickSlot(slot) => self
                .io
                .send(io::Job::QuickSlot(crate::quick_slot::clamp(slot))),
            Command::SetCoreOption { key, value } => self.core.set_core_option(key, value),
            Command::Reset => {
                self.core.reset();
                self.set_unsaved(true);
            }
            Command::Rewind(on) => self.set_rewinding(on),
            Command::SetRewindLength(seconds) => self.set_rewind_length(seconds),
            Command::SetFastAudio(policy) => self.set_fast_audio(policy),
            Command::Screenshot => self.screenshot(SystemTime::now()),
            Command::Flush => self.flush(now),
            Command::Stop => return Flow::Stop,
        }
        Flow::Continue
    }

    /// A pacing restart: the next frame is due now, the FPS window and the diagnostics start over.
    fn restart_pacing(&mut self, now: Instant) {
        self.next_frame = now;
        self.fps.restart(now);
        self.pace = PaceLog::new(now);
    }

    fn set_speed(&mut self, speed: Speed, now: Instant) {
        log::info!("pacing: speed set to {speed:?}");
        self.speed = speed;
        self.meter = SpeedMeter::new(self.base_fps, now);
        self.core.set_audio_speed(audio_speed(speed));
        if !speed.is_normal() && self.fast_audio == FastAudio::Mute {
            self.shared.audio.clear();
        }
        self.shared.stats.lock().speed = Some(speed);
        self.restart_pacing(now);
    }

    fn set_paused(&mut self, paused: bool, now: Instant) {
        self.paused = paused;
        {
            // Paused, nothing runs: 0 frames/s, not the last second before it.
            let mut stats = self.shared.stats.lock();
            stats.paused = paused;
            stats.emu_fps = 0.0;
        }
        self.restart_pacing(now);
    }

    /// A quick save: serialized here, written by the worker (which says `stateSaved`). The
    /// worker keeps what the slot held; «Annulla» asks it back.
    fn save_state(&mut self, slot: u8) {
        if self.save_async(slot) {
            self.undo = Some(Undo::Save);
            // The user's save holds the game as it is (the worker sets it again if the write
            // fails).
            self.set_unsaved(false);
        }
    }

    /// [`super::Stats::unsaved`].
    fn set_unsaved(&self, unsaved: bool) {
        self.shared.stats.lock().unsaved = unsaved;
    }

    /// Serializes into `slot` and hands the write to the worker; false (and an error event) when
    /// the core cannot serialize.
    fn save_async(&mut self, slot: u8) -> bool {
        let started = Instant::now();
        match self.core.serialize() {
            Ok(state) => {
                // A 3DS state on Vulkan takes a quarter of a second (Trello #98): seen in the log.
                log::info!(
                    "quick save: slot {slot}, state of {} KB taken in {:.1} ms",
                    state.len() / 1024,
                    started.elapsed().as_secs_f64() * 1000.0
                );
                let frame = self.shared.frame.lock().clone();
                self.io.send(io::Job::Slot { slot, state, frame });
                true
            }
            Err(e) => {
                log::warn!("quick save: slot {slot} not serialized: {e:#}");
                self.send(Event::Error {
                    error: ErrorCode::StateSaveFailed {
                        slot,
                        cause: crate::IoCause::of_any(&e),
                    },
                });
                false
            }
        }
    }

    fn load_state(&mut self, slot: u8) {
        if self.load_refused() {
            return;
        }
        // A save still being written lands first: the load reads the new file.
        self.io.sync();
        let loaded = self.load_undoable(slot);
        if loaded.is_ok() {
            self.io.send(io::Job::ForgetUndo);
            // The game is what the slot holds.
            self.set_unsaved(false);
        }
        self.send(match loaded {
            Ok(()) => Event::StateLoaded { slot },
            Err(error) => Event::Error { error },
        });
    }

    /// [`load_slot`], keeping the state from before it for «Annulla» when it succeeds.
    fn load_undoable(&mut self, slot: u8) -> Result<(), ErrorCode> {
        let before = self.core.serialize().ok();
        load_slot(&mut self.core, &self.files, slot)?;
        self.undo = before.map(Undo::Load);
        Ok(())
    }

    fn undo_load(&mut self) {
        if self.load_refused() {
            return;
        }
        let event = match self.undo.take() {
            Some(Undo::Load(data)) => match self.core.unserialize(&data) {
                Ok(()) => {
                    // Back to the state from before the load: neither saved nor loaded.
                    self.set_unsaved(true);
                    Event::LoadUndone
                }
                Err(e) => {
                    log::warn!("undo load: the core refused the state from before: {e:#}");
                    Event::Error {
                        error: ErrorCode::UndoFailed,
                    }
                }
            },
            kept @ (Some(Undo::Save) | None) => {
                self.undo = kept;
                Event::Error {
                    error: ErrorCode::NothingToUndo,
                }
            }
        };
        self.send(event);
    }

    fn undo_save(&mut self) {
        match self.undo.take() {
            // The worker puts the slot back and says `saveUndone` (or why not).
            Some(Undo::Save) => {
                self.io.send(io::Job::UndoSave);
                // The slot gets back an older state than the game's.
                self.set_unsaved(true);
            }
            kept @ (Some(Undo::Load(_)) | None) => {
                self.undo = kept;
                self.send(Event::Error {
                    error: ErrorCode::NothingToUndo,
                });
            }
        }
    }

    fn set_rewinding(&mut self, on: bool) {
        // Rewinding loads states; letting it go is always fine (it never started).
        if on && self.load_refused() {
            return;
        }
        if on != self.rewinding {
            self.rewinding = on;
            // Silence while going back.
            self.core.set_audio_enabled(!on);
            self.shared.audio.clear();
            self.shared.stats.lock().rewinding = on;
        }
    }

    fn set_rewind_length(&mut self, seconds: Option<u32>) {
        // With loads refused rewind stays off whatever `speed.rew` says.
        self.rewind
            .set_seconds(seconds.filter(|_| self.load_block.is_none()));
        let (states, bytes) = self.rewind.usage();
        let mut stats = self.shared.stats.lock();
        (stats.rewind_states, stats.rewind_bytes) = (states, bytes);
    }

    fn set_fast_audio(&mut self, policy: FastAudio) {
        self.fast_audio = policy;
        self.core.set_fast_audio(policy);
    }

    fn screenshot(&mut self, at: SystemTime) {
        let frame = self.shared.frame.lock().clone();
        self.io.send(io::Job::Screenshot {
            frame,
            dir: self.screenshots.clone(),
            at,
        });
    }

    /// What closing would write, without closing (Android going to the background, where the
    /// process may be killed after this): waited for, after what is queued.
    fn flush(&mut self, now: Instant) {
        self.io.sync();
        // Failures are logged and sent as events; the next flush tries again.
        let _ = self.exit_writes();
        self.last_sram_flush = now;
    }

    // ---- Frames ----

    /// One round of the loop after the commands: paused, nothing; otherwise one frame (or one
    /// step back while rewinding) and its bookkeeping at `now`.
    pub(super) fn step(&mut self, now: Instant) {
        if self.paused {
            return;
        }
        // The input this frame reads (the core takes it again at its own `input_poll`).
        self.core.refresh_input();
        if self.rewinding {
            // One state back, then one frame run to show it. With nothing left the game stays on
            // its picture (the frame count still moves: the thread is alive).
            let core = &mut self.core;
            match self.rewind.step_back(|state| core.unserialize(state)) {
                Some(Ok(())) => self.core.run(),
                Some(Err(e)) => log::warn!("rewind: state not loaded: {e:#}"),
                None => {}
            }
        } else {
            self.core.run();
            let core = &self.core;
            self.rewind.record(|buf| core.serialize_into(buf).is_ok());
        }
        {
            let (states, bytes) = self.rewind.usage();
            let mut stats = self.shared.stats.lock();
            stats.frames += 1;
            // A frame forward or a step back: the game is no longer what was saved or loaded.
            stats.unsaved = true;
            (stats.rewind_states, stats.rewind_bytes) = (states, bytes);
        }

        // The core changed its timing (`SET_SYSTEM_AV_INFO`, e.g. a mode switch): pacing, the
        // speed meter and rewind follow it from the next frame.
        if let Some(av) = self.core.take_av_change() {
            if let Some(new_fps) = changed_fps(self.base_fps, av.timing.fps) {
                log::info!(
                    "pacing: the core changed its rate from {:.4} to {new_fps:.4}",
                    self.base_fps
                );
                self.base_fps = new_fps;
                self.meter = SpeedMeter::new(new_fps, now);
                self.rewind.set_fps(new_fps);
                self.next_frame = now;
                self.fps.restart(now);
            }
        }

        // Auto quick save every `saves.autoq`, only while the game runs.
        if let Some(every) = self.autosave_every {
            if now.saturating_duration_since(self.last_autosave) >= every {
                // A failure is sent as an event; the next one tries again.
                self.save_async(AUTO_SLOT);
                self.last_autosave = now;
            }
        }

        // At Max the audio follows the measured speed.
        if self.speed == Speed::Max {
            if let Some(s) = self.meter.frame(now) {
                self.core.set_audio_speed(Some(s));
            }
        }

        if let Some(rate) = self.fps.frame(now) {
            self.shared.stats.lock().emu_fps = rate;
        }

        // Game save every 30 s: copied here, written by the worker (a failure is logged; the
        // next flush tries again).
        if now.saturating_duration_since(self.last_sram_flush) >= SRAM_FLUSH_EVERY {
            if let Some(ram) = self.core.sram() {
                self.io.send(io::Job::Sram(ram.to_vec()));
            }
            self.last_sram_flush = now;
        }
    }

    /// After a frame's work, at `now`: how long to sleep before the next one (at a fixed speed;
    /// at Max, or paused, never). Too far behind (a window dragged) it starts again from `now`
    /// without catching up.
    pub(super) fn pace(&mut self, now: Instant) -> Option<Duration> {
        if self.paused {
            return None;
        }
        let mut sleep = None;
        if let Speed::Factor(f) = self.speed {
            let period = Duration::from_secs_f64(1.0 / (self.base_fps * f64::from(f.max(0.05))));
            self.next_frame += period;
            if self.next_frame > now {
                sleep = Some(self.next_frame - now);
                self.pace.sleeps += 1;
            } else if now - self.next_frame > period * 4 {
                self.next_frame = now;
                self.pace.resets += 1;
            }
        }
        self.pace.frames += 1;
        let window = now.saturating_duration_since(self.pace.since);
        if window >= PACE_LOG_EVERY {
            log::info!(
                "pacing: {:.2} frames/s over {:.1} s (speed {:?}, base {:.4}, {} sleeps, {} resets)",
                f64::from(self.pace.frames) / window.as_secs_f64(),
                window.as_secs_f64(),
                self.speed,
                self.base_fps,
                self.pace.sleeps,
                self.pace.resets,
            );
            self.pace = PaceLog::new(now);
        }
        sleep
    }

    /// The frame's work, recorded while the headless example's `--timing` asks for it.
    pub(super) fn record_work(&self, work: Duration) {
        if let Some(times) = self.shared.frame_times.lock().as_mut() {
            times.push(u32::try_from(work.as_micros()).unwrap_or(u32::MAX));
        }
    }

    // ---- The end ----

    /// The in-game save, and the auto quick save with `saves.autoexit`, written here and waited
    /// for: closing and [`Command::Flush`] only.
    fn exit_writes(&self) -> ExitWrites {
        write_on_exit(&self.core, &self.writer(), self.autosave_on_exit, |core| {
            self.save_auto_now(core)
        })
    }

    fn save_auto_now(&self, core: &C) -> Result<(), ErrorCode> {
        let writer = self.writer();
        match writer.state(core, AUTO_SLOT) {
            Ok(true) => {
                writer.thumb(AUTO_SLOT, &self.shared);
                if let Err(e) =
                    crate::saves::write_slot_meta(&self.files, AUTO_SLOT, &self.slot_meta)
                {
                    log::warn!("slot {AUTO_SLOT} record not written: {e}");
                }
                self.shared.stats.lock().auto_saved_at = Some(SystemTime::now());
                Ok(())
            }
            Ok(false) => Ok(()),
            Err(e) => {
                log::warn!("auto quick save not written: {e:#}");
                let error = ErrorCode::StateSaveFailed {
                    slot: AUTO_SLOT,
                    cause: crate::IoCause::of_any(&e),
                };
                self.send(Event::Error {
                    error: error.clone(),
                });
                Err(error)
            }
        }
    }

    /// The session's end: what the worker still has queued lands first (the exit writes are the
    /// newest), then the exit writes, the core unloaded, `Stopped` sent.
    pub(super) fn finish(mut self) -> ExitWrites {
        let io = std::mem::replace(&mut self.io, io::Io::Finished);
        io.finish();
        let written = self.exit_writes();
        let Self { core, events, .. } = self;
        core.end(false);
        let _ = events.send(Event::Stopped);
        written
    }

    /// A session that was let go: another session may be using this core's module now (the core
    /// is refused, but the module's globals are shared), so neither files nor unloading. The
    /// core stays loaded until the process ends; the worker is not waited for (it sees the
    /// session dead and writes nothing more).
    pub(super) fn let_go(self) {
        log::warn!("a let-go emulation thread ended: nothing written, the core left loaded");
        let Self { core, io, .. } = self;
        core.end(true);
        drop(io);
    }
}

/// The new frame rate when the core set a usable one that differs from `base` (a zero or broken
/// rate keeps the one in use).
pub(super) fn changed_fps(base: f64, new: f64) -> Option<f64> {
    (new.is_finite() && new > 0.0 && (new - base).abs() > 1e-9).then_some(new)
}

/// The speed the audio is made at: the factor, or None at Max until it is measured.
fn audio_speed(speed: Speed) -> Option<f64> {
    match speed {
        Speed::Factor(f) => Some(f64::from(f)),
        Speed::Max => None,
    }
}

/// Pacing diagnostics over one window: frames run, sleeps and resets.
struct PaceLog {
    since: Instant,
    frames: u32,
    sleeps: u32,
    resets: u32,
}

impl PaceLog {
    fn new(now: Instant) -> Self {
        Self {
            since: now,
            frames: 0,
            sleeps: 0,
            resets: 0,
        }
    }
}

/// Emulated frames per second, measured over windows of at least one second of running game.
///
/// Paused time is not running time: the window restarts on pause and on resume, otherwise
/// the last value before a pause would stay on show for as long as the game is paused
/// (Trello #57: a frozen `core 60.66` read as a game running 1.5 % fast) and the first
/// window after it would count the pause.
pub(super) struct FpsMeter {
    since: Instant,
    frames: u32,
}

impl FpsMeter {
    const WINDOW: Duration = Duration::from_secs(1);

    pub(super) fn new(now: Instant) -> Self {
        Self {
            since: now,
            frames: 0,
        }
    }

    /// Starts a new window with no frames (a pause, a resume, a speed change).
    pub(super) fn restart(&mut self, now: Instant) {
        *self = Self::new(now);
    }

    /// One frame run at `now`: the rate when a window just closed.
    pub(super) fn frame(&mut self, now: Instant) -> Option<f64> {
        self.frames += 1;
        let elapsed = now.saturating_duration_since(self.since);
        if elapsed < Self::WINDOW {
            return None;
        }
        let fps = f64::from(self.frames) / elapsed.as_secs_f64();
        self.restart(now);
        Some(fps)
    }
}

/// What «Annulla» goes back to: the newest save or load only.
enum Undo {
    /// The game's state from before the last load.
    Load(Vec<u8>),
    /// The last save: the I/O worker holds what its slot held before it.
    Save,
}

/// Reads a slot and hands it to the core, telling an empty slot from a broken one.
fn load_slot(core: &mut impl Core, files: &GameFiles, slot: u8) -> Result<(), ErrorCode> {
    // An `if` rather than a `match`: io::ErrorKind is non-exhaustive, so a match would need
    // the `_ =>` arm the workspace lints forbid.
    let data = std::fs::read(files.slot(slot)).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ErrorCode::SlotEmpty { slot }
        } else {
            log::warn!("slot {slot} not read: {e}");
            ErrorCode::StateLoadFailed {
                slot,
                cause: crate::IoCause::of(&e),
            }
        }
    })?;
    core.unserialize(&data)
        .map_err(|_| ErrorCode::StateInvalid { slot })
}

/// Copies the in-game save on disk into a freshly loaded core.
pub(super) fn load_sram(core: &mut impl Core, files: &GameFiles) {
    if let (Some(ram), Ok(data)) = (core.sram_mut(), std::fs::read(&files.sram)) {
        let n = ram.len().min(data.len());
        ram[..n].copy_from_slice(&data[..n]);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, Receiver};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::{EmuLoop, Flow};
    use crate::core::GameFiles;
    use crate::core_profile::StateLoadBlock;
    use crate::error::ErrorCode;
    use crate::session::{Command, Event, Outbox, SessionOptions, Shared, Speed, AUTO_SLOT};
    use crate::settings::test_dir::TestDir;
    use crate::speed_audio::FastAudio;
    use crate::testing::FakeCore;

    struct Rig {
        lp: EmuLoop<FakeCore>,
        events: Receiver<Event>,
        files: GameFiles,
        dead: Arc<AtomicBool>,
        t0: Instant,
        _dir: TestDir,
    }

    fn options(dir: &TestDir) -> SessionOptions {
        SessionOptions {
            core_path: dir.path("fake_libretro.dll"),
            game_path: dir.path("Game.gba"),
            game_file: None,
            system_dir: dir.path("system"),
            save_dir: dir.path("saves"),
            game: crate::GameKey::parse("gba:9a3e").unwrap(),
            autosave_every: None,
            autosave_on_exit: true,
            resume_auto: false,
            app_version: "test".into(),
            core_options: Vec::new(),
            rewind_seconds: None,
            state_load_block: None,
            split: crate::screens::Split::Whole,
            username: None,
            fast_audio: FastAudio::Mute,
            video: Default::default(),
        }
    }

    fn rig_with(name: &str, setup: impl FnOnce(&mut SessionOptions, &TestDir)) -> Rig {
        let dir = TestDir::new(name);
        let mut opts = options(&dir);
        setup(&mut opts, &dir);
        let files = GameFiles::new(&opts.save_dir, &opts.game);
        std::fs::create_dir_all(files.dir()).unwrap();
        let (tx, events) = mpsc::channel();
        let dead = Arc::new(AtomicBool::new(false));
        let outbox = Outbox {
            tx,
            dead: dead.clone(),
        };
        let t0 = Instant::now();
        let shared = Arc::new(Shared::default());
        let lp = EmuLoop::start(FakeCore::new(), &opts, shared, outbox, dead.clone(), t0);
        Rig {
            lp,
            events,
            files,
            dead,
            t0,
            _dir: dir,
        }
    }

    fn rig(name: &str) -> Rig {
        rig_with(name, |_, _| {})
    }

    impl Rig {
        /// `n` frames, all at the start time (no period passes).
        fn frames(&mut self, n: u32) {
            for _ in 0..n {
                self.lp.step(self.t0);
            }
        }

        fn at(&self, secs: u64) -> Instant {
            self.t0 + Duration::from_secs(secs)
        }

        fn cmd(&mut self, cmd: Command) -> Flow {
            let now = self.t0;
            self.lp.command(cmd, now)
        }

        fn counter(&self) -> u64 {
            self.lp.core.frames
        }

        /// Every event so far, once the worker's writes are done.
        fn events(&mut self) -> Vec<Event> {
            self.lp.io.sync();
            self.events.try_iter().collect()
        }

        fn on_disk(&self, path: &std::path::Path) -> Option<u64> {
            FakeCore::counter(&std::fs::read(path).ok()?)
        }
    }

    fn is_error(event: &Event, code: &ErrorCode) -> bool {
        matches!(event, Event::Error { error } if error == code)
    }

    #[test]
    fn starting_says_started_and_resumes_the_auto_quick_save_when_asked() {
        let mut r = rig_with("loop-resume", |opts, dir| {
            opts.resume_auto = true;
            let auto = GameFiles::new(&opts.save_dir, &opts.game).slot(AUTO_SLOT);
            std::fs::create_dir_all(auto.parent().unwrap()).unwrap();
            std::fs::write(&auto, 9u64.to_le_bytes()).unwrap();
            let _ = dir;
        });
        assert_eq!(r.counter(), 9);
        let events = r.events();
        assert!(matches!(events.first(), Some(Event::Started { .. })));
        assert!(matches!(
            events.get(1),
            Some(Event::StateLoaded { slot: AUTO_SLOT })
        ));
        // The resume is undoable like any load.
        r.frames(2);
        r.cmd(Command::UndoLoad);
        assert_eq!(r.counter(), 0);
    }

    #[test]
    fn undoing_a_load_goes_back_once() {
        let mut r = rig("loop-undo-load");
        r.frames(3);
        r.cmd(Command::SaveState(1));
        r.frames(2);
        r.cmd(Command::LoadState(1));
        assert_eq!(r.counter(), 3);
        r.cmd(Command::UndoLoad);
        assert_eq!(r.counter(), 5);
        r.cmd(Command::UndoLoad);
        assert_eq!(r.counter(), 5);
        let events = r.events();
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StateSaved { slot: 1 })));
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StateLoaded { slot: 1 })));
        assert!(events.iter().any(|e| matches!(e, Event::LoadUndone)));
        assert!(events
            .last()
            .is_some_and(|e| is_error(e, &ErrorCode::NothingToUndo)));
    }

    #[test]
    fn undoing_a_save_puts_the_slot_back_until_a_load_replaces_it() {
        let mut r = rig("loop-undo-save");
        r.frames(2);
        r.cmd(Command::SaveState(1));
        r.frames(2);
        r.cmd(Command::SaveState(1));
        r.cmd(Command::UndoSave);
        let events = r.events();
        assert!(matches!(events.last(), Some(Event::SaveUndone { slot: 1 })));
        assert_eq!(r.on_disk(&r.files.slot(1)), Some(2));

        // A load is the newest action: the save's undo is gone.
        r.cmd(Command::SaveState(2));
        r.cmd(Command::LoadState(1));
        r.cmd(Command::UndoSave);
        let events = r.events();
        assert!(events
            .last()
            .is_some_and(|e| is_error(e, &ErrorCode::NothingToUndo)));
        // ...and the load's undo is still there.
        r.cmd(Command::UndoLoad);
        assert_eq!(r.counter(), 4);
    }

    #[test]
    fn loading_an_empty_or_broken_slot_says_which() {
        let mut r = rig("loop-load-errors");
        r.cmd(Command::LoadState(5));
        std::fs::write(r.files.slot(6), b"junk").unwrap();
        r.cmd(Command::LoadState(6));
        let events = r.events();
        assert!(events
            .iter()
            .any(|e| is_error(e, &ErrorCode::SlotEmpty { slot: 5 })));
        assert!(events
            .iter()
            .any(|e| is_error(e, &ErrorCode::StateInvalid { slot: 6 })));
        assert_eq!(r.counter(), 0);
    }

    #[test]
    fn rewind_goes_back_silently_and_a_pause_holds_it() {
        let mut r = rig_with("loop-rewind", |opts, _| opts.rewind_seconds = Some(10));
        // One frame at a time, the encoder keeping up (it skips states when it is behind).
        for _ in 0..20 {
            r.frames(1);
            r.lp.rewind.sync();
        }
        r.cmd(Command::Rewind(true));
        assert_eq!(r.lp.core.calls.lock().audio_enabled, Some(false));
        assert!(r.lp.shared.stats.lock().rewinding);
        r.frames(2);
        let back = r.counter();
        assert!(back < 20, "{back}");

        // Paused while rewinding: nothing moves, no pacing.
        r.cmd(Command::SetPaused(true));
        r.frames(3);
        assert_eq!(r.counter(), back);
        assert_eq!(r.lp.pace(r.t0), None);
        // Resumed, still rewinding: further back.
        r.cmd(Command::SetPaused(false));
        r.frames(2);
        assert!(r.counter() < back, "{} then {}", back, r.counter());

        // Rewind off: the sound comes back and the game goes forward again.
        r.cmd(Command::Rewind(false));
        assert_eq!(r.lp.core.calls.lock().audio_enabled, Some(true));
        let now = r.counter();
        r.frames(1);
        assert_eq!(r.counter(), now + 1);
    }

    /// Trello #93: a core that would crash loading a state never gets one; saving still works.
    #[test]
    fn with_state_loads_blocked_no_load_or_rewind_reaches_the_core() {
        let refused = ErrorCode::StateLoadUnsupported {
            reason: StateLoadBlock::VulkanNoTitleId,
        };
        let mut r = rig_with("loop-load-blocked", |opts, _| {
            opts.state_load_block = Some(StateLoadBlock::VulkanNoTitleId);
            opts.rewind_seconds = Some(10);
            // An auto quick save to resume from at the start: refused too.
            opts.resume_auto = true;
            let auto = GameFiles::new(&opts.save_dir, &opts.game).slot(AUTO_SLOT);
            std::fs::create_dir_all(auto.parent().unwrap()).unwrap();
            std::fs::write(&auto, 9u64.to_le_bytes()).unwrap();
        });
        assert_eq!(r.counter(), 0, "the auto quick save was not loaded");

        // Saving works.
        r.frames(3);
        r.cmd(Command::SaveState(1));
        r.frames(2);
        r.cmd(Command::LoadState(1));
        r.cmd(Command::LoadState(AUTO_SLOT));
        r.cmd(Command::UndoLoad);
        assert_eq!(r.counter(), 5);

        // Rewind: nothing recorded, nothing played back, the sound stays on.
        r.lp.rewind.sync();
        assert_eq!(r.lp.rewind.usage().0, 0);
        r.cmd(Command::SetRewindLength(Some(20)));
        r.frames(2);
        r.lp.rewind.sync();
        assert_eq!(r.lp.rewind.usage().0, 0);
        r.cmd(Command::Rewind(true));
        assert!(!r.lp.shared.stats.lock().rewinding);
        assert_eq!(r.lp.core.calls.lock().audio_enabled, None);
        r.frames(1);
        assert_eq!(r.counter(), 8, "the game went on forward");
        r.cmd(Command::Rewind(false));

        assert_eq!(r.lp.core.calls.lock().unserializes, 0);
        let events = r.events();
        assert_eq!(r.on_disk(&r.files.slot(1)), Some(3));
        let refusals = events.iter().filter(|e| is_error(e, &refused)).count();
        // The resume, two loads, the undo, the rewind; not its release.
        assert_eq!(refusals, 5, "{events:?}");
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::StateSaved { slot: 1 })));
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::StateLoaded { .. } | Event::LoadUndone)));
    }

    #[test]
    fn the_auto_quick_save_follows_its_period_only_while_the_game_runs() {
        let mut r = rig_with("loop-autosave", |opts, _| {
            opts.autosave_every = Some(Duration::from_secs(60));
        });
        let auto = r.files.slot(AUTO_SLOT);
        r.frames(1);
        r.lp.step(r.at(30));
        r.lp.io.sync();
        assert!(!auto.exists());
        r.lp.step(r.at(61));
        r.lp.io.sync();
        assert_eq!(r.on_disk(&auto), Some(3));
        assert!(r.lp.shared.stats.lock().auto_saved_at.is_some());
        // The auto quick save says nothing (no «Salvato» toast).
        assert!(!r
            .events()
            .iter()
            .any(|e| matches!(e, Event::StateSaved { .. })));

        // Paused, no auto quick save however long it waits.
        std::fs::remove_file(&auto).unwrap();
        r.cmd(Command::SetPaused(true));
        r.lp.step(r.at(500));
        r.lp.io.sync();
        assert!(!auto.exists());
        r.cmd(Command::SetPaused(false));
        r.lp.step(r.at(500));
        r.lp.io.sync();
        assert!(auto.exists());
    }

    #[test]
    fn the_game_save_is_written_every_30_seconds() {
        let mut r = rig("loop-sram");
        r.frames(2);
        r.lp.step(r.at(10));
        r.lp.io.sync();
        assert!(!r.files.sram.exists());
        r.lp.step(r.at(31));
        r.lp.io.sync();
        assert_eq!(r.on_disk(&r.files.sram), Some(4));
    }

    fn near(sleep: Option<Duration>, ms: f64) -> bool {
        sleep.is_some_and(|d| (d.as_secs_f64() * 1000.0 - ms).abs() < 0.01)
    }

    #[test]
    fn pacing_sleeps_one_period_and_restarts_when_far_behind() {
        let mut r = rig("loop-pacing");
        r.frames(1);
        assert!(near(r.lp.pace(r.t0), 1000.0 / 60.0));
        // A second late (a window dragged): no burst to catch up, the clock starts again.
        let late = r.at(1);
        assert_eq!(r.lp.pace(late), None);
        assert!(near(r.lp.pace(late), 1000.0 / 60.0));
    }

    #[test]
    fn speed_pause_and_a_new_core_rate_restart_the_pacing() {
        let mut r = rig("loop-pacing-resets");
        r.frames(1);
        let _ = r.lp.pace(r.t0);
        let _ = r.lp.pace(r.t0);
        // Twice as fast: half a period, counted from the change.
        let t1 = r.at(5);
        r.lp.command(Command::SetSpeed(Speed::Factor(2.0)), t1);
        assert!(near(r.lp.pace(t1), 1000.0 / 120.0));
        assert_eq!(
            r.lp.core.calls.lock().audio_speed,
            Some(Some(2.0)),
            "the audio follows the speed"
        );
        // At Max nothing waits.
        r.lp.command(Command::SetSpeed(Speed::Max), t1);
        assert_eq!(r.lp.pace(t1), None);
        assert_eq!(r.lp.core.calls.lock().audio_speed, Some(None));
        // A pause and a resume start the clock from the resume.
        let t2 = r.at(9);
        r.lp.command(Command::SetSpeed(Speed::Factor(1.0)), t2);
        r.lp.command(Command::SetPaused(true), t2);
        let t3 = r.at(40);
        r.lp.command(Command::SetPaused(false), t3);
        assert!(near(r.lp.pace(t3), 1000.0 / 60.0));
        assert_eq!(r.lp.shared.stats.lock().emu_fps, 0.0);
        // The core changes its rate: the period follows from the next frame.
        r.lp.core.next_fps = Some(30.0);
        let t4 = r.at(41);
        r.lp.step(t4);
        assert!(near(r.lp.pace(t4), 1000.0 / 30.0));
    }

    #[test]
    fn reset_options_and_audio_reach_the_core() {
        let mut r = rig("loop-core-calls");
        r.frames(3);
        r.cmd(Command::Reset);
        assert_eq!(r.counter(), 0);
        r.cmd(Command::SetCoreOption {
            key: "mgba_skip_bios".into(),
            value: "ON".into(),
        });
        r.cmd(Command::SetFastAudio(FastAudio::KeepPitch));
        let calls = r.lp.core.calls.lock().clone();
        assert_eq!(calls.resets, 1);
        assert_eq!(
            calls.options,
            vec![("mgba_skip_bios".to_owned(), "ON".to_owned())]
        );
        assert_eq!(calls.fast_audio, Some(FastAudio::KeepPitch));
    }

    #[test]
    fn a_flush_writes_what_closing_would_and_the_game_goes_on() {
        let mut r = rig("loop-flush");
        r.frames(4);
        assert_eq!(r.cmd(Command::Flush), Flow::Continue);
        assert_eq!(r.on_disk(&r.files.sram), Some(4));
        assert_eq!(r.on_disk(&r.files.slot(AUTO_SLOT)), Some(4));
        r.frames(1);
        assert_eq!(r.counter(), 5);
    }

    #[test]
    fn stopping_writes_the_exit_saves_and_unloads_the_core() {
        let mut r = rig("loop-finish");
        r.frames(6);
        assert_eq!(r.cmd(Command::Stop), Flow::Stop);
        let calls = r.lp.core.calls.clone();
        let Rig {
            lp, events, files, ..
        } = r;
        assert_eq!(lp.finish(), Ok(()));
        assert_eq!(calls.lock().ended, Some(false));
        assert_eq!(
            FakeCore::counter(&std::fs::read(&files.sram).unwrap()),
            Some(6)
        );
        assert_eq!(
            FakeCore::counter(&std::fs::read(files.slot(AUTO_SLOT)).unwrap()),
            Some(6)
        );
        assert!(matches!(events.try_iter().last(), Some(Event::Stopped)));
    }

    #[test]
    fn a_let_go_loop_writes_nothing_and_leaves_the_core_loaded() {
        let mut r = rig("loop-let-go");
        r.frames(6);
        r.dead.store(true, Ordering::SeqCst);
        assert!(!r.lp.alive());
        let calls = r.lp.core.calls.clone();
        let Rig { lp, files, .. } = r;
        lp.let_go();
        assert_eq!(calls.lock().ended, Some(true));
        assert!(!files.sram.exists());
        assert!(!files.slot(AUTO_SLOT).exists());
    }

    #[test]
    fn the_quick_slot_is_recorded_clamped_and_restored_when_the_game_opens_again() {
        let mut r = rig("loop-quick-slot");
        assert_eq!(crate::quick_slot::restore(&r.files), 1);
        r.cmd(Command::SetQuickSlot(4));
        assert!(r.events().iter().all(|e| !matches!(e, Event::Error { .. })));
        assert_eq!(
            crate::quick_slot::read(&r.files),
            crate::quick_slot::Found::Slot(4)
        );
        // Out of range: the nearest quick slot.
        r.cmd(Command::SetQuickSlot(0));
        r.events();
        assert_eq!(crate::quick_slot::restore(&r.files), 1);
        r.cmd(Command::SetQuickSlot(77));
        r.cmd(Command::Stop);
        let Rig { lp, files, .. } = r;
        assert_eq!(lp.finish(), Ok(()));
        // The next open reads what the last session wrote.
        assert_eq!(crate::quick_slot::restore(&files), crate::quick_slot::LAST);
    }

    #[test]
    fn an_unreadable_quick_slot_is_left_as_it_is_until_the_user_moves_it() {
        let mut r = rig("loop-quick-slot-broken");
        let path = r.files.quick_slot();
        std::fs::write(&path, b"{ broken").unwrap();
        assert_eq!(crate::quick_slot::restore(&r.files), 1);
        r.frames(3);
        r.cmd(Command::SaveState(2));
        r.cmd(Command::Flush);
        r.events();
        assert_eq!(std::fs::read(&path).unwrap(), b"{ broken");
        // A change the user makes is the one thing that writes it again.
        r.cmd(Command::SetQuickSlot(3));
        r.events();
        assert_eq!(crate::quick_slot::restore(&r.files), 3);
    }

    fn unsaved(r: &Rig) -> bool {
        r.lp.shared.stats.lock().unsaved
    }

    /// «Chiudere senza salvare?»: the game moved since the user's last quick save or load.
    #[test]
    fn unsaved_follows_the_frames_and_the_users_saves_and_loads() {
        let mut r = rig_with("loop-unsaved", |opts, _| {
            opts.autosave_every = Some(Duration::from_secs(60));
            opts.rewind_seconds = Some(10);
        });
        assert!(!unsaved(&r), "nothing ran yet");
        // Paused, nothing runs: still nothing to save.
        r.cmd(Command::SetPaused(true));
        r.frames(3);
        assert!(!unsaved(&r));
        r.cmd(Command::SetPaused(false));
        r.frames(1);
        assert!(unsaved(&r), "a frame ran");

        r.cmd(Command::SaveState(2));
        assert!(!unsaved(&r), "the user saved");
        r.events();
        assert!(!unsaved(&r), "the write landed");
        r.frames(1);
        assert!(unsaved(&r));

        // The auto quick save the app writes does not count.
        r.lp.step(r.at(61));
        r.lp.io.sync();
        assert!(r.files.slot(AUTO_SLOT).exists());
        assert!(unsaved(&r));

        r.cmd(Command::LoadState(2));
        assert!(!unsaved(&r), "the user loaded");
        r.cmd(Command::UndoLoad);
        assert!(unsaved(&r), "the load was undone");

        r.cmd(Command::SaveState(3));
        assert!(!unsaved(&r));
        r.cmd(Command::UndoSave);
        assert!(unsaved(&r), "the save was undone");

        // Nothing to undo leaves it as it is.
        r.cmd(Command::SaveState(3));
        r.cmd(Command::UndoLoad);
        assert!(!unsaved(&r));

        // Rewinding changes the game too.
        for _ in 0..5 {
            r.frames(1);
            r.lp.rewind.sync();
        }
        r.cmd(Command::SaveState(3));
        r.cmd(Command::Rewind(true));
        r.frames(1);
        assert!(unsaved(&r), "rewound");
        r.cmd(Command::Rewind(false));

        r.cmd(Command::SaveState(3));
        r.cmd(Command::Reset);
        assert!(unsaved(&r), "reset");
    }

    #[test]
    fn a_quick_save_that_fails_leaves_the_game_unsaved() {
        let mut r = rig("loop-unsaved-failed");
        r.frames(2);
        // A folder where the slot's file goes: the write cannot land.
        std::fs::create_dir_all(r.files.slot(4)).unwrap();
        r.cmd(Command::SaveState(4));
        let events = r.events();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Error {
                error: ErrorCode::StateSaveFailed { slot: 4, .. }
            }
        )));
        assert!(unsaved(&r));
        // A core that cannot serialize never cleared it.
        r.lp.core.no_states = true;
        r.cmd(Command::SaveState(5));
        assert!(unsaved(&r));
    }

    #[test]
    fn a_let_go_session_does_not_record_the_quick_slot() {
        let mut r = rig("loop-quick-slot-let-go");
        r.dead.store(true, Ordering::SeqCst);
        r.cmd(Command::SetQuickSlot(6));
        r.events();
        assert!(!r.files.quick_slot().exists());
    }
}
