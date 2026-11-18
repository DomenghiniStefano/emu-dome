//! The window and the interface's webview. Tauri's API differs by form factor
//! (`desktop` / `mobile` are Tauri's cfg aliases, not operating systems):
//!
//! - desktop: a bare window with the transparent "ui" child webview on top, so the native
//!   video can draw on the window underneath (needs Tauri's `unstable` feature);
//! - mobile: the single webview window the system gives; the native video gets a
//!   surface of its own (see `emudome_platform::attach_external_surface`).

use std::path::PathBuf;

use tauri::{App, Webview, WebviewUrl, Window};

pub struct Options {
    /// False in bench mode: the window appears without taking the focus.
    pub focused: bool,
    /// The webview's profile folder, when the data folder is not the default one
    /// (`EMUDOME_DATA_DIR`); `None` leaves Tauri's default.
    pub webview_data_dir: Option<PathBuf>,
}

#[cfg(desktop)]
pub fn create(app: &App, options: &Options) -> tauri::Result<(Window, Webview)> {
    use tauri::LogicalPosition;

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

#[cfg(mobile)]
pub fn create(app: &App, options: &Options) -> tauri::Result<(Window, Webview)> {
    let mut builder =
        tauri::WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .transparent(true)
            .focused(options.focused);
    if let Some(dir) = &options.webview_data_dir {
        builder = builder.data_directory(dir.clone());
    }
    let main = builder.build()?;
    let ui: &Webview = main.as_ref();
    Ok((ui.window(), ui.clone()))
}
