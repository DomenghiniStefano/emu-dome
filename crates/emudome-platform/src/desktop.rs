//! What the desktop systems (Windows, Linux, macOS) share: logs on stderr, the game
//! from the command line, cores as files, no external surface (the native video
//! draws on Tauri's window).
use std::path::PathBuf;

use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, WindowHandle};

use crate::SurfaceListener;

pub fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

pub fn startup_game() -> Option<String> {
    std::env::args().skip(1).find(|a| !a.starts_with('-'))
}

pub fn packaged_core(_name: &str) -> Option<PathBuf> {
    None
}

pub const EXTERNAL_SURFACE: bool = false;
pub const PHONE: bool = false;

/// Never built on the desktop: the native video uses the window.
pub enum SurfaceWindow {}

impl HasWindowHandle for SurfaceWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        match *self {}
    }
}

pub fn surface_display_handle(window: &SurfaceWindow) -> Result<DisplayHandle<'_>, HandleError> {
    match *window {}
}

pub fn attach_external_surface<R: tauri::Runtime>(
    _webview: &tauri::Webview<R>,
    _on_event: SurfaceListener,
) {
}
