//! Dynamic loading of a libretro core and access to its functions.

use std::ffi::{c_void, CStr, CString};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use libloading::Library;

use crate::callbacks::Callbacks;
use crate::GameKey;
use libretro_sys::symbols::{
    RetroApiVersion, RetroDeinit, RetroGetMemoryData, RetroGetMemorySize, RetroGetSystemAvInfo,
    RetroGetSystemInfo, RetroInit, RetroLoadGame, RetroReset, RetroRun, RetroSerialize,
    RetroSerializeSize, RetroSetAudioSample, RetroSetAudioSampleBatch,
    RetroSetControllerPortDevice, RetroSetEnvironment, RetroSetInputPoll, RetroSetInputState,
    RetroSetVideoRefresh, RetroUnloadGame, RetroUnserialize,
};
use libretro_sys::{
    retro_game_info, retro_game_info_ext, retro_system_av_info, retro_system_info,
    RETRO_API_VERSION, RETRO_DEVICE_JOYPAD, RETRO_MEMORY_SAVE_RAM,
};

/// Why a game could not start, as the interface needs it to say what to do (mapped to an
/// [`crate::ErrorCode`] by `From`). `detail` is technical text (English, from the OS or the
/// loader), never a sentence for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartError {
    /// A game on this core was let go in this run: the core is refused until EmuDome restarts.
    CoreNeedsRestart,
    /// The core file cannot be loaded as a library (damaged, wrong architecture, a missing
    /// system library).
    CoreUnloadable { detail: String },
    /// The file loads but is not a libretro core EmuDome can drive (another API version, a
    /// function missing).
    CoreIncompatible { detail: String },
    /// The game file cannot be read.
    RomUnreadable { detail: String },
    /// The core looked at the game and refused it (`retro_load_game` returned false): a bad dump,
    /// an encrypted 3DS game, a format this core does not run.
    CoreRefusedGame,
    /// The emulation thread could not be started, or ended while starting.
    ThreadFailed { detail: String },
    /// The game's save folder ([`GameFiles::dir`]) could not be created. Refused before the core
    /// loads: a core that cannot reach it writes elsewhere (Azahar falls back to its own user
    /// folder, shared by every game; Trello #97).
    SaveFolderUnavailable { folder: PathBuf, detail: String },
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::CoreNeedsRestart => write!(f, "the core needs EmuDome to restart"),
            StartError::CoreUnloadable { detail } => write!(f, "cannot load the core: {detail}"),
            StartError::CoreIncompatible { detail } => {
                write!(f, "not a usable libretro core: {detail}")
            }
            StartError::RomUnreadable { detail } => write!(f, "cannot read the game: {detail}"),
            StartError::CoreRefusedGame => write!(f, "the core refused the game"),
            StartError::ThreadFailed { detail } => {
                write!(f, "the emulation thread failed: {detail}")
            }
            StartError::SaveFolderUnavailable { folder, detail } => {
                write!(
                    f,
                    "cannot create the save folder {}: {detail}",
                    folder.display()
                )
            }
        }
    }
}

impl std::error::Error for StartError {}

/// A libretro core loaded in memory. Every call must happen
/// on the same thread (the emulation one).
pub struct LoadedCore {
    pub run: RetroRun,
    pub reset: RetroReset,
    pub serialize_size: RetroSerializeSize,
    pub serialize: RetroSerialize,
    pub unserialize: RetroUnserialize,
    pub get_memory_data: RetroGetMemoryData,
    pub get_memory_size: RetroGetMemorySize,
    pub set_controller_port_device: RetroSetControllerPortDevice,
    /// The timing and geometry the core gave at load, updated by the session when the core
    /// changes them (`SET_SYSTEM_AV_INFO`).
    pub av_info: retro_system_av_info,
    pub library_name: String,
    pub library_version: String,
    /// The core wants the game's path only (`retro_system_info::need_fullpath`): it reads the
    /// file itself, and the host must not read it into memory (3DS games are gigabytes).
    pub need_fullpath: bool,
    load_game_fn: RetroLoadGame,
    get_av_info: RetroGetSystemAvInfo,
    unload_game: RetroUnloadGame,
    deinit: RetroDeinit,
    game_loaded: bool,
    // Kept alive while the core is in use; must be released last.
    _lib: Library,
}

/// Every function the host calls, looked up before `retro_init`: a missing one refuses the core
/// before it ran any code of its own, so there is never a `retro_init` without its
/// `retro_deinit`.
struct Symbols {
    set_environment: RetroSetEnvironment,
    set_video: RetroSetVideoRefresh,
    set_audio: RetroSetAudioSample,
    set_audio_batch: RetroSetAudioSampleBatch,
    set_input_poll: RetroSetInputPoll,
    set_input_state: RetroSetInputState,
    init: RetroInit,
    get_system_info: RetroGetSystemInfo,
    run: RetroRun,
    reset: RetroReset,
    serialize_size: RetroSerializeSize,
    serialize: RetroSerialize,
    unserialize: RetroUnserialize,
    get_memory_data: RetroGetMemoryData,
    get_memory_size: RetroGetMemorySize,
    set_controller_port_device: RetroSetControllerPortDevice,
    load_game: RetroLoadGame,
    get_av_info: RetroGetSystemAvInfo,
    unload_game: RetroUnloadGame,
    deinit: RetroDeinit,
}

macro_rules! sym {
    ($lib:expr, $ty:ty, $name:literal) => {{
        let s: libloading::Symbol<$ty> =
            $lib.get(concat!($name, "\0").as_bytes()).map_err(|e| {
                StartError::CoreIncompatible {
                    detail: format!("the core does not export {}: {e}", $name),
                }
            })?;
        *s
    }};
}

impl Symbols {
    /// # Safety
    /// `lib` must be a libretro core: the symbols are read with libretro's signatures.
    unsafe fn resolve(lib: &Library) -> Result<Self, StartError> {
        Ok(Self {
            set_environment: sym!(lib, RetroSetEnvironment, "retro_set_environment"),
            set_video: sym!(lib, RetroSetVideoRefresh, "retro_set_video_refresh"),
            set_audio: sym!(lib, RetroSetAudioSample, "retro_set_audio_sample"),
            set_audio_batch: sym!(
                lib,
                RetroSetAudioSampleBatch,
                "retro_set_audio_sample_batch"
            ),
            set_input_poll: sym!(lib, RetroSetInputPoll, "retro_set_input_poll"),
            set_input_state: sym!(lib, RetroSetInputState, "retro_set_input_state"),
            init: sym!(lib, RetroInit, "retro_init"),
            get_system_info: sym!(lib, RetroGetSystemInfo, "retro_get_system_info"),
            run: sym!(lib, RetroRun, "retro_run"),
            reset: sym!(lib, RetroReset, "retro_reset"),
            serialize_size: sym!(lib, RetroSerializeSize, "retro_serialize_size"),
            serialize: sym!(lib, RetroSerialize, "retro_serialize"),
            unserialize: sym!(lib, RetroUnserialize, "retro_unserialize"),
            get_memory_data: sym!(lib, RetroGetMemoryData, "retro_get_memory_data"),
            get_memory_size: sym!(lib, RetroGetMemorySize, "retro_get_memory_size"),
            set_controller_port_device: sym!(
                lib,
                RetroSetControllerPortDevice,
                "retro_set_controller_port_device"
            ),
            load_game: sym!(lib, RetroLoadGame, "retro_load_game"),
            get_av_info: sym!(lib, RetroGetSystemAvInfo, "retro_get_system_av_info"),
            unload_game: sym!(lib, RetroUnloadGame, "retro_unload_game"),
            deinit: sym!(lib, RetroDeinit, "retro_deinit"),
        })
    }
}

impl LoadedCore {
    /// Loads the library, wires `callbacks` (the ones of the core's slot) and calls `retro_init`.
    ///
    /// # Safety
    /// Loads and runs arbitrary native code: the file must be a libretro core.
    pub(crate) unsafe fn load(path: &Path, callbacks: Callbacks) -> Result<Self, StartError> {
        let lib = Library::new(path).map_err(|e| StartError::CoreUnloadable {
            detail: format!("{}: {e}", path.display()),
        })?;

        let api_version: RetroApiVersion = sym!(lib, RetroApiVersion, "retro_api_version");
        if api_version() != RETRO_API_VERSION {
            return Err(StartError::CoreIncompatible {
                detail: format!("libretro API version {}", api_version()),
            });
        }
        let s = Symbols::resolve(&lib)?;

        (s.set_environment)(callbacks.environment);
        (s.set_video)(callbacks.video_refresh);
        (s.set_audio)(callbacks.audio_sample);
        (s.set_audio_batch)(callbacks.audio_sample_batch);
        (s.set_input_poll)(callbacks.input_poll);
        (s.set_input_state)(callbacks.input_state);
        (s.init)();

        let mut info: retro_system_info = std::mem::zeroed();
        (s.get_system_info)(&mut info);
        let cstr = |p: *const std::os::raw::c_char| {
            if p.is_null() {
                String::new()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };
        let core = Self {
            run: s.run,
            reset: s.reset,
            serialize_size: s.serialize_size,
            serialize: s.serialize,
            unserialize: s.unserialize,
            get_memory_data: s.get_memory_data,
            get_memory_size: s.get_memory_size,
            set_controller_port_device: s.set_controller_port_device,
            av_info: retro_system_av_info::default(),
            library_name: cstr(info.library_name),
            library_version: cstr(info.library_version),
            need_fullpath: info.need_fullpath,
            load_game_fn: s.load_game,
            get_av_info: s.get_av_info,
            unload_game: s.unload_game,
            deinit: s.deinit,
            game_loaded: false,
            _lib: lib,
        };
        log::info!(
            "core {} {} loaded (need_fullpath: {})",
            core.library_name,
            core.library_version,
            core.need_fullpath
        );
        Ok(core)
    }

    /// Loads the game: `content` holds its path, and its bytes unless the core reads the file
    /// itself ([`GameContent::read`]).
    pub(crate) unsafe fn load_game(&mut self, content: &GameContent) -> Result<(), StartError> {
        let info = content.info();
        if !(self.load_game_fn)(&info) {
            return Err(StartError::CoreRefusedGame);
        }
        self.game_loaded = true;
        (self.get_av_info)(&mut self.av_info);
        (self.set_controller_port_device)(0, RETRO_DEVICE_JOYPAD);
        Ok(())
    }

    /// Full emulation state (for the slots and, later, rewind).
    pub fn save_state(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.save_state_into(&mut buf)?;
        Ok(buf)
    }

    /// [`Self::save_state`] into `buf`, reusing its memory (rewind takes a state every few
    /// frames: no allocation once the buffers are there).
    pub fn save_state_into(&self, buf: &mut Vec<u8>) -> Result<()> {
        unsafe {
            let size = (self.serialize_size)();
            if size == 0 {
                bail!("this core does not support save states");
            }
            buf.resize(size, 0);
            if !(self.serialize)(buf.as_mut_ptr() as *mut c_void, size) {
                bail!("the core failed to save the state");
            }
            Ok(())
        }
    }

    pub fn load_state(&self, data: &[u8]) -> Result<()> {
        unsafe {
            if !(self.unserialize)(data.as_ptr() as *const c_void, data.len()) {
                bail!("state not valid for this game or this core version");
            }
        }
        Ok(())
    }

    /// The game's save memory (cartridge battery), read only.
    pub fn save_ram(&self) -> Option<&[u8]> {
        let (ptr, len) = self.save_ram_raw()?;
        Some(unsafe { std::slice::from_raw_parts(ptr, len) })
    }

    /// The game's save memory, to restore it from the `.sav` file.
    pub fn save_ram_mut(&mut self) -> Option<&mut [u8]> {
        let (ptr, len) = self.save_ram_raw()?;
        Some(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
    }

    fn save_ram_raw(&self) -> Option<(*mut u8, usize)> {
        unsafe {
            let ptr = (self.get_memory_data)(RETRO_MEMORY_SAVE_RAM) as *mut u8;
            let len = (self.get_memory_size)(RETRO_MEMORY_SAVE_RAM);
            (!ptr.is_null() && len > 0).then_some((ptr, len))
        }
    }
}

impl Drop for LoadedCore {
    fn drop(&mut self) {
        unsafe {
            if self.game_loaded {
                (self.unload_game)();
            }
            (self.deinit)();
        }
    }
}

/// What `retro_load_game` and `GET_GAME_INFO_EXT` get for a game: its path always, its bytes
/// only for a core that does not read the file itself.
pub(crate) struct GameContent {
    path: CString,
    /// `None` for a `need_fullpath` core: the file is never read by the host.
    data: Option<Vec<u8>>,
    dir: CString,
    name: CString,
    ext: CString,
}

impl GameContent {
    /// The game at `game` for a core that does (`need_fullpath`) or does not read the file
    /// itself. `read` reads the whole file (`std::fs::read`; a seam for the tests) and is called
    /// only when the core wants the bytes.
    pub(crate) fn read(
        game: &Path,
        need_fullpath: bool,
        read: impl FnOnce(&Path) -> std::io::Result<Vec<u8>>,
    ) -> Result<Self, StartError> {
        let c = |s: &std::ffi::OsStr| {
            CString::new(s.to_string_lossy().as_bytes()).map_err(|_| StartError::RomUnreadable {
                detail: format!("a NUL byte in the path {}", game.display()),
            })
        };
        let path = c(game.as_os_str())?;
        let data = if need_fullpath {
            None
        } else {
            Some(read(game).map_err(|e| StartError::RomUnreadable {
                detail: format!("{}: {e}", game.display()),
            })?)
        };
        Ok(Self {
            path,
            data,
            dir: c(game.parent().map(Path::as_os_str).unwrap_or_default())?,
            name: c(game.file_stem().unwrap_or_default())?,
            // libretro wants the extension in lower case.
            ext: CString::new(
                game.extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
        })
    }

    /// The `retro_game_info` for `retro_load_game`: `data = null, size = 0` when the core reads
    /// the file itself. Valid while `self` is.
    pub(crate) fn info(&self) -> retro_game_info {
        let (data, size) = self.bytes();
        retro_game_info {
            path: self.path.as_ptr(),
            data,
            size,
            meta: std::ptr::null(),
        }
    }

    /// The `retro_game_info_ext` for `GET_GAME_INFO_EXT`. The bytes are not persistent (dropped
    /// after `retro_load_game`), as `persistent_data = false` tells the core.
    pub(crate) fn info_ext(&self, with_data: bool) -> retro_game_info_ext {
        let (data, size) = if with_data {
            self.bytes()
        } else {
            (std::ptr::null(), 0)
        };
        retro_game_info_ext {
            full_path: self.path.as_ptr(),
            archive_path: std::ptr::null(),
            archive_file: std::ptr::null(),
            dir: self.dir.as_ptr(),
            name: self.name.as_ptr(),
            ext: self.ext.as_ptr(),
            meta: std::ptr::null(),
            data,
            size,
            file_in_archive: false,
            persistent_data: false,
        }
    }

    /// The strings `info_ext` points into, to keep alive while the core may ask for them.
    pub(crate) fn into_strings(self) -> GameStrings {
        GameStrings {
            _path: self.path,
            _dir: self.dir,
            _name: self.name,
            _ext: self.ext,
        }
    }

    fn bytes(&self) -> (*const c_void, usize) {
        match &self.data {
            Some(d) => (d.as_ptr() as *const c_void, d.len()),
            None => (std::ptr::null(), 0),
        }
    }
}

/// The path strings of a loaded game, kept while the core runs (`GET_GAME_INFO_EXT` points into
/// them after the bytes are dropped).
pub(crate) struct GameStrings {
    _path: CString,
    _dir: CString,
    _name: CString,
    _ext: CString,
}

/// The files of one game, all in its own folder `<saves root>/<console>/<hex>/` (its
/// [`GameKey`], never its file name):
/// - `game.sav`: the in-game save (cartridge battery);
/// - `auto.state`, `slotN.state` (+ `.png` thumbnail, + `.json` record): quick saves;
/// - `backups/<unix ms>.<reason>.sav`, `broken-<unix ms>.sav` ([`crate::saves`]).
///
/// The same folder is the core's save directory for the session (`GET_SAVE_DIRECTORY`), so what
/// a core writes by itself stays with the game too.
#[derive(Clone)]
pub struct GameFiles {
    pub sram: PathBuf,
    dir: PathBuf,
    root: PathBuf,
}

impl GameFiles {
    /// The files of `game` under the saves root `saves_root`.
    pub fn new(saves_root: &Path, game: &GameKey) -> Self {
        let dir = saves_root.join(game.dir());
        Self {
            sram: dir.join("game.sav"),
            dir,
            root: saves_root.to_path_buf(),
        }
    }

    /// The file of a quick save slot; slot 0 ([`crate::AUTO_SLOT`]) is the auto quick
    /// save («Salvataggio rapido automatico»), the others are the user's slots.
    pub fn slot(&self, slot: u8) -> PathBuf {
        if slot == crate::AUTO_SLOT {
            return self.dir.join("auto.state");
        }
        self.dir.join(format!("slot{slot}.state"))
    }

    /// The thumbnail of a slot: the frame on screen when it was saved.
    pub fn thumb(&self, slot: u8) -> PathBuf {
        self.slot(slot).with_extension("png")
    }

    /// What wrote a slot (core and app versions, [`crate::saves::SlotMeta`]).
    pub fn meta(&self, slot: u8) -> PathBuf {
        self.slot(slot).with_extension("json")
    }

    /// The game's own folder: its saves, slots, backups, and the core's save directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The saves root, shared by every game (`cores.json`, `screenshots/`).
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::ffi::CStr;
    use std::path::Path;

    use super::{GameContent, StartError};
    use crate::settings::test_dir::TestDir;

    /// A 3DS game is 0.5-4 GB: for a `need_fullpath` core the host passes the path only and
    /// never reads the file. The reader here stands for the 1 GB file: calling it fails the test.
    #[test]
    fn a_need_fullpath_core_gets_the_path_and_the_file_is_never_read() {
        let game = Path::new("D:/games/Big Game (1 GB).3ds");
        let read = Cell::new(0);
        let content = GameContent::read(game, true, |_| {
            read.set(read.get() + 1);
            panic!("a need_fullpath game was read into memory")
        })
        .unwrap();
        assert_eq!(read.get(), 0);
        let info = content.info();
        assert!(info.data.is_null());
        assert_eq!(info.size, 0);
        let path = unsafe { CStr::from_ptr(info.path) };
        assert_eq!(path.to_str().unwrap(), "D:/games/Big Game (1 GB).3ds");
        let ext = content.info_ext(true);
        assert!(ext.data.is_null() && ext.size == 0 && !ext.persistent_data);
        let s = |p| unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_owned();
        assert_eq!(s(ext.full_path), "D:/games/Big Game (1 GB).3ds");
        assert_eq!(s(ext.dir), "D:/games");
        assert_eq!(s(ext.name), "Big Game (1 GB)");
        assert_eq!(s(ext.ext), "3ds");
    }

    /// The same with a real file on disk (a small one: the size is not what is checked, the read
    /// is): a missing read is a `need_fullpath` game opening without touching its bytes.
    #[test]
    fn a_need_fullpath_game_opens_even_when_its_bytes_cannot_be_read() {
        let dir = TestDir::new("core-fullpath");
        let missing = dir.path("not-there.3ds");
        assert!(GameContent::read(&missing, true, |p| std::fs::read(p)).is_ok());
        assert!(matches!(
            GameContent::read(&missing, false, |p| std::fs::read(p)),
            Err(StartError::RomUnreadable { .. })
        ));
    }

    #[test]
    fn other_cores_get_the_bytes_too() {
        let dir = TestDir::new("core-bytes");
        dir.write("Game.GBA", "rom bytes");
        let content =
            GameContent::read(&dir.path("Game.GBA"), false, |p| std::fs::read(p)).unwrap();
        let info = content.info();
        assert_eq!(info.size, 9);
        let bytes = unsafe { std::slice::from_raw_parts(info.data as *const u8, info.size) };
        assert_eq!(bytes, b"rom bytes");
        let ext = content.info_ext(true);
        assert_eq!(ext.size, 9);
        assert_eq!(unsafe { CStr::from_ptr(ext.ext) }.to_str().unwrap(), "gba");
        // After `retro_load_game` the bytes are gone: `GET_GAME_INFO_EXT` says so.
        assert!(content.info_ext(false).data.is_null());
    }
}
