//! The window and the interface's webview. Tauri's API differs by form factor, so both live in
//! `emudome_platform::create_window`; this module keeps the names the app uses.

pub use emudome_platform::{create_window as create, WindowOptions as Options};
