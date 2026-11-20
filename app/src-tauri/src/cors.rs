//! Who may read the `emu://` protocol's responses from script: the interface's own page only.
//!
//! The page is on another origin than `emu://` (`http://tauri.localhost` against
//! `http://emu.localhost` on Windows and Android, `tauri://localhost` against `emu://localhost`
//! elsewhere), so the canvas's `fetch` of the frames and the cover colours read through a canvas
//! (`img.crossOrigin`) need `Access-Control-Allow-Origin`. It names the request's `Origin` only
//! when that is the app's own page (or, in a debug build, the Vite dev server of `devUrl`); any
//! other page gets no CORS header and cannot read covers, thumbnails or frames.

use tauri::http::{header, HeaderValue, Request, Response};

/// The origin of the interface's page as Tauri serves it on this system.
fn app_origin() -> String {
    emudome_platform::protocol_url("tauri", "")
        .trim_end_matches('/')
        .to_owned()
}

/// Whether `origin` is one of `allowed` (exact match, as browsers send it: scheme, host, port).
fn origin_allowed(origin: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|a| a == origin)
}

/// Adds `Access-Control-Allow-Origin` to `response` when the request comes from the app's page.
pub fn allow_app(
    app: &tauri::AppHandle,
    request: &Request<Vec<u8>>,
    mut response: Response<Vec<u8>>,
) -> Response<Vec<u8>> {
    let Some(origin) = request.headers().get(header::ORIGIN) else {
        return response;
    };
    let mut allowed = vec![app_origin()];
    if cfg!(debug_assertions) {
        if let Some(dev) = &app.config().build.dev_url {
            allowed.push(dev.origin().ascii_serialization());
        }
    }
    let headers = response.headers_mut();
    // The answer depends on the Origin: a cache must not hand it to another one.
    headers.append(header::VARY, HeaderValue::from_static("Origin"));
    if origin
        .to_str()
        .is_ok_and(|origin| origin_allowed(origin, &allowed))
    {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
    } else {
        log::warn!("emu:// read from a foreign origin {origin:?}: refused");
    }
    response
}

#[cfg(test)]
mod tests {
    use super::{app_origin, origin_allowed};

    #[test]
    fn only_the_app_page_may_read_the_protocol() {
        let allowed = vec![app_origin(), "http://localhost:5173".to_owned()];
        assert!(origin_allowed(&app_origin(), &allowed));
        assert!(origin_allowed("http://localhost:5173", &allowed));
        for foreign in [
            "null",
            "https://evil.example",
            "http://localhost:5174",
            "http://tauri.localhost.evil.example",
            "",
        ] {
            assert!(!origin_allowed(foreign, &allowed), "{foreign}");
        }
        assert!(!app_origin().ends_with('/'));
    }
}
