//! Dynamic loading of a libretro core and access to its functions.

use std::ffi::{c_void, CStr, CString};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use libloading::Library;
use libretro_sys::symbols::{
    RetroApiVersion, RetroDeinit, RetroGetMemoryData, RetroGetMemorySize, RetroGetSystemAvInfo,
    RetroGetSystemInfo, RetroInit, RetroLoadGame, RetroReset, RetroRun, RetroSerialize,
    RetroSerializeSize, RetroSetAudioSample, RetroSetAudioSampleBatch,
    RetroSetControllerPortDevice, RetroSetEnvironment, RetroSetInputPoll, RetroSetInputState,
    RetroSetVideoRefresh, RetroUnloadGame, RetroUnserialize,
};
use libretro_sys::{
    retro_game_info, retro_system_av_info, retro_system_info, RETRO_API_VERSION,
    RETRO_DEVICE_JOYPAD, RETRO_MEMORY_SAVE_RAM,
};

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
    pub av_info: retro_system_av_info,
    pub library_name: String,
    pub library_version: String,
    unload_game: RetroUnloadGame,
    deinit: RetroDeinit,
    game_loaded: bool,
    // Kept alive while the core is in use; must be released last.
    _lib: Library,
}

macro_rules! sym {
    ($lib:expr, $ty:ty, $name:literal) => {{
        let s: libloading::Symbol<$ty> = $lib
            .get(concat!($name, "\0").as_bytes())
            .with_context(|| format!("the core does not export {}", $name))?;
        *s
    }};
}

impl LoadedCore {
    /// Loads the library, wires the callbacks and calls `retro_init`.
    ///
    /// # Safety
    /// Loads and runs arbitrary native code: the file must be a libretro core.
    pub unsafe fn load(path: &Path) -> Result<Self> {
        let lib = Library::new(path)
            .with_context(|| format!("cannot load the core {}", path.display()))?;

        let api_version: RetroApiVersion = sym!(lib, RetroApiVersion, "retro_api_version");
        if api_version() != RETRO_API_VERSION {
            bail!("unsupported libretro API version: {}", api_version());
        }

        let set_environment: RetroSetEnvironment =
            sym!(lib, RetroSetEnvironment, "retro_set_environment");
        let set_video: RetroSetVideoRefresh =
            sym!(lib, RetroSetVideoRefresh, "retro_set_video_refresh");
        let set_audio: RetroSetAudioSample =
            sym!(lib, RetroSetAudioSample, "retro_set_audio_sample");
        let set_audio_batch: RetroSetAudioSampleBatch = sym!(
            lib,
            RetroSetAudioSampleBatch,
            "retro_set_audio_sample_batch"
        );
        let set_input_poll: RetroSetInputPoll =
            sym!(lib, RetroSetInputPoll, "retro_set_input_poll");
        let set_input_state: RetroSetInputState =
            sym!(lib, RetroSetInputState, "retro_set_input_state");
        let init: RetroInit = sym!(lib, RetroInit, "retro_init");
        let get_system_info: RetroGetSystemInfo =
            sym!(lib, RetroGetSystemInfo, "retro_get_system_info");

        set_environment(crate::callbacks::environment);
        set_video(crate::callbacks::video_refresh);
        set_audio(crate::callbacks::audio_sample);
        set_audio_batch(crate::callbacks::audio_sample_batch);
        set_input_poll(crate::callbacks::input_poll);
        set_input_state(crate::callbacks::input_state);
        init();

        let mut info: retro_system_info = std::mem::zeroed();
        get_system_info(&mut info);
        let cstr = |p: *const std::os::raw::c_char| {
            if p.is_null() {
                String::new()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };

        Ok(Self {
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
            unload_game: sym!(lib, RetroUnloadGame, "retro_unload_game"),
            deinit: sym!(lib, RetroDeinit, "retro_deinit"),
            av_info: retro_system_av_info::default(),
            library_name: cstr(info.library_name),
            library_version: cstr(info.library_version),
            game_loaded: false,
            _lib: lib,
        })
    }

    /// Loads the game. For simplicity the file is always read into memory
    /// and its path passed too: this suits both `need_fullpath` cores and the others.
    pub unsafe fn load_game(&mut self, game: &Path) -> Result<()> {
        let data =
            std::fs::read(game).with_context(|| format!("cannot read {}", game.display()))?;
        let path = CString::new(game.to_string_lossy().as_bytes())?;
        let info = retro_game_info {
            path: path.as_ptr(),
            data: data.as_ptr() as *const c_void,
            size: data.len(),
            meta: std::ptr::null(),
        };
        let load_game: RetroLoadGame = sym!(self._lib, RetroLoadGame, "retro_load_game");
        if !load_game(&info) {
            bail!("the core refused the game {}", game.display());
        }
        self.game_loaded = true;
        let get_av: RetroGetSystemAvInfo =
            sym!(self._lib, RetroGetSystemAvInfo, "retro_get_system_av_info");
        get_av(&mut self.av_info);
        (self.set_controller_port_device)(0, RETRO_DEVICE_JOYPAD);
        Ok(())
    }

    /// Full emulation state (for the slots and, later, rewind).
    pub fn save_state(&self) -> Result<Vec<u8>> {
        unsafe {
            let size = (self.serialize_size)();
            if size == 0 {
                bail!("this core does not support save states");
            }
            let mut buf = vec![0u8; size];
            if !(self.serialize)(buf.as_mut_ptr() as *mut c_void, size) {
                bail!("the core failed to save the state");
            }
            Ok(buf)
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

/// A game's file paths: cartridge save and slots.
pub struct GameFiles {
    pub sram: PathBuf,
    save_dir: PathBuf,
    stem: String,
}

impl GameFiles {
    pub fn new(save_dir: &Path, game: &Path) -> Self {
        let stem = game
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "game".into());
        Self {
            sram: save_dir.join(format!("{stem}.sav")),
            save_dir: save_dir.to_path_buf(),
            stem,
        }
    }

    /// The file of a quick save slot; slot 0 ([`crate::AUTO_SLOT`]) is the auto quick
    /// save («Salvataggio rapido automatico»), the others are the user's slots.
    pub fn slot(&self, slot: u8) -> PathBuf {
        if slot == crate::AUTO_SLOT {
            return self.save_dir.join(format!("{}.auto.state", self.stem));
        }
        self.save_dir
            .join(format!("{}.slot{}.state", self.stem, slot))
    }

    /// The thumbnail of a slot: the frame on screen when it was saved.
    pub fn thumb(&self, slot: u8) -> PathBuf {
        self.slot(slot).with_extension("png")
    }
}
