//! EmuDome's heart: loads a libretro core, runs it on a dedicated thread
//! and exposes frames, audio, input and commands to the shell.
//!
//! The shell (Tauri) never calls the core directly: it sends [`Command`]
//! to the [`Session`] and reads the shared state in [`Shared`].

mod callbacks;
mod core;
mod error;
mod frame;
mod session;

pub mod audio;
pub mod hotkeys;
pub mod input;
pub mod library;
pub mod settings;

pub use error::ErrorCode;
pub use frame::Frame;
pub use session::{
    has_auto_save, slot_infos, Command, Event, Session, SessionOptions, Shared, SlotInfo, Speed,
    Stats, AUTO_SLOT,
};
