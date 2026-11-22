//! The open game and everything that lives exactly as long as it does (Trello #81).
//!
//! One [`GameSession`] in the app owns the running [`Session`], the real-time readers' handle
//! ([`CurrentShared`]), the key of the game whose files are in use, and the per-game parts that
//! start and stop with it ([`Hooks`]: the native renderer's source, the settings session, the core
//! options that follow the settings, the library's play time). Every way a game starts goes
//! through [`GameSession::open`]; every way it ends goes through [`GameSession::take`] (the close
//! flow, `closing.rs`), [`GameSession::close`] (in place, `RunEvent::Exit` only) or
//! [`GameSession::abandon`] (E11). A new per-game part is one more `Hooks`, in one place.
//!
//! No Tauri here: the app builds the hooks around its `AppHandle` (`lib.rs`), the tests run real
//! sessions on `emudome_core::testing::FakeCore`.

use std::sync::Arc;
use std::time::Duration;

use emudome_core::core_profile::CoreId;
use emudome_core::settings::core_values::MappedOption;
use emudome_core::{Closed, Command, ConsoleId, ErrorCode, ExitWrites, GameKey, Session, Shared};
use parking_lot::Mutex;

use crate::CurrentShared;

/// A game that just started: what the per-game parts need to know.
pub struct Opened {
    pub key: GameKey,
    pub core: CoreId,
    pub console: ConsoleId,
    /// A library game: its play time counts.
    pub library: bool,
    /// The core options it opened with (from the settings).
    pub core_options: Vec<MappedOption>,
    pub shared: Arc<Shared>,
}

type OpenedFn = Box<dyn Fn(&Opened) + Send + Sync>;
type ClosedFn = Box<dyn Fn() + Send + Sync>;

/// A check of what one game's session left behind, run once after its thread ended (Azahar's own
/// folder, Trello #97): taken with [`Leaving::take_after_end`] by the close flow's worker and run
/// off every lock. A game let go ([`Leaving::abandon`], [`Leaving::close`] on `RunEvent::Exit`)
/// drops it unrun: its thread may still be writing, or the app is exiting.
pub type AfterEnd = Box<dyn FnOnce() + Send>;

/// A part of the app that lives as long as a game is open: told when one opens, and when it
/// leaves (before its thread is waited for). Opened in the order given, closed in reverse.
pub struct Hooks {
    opened: OpenedFn,
    closed: ClosedFn,
}

impl Hooks {
    pub fn new(
        opened: impl Fn(&Opened) + Send + Sync + 'static,
        closed: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            opened: Box::new(opened),
            closed: Box::new(closed),
        }
    }
}

/// The running game.
struct Running {
    session: Session,
    key: GameKey,
    core: CoreId,
    after_end: Option<AfterEnd>,
}

/// The key of the game whose files are in use: set when it opens, cleared once its thread has
/// ended (the Saves screen refuses that game's files until then).
type InUse = Arc<Mutex<Option<GameKey>>>;

pub struct GameSession {
    running: Mutex<Option<Running>>,
    current: CurrentShared,
    in_use: InUse,
    hooks: Vec<Hooks>,
    /// Held for the whole of an in-place close or an abandon: a second one waits for the first,
    /// so the app never quits before the exit writes are done. The emulation thread never takes
    /// it.
    ending: Mutex<()>,
}

impl GameSession {
    pub fn new(current: CurrentShared, hooks: Vec<Hooks>) -> Self {
        Self {
            running: Mutex::new(None),
            current,
            in_use: Arc::default(),
            hooks,
            ending: Mutex::new(()),
        }
    }

    /// The open game's shared state, for the real-time readers (no lock).
    pub fn current(&self) -> &CurrentShared {
        &self.current
    }

    pub fn is_open(&self) -> bool {
        self.running.lock().is_some()
    }

    /// The game whose files are in use: open, or closing until its thread ends.
    pub fn in_use(&self) -> Option<GameKey> {
        self.in_use.lock().clone()
    }

    /// A command to the open game; `noGame` without one.
    pub fn send(&self, cmd: Command) -> Result<(), ErrorCode> {
        match self.running.lock().as_ref() {
            Some(r) => {
                r.session.send(cmd);
                Ok(())
            }
            None => Err(ErrorCode::NoGame),
        }
    }

    /// A started session becomes the open game: the real-time readers see it, then each part
    /// is told. The previous game must be closed first (the close flow does it). `after_end`
    /// runs once its thread ended ([`AfterEnd`]).
    pub fn open(&self, session: Session, opened: &Opened, after_end: Option<AfterEnd>) {
        let previous = {
            let mut running = self.running.lock();
            let previous = running.take();
            *running = Some(Running {
                session,
                key: opened.key.clone(),
                core: opened.core,
                after_end,
            });
            previous
        };
        if let Some(previous) = previous {
            // Never happens through the commands (opening closes first): closed in place.
            log::error!("a game opened over another one: closing the old one in place");
            drop(previous.session);
        }
        self.current.store(Some(opened.shared.clone()));
        *self.in_use.lock() = Some(opened.key.clone());
        for hooks in &self.hooks {
            (hooks.opened)(opened);
        }
    }

    /// Takes the open game out of the app (the parts told, in reverse order; the real-time
    /// readers let go), its thread still running: the caller stops it and waits for it.
    pub fn take(&self) -> Option<Leaving> {
        for hooks in self.hooks.iter().rev() {
            (hooks.closed)();
        }
        self.current.store(None);
        // Taken out first: the lock is not held while the thread is waited for.
        let running = self.running.lock().take()?;
        Some(Leaving {
            session: running.session,
            key: running.key,
            core: running.core,
            in_use: self.in_use.clone(),
            after_end: running.after_end,
        })
    }

    /// Ends the open game in place, bounded by `timeout`: only for `RunEvent::Exit`, which cannot
    /// wait (every other way out goes through the close flow). `Some` with how it ended.
    pub fn close(&self, timeout: Duration) -> Option<(CoreId, Closed)> {
        let _ending = self.ending.lock();
        let leaving = self.take()?;
        let core = leaving.core;
        Some((core, leaving.close(timeout)))
    }

    /// E11, the core stopped answering: lets go of its thread without waiting for it. The session
    /// is dead (it writes nothing more) and its core refused until EmuDome restarts.
    pub fn abandon(&self) -> Option<CoreId> {
        let _ending = self.ending.lock();
        let leaving = self.take()?;
        let core = leaving.core;
        leaving.abandon();
        Some(core)
    }
}

/// A game taken out of the app whose thread has not ended yet. Ending it ([`Leaving::wait`]
/// answering, [`Leaving::close`], [`Leaving::abandon`]) frees its files.
pub struct Leaving {
    session: Session,
    key: GameKey,
    core: CoreId,
    in_use: InUse,
    after_end: Option<AfterEnd>,
}

impl Leaving {
    pub fn core(&self) -> CoreId {
        self.core
    }

    /// The check to run now that its thread ended ([`Leaving::wait`] answered), once; the caller
    /// runs it outside its locks.
    pub fn take_after_end(&mut self) -> Option<AfterEnd> {
        self.after_end.take()
    }

    /// Asks the thread to stop (it writes the in-game save, and the auto quick save with
    /// `saves.autoexit`).
    pub fn stop(&self) {
        self.session.stop();
    }

    /// Waits up to `timeout` for the thread after [`Leaving::stop`]: `Some` with the exit writes'
    /// outcome once it ended (its files are free), `None` while it runs. Never lets it go.
    pub fn wait(&mut self, timeout: Duration) -> Option<ExitWrites> {
        let written = self.session.wait(timeout)?;
        self.free();
        Some(written)
    }

    /// Stops the thread and waits up to `timeout`, then lets it go.
    pub fn close(self, timeout: Duration) -> Closed {
        let Leaving {
            session,
            key,
            in_use,
            ..
        } = self;
        let closed = session.close(timeout);
        free(&in_use, &key);
        closed
    }

    /// Lets the thread go at once (E11, «Close anyway»).
    pub fn abandon(self) {
        let Leaving {
            session,
            key,
            in_use,
            ..
        } = self;
        session.abandon();
        free(&in_use, &key);
    }

    fn free(&self) {
        free(&self.in_use, &self.key);
    }
}

/// The game's files are free again, unless another game took them meanwhile.
fn free(in_use: &InUse, key: &GameKey) {
    let mut in_use = in_use.lock();
    if in_use.as_ref() == Some(key) {
        *in_use = None;
    }
}

#[cfg(test)]
mod tests {
    //! Rule 1 of the audit: the data survives every way a game ends, through `GameSession`, on
    //! real session threads running the fake core.

    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use emudome_core::core_profile::CoreId;
    use emudome_core::testing::{FakeCore, Gate};
    use emudome_core::{
        core_needs_restart, Closed, Command, ConsoleId, GameFiles, GameKey, Session,
        SessionOptions, AUTO_SLOT, CLOSE_TIMEOUT,
    };
    use parking_lot::Mutex;

    use super::{GameSession, Hooks, Opened};

    /// A fresh folder under the system temp dir, removed when dropped.
    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "emudome-app-test-{name}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn key(hex: &str) -> GameKey {
        GameKey::parse(&format!("gba:{hex}")).unwrap()
    }

    fn options(dir: &Path, game: &GameKey, core: &str, autosave_on_exit: bool) -> SessionOptions {
        SessionOptions {
            core_path: dir.join(core),
            game_path: dir.join("Game.gba"),
            game_file: None,
            system_dir: dir.join("system"),
            save_dir: dir.join("saves"),
            game: game.clone(),
            autosave_every: None,
            autosave_on_exit,
            resume_auto: false,
            app_version: "test".into(),
            core_options: Vec::new(),
            rewind_seconds: None,
            state_load_block: None,
            split: emudome_core::screens::Split::Whole,
            username: None,
            fast_audio: emudome_core::speed_audio::FastAudio::Mute,
            video: Default::default(),
        }
    }

    /// What the parts were told, in order.
    type Log = Arc<Mutex<Vec<String>>>;

    fn games(log: &Log) -> GameSession {
        let hooks = ["renderer", "settings", "library"]
            .into_iter()
            .map(|name| {
                let (a, b) = (log.clone(), log.clone());
                Hooks::new(
                    move |g: &Opened| a.lock().push(format!("{name} opened {}", g.key)),
                    move || b.lock().push(format!("{name} closed")),
                )
            })
            .collect();
        GameSession::new(Arc::default(), hooks)
    }

    /// Opens `game` on a fake core (`core` names its file, for `coreNeedsRestart`) and waits
    /// until it ran a few frames.
    fn open(games: &GameSession, dir: &Path, game: &GameKey, core: &str, fake: FakeCore) {
        let opts = options(dir, game, core, true);
        open_with(games, opts, fake, true);
    }

    fn open_with(games: &GameSession, opts: SessionOptions, fake: FakeCore, wait_frames: bool) {
        open_after(games, opts, fake, wait_frames, None);
    }

    fn open_after(
        games: &GameSession,
        opts: SessionOptions,
        fake: FakeCore,
        wait_frames: bool,
        after_end: Option<super::AfterEnd>,
    ) {
        let game = opts.game.clone();
        let (tx, _rx) = std::sync::mpsc::channel();
        let session = Session::start_with(opts, tx, fake).unwrap();
        let shared = session.shared().clone();
        games.open(
            session,
            &Opened {
                key: game,
                core: CoreId::Mgba,
                console: ConsoleId::Gba,
                library: true,
                core_options: Vec::new(),
                shared: shared.clone(),
            },
            after_end,
        );
        if wait_frames {
            let started = Instant::now();
            while shared.stats.lock().frames < 3 {
                assert!(started.elapsed() < Duration::from_secs(10), "no frames");
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }

    fn counter(path: &Path) -> Option<u64> {
        FakeCore::counter(&std::fs::read(path).ok()?)
    }

    /// The in-game save and the auto quick save were written at the same frame, past the start.
    fn assert_exit_writes(files: &GameFiles) {
        let sram = counter(&files.sram).expect("the in-game save");
        let auto = counter(&files.slot(AUTO_SLOT)).expect("the auto quick save");
        assert!(sram >= 3 && sram == auto, "{sram} {auto}");
    }

    #[test]
    fn closing_through_the_flow_writes_the_saves_and_frees_the_files_after() {
        let dir = Dir::new("gs-flow");
        let log = Log::default();
        let games = games(&log);
        let game = key("01");
        open(&games, &dir.0, &game, "a_libretro.dll", FakeCore::new());
        assert!(games.is_open());
        assert_eq!(games.in_use(), Some(game.clone()));
        assert!(games.current().load().is_some());

        let mut leaving = games.take().unwrap();
        // Out of the app at once; its files stay refused until the thread ends.
        assert!(!games.is_open());
        assert!(games.current().load().is_none());
        assert_eq!(
            games.send(Command::Flush),
            Err(emudome_core::ErrorCode::NoGame)
        );
        assert_eq!(games.in_use(), Some(game.clone()));
        leaving.stop();
        let written = loop {
            if let Some(written) = leaving.wait(Duration::from_millis(100)) {
                break written;
            }
        };
        assert_eq!(written, Ok(()));
        assert_eq!(games.in_use(), None);
        assert_exit_writes(&GameFiles::new(&dir.0.join("saves"), &game));
        assert_eq!(
            *log.lock(),
            [
                "renderer opened gba:01",
                "settings opened gba:01",
                "library opened gba:01",
                "library closed",
                "settings closed",
                "renderer closed",
            ]
        );
    }

    #[test]
    fn the_after_end_check_is_handed_over_once_the_thread_ended() {
        let dir = Dir::new("gs-after-end");
        let log = Log::default();
        let games = games(&log);
        let game = key("0a");
        let ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = ran.clone();
        open_after(
            &games,
            options(&dir.0, &game, "k_libretro.dll", true),
            FakeCore::new(),
            true,
            Some(Box::new(move || flag.store(true, Ordering::SeqCst))),
        );
        let mut leaving = games.take().unwrap();
        leaving.stop();
        while leaving.wait(Duration::from_millis(100)).is_none() {}
        let check = leaving.take_after_end().expect("the check");
        assert!(leaving.take_after_end().is_none(), "handed over once");
        assert!(!ran.load(Ordering::SeqCst));
        check();
        assert!(ran.load(Ordering::SeqCst));
    }

    #[test]
    fn closing_in_place_on_exit_writes_the_saves() {
        let dir = Dir::new("gs-exit");
        let games = games(&Log::default());
        let game = key("02");
        open(&games, &dir.0, &game, "b_libretro.dll", FakeCore::new());
        assert_eq!(
            games.close(CLOSE_TIMEOUT),
            Some((CoreId::Mgba, Closed::Joined))
        );
        assert_eq!(games.in_use(), None);
        assert_exit_writes(&GameFiles::new(&dir.0.join("saves"), &game));
        // Nothing open: nothing to close.
        assert_eq!(games.close(CLOSE_TIMEOUT), None);
    }

    #[test]
    fn without_save_on_exit_only_the_in_game_save_is_written() {
        let dir = Dir::new("gs-no-autoexit");
        let games = games(&Log::default());
        let game = key("03");
        open_with(
            &games,
            options(&dir.0, &game, "c_libretro.dll", false),
            FakeCore::new(),
            true,
        );
        games.close(CLOSE_TIMEOUT);
        let files = GameFiles::new(&dir.0.join("saves"), &game);
        assert!(counter(&files.sram).is_some());
        assert!(!files.slot(AUTO_SLOT).exists());
    }

    #[test]
    fn opening_another_game_closes_the_first_with_its_saves() {
        let dir = Dir::new("gs-switch");
        let games = games(&Log::default());
        let (first, second) = (key("04"), key("05"));
        open(&games, &dir.0, &first, "d_libretro.dll", FakeCore::new());
        // What opening does: the close flow first, then the new game.
        let mut leaving = games.take().unwrap();
        leaving.stop();
        while leaving.wait(Duration::from_millis(100)).is_none() {}
        open(&games, &dir.0, &second, "d_libretro.dll", FakeCore::new());
        assert_exit_writes(&GameFiles::new(&dir.0.join("saves"), &first));
        assert_eq!(games.in_use(), Some(second.clone()));
        // The second one's in-game save, written when it closes, reads back as that game's.
        games.close(CLOSE_TIMEOUT);
        assert_exit_writes(&GameFiles::new(&dir.0.join("saves"), &second));
    }

    #[test]
    fn going_to_the_background_flushes_and_the_game_keeps_running() {
        let dir = Dir::new("gs-flush");
        let games = games(&Log::default());
        let game = key("06");
        open(&games, &dir.0, &game, "e_libretro.dll", FakeCore::new());
        let files = GameFiles::new(&dir.0.join("saves"), &game);
        assert_eq!(games.send(Command::Flush), Ok(()));
        let started = Instant::now();
        while !files.slot(AUTO_SLOT).exists() {
            assert!(started.elapsed() < Duration::from_secs(10), "not flushed");
            std::thread::sleep(Duration::from_millis(5));
        }
        // Written atomically: what is there is complete.
        let flushed = counter(&files.sram).unwrap();
        assert!(flushed >= 3);
        assert!(games.is_open());
        // ...and it keeps running, its closing save newer.
        std::thread::sleep(Duration::from_millis(100));
        games.close(CLOSE_TIMEOUT);
        assert!(counter(&files.sram).unwrap() > flushed);
    }

    #[test]
    fn abandoning_a_hung_game_writes_nothing_over_its_saves() {
        let dir = Dir::new("gs-abandon");
        let games = games(&Log::default());
        let game = key("07");
        let files = GameFiles::new(&dir.0.join("saves"), &game);
        std::fs::create_dir_all(files.dir()).unwrap();
        std::fs::write(&files.sram, 41u64.to_le_bytes()).unwrap();
        let gate = Gate::new();
        let opts = options(&dir.0, &game, "hung_libretro.dll", true);
        let core_path = opts.core_path.clone();
        open_with(&games, opts, FakeCore::stuck(gate.clone()), false);
        assert!(
            gate.wait_held(Duration::from_secs(10)),
            "the core never ran"
        );
        let started = Instant::now();
        assert_eq!(games.abandon(), Some(CoreId::Mgba));
        assert!(started.elapsed() < Duration::from_secs(1), "abandon waited");
        assert_eq!(games.in_use(), None);
        assert!(core_needs_restart(&core_path));
        // The thread wakes up later: it is dead and writes nothing.
        gate.open();
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(counter(&files.sram), Some(41));
        assert!(!files.slot(AUTO_SLOT).exists());
    }

    #[test]
    fn a_close_that_times_out_lets_the_game_go_and_keeps_the_old_saves() {
        let dir = Dir::new("gs-timeout");
        let games = games(&Log::default());
        let game = key("08");
        let files = GameFiles::new(&dir.0.join("saves"), &game);
        std::fs::create_dir_all(files.dir()).unwrap();
        std::fs::write(&files.sram, 42u64.to_le_bytes()).unwrap();
        let gate = Gate::new();
        let opts = options(&dir.0, &game, "stuck_libretro.dll", true);
        let core_path = opts.core_path.clone();
        open_with(&games, opts, FakeCore::stuck(gate.clone()), false);
        assert!(
            gate.wait_held(Duration::from_secs(10)),
            "the core never ran"
        );
        assert_eq!(
            games.close(Duration::from_millis(200)),
            Some((CoreId::Mgba, Closed::Abandoned))
        );
        assert!(core_needs_restart(&core_path));
        gate.open();
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(counter(&files.sram), Some(42));
    }

    #[test]
    fn the_flows_timeout_then_close_anyway_lets_the_game_go() {
        let dir = Dir::new("gs-close-anyway");
        let games = games(&Log::default());
        let game = key("09");
        let gate = Gate::new();
        let opts = options(&dir.0, &game, "slow_libretro.dll", true);
        let core_path = opts.core_path.clone();
        open_with(&games, opts, FakeCore::stuck(gate.clone()), false);
        assert!(
            gate.wait_held(Duration::from_secs(10)),
            "the core never ran"
        );
        let mut leaving = games.take().unwrap();
        leaving.stop();
        // «Wait»: still running, not let go.
        assert_eq!(leaving.wait(Duration::from_millis(100)), None);
        assert!(!core_needs_restart(&core_path));
        assert_eq!(games.in_use(), Some(game.clone()));
        // «Close anyway».
        leaving.abandon();
        assert!(core_needs_restart(&core_path));
        assert_eq!(games.in_use(), None);
        gate.open();
    }
}
