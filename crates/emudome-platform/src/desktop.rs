//! What the desktop systems (Windows, Linux, macOS) share: logs on stderr, the game
//! from the command line, cores as files, no external surface (the native video
//! draws on Tauri's window), the window with its child webview, the native file picker.
use std::path::{Path, PathBuf};

use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, WindowHandle};

use crate::{
    BackListener, ContentError, ContentFile, DataDirs, SurfaceListener, WindowOptions,
    OLD_IDENTIFIER,
};

/// The system's data folders are `<base>/<identifier>` (`%APPDATA%` and `%LOCALAPPDATA%` on
/// Windows, `$XDG_DATA_HOME` for both on Linux, `~/Library/Application Support` on macOS): the
/// old identifier's are beside the current ones.
pub fn old_data_dirs(data: &Path, local: &Path) -> Option<DataDirs> {
    let beside = |dir: &Path| {
        let old = dir.parent()?.join(OLD_IDENTIFIER);
        (old != dir).then_some(old)
    };
    Some(DataDirs {
        data: beside(data)?,
        local: beside(local)?,
    })
}

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
        // The design system's `bg` (#0e0f13): what shows before the page's splash paints and
        // before the native renderer's first frame (Trello #101). The child webview keeps its
        // own background (transparent over the native video).
        .background_color(tauri::window::Color(0x0e, 0x0f, 0x13, 0xff))
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

/// The desktop opens games by path only.
pub fn open_content(_uri: &str, _cache_dir: &Path) -> Result<ContentFile, ContentError> {
    Err(ContentError::Unsupported)
}

/// No back button on the desktop, and the startup game is the command line's: nothing to attach.
pub fn attach_system<R: tauri::Runtime>(_webview: &tauri::Webview<R>, _on_back: BackListener) {}

pub fn set_fullscreen(window: &tauri::Window, on: bool) -> tauri::Result<()> {
    window.set_fullscreen(on)
}

#[cfg(test)]
mod tests {
    use super::old_data_dirs;
    use crate::DataDirs;
    use std::path::{Path, PathBuf};

    #[test]
    fn the_old_folders_are_beside_the_new_ones() {
        let base = PathBuf::from("base");
        assert_eq!(
            old_data_dirs(
                &base.join("Roaming").join("it.domehub.emudome"),
                &base.join("Local").join("it.domehub.emudome"),
            ),
            Some(DataDirs {
                data: base.join("Roaming").join("com.domenghini.emudome"),
                local: base.join("Local").join("com.domenghini.emudome"),
            })
        );
        // No parent, or already the old name: nothing to copy from.
        assert_eq!(old_data_dirs(Path::new(""), Path::new("")), None);
        let old = base.join("com.domenghini.emudome");
        assert_eq!(old_data_dirs(&old, &old), None);
    }
}
