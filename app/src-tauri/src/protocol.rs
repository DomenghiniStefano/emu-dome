//! The `emu://` scheme: the images and frames the interface reads from Rust. One route per
//! resource; an unknown path answers 404 (it used to fall through to the frame).
//!
//! - `emu://frame?since=N`: the latest frame as raw RGBA, for the webview's canvas;
//! - `emu://cover/<id>`: covers (`library.rs`);
//! - `emu://background/<name>`: the user's background, `video.surroundImg` (`settings.rs`);
//! - `emu://slot/<n>`: slot thumbnails of the open game;
//! - `emu://savethumb/<game>/<n>`: those of any game, for the Saves section (`saves.rs`);
//! - `emu://screenshot/<game>/<name>`: a game's screenshots, for «Da uno screenshot» (`library.rs`);
//! - `emu://picked/<token>`: the image picked for a cover, for «Adatta la copertina» (`library.rs`).
//!
//! Only the interface's own page may read them from script (`cors.rs`).

use tauri::http::{header, Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

use crate::{cors, library, saves, settings, AppState};

/// A resource of the scheme, from the request's path.
#[derive(Debug, PartialEq, Eq)]
enum Route<'a> {
    Frame,
    Cover(&'a str),
    Background(&'a str),
    SaveThumb(&'a str),
    SlotThumb(&'a str),
    Screenshot(&'a str),
    Picked(&'a str),
    NotFound,
}

fn route(path: &str) -> Route<'_> {
    if let Some(id) = path.strip_prefix("/cover/") {
        Route::Cover(id)
    } else if let Some(name) = path.strip_prefix("/background/") {
        Route::Background(name)
    } else if let Some(rest) = path.strip_prefix("/savethumb/") {
        Route::SaveThumb(rest)
    } else if let Some(slot) = path.strip_prefix("/slot/") {
        Route::SlotThumb(slot)
    } else if let Some(rest) = path.strip_prefix("/screenshot/") {
        Route::Screenshot(rest)
    } else if let Some(token) = path.strip_prefix("/picked/") {
        Route::Picked(token)
    } else if path == "/frame" || path == "/frame/" {
        Route::Frame
    } else {
        Route::NotFound
    }
}

/// Answers one request of the scheme.
pub fn handle(app: &AppHandle, req: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let response = match route(req.uri().path()) {
        Route::Frame => frame_response(app, req),
        Route::Cover(id) => library::cover_response(app, id),
        Route::Background(name) => settings::background_response(app, name),
        Route::SaveThumb(rest) => saves::thumb_response(app, rest),
        Route::SlotThumb(slot) => slot_thumb_response(app, slot),
        Route::Screenshot(rest) => library::screenshot_response(app, rest),
        Route::Picked(token) => library::picked_response(app, token),
        Route::NotFound => {
            log::warn!("emu:// has no resource at {}", req.uri().path());
            not_found()
        }
    };
    cors::allow_app(app, req, response)
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Vec::new())
        .unwrap_or_else(response_error)
}

/// `emu://slot/<n>`: the thumbnail of a slot of the open game.
fn slot_thumb_response(app: &AppHandle, slot: &str) -> Response<Vec<u8>> {
    let bytes = slot.parse::<u8>().ok().and_then(|slot| {
        let state = app.try_state::<AppState>()?;
        let game = state.games.in_use()?;
        let info = emudome_core::slot_infos(&state.save_dir(), &game, slot)
            .into_iter()
            .find(|s| s.slot == slot)?;
        std::fs::read(info.thumb?).ok()
    });
    let builder = Response::builder();
    match bytes {
        // The URL carries the save time (`?v=`): a new save is a new URL.
        Some(b) => builder
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(b),
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    }
    .unwrap_or_else(response_error)
}

/// `emu://frame`: the latest frame as raw RGBA, for the webview's canvas. `?since=N` answers
/// 204 when there is no frame newer than N.
fn frame_response(app: &AppHandle, req: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let since: Option<u64> = req
        .uri()
        .query()
        .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("since=")))
        .and_then(|v| v.parse().ok());
    let builder = Response::builder()
        .header(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            "x-width, x-height, x-seq",
        )
        .header(header::CACHE_CONTROL, "no-store");
    let shared = app
        .try_state::<AppState>()
        .and_then(|state| state.games.current().load_full());
    let Some(shared) = shared else {
        return builder
            .status(StatusCode::NO_CONTENT)
            .body(Vec::new())
            .unwrap_or_else(response_error);
    };
    // A clone of the frame (its pixels' `Arc`) under the lock; the copy into the response after.
    let frame = shared.frame.lock().clone();
    if frame.rgba.is_empty() || since == Some(frame.seq) {
        return builder
            .status(StatusCode::NO_CONTENT)
            .body(Vec::new())
            .unwrap_or_else(response_error);
    }
    builder
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-width", frame.width)
        .header("x-height", frame.height)
        .header("x-seq", frame.seq)
        .body(frame.rgba.to_vec())
        .unwrap_or_else(response_error)
}

/// A response the builder refused (an invalid header value): logged, and answered with an
/// empty 500 so the reader skips it instead of the protocol handler panicking.
pub fn response_error(e: tauri::http::Error) -> Response<Vec<u8>> {
    log::error!("emu:// response: {e}");
    let mut res = Response::new(Vec::new());
    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    res
}

#[cfg(test)]
mod tests {
    use super::{route, Route};

    #[test]
    fn each_path_has_its_resource_and_an_unknown_one_none() {
        assert_eq!(route("/frame"), Route::Frame);
        assert_eq!(route("/cover/gba-0a1b"), Route::Cover("gba-0a1b"));
        assert_eq!(route("/background/sky.png"), Route::Background("sky.png"));
        assert_eq!(
            route("/savethumb/gba-0a1b/3"),
            Route::SaveThumb("gba-0a1b/3")
        );
        assert_eq!(route("/slot/2"), Route::SlotThumb("2"));
        assert_eq!(
            route("/screenshot/gba-0a1b/2026-11-22_10-11-12.png"),
            Route::Screenshot("gba-0a1b/2026-11-22_10-11-12.png")
        );
        assert_eq!(route("/picked/3"), Route::Picked("3"));
        assert_eq!(route("/"), Route::NotFound);
        assert_eq!(route("/framez"), Route::NotFound);
        assert_eq!(route("/covers/x"), Route::NotFound);
    }
}
