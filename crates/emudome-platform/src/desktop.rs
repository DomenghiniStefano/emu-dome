//! What the desktop systems (Windows, Linux, macOS) share: logs on stderr, the game
//! from the command line, cores as files, no external surface (the native video
//! draws on Tauri's window), the window with its child webview, the native file picker.
use std::path::PathBuf;

use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, WindowHandle};

use crate::{SurfaceListener, WindowOptions};

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

pub fn create_window(
    app: &tauri::App,
    options: &WindowOptions,
) -> tauri::Result<(tauri::Window, tauri::Webview)> {
    use tauri::{LogicalPosition, WebviewUrl};

    let window = tauri::window::WindowBuilder::new(app, "main")
        .title("EmuDome")
        .inner_size(1280.0, 800.0)
        .min_inner_size(640.0, 480.0)
        .focused(options.focused)
        .build()?;
    let scale = window.scale_factor()?;
    let size = window.inner_size()?;
    let mut ui = tauri::webview::WebviewBuilder::new("ui", WebviewUrl::App("index.html".into()))
        .transparent(true)
        .focused(options.focused)
        .auto_resize();
    if let Some(dir) = &options.webview_data_dir {
        ui = ui.data_directory(dir.clone());
    }
    let ui = window.add_child(
        ui,
        LogicalPosition::new(0.0, 0.0),
        size.to_logical::<f64>(scale),
    )?;
    Ok((window, ui))
}

pub fn pick_file(app: &tauri::AppHandle, filter: &str, extensions: &[&str]) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter(filter, extensions)
        .blocking_pick_file()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("picked file: {e}"))
        .ok()
}
