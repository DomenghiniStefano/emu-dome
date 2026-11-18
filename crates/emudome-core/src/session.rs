//! A game in progress: emulation thread, frame pacing, commands.

use std::ffi::CString;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use libretro_sys::retro_system_av_info;
use parking_lot::Mutex;

use crate::audio::AudioQueue;
use crate::callbacks::{self, CallbackCtx};
use crate::core::{GameFiles, LoadedCore};
use crate::core_options::CoreOptions;
use crate::error::ErrorCode;
use crate::frame::Frame;
use crate::input::InputState;
use crate::rewind::Rewind;

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
    /// Saves the frame on screen as a PNG next to the saves ([`crate::screenshot`]).
    Screenshot,
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
    pub audio: Mutex<AudioQueue>,
    pub input: InputState,
    pub stats: Mutex<Stats>,
    /// «Muto» in the quick menu: the audio output plays silence for this game.
    pub muted: std::sync::atomic::AtomicBool,
}

pub struct SessionOptions {
    pub core_path: PathBuf,
    pub game_path: PathBuf,
    /// Folder of the BIOS and system files (empty is fine: the cores have replacements).
    pub system_dir: PathBuf,
    /// Folder of the game saves and the slots.
    pub save_dir: PathBuf,
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
}

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
pub fn slot_infos(
    save_dir: &std::path::Path,
    game_path: &std::path::Path,
    count: u8,
) -> Vec<SlotInfo> {
    let files = GameFiles::new(save_dir, game_path);
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

/// Writes the thumbnail of a slot from the frame on screen (failures only logged: a slot
/// without a picture still loads).
fn write_thumb(files: &GameFiles, slot: u8, shared: &Shared) {
    let png = shared.frame.lock().to_png();
    match png {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(files.thumb(slot), bytes) {
                log::warn!("slot {slot} thumbnail not written: {e}");
            }
        }
        Err(e) => log::warn!("slot {slot} thumbnail not encoded: {e}"),
    }
}

/// Whether a game has an auto quick save to resume from.
pub fn has_auto_save(save_dir: &std::path::Path, game_path: &std::path::Path) -> bool {
    GameFiles::new(save_dir, game_path)
        .slot(AUTO_SLOT)
        .is_file()
}

pub struct Session {
    tx: Sender<Command>,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Session {
    /// Starts the emulation thread and waits for the game to load.
    /// Later events arrive on `events`.
    pub fn start(opts: SessionOptions, events: Sender<Event>) -> Result<Self> {
        std::fs::create_dir_all(&opts.system_dir).ok();
        std::fs::create_dir_all(&opts.save_dir).ok();

        let shared = Arc::new(Shared::default());
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<()>>(1);

        let thread_shared = shared.clone();
        let thread = std::thread::Builder::new()
            .name("emudome-emu".into())
            .spawn(move || emu_thread(opts, thread_shared, rx, events, ready_tx))
            .context("cannot start the emulation thread")?;

        ready_rx
            .recv()
            .context("the emulation thread exited while starting")??;

        Ok(Self {
            tx,
            shared,
            thread: Some(thread),
        })
    }

    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }

    pub fn shared(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// Lets go of an emulation thread that stopped answering (E11): asks it to stop but does not
    /// wait for it, since joining a core stuck inside `retro_run` would block forever.
    pub fn abandon(mut self) {
        let _ = self.tx.send(Command::Stop);
        if let Some(t) = self.thread.take() {
            log::warn!("emulation thread abandoned: {:?}", t.thread().name());
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn emu_thread(
    opts: SessionOptions,
    shared: Arc<Shared>,
    rx: Receiver<Command>,
    events: Sender<Event>,
    ready: mpsc::SyncSender<Result<()>>,
) {
    let to_c = |p: &PathBuf| CString::new(p.to_string_lossy().as_bytes()).unwrap_or_default();
    callbacks::install(CallbackCtx {
        shared: shared.clone(),
        pixel_format: libretro_sys::RETRO_PIXEL_FORMAT_0RGB1555,
        system_dir: to_c(&opts.system_dir),
        save_dir: to_c(&opts.save_dir),
        audio_enabled: true,
        options: CoreOptions::new(opts.core_options.iter().cloned()),
    });

    let started = (|| -> Result<(LoadedCore, GameFiles)> {
        let mut core = unsafe { LoadedCore::load(&opts.core_path)? };
        unsafe { core.load_game(&opts.game_path)? };
        let files = GameFiles::new(&opts.save_dir, &opts.game_path);
        if let (Some(ram), Ok(data)) = (core.save_ram_mut(), std::fs::read(&files.sram)) {
            let n = ram.len().min(data.len());
            ram[..n].copy_from_slice(&data[..n]);
        }
        Ok((core, files))
    })();

    let (core, files) = match started {
        Ok(v) => {
            let _ = ready.send(Ok(()));
            v
        }
        Err(e) => {
            callbacks::clear();
            let _ = ready.send(Err(e));
            return;
        }
    };

    // The core's version: quick saves made by another one might not load (Saves, V02).
    let slot_meta = crate::saves::SlotMeta {
        core: core.library_name.clone(),
        core_version: core.library_version.clone(),
        app_version: opts.app_version.clone(),
    };
    if let Err(e) =
        crate::saves::note_core_version(&opts.save_dir, &slot_meta.core, &slot_meta.core_version)
    {
        log::warn!("core version not recorded: {e}");
    }
    // What wrote a slot, beside it (failures only logged: the slot still loads).
    let write_meta = |slot: u8| {
        if let Err(e) = crate::saves::write_slot_meta(&files, slot, &slot_meta) {
            log::warn!("slot {slot} record not written: {e}");
        }
    };

    let av = core.av_info;
    {
        let mut stats = shared.stats.lock();
        stats.av_info = av;
        stats.speed = Some(Speed::Factor(1.0));
    }
    shared.audio.lock().set_rate(av.timing.sample_rate);
    let _ = events.send(Event::Started {
        core: format!("{} {}", core.library_name, core.library_version),
        game: opts
            .game_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        fps: av.timing.fps,
        width: av.geometry.base_width,
        height: av.geometry.base_height,
    });

    let base_fps = if av.timing.fps > 0.0 {
        av.timing.fps
    } else {
        60.0
    };
    let mut speed = Speed::Factor(1.0);
    let mut paused = false;
    let mut rewind = Rewind::new(opts.rewind_seconds, base_fps);
    let mut rewinding = false;
    let mut next_frame = Instant::now();
    let mut fps_window = (Instant::now(), 0u32);
    let mut last_sram_flush = Instant::now();
    let mut last_autosave = Instant::now();
    // Pacing diagnostics (Trello #57): frames run, sleeps and resets per 10 s window, logged.
    let mut pace = (Instant::now(), 0u32, 0u32, 0u32);

    let save_auto = |core: &LoadedCore| match core.save_state().and_then(|data| {
        std::fs::write(files.slot(AUTO_SLOT), data).context("cannot write the auto save")
    }) {
        Ok(()) => {
            write_thumb(&files, AUTO_SLOT, &shared);
            write_meta(AUTO_SLOT);
            shared.stats.lock().auto_saved_at = Some(std::time::SystemTime::now());
        }
        Err(e) => {
            let _ = events.send(Event::Error {
                error: ErrorCode::StateSaveFailed {
                    slot: AUTO_SLOT,
                    detail: format!("{e:#}"),
                },
            });
        }
    };

    // What the last save or load can go back to («Annulla»): only the newest one, until the
    // next save or load. Always kept; the interface offers the load's only when `saves.undo`
    // is on.
    let mut undo: Option<Undo> = None;

    if opts.resume_auto && files.slot(AUTO_SLOT).is_file() {
        let _ = events.send(match load_undoable(&core, &files, AUTO_SLOT, &mut undo) {
            Ok(()) => Event::StateLoaded { slot: AUTO_SLOT },
            Err(error) => Event::Error { error },
        });
    }

    let write_sram = |core: &LoadedCore| {
        if let Some(ram) = core.save_ram() {
            if let Err(e) = crate::saves::write_atomic(&files.sram, ram) {
                log::warn!("game save not written: {e}");
            }
        }
    };

    'main: loop {
        // Commands: paused, wait for them; otherwise check without blocking.
        let first = if paused {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(c) => Some(c),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            rx.try_recv().ok()
        };
        for cmd in first
            .into_iter()
            .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        {
            match cmd {
                Command::SetSpeed(s) => {
                    log::info!("pacing: speed set to {s:?}");
                    speed = s;
                    callbacks::set_audio_enabled(s.is_normal());
                    if !s.is_normal() {
                        shared.audio.lock().samples.clear();
                    }
                    shared.stats.lock().speed = Some(s);
                    next_frame = Instant::now();
                    fps_window = (Instant::now(), 0);
                }
                Command::SetPaused(p) => {
                    paused = p;
                    shared.stats.lock().paused = p;
                    next_frame = Instant::now();
                }
                Command::SaveState(slot) => {
                    let before = crate::saves::SlotContent::read(&files, slot)
                        .inspect_err(|e| log::warn!("slot {slot} not set aside for undo: {e}"))
                        .ok();
                    let res = core.save_state().and_then(|data| {
                        std::fs::write(files.slot(slot), data).context("cannot write the slot")
                    });
                    if res.is_ok() {
                        write_thumb(&files, slot, &shared);
                        write_meta(slot);
                        undo = before.map(|before| Undo::Save { slot, before });
                    }
                    let _ = events.send(match res {
                        Ok(()) => Event::StateSaved { slot },
                        Err(e) => Event::Error {
                            error: ErrorCode::StateSaveFailed {
                                slot,
                                detail: format!("{e:#}"),
                            },
                        },
                    });
                }
                Command::LoadState(slot) => {
                    let _ = events.send(match load_undoable(&core, &files, slot, &mut undo) {
                        Ok(()) => Event::StateLoaded { slot },
                        Err(error) => Event::Error { error },
                    });
                }
                Command::UndoLoad => {
                    let _ = events.send(match undo.take() {
                        Some(Undo::Load(data)) => match core.load_state(&data) {
                            Ok(()) => Event::LoadUndone,
                            Err(e) => Event::Error {
                                error: ErrorCode::UndoFailed {
                                    detail: format!("{e:#}"),
                                },
                            },
                        },
                        kept @ (Some(Undo::Save { .. }) | None) => {
                            undo = kept;
                            Event::Error {
                                error: ErrorCode::NothingToUndo,
                            }
                        }
                    });
                }
                Command::UndoSave => {
                    let _ = events.send(match undo.take() {
                        Some(Undo::Save { slot, before }) => match before.restore(&files, slot) {
                            Ok(()) => Event::SaveUndone { slot },
                            Err(e) => Event::Error {
                                error: ErrorCode::UndoFailed {
                                    detail: e.to_string(),
                                },
                            },
                        },
                        kept @ (Some(Undo::Load(_)) | None) => {
                            undo = kept;
                            Event::Error {
                                error: ErrorCode::NothingToUndo,
                            }
                        }
                    });
                }
                Command::SetCoreOption { key, value } => callbacks::set_core_option(key, value),
                Command::Reset => unsafe { (core.reset)() },
                Command::Rewind(on) => {
                    if on != rewinding {
                        rewinding = on;
                        // Silence while going back; the speed decides again afterwards.
                        callbacks::set_audio_enabled(!on && speed.is_normal());
                        shared.audio.lock().samples.clear();
                        shared.stats.lock().rewinding = on;
                    }
                }
                Command::SetRewindLength(seconds) => {
                    rewind.set_seconds(seconds);
                    let (states, bytes) = rewind.usage();
                    let mut stats = shared.stats.lock();
                    (stats.rewind_states, stats.rewind_bytes) = (states, bytes);
                }
                Command::Screenshot => {
                    let png = shared.frame.lock().to_png();
                    let res = png.map_err(|e| e.to_string()).and_then(|png| {
                        crate::screenshot::write(&files, &png, std::time::SystemTime::now())
                            .map_err(|e| e.to_string())
                    });
                    let _ = events.send(match res {
                        Ok(path) => Event::ScreenshotSaved {
                            path: path.to_string_lossy().into_owned(),
                        },
                        Err(detail) => Event::Error {
                            error: ErrorCode::ScreenshotFailed { detail },
                        },
                    });
                }
                Command::Stop => break 'main,
            }
        }
        if paused {
            continue;
        }

        if rewinding {
            // One state back, then one frame run to show it. With nothing left the game
            // stays on its picture (the frame count still moves: the thread is alive).
            if let Some(state) = rewind.step_back() {
                match core.load_state(&state) {
                    Ok(()) => unsafe { (core.run)() },
                    Err(e) => log::warn!("rewind: state not loaded: {e:#}"),
                }
            }
        } else {
            unsafe { (core.run)() };
            rewind.record(|| core.save_state().ok());
        }
        {
            let (states, bytes) = rewind.usage();
            let mut stats = shared.stats.lock();
            stats.frames += 1;
            (stats.rewind_states, stats.rewind_bytes) = (states, bytes);
        }

        // Auto quick save every `saves.autoq`, only while the game runs.
        if let Some(every) = opts.autosave_every {
            if last_autosave.elapsed() >= every {
                save_auto(&core);
                last_autosave = Instant::now();
            }
        }

        // Emulation FPS.
        fps_window.1 += 1;
        let elapsed = fps_window.0.elapsed();
        if elapsed >= Duration::from_secs(1) {
            shared.stats.lock().emu_fps = fps_window.1 as f64 / elapsed.as_secs_f64();
            fps_window = (Instant::now(), 0);
        }

        // Game save every 30 s, and on exit.
        if last_sram_flush.elapsed() >= Duration::from_secs(30) {
            write_sram(&core);
            last_sram_flush = Instant::now();
        }

        // Pacing: at a fixed speed wait for the next frame, at Max do not.
        if let Speed::Factor(f) = speed {
            let period = Duration::from_secs_f64(1.0 / (base_fps * f.max(0.05) as f64));
            next_frame += period;
            let now = Instant::now();
            if next_frame > now {
                std::thread::sleep(next_frame - now);
                pace.2 += 1;
            } else if now - next_frame > period * 4 {
                // Too far behind (e.g. window dragged): restart without catching up.
                next_frame = now;
                pace.3 += 1;
            }
        }
        pace.1 += 1;
        let window = pace.0.elapsed();
        if window >= Duration::from_secs(10) {
            log::info!(
                "pacing: {:.2} frames/s over {:.1} s (speed {speed:?}, base {base_fps:.4},                  {} sleeps, {} resets)",
                f64::from(pace.1) / window.as_secs_f64(),
                window.as_secs_f64(),
                pace.2,
                pace.3,
            );
            pace = (Instant::now(), 0, 0, 0);
        }
    }

    if opts.autosave_on_exit {
        save_auto(&core);
    }
    write_sram(&core);
    drop(core);
    callbacks::clear();
    let _ = events.send(Event::Stopped);
}

/// What «Annulla» goes back to: the newest save or load only.
enum Undo {
    /// The game's state from before the last load.
    Load(Vec<u8>),
    /// What the slot of the last save held before it.
    Save {
        slot: u8,
        before: crate::saves::SlotContent,
    },
}

/// [`load_slot`], keeping the state from before it in `undo` when it succeeds.
fn load_undoable(
    core: &LoadedCore,
    files: &GameFiles,
    slot: u8,
    undo: &mut Option<Undo>,
) -> Result<(), ErrorCode> {
    let state = core.save_state().ok();
    load_slot(core, files, slot)?;
    *undo = state.map(Undo::Load);
    Ok(())
}

/// Reads a slot and hands it to the core, telling an empty slot from a broken one.
fn load_slot(core: &LoadedCore, files: &GameFiles, slot: u8) -> Result<(), ErrorCode> {
    // An `if` rather than a `match`: io::ErrorKind is non-exhaustive, so a match would need
    // the `_ =>` arm the workspace lints forbid.
    let data = std::fs::read(files.slot(slot)).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ErrorCode::SlotEmpty { slot }
        } else {
            ErrorCode::StateLoadFailed {
                slot,
                detail: e.to_string(),
            }
        }
    })?;
    core.load_state(&data)
        .map_err(|_| ErrorCode::StateInvalid { slot })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{has_auto_save, slot_infos, GameFiles, AUTO_SLOT};

    #[test]
    fn the_auto_quick_save_has_its_own_file() {
        let files = GameFiles::new(Path::new("saves"), Path::new("games/Game.gba"));
        assert!(files.slot(AUTO_SLOT).ends_with("Game.auto.state"));
        assert!(files.slot(1).ends_with("Game.slot1.state"));
        assert!(files.thumb(3).ends_with("Game.slot3.png"));
        let empty = slot_infos(Path::new("no-such-dir"), Path::new("games/Game.gba"), 3);
        assert_eq!(empty.len(), 4);
        assert!(empty
            .iter()
            .all(|s| s.saved_at.is_none() && s.thumb.is_none()));
        assert!(!has_auto_save(
            Path::new("no-such-dir"),
            Path::new("games/Game.gba")
        ));
    }
}
