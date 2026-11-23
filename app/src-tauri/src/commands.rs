//! The commands on the open game that only pass a value to it (speed, pause, slots, input,
//! stats...), and the window's video and fullscreen commands. Opening and closing a game are in
//! `game.rs`.

use std::sync::atomic::Ordering;

use emudome_core::input::{Button, Source};
use emudome_core::screens::{Area, Inset};
use emudome_core::{Command, ErrorCode, Speed};
use serde::Serialize;
use tauri::Manager;
use ts_rs::TS;

use crate::{AppState, VideoMode};

/// One quick save slot for the quick menu (G07). TypeScript: `SlotInfo` (generated).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "SlotInfo")]
pub(crate) struct SlotDto {
    slot: u8,
    /// Unix ms of the last save; null = empty.
    saved_at: Option<u64>,
    /// `emu://slot/<n>?v=<ms>`, the frame on screen when it was saved.
    thumb_url: Option<String>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "app/", rename = "Stats")]
pub(crate) struct StatsDto {
    running: bool,
    emu_fps: f64,
    target_fps: f64,
    speed: Option<Speed>,
    paused: bool,
    /// Frames emulated since the game started (the interface's hang watchdog, E11).
    frames: u64,
    /// Unix ms of the last auto quick save in this session.
    auto_saved_at: Option<u64>,
    /// The game moved since the user's last quick save or load (`Stats::unsaved`): the quick
    /// menu's «Chiudi il gioco» asks first.
    unsaved: bool,
}

pub(crate) fn unix_ms(t: std::time::SystemTime) -> Option<u64> {
    t.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
}

/// The auto quick save (slot 0) and slots `1..=count` of the open game.
#[tauri::command]
pub(crate) fn list_slots(state: tauri::State<'_, AppState>, count: u8) -> Vec<SlotDto> {
    let Some(game) = state.games.in_use() else {
        return Vec::new();
    };
    emudome_core::slot_infos(&state.save_dir(), &game, count)
        .into_iter()
        .map(|s| {
            let saved_at = s.saved_at.and_then(unix_ms);
            SlotDto {
                slot: s.slot,
                saved_at,
                thumb_url: s.thumb.map(|_| {
                    emudome_platform::protocol_url(
                        "emu",
                        &format!("slot/{}?v={}", s.slot, saved_at.unwrap_or(0)),
                    )
                }),
            }
        })
        .collect()
}

#[tauri::command]
pub(crate) fn video_mode(state: tauri::State<'_, AppState>) -> (VideoMode, bool) {
    (*state.video_mode.lock(), state.renderer.is_some())
}

/// Switches mode live (only where the native renderer exists), to compare the two.
#[tauri::command]
pub(crate) fn set_video_mode(
    webview: tauri::Webview,
    state: tauri::State<'_, AppState>,
    mode: VideoMode,
) -> Result<(), ErrorCode> {
    if mode == VideoMode::Native && state.renderer.is_none() {
        return Err(ErrorCode::NativeVideoUnavailable);
    }
    webview
        .set_background_color(Some(mode.webview_background()))
        .map_err(|e| {
            log::warn!("webview background not changed: {e}");
            ErrorCode::WebviewBackgroundFailed
        })?;
    *state.video_mode.lock() = mode;
    let current = state.games.current().load_full();
    state.native_source(current);
    Ok(())
}

/// The edges of the window kept clear of the native game (shares of its width and height): the
/// quick menu's bar, sheet or side panel, the touch controls' band or columns; all zero = the
/// whole window again. The webview canvas does the same by itself.
#[tauri::command]
pub(crate) fn set_video_inset(state: tauri::State<'_, AppState>, inset: Inset) {
    if let Some(r) = &state.renderer {
        r.set_inset(inset);
    }
}

/// F11 and the quick menu's «A schermo intero»: fullscreen on / off. Answers whether the
/// window is now fullscreen. No controller shortcut (docs/combo-tasti.md).
#[tauri::command]
pub(crate) fn toggle_fullscreen(app: tauri::AppHandle) -> bool {
    let Some(window) = app.get_window("main") else {
        return false;
    };
    let on = window.is_fullscreen().unwrap_or(false);
    if let Err(e) = emudome_platform::set_fullscreen(&window, !on) {
        log::warn!("fullscreen {}: {e}", !on);
        return on;
    }
    !on
}

/// Whether the window is fullscreen (the quick menu's tile shows it pressed).
#[tauri::command]
pub(crate) fn is_fullscreen(app: tauri::AppHandle) -> bool {
    app.get_window("main")
        .and_then(|w| w.is_fullscreen().ok())
        .unwrap_or(false)
}

/// The interface tells when the quick menu opens and closes, so the controllers
/// navigate it instead of reaching the game. Opening it lets go of the keyboard's buttons: a
/// key released while the menu has the keys never reaches the game (Trello #74).
#[tauri::command]
pub(crate) fn set_menu_open(state: tauri::State<'_, AppState>, open: bool) {
    state.menu_open.store(open, Ordering::Relaxed);
    if open {
        state.release_keyboard();
    }
}

/// The page lost the focus (Alt-Tab, another window, minimized) or got it back: losing it lets
/// go of the keys held now (they are released elsewhere), and both carry out `general.bgpause`
/// (`background_pause.rs`).
#[tauri::command]
pub(crate) fn window_focus(app: tauri::AppHandle, focused: bool) {
    crate::background_pause::focus_changed(&app, focused);
}

#[tauri::command]
pub(crate) fn set_speed(state: tauri::State<'_, AppState>, speed: Speed) -> Result<(), ErrorCode> {
    state.games.send(Command::SetSpeed(speed))
}

#[tauri::command]
pub(crate) fn set_paused(state: tauri::State<'_, AppState>, paused: bool) -> Result<(), ErrorCode> {
    // The interface decides now: a pause `general.bgpause` made is the user's from here.
    state.background.lock().forget();
    state.games.send(Command::SetPaused(paused))
}

#[tauri::command]
pub(crate) fn save_state(state: tauri::State<'_, AppState>, slot: u8) -> Result<(), ErrorCode> {
    state.games.send(Command::SaveState(slot))
}

#[tauri::command]
pub(crate) fn load_state(state: tauri::State<'_, AppState>, slot: u8) -> Result<(), ErrorCode> {
    state.games.send(Command::LoadState(slot))
}

/// «Annulla il caricamento» (`saves.undo`): back to before the last load.
#[tauri::command]
pub(crate) fn undo_load(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    state.games.send(Command::UndoLoad)
}

/// «Salvato nello slot N · Annulla»: the slot of the last save gets back what it held.
#[tauri::command]
pub(crate) fn undo_save(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    state.games.send(Command::UndoSave)
}

/// «Muto» (quick menu): silence for the open game only; the next game starts with sound.
#[tauri::command]
pub(crate) fn set_muted(state: tauri::State<'_, AppState>, muted: bool) -> Result<(), ErrorCode> {
    let shared = state.games.current().load_full().ok_or(ErrorCode::NoGame)?;
    shared.muted.store(muted, Ordering::Relaxed);
    Ok(())
}

/// Backspace (held, or toggled with `a11y.toggle`): rewind on / off.
#[tauri::command]
pub(crate) fn rewind(state: tauri::State<'_, AppState>, on: bool) -> Result<(), ErrorCode> {
    state.games.send(Command::Rewind(on))
}

/// F12, L3 + B, quick menu › Strumenti: a PNG of the game next to the saves
/// (`screenshotSaved` event, or `screenshotFailed`).
#[tauri::command]
pub(crate) fn screenshot(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    state.games.send(Command::Screenshot)
}

#[tauri::command]
pub(crate) fn reset_game(state: tauri::State<'_, AppState>) -> Result<(), ErrorCode> {
    state.games.send(Command::Reset)
}

/// A key of the keyboard (`KeyboardEvent.code`), forwarded raw by the webview, which has the
/// focus: Rust routes it (`emudome_core::keyboard`) to the game's button or a shortcut, carried
/// out as the controllers' are (`hotkeys.rs`). The menu key, fullscreen and the menus' focus
/// keys stay in the interface.
#[tauri::command]
pub(crate) fn key(state: tauri::State<'_, AppState>, code: String, pressed: bool, repeat: bool) {
    let current = state.games.current().load_full();
    let game_has_keys = current.is_some() && !state.menu_open.load(Ordering::Relaxed);
    let button = state.hotkeys.key(&code, pressed, repeat, game_has_keys);
    if let (Some((button, down)), Some(shared)) = (button, current) {
        shared.input.set(Source::Keyboard, button, down);
    }
}

/// A game button from the touch controls (Android's overlay): their own source, so letting go
/// of the keyboard (the quick menu opening, the window losing the focus) never drops a button a
/// finger still holds, nor the other way round.
#[tauri::command]
pub(crate) fn input(state: tauri::State<'_, AppState>, button: Button, pressed: bool) {
    if let Some(shared) = state.games.current().load().as_ref() {
        shared.input.set(Source::Touch, button, pressed);
    }
}

/// Where a pointer position is measured (`pointer`): the window in the renderer's unit (physical
/// pixels for the native video, CSS pixels for the webview canvas) and the edges kept clear of
/// the game (the quick menu, the touch controls). TypeScript: `PointerArea` (generated).
#[derive(serde::Deserialize, TS)]
#[ts(export_to = "app/")]
pub(crate) struct PointerArea {
    width: f32,
    height: f32,
    inset: Inset,
}

impl PointerArea {
    /// The area as the screens are composed in it.
    pub(crate) fn area(&self) -> Area {
        Area {
            width: self.width,
            height: self.height,
            inset: self.inset,
        }
    }
}

/// Touch screen (NDS, 3DS): the mouse or a finger at (`x`, `y`) in `area`, pressed or not. Rust
/// maps it through the bottom screen as the open game's layout composes it (`screens.*`, swapped
/// or not: `crate::screens::layout`, `screens::pointer_at`); off the touch screen it is a
/// release, as on the console. One-screen consoles ignore it.
#[tauri::command]
pub(crate) fn pointer(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    x: f32,
    y: f32,
    pressed: bool,
    area: PointerArea,
) {
    let Some(shared) = state.games.current().load_full() else {
        return;
    };
    let at = if pressed {
        let (frame, split) = {
            let f = shared.frame.lock();
            ((f.width, f.height), f.split)
        };
        let layout = crate::screens::layout(&app);
        emudome_core::screens::pointer_at(&layout, split, frame, area.area(), x, y)
    } else {
        None
    };
    shared.input.set_pointer(at);
}

#[tauri::command]
pub(crate) fn stats(state: tauri::State<'_, AppState>) -> StatsDto {
    match state.games.current().load().as_ref() {
        Some(shared) => {
            let s = shared.stats.lock();
            StatsDto {
                running: true,
                emu_fps: s.emu_fps,
                target_fps: s.av_info.timing.fps,
                speed: s.speed,
                paused: s.paused,
                frames: s.frames,
                auto_saved_at: s.auto_saved_at.and_then(unix_ms),
                unsaved: s.unsaved,
            }
        }
        None => StatsDto {
            running: false,
            emu_fps: 0.0,
            target_fps: 0.0,
            speed: None,
            paused: false,
            frames: 0,
            auto_saved_at: None,
            unsaved: false,
        },
    }
}
