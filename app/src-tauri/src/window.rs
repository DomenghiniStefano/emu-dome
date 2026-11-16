//! The window and the interface's webview. Tauri's API differs by form factor
//! (`desktop` / `mobile` are Tauri's cfg aliases, not operating systems):
//!
//! - desktop: a bare window with the transparent "ui" child webview on top, so the native
//!   video can draw on the window underneath (needs Tauri's `unstable` feature);
//! - mobile: the single webview window the system gives; the native video gets a
//!   surface of its own (see `emudome_platform::attach_external_surface`).

use tauri::{App, Webview, WebviewUrl, Window};

#[cfg(desktop)]
pub fn create(app: &App) -> tauri::Result<(Window, Webview)> {
    use tauri::LogicalPosition;

    let window = tauri::window::WindowBuilder::new(app, "main")
        .title("EmuDome")
        .inner_size(1280.0, 800.0)
        .min_inner_size(640.0, 480.0)
        .build()?;
    let scale = window.scale_factor()?;
    let size = window.inner_size()?;
    let ui = window.add_child(
        tauri::webview::WebviewBuilder::new("ui", WebviewUrl::App("index.html".into()))
            .transparent(true)
            .auto_resize(),
        LogicalPosition::new(0.0, 0.0),
        size.to_logical::<f64>(scale),
    )?;
    Ok((window, ui))
}

#[cfg(mobile)]
pub fn create(app: &App) -> tauri::Result<(Window, Webview)> {
    let main = tauri::WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .transparent(true)
        .build()?;
    let ui: &Webview = main.as_ref();
    Ok((ui.window(), ui.clone()))
}
