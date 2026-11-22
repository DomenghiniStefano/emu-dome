//! The window and the interface's webview. Tauri's API differs by form factor, so both live in
//! `emudome_platform::create_window`; this module keeps the names the app uses, and carries out
//! the window's settings (desktop rows; on Android the platform calls do nothing):
//!
//! - `video.window`: windowed, fullscreen or borderless, at startup and when it changes;
//! - `video.winsize`: in a window with a game open, its inside is the game console's picture at
//!   1× ([`console_frame`]) times 1–4, the largest multiple that fits the monitor's work area
//!   ([`inner_size`], physical pixels: one console pixel is N screen pixels), never under the
//!   window's smallest size; applied when a game opens and when the setting changes while one
//!   is open ([`size_target`]). With no game the window keeps its size (the library's: the
//!   default at startup, or what the user gave it). «Free» keeps the size the user gave the
//!   window (nothing is stored: a new start opens at the default size). A maximized window is
//!   left as it is, except when the setting itself changes. F11 keeps toggling the system's
//!   fullscreen.

use emudome_core::screens::Split;
use emudome_core::settings::{keys, ConsoleId, Scope};
use emudome_platform::WindowMode;
use parking_lot::Mutex;
use tauri::{AppHandle, Manager};

pub use emudome_platform::{create_window as create, WindowOptions as Options};

/// `video.winsize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeChoice {
    /// The console's picture times this.
    Times(u32),
    /// The size the user gives the window.
    Free,
}

/// `video.window`'s option; an unknown id is a window.
pub fn mode_from_id(id: &str) -> WindowMode {
    match id {
        "fullscreen" => WindowMode::Fullscreen,
        "borderless" => WindowMode::Borderless,
        _ => WindowMode::Windowed,
    }
}

/// `video.winsize`'s option; an unknown id leaves the size alone.
pub fn size_from_id(id: &str) -> SizeChoice {
    match id {
        "1x" => SizeChoice::Times(1),
        "2x" => SizeChoice::Times(2),
        "3x" => SizeChoice::Times(3),
        "4x" => SizeChoice::Times(4),
        _ => SizeChoice::Free,
    }
}

/// The console's picture at 1×, as EmuDome composes it by default (the two screens stacked);
/// GBA's with no game.
pub fn console_frame(console: Option<ConsoleId>) -> (u32, u32) {
    match console {
        Some(ConsoleId::Gb | ConsoleId::Gbc) => (160, 144),
        Some(ConsoleId::Gba) | None => (240, 160),
        Some(ConsoleId::Nds) => Split::DsStacked { gap: 0 }
            .frame_at_1x()
            .unwrap_or((256, 384)),
        Some(ConsoleId::N3ds) => Split::N3dsStacked.frame_at_1x().unwrap_or((400, 480)),
    }
}

/// The window's inside for `frame` × `times`: the largest multiple up to `times` that fits in
/// `max` (a fraction of 1× when even that does not), then at least `min` on each side.
pub fn inner_size(frame: (u32, u32), times: u32, max: (u32, u32), min: (u32, u32)) -> (u32, u32) {
    let (w, h) = (frame.0.max(1), frame.1.max(1));
    let fits = |n: u32| w * n <= max.0 && h * n <= max.1;
    let (width, height) = match (1..=times.max(1)).rev().find(|n| fits(*n)) {
        Some(n) => (w * n, h * n),
        None => {
            let scale = (f64::from(max.0) / f64::from(w)).min(f64::from(max.1) / f64::from(h));
            (
                (f64::from(w) * scale).floor() as u32,
                (f64::from(h) * scale).floor() as u32,
            )
        }
    };
    (width.max(min.0), height.max(min.1))
}

/// Why the window's settings are applied now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// EmuDome started (the window was made windowed at the default size).
    Startup,
    /// `video.window` or `video.winsize` changed.
    Setting,
    /// A game of this console opened.
    Game(ConsoleId),
}

/// What happens to a window's size (in windowed mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeTarget {
    /// It keeps its size.
    Leave,
    /// This console's picture times N.
    Game(ConsoleId, u32),
    /// The default size (back from borderless, which covered the monitor).
    Default,
}

/// The size decision. `video.winsize` sizes the window for a GAME only (owner's PM, 2026-11-22:
/// the library keeps its window): when a game opens, and when the setting changes while one is
/// open. With no game open (`console` `None`: startup, a change in the library) the window keeps
/// its size; coming back from borderless it gets the default size. A maximized window is left as
/// it is unless the setting itself changed.
pub fn size_target(
    size: SizeChoice,
    console: Option<ConsoleId>,
    why: Why,
    maximized: bool,
    from_borderless: bool,
) -> SizeTarget {
    match (size, console) {
        (SizeChoice::Times(_), Some(_)) if maximized && why != Why::Setting => SizeTarget::Leave,
        (SizeChoice::Times(n), Some(console)) if why != Why::Startup => {
            SizeTarget::Game(console, n)
        }
        (SizeChoice::Times(_) | SizeChoice::Free, _) if from_borderless => SizeTarget::Default,
        (SizeChoice::Times(_) | SizeChoice::Free, _) => SizeTarget::Leave,
    }
}

/// The window mode last applied (`None` before startup).
#[derive(Default)]
pub struct WindowState(Mutex<Option<WindowMode>>);

/// The keys this module follows.
pub fn is_window_key(key: &str) -> bool {
    key == keys::VIDEO_WINDOW.as_str() || key == keys::VIDEO_WINDOW_SIZE.as_str()
}

/// A setting changed: the window follows when it is one of its own.
pub fn settings_changed(app: &AppHandle, keys: &[&str]) {
    if keys.iter().any(|k| is_window_key(k)) {
        apply(app, Why::Setting);
    }
}

/// The open game's console and the scope its settings are read from (`video.winsize` can be
/// set per console); «Tutti i giochi» without a game.
fn scope_now(app: &AppHandle) -> (Option<ConsoleId>, Scope) {
    let session = app
        .try_state::<crate::settings::SettingsState>()
        .and_then(|s| s.0.lock().session_scope());
    match session {
        Some(Scope::Game { console, .. }) => (Some(console), Scope::Session),
        Some(Scope::Global | Scope::Console { .. } | Scope::Session) | None => {
            (None, Scope::Global)
        }
    }
}

/// Applies `video.window` and `video.winsize` to the main window.
pub fn apply(app: &AppHandle, why: Why) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let (session_console, scope) = scope_now(app);
    let console = match why {
        Why::Game(console) => Some(console),
        Why::Startup | Why::Setting => session_console,
    };
    let settings = keys::Reader::new(crate::settings::resolver(app, &scope));
    let mode = mode_from_id(&settings.text(keys::VIDEO_WINDOW));
    let size = size_from_id(&settings.text(keys::VIDEO_WINDOW_SIZE));
    let previous = app
        .try_state::<WindowState>()
        .and_then(|s| s.0.lock().replace(mode));
    // The window starts windowed; a game opening never changes the mode.
    let change_mode = match why {
        Why::Startup => mode != WindowMode::Windowed,
        Why::Setting => previous != Some(mode),
        Why::Game(_) => false,
    };
    if change_mode {
        if let Err(e) = emudome_platform::set_window_mode(&window, mode) {
            log::warn!("window mode {mode:?}: {e}");
        }
    }
    if mode != WindowMode::Windowed {
        return;
    }
    let Some(room) = emudome_platform::window_room(&window) else {
        return;
    };
    // F11's fullscreen stays until F11 again.
    if room.fullscreen && !change_mode {
        return;
    }
    let from_borderless = change_mode && previous == Some(WindowMode::Borderless);
    let target = match size_target(size, console, why, room.maximized, from_borderless) {
        SizeTarget::Leave => None,
        SizeTarget::Game(console, n) => Some(inner_size(
            console_frame(Some(console)),
            n,
            room.max_inner,
            room.min_inner,
        )),
        SizeTarget::Default => {
            let (w, h) = emudome_platform::WINDOW_SIZE;
            let physical = |v: f64, max: u32| ((v * room.scale).round() as u32).min(max);
            Some((physical(w, room.max_inner.0), physical(h, room.max_inner.1)))
        }
    };
    if let Some(size) = target {
        log::info!("window: {size:?} ({why:?}, {mode:?}, {:?})", console);
        if let Err(e) = emudome_platform::set_window_inner_size(&window, size) {
            log::warn!("window size {size:?}: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        console_frame, inner_size, mode_from_id, size_from_id, size_target, SizeChoice, SizeTarget,
        Why,
    };
    use emudome_core::settings::ConsoleId;
    use emudome_platform::WindowMode;

    #[test]
    fn the_size_follows_a_game_only() {
        let three = SizeChoice::Times(3);
        let nds = Some(ConsoleId::Nds);
        // No game: startup and a change in the library leave the window as it is.
        assert_eq!(
            size_target(three, None, Why::Startup, false, false),
            SizeTarget::Leave
        );
        assert_eq!(
            size_target(three, None, Why::Setting, false, false),
            SizeTarget::Leave
        );
        // A game opens, or the setting changes while it is open.
        assert_eq!(
            size_target(three, nds, Why::Game(ConsoleId::Nds), false, false),
            SizeTarget::Game(ConsoleId::Nds, 3)
        );
        assert_eq!(
            size_target(three, nds, Why::Setting, false, false),
            SizeTarget::Game(ConsoleId::Nds, 3)
        );
        // Maximized: kept, unless the setting itself changed.
        assert_eq!(
            size_target(three, nds, Why::Game(ConsoleId::Nds), true, false),
            SizeTarget::Leave
        );
        assert_eq!(
            size_target(three, nds, Why::Setting, true, false),
            SizeTarget::Game(ConsoleId::Nds, 3)
        );
        // Free never sizes; back from borderless the window gets the default size.
        assert_eq!(
            size_target(SizeChoice::Free, nds, Why::Setting, false, false),
            SizeTarget::Leave
        );
        assert_eq!(
            size_target(SizeChoice::Free, None, Why::Setting, false, true),
            SizeTarget::Default
        );
        assert_eq!(
            size_target(three, None, Why::Setting, false, true),
            SizeTarget::Default
        );
    }

    #[test]
    fn the_options_map_to_modes_and_sizes() {
        assert_eq!(mode_from_id("windowed"), WindowMode::Windowed);
        assert_eq!(mode_from_id("fullscreen"), WindowMode::Fullscreen);
        assert_eq!(mode_from_id("borderless"), WindowMode::Borderless);
        assert_eq!(size_from_id("1x"), SizeChoice::Times(1));
        assert_eq!(size_from_id("4x"), SizeChoice::Times(4));
        assert_eq!(size_from_id("free"), SizeChoice::Free);
        assert_eq!(size_from_id("9x"), SizeChoice::Free, "unknown: left alone");
    }

    #[test]
    fn each_console_has_its_picture_at_1x() {
        assert_eq!(console_frame(None), (240, 160), "GBA with no game");
        assert_eq!(console_frame(Some(ConsoleId::Gb)), (160, 144));
        assert_eq!(console_frame(Some(ConsoleId::Gbc)), (160, 144));
        assert_eq!(console_frame(Some(ConsoleId::Nds)), (256, 384));
        assert_eq!(console_frame(Some(ConsoleId::N3ds)), (400, 480));
    }

    #[test]
    fn the_size_is_the_largest_multiple_that_fits() {
        let min = (640, 480);
        // A 1080p work area less the frame.
        let max = (1920, 1040);
        assert_eq!(inner_size((240, 160), 3, max, min), (720, 480));
        assert_eq!(inner_size((240, 160), 4, max, min), (960, 640));
        // 1x and 2x are smaller than the window may be.
        assert_eq!(inner_size((240, 160), 1, max, min), (640, 480));
        // A DS at 4x (1024 x 1536) does not fit: 2x does.
        assert_eq!(
            inner_size(console_frame(Some(ConsoleId::Nds)), 4, max, min),
            (640, 768)
        );
        // Not even 1x fits: the largest fraction, then the smallest window.
        assert_eq!(inner_size((400, 480), 1, (800, 300), (0, 0)), (250, 300));
    }
}
