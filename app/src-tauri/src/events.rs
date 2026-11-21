//! The names of the events the shell sends to the interface, `<domain>://<event>`, in one place
//! (the listeners are in `app/src/emu.ts`, the only file that names them there).

/// The open game's session events (`emudome_core::Event`).
pub const EMU_EVENT: &str = "emu://event";
