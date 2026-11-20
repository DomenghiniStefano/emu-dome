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
pub mod effects;
pub mod first_run;
pub mod hotkeys;
pub mod input;
pub mod library;
pub mod nds;
pub mod rewind;
pub mod saves;
pub mod screens;
pub mod screenshot;
pub mod settings;
pub mod speed_audio;
pub mod tuning;

pub use atomic::write_atomic;
pub use core::GameFiles;
pub use error::ErrorCode;
pub use frame::Frame;
pub use session::{
    core_needs_restart, has_auto_save, slot_infos, Closed, Command, CoreOptionInfo, Event,
    ExitWrites, Session, SessionOptions, Shared, SlotInfo, Speed, Stats, AUTO_SLOT, CLOSE_TIMEOUT,
};
