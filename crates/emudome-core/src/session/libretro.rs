//! The libretro core as the session's [`Core`]: a [`LoadedCore`] and the callback slot it calls
//! back into, loaded on the emulation thread by [`boot`].

use std::ffi::CString;
use std::sync::Arc;

use libretro_sys::retro_system_av_info;

use super::emu_loop::{Core, CoreInfo};
use super::{SessionOptions, Shared, VideoSetup};
use crate::callbacks::{CallbackCtx, Slot};
use crate::core::{GameContent, GameFiles, LoadedCore, StartError};
use crate::core_options::CoreOptions;
use crate::speed_audio::{FastAudio, SpeedAudio};

/// A loaded libretro core with its game, and its callbacks' slot.
pub(super) struct Libretro {
    core: LoadedCore,
    slot: Slot,
}

/// Claims a callback slot, loads the core and the game (on the emulation thread). On failure the
/// core, if it loaded, is unloaded and the slot freed.
pub(super) fn boot(
    opts: &mut SessionOptions,
    shared: &Arc<Shared>,
    files: &GameFiles,
) -> Result<Libretro, StartError> {
    let to_c =
        |p: &std::path::Path| CString::new(p.to_string_lossy().as_bytes()).unwrap_or_default();
    let mut ctx = CallbackCtx {
        audio_enabled: true,
        speed_audio: SpeedAudio::new(opts.fast_audio),
        options: CoreOptions::new(opts.core_options.iter().cloned()),
        split: opts.split,
        username: opts
            .username
            .as_deref()
            .filter(|n| !n.is_empty())
            .and_then(|n| CString::new(n).ok()),
        // The game's own folder: what a core writes by itself stays with the game.
        ..CallbackCtx::new(shared.clone(), to_c(&opts.system_dir), to_c(files.dir()))
    };
    let VideoSetup { sink, hw_render } = std::mem::take(&mut opts.video);
    if let Some(make) = sink {
        ctx.video = make(shared);
    }
    if let Some(hw) = hw_render {
        ctx.hw = hw;
    }
    let slot = Slot::claim(ctx).map_err(|e| StartError::ThreadFailed {
        detail: format!("{e:#}"),
    })?;

    let loaded = (|| -> Result<LoadedCore, StartError> {
        let mut core = unsafe { LoadedCore::load(&opts.core_path, slot.callbacks())? };
        // A `need_fullpath` core (Azahar) reads the file itself: never read into memory here.
        let content = GameContent::read(&opts.game_path, core.need_fullpath, |p| std::fs::read(p))?;
        slot.begin_game(content.info_ext(true));
        let game = unsafe { core.load_game(&content) };
        slot.end_game_load(content.into_strings());
        game?;
        Ok(core)
    })();
    match loaded {
        Ok(mut core) => {
            // A hardware context the core asked for is ready now: the core creates its GPU
            // objects (Azahar loads the game there). Outside the slot's lock: it calls back.
            if let Some(reset) = slot.hw_context_reset() {
                let started = std::time::Instant::now();
                unsafe { reset() };
                log::info!(
                    "hw render: context_reset took {} ms",
                    started.elapsed().as_millis()
                );
            }
            // A timing set while loading is already in `av_info` (read after `retro_load_game`);
            // one set in `context_reset` is taken now.
            if let Some(av) = slot.take_av_change() {
                core.av_info = av;
            }
            Ok(Libretro { core, slot })
        }
        Err(e) => {
            // The core, if it loaded, was unloaded with the closure: the slot is free.
            slot.release();
            Err(e)
        }
    }
}

impl Core for Libretro {
    fn info(&self) -> CoreInfo {
        CoreInfo {
            name: self.core.library_name.clone(),
            version: self.core.library_version.clone(),
            av: self.core.av_info,
        }
    }

    fn run(&mut self) {
        unsafe { (self.core.run)() }
    }

    fn reset(&mut self) {
        unsafe { (self.core.reset)() }
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        self.core.save_state()
    }

    fn serialize_into(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        self.core.save_state_into(buf)
    }

    fn unserialize(&mut self, state: &[u8]) -> anyhow::Result<()> {
        self.core.load_state(state)
    }

    fn sram(&self) -> Option<&[u8]> {
        self.core.save_ram()
    }

    fn sram_mut(&mut self) -> Option<&mut [u8]> {
        self.core.save_ram_mut()
    }

    fn refresh_input(&mut self) {
        self.slot.refresh_input();
    }

    fn take_av_change(&mut self) -> Option<retro_system_av_info> {
        let av = self.slot.take_av_change()?;
        self.core.av_info = av;
        Some(av)
    }

    fn set_audio_speed(&mut self, speed: Option<f64>) {
        self.slot.set_audio_speed(speed);
    }

    fn set_audio_enabled(&mut self, enabled: bool) {
        self.slot.set_audio_enabled(enabled);
    }

    fn set_fast_audio(&mut self, policy: FastAudio) {
        self.slot.set_fast_audio(policy);
    }

    fn set_core_option(&mut self, key: String, value: String) {
        self.slot.set_core_option(key, value);
    }

    fn end(self, let_go: bool) {
        let Libretro { core, slot } = self;
        if let_go {
            // Left loaded: its thread may still be inside it; the slot keeps it, empty, forever.
            std::mem::forget(core);
            slot.retire();
        } else {
            // The sink lets go of the core's images, the core destroys its GPU context (the
            // device, the shell's, lives on), then `retro_unload_game` / `retro_deinit`, then
            // the slot serves another core.
            if let Some(destroy) = slot.hw_shut_down() {
                unsafe { destroy() };
            }
            drop(core);
            slot.release();
        }
    }
}
