//! What the desktop systems (Windows, Linux, macOS) share: logs on stderr, the game
//! from the command line, cores as files, no external surface (the native video
//! draws on Tauri's window), the window with its child webview, the native file picker.
use std::path::{Path, PathBuf};

use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, WindowHandle};

use crate::{
    BackListener, ContentError, ContentFile, DataDirs, SurfaceListener, WindowMode, WindowOptions,
    WindowRoom, OLD_IDENTIFIER, WINDOW_MIN_SIZE, WINDOW_SIZE,
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
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let level = logger.filter();
    // Also into the log file (`log_file.rs`); stderr stays as it was.
    if log::set_boxed_logger(Box::new(crate::log_file::Tee::new(Box::new(logger)))).is_ok() {
        log::set_max_level(level);
    }
}

pub fn startup_game() -> Option<String> {
    std::env::args().skip(1).find(|a| !a.starts_with('-'))
}

/// The desktop has no background to leave to: the window stays as it is.
pub fn move_to_background() -> bool {
    false
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
        .inner_size(WINDOW_SIZE.0, WINDOW_SIZE.1)
        .min_inner_size(WINDOW_MIN_SIZE.0, WINDOW_MIN_SIZE.1)
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

pub fn save_file(
    app: &tauri::AppHandle,
    name: &str,
    filter: &str,
    extensions: &[&str],
) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_file_name(name)
        .add_filter(filter, extensions)
        .blocking_save_file()?;
    picked
        .into_path()
        .inspect_err(|e| log::warn!("save target: {e}"))
        .ok()
}

/// The desktop opens games by path only.
pub fn open_content(_uri: &str, _cache_dir: &Path) -> Result<ContentFile, ContentError> {
    Err(ContentError::Unsupported)
}

/// No back button on the desktop, and the startup game is the command line's: nothing to attach.
pub fn attach_system<R: tauri::Runtime>(_webview: &tauri::Webview<R>, _on_back: BackListener) {}

/// No vibrator on the desktop.
pub fn haptic(_strength: u8) {}

pub fn set_fullscreen(window: &tauri::Window, on: bool) -> tauri::Result<()> {
    window.set_fullscreen(on)
}

/// The window's frame (outer size less inner size), physical pixels.
fn frame_size(window: &tauri::Window) -> tauri::Result<(u32, u32)> {
    let (inner, outer) = (window.inner_size()?, window.outer_size()?);
    Ok((
        outer.width.saturating_sub(inner.width),
        outer.height.saturating_sub(inner.height),
    ))
}

pub fn window_room(window: &tauri::Window) -> Option<WindowRoom> {
    let monitor = window.current_monitor().ok().flatten()?;
    let work = monitor.work_area().size;
    let frame = frame_size(window).ok()?;
    let scale = window.scale_factor().ok()?;
    let physical = |v: f64| (v * scale).round() as u32;
    Some(WindowRoom {
        max_inner: (
            work.width.saturating_sub(frame.0),
            work.height.saturating_sub(frame.1),
        ),
        min_inner: (physical(WINDOW_MIN_SIZE.0), physical(WINDOW_MIN_SIZE.1)),
        scale,
        maximized: window.is_maximized().unwrap_or(false),
        fullscreen: window.is_fullscreen().unwrap_or(false),
    })
}

pub fn set_window_mode(window: &tauri::Window, mode: WindowMode) -> tauri::Result<()> {
    match mode {
        WindowMode::Fullscreen => {
            window.set_decorations(true)?;
            window.set_fullscreen(true)
        }
        WindowMode::Windowed => {
            window.set_fullscreen(false)?;
            window.set_decorations(true)
        }
        WindowMode::Borderless => {
            window.set_fullscreen(false)?;
            window.set_decorations(false)?;
            if window.is_maximized()? {
                window.unmaximize()?;
            }
            if let Some(monitor) = window.current_monitor()? {
                window.set_position(*monitor.position())?;
                window.set_size(*monitor.size())?;
            }
            Ok(())
        }
    }
}

pub fn set_window_inner_size(window: &tauri::Window, size: (u32, u32)) -> tauri::Result<()> {
    use tauri::{PhysicalPosition, PhysicalSize};

    if window.is_maximized()? {
        window.unmaximize()?;
    }
    let frame = frame_size(window)?;
    window.set_size(PhysicalSize::new(size.0, size.1))?;
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };
    let area = monitor.work_area();
    let (left, top) = (area.position.x, area.position.y);
    let (room_w, room_h) = (i64::from(area.size.width), i64::from(area.size.height));
    let (outer_w, outer_h) = (
        i64::from(size.0.saturating_add(frame.0)),
        i64::from(size.1.saturating_add(frame.1)),
    );
    let at = window.outer_position()?;
    let (x, y) = (
        i64::from(at.x) - i64::from(left),
        i64::from(at.y) - i64::from(top),
    );
    if x >= 0 && y >= 0 && x + outer_w <= room_w && y + outer_h <= room_h {
        return Ok(());
    }
    // Sticking out: centred in the work area (its top left corner when larger than it).
    let centre = |room: i64, outer: i64| i32::try_from(((room - outer) / 2).max(0)).unwrap_or(0);
    window.set_position(PhysicalPosition::new(
        left + centre(room_w, outer_w),
        top + centre(room_h, outer_h),
    ))
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
