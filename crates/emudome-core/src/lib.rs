//! EmuDome's heart: loads a libretro core, runs it on a dedicated thread
//! and exposes frames, audio, input and commands to the shell.
//!
//! The shell (Tauri) never calls the core directly: it sends [`Command`]
//! to the [`Session`] and reads the shared state in [`Shared`].

mod atomic;
mod callbacks;
mod core;
mod core_options;
mod error;
mod frame;
mod session;

pub mod audio;
pub mod bindings;
pub mod bios;
pub mod close_flow;
pub mod console;
pub mod contract;
pub mod core_profile;
pub mod data_move;
pub mod effects;
pub mod first_run;
pub mod game_key;
pub mod hotkeys;
pub mod hw_render;
pub mod input;
pub mod keyboard;
pub mod library;
pub mod migration;
pub mod n3ds;
pub mod nds;
pub mod notifications;
pub mod rewind;
pub mod save_path;
pub mod saves;
pub mod screens;
pub mod screenshot;
pub mod settings;
pub mod speed_audio;
#[doc(hidden)]
pub mod testing;
pub mod video_sink;

pub use atomic::write_atomic;
pub use console::ConsoleId;
pub use core::{GameFiles, StartError};
pub use error::ErrorCode;
pub use frame::{rgba_png, Frame};
pub use game_key::GameKey;
pub use session::{
    core_needs_restart, has_auto_save, slot_infos, Closed, Command, Core, CoreInfo, CoreOptionInfo,
    Event, ExitWrites, Session, SessionOptions, Shared, SinkFactory, SlotInfo, Speed, Stats,
    VideoSetup, AUTO_SLOT, CLOSE_TIMEOUT,
};
