//! Screen effects (`fx.*`) on the native video path.
//!
//! The built-in presets (`app/src-tauri/effects/`, see THIRD_PARTY.md) are part of the binary
//! and written once to `<app local data>/effects/<version>/`, where librashader reads them like
//! any `.slangp` (Android included, where the app has no files of its own on disk). The values
//! resolved for the open game (its session «in prova» included) go to the renderer through
//! [`apply`], called whenever an `fx.*` setting changes and when a game opens or closes.

use std::path::{Path, PathBuf};

use emudome_core::effects::{self, EffectSource};
use emudome_core::settings::{keys, Scope, Value};
use emudome_core::ErrorCode;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::SettingsState;
use crate::video::EffectSpec;
use crate::AppState;

/// Every file of the built-in effects, by its path under the effects folder.
const BUILTIN_FILES: [(&str, &[u8]); 11] = [
    ("lcd.slangp", include_bytes!("../effects/lcd.slangp")),
    (
        "lcd_grid.slangp",
        include_bytes!("../effects/lcd_grid.slangp"),
    ),
    ("crt.slangp", include_bytes!("../effects/crt.slangp")),
    (
        "crt_curved.slangp",
        include_bytes!("../effects/crt_curved.slangp"),
    ),
    (
        "soft_pixels.slangp",
        include_bytes!("../effects/soft_pixels.slangp"),
    ),
    (
        "shaders/zfast_lcd.slang",
        include_bytes!("../effects/shaders/zfast_lcd.slang"),
    ),
    (
        "shaders/lcd3x.slang",
        include_bytes!("../effects/shaders/lcd3x.slang"),
    ),
    (
        "shaders/xbr-lv2-standalone.slang",
        include_bytes!("../effects/shaders/xbr-lv2-standalone.slang"),
    ),
    (
        "shaders/zfast_crt/zfast_crt_finemask.slang",
        include_bytes!("../effects/shaders/zfast_crt/zfast_crt_finemask.slang"),
    ),
    (
        "shaders/zfast_crt/zfast_crt_impl.inc",
        include_bytes!("../effects/shaders/zfast_crt/zfast_crt_impl.inc"),
    ),
    (
        "shaders/zfast_crt/zfast_crt_curvature.slang",
        include_bytes!("../effects/shaders/zfast_crt/zfast_crt_curvature.slang"),
    ),
];

/// Where the built-in presets are on disk (`None`: they could not be written, so only the
/// user's own files work).
pub struct EffectsState {
    builtin_dir: Option<PathBuf>,
}

/// Writes the built-in presets under `base` (a file already equal is left alone).
pub fn write_builtins(base: &Path) -> std::io::Result<PathBuf> {
    let dir = base.join(env!("CARGO_PKG_VERSION"));
    for (rel, bytes) in BUILTIN_FILES {
        let path = dir.join(rel);
        if std::fs::read(&path).is_ok_and(|current| current == bytes) {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes)?;
    }
    Ok(dir)
}

/// Writes the built-in presets and tells the renderer who to warn when an effect fails.
pub fn open(app: &tauri::App) -> EffectsState {
    // `dirs::local`, so `EMUDOME_DATA_DIR` moves these too (a test run never writes the real
    // app data).
    let builtin_dir = crate::dirs::local(app)
        .map_err(|e| e.to_string())
        .and_then(|d| write_builtins(&d.join("effects")).map_err(|e| e.to_string()))
        .inspect_err(|e| log::error!("built-in screen effects not written: {e}"))
        .ok();
    if let Some(renderer) = app.try_state::<AppState>().and_then(|s| s.renderer.clone()) {
        let handle = app.handle().clone();
        renderer.on_effect_error(Box::new(move |preset, detail| {
            let error = ErrorCode::EffectNotLoaded {
                name: effect_name(preset),
                detail,
            };
            if let Err(e) = handle.emit(crate::events::EFFECT_ERROR, error) {
                log::warn!("emu://effect-error: {e}");
            }
        }));
    }
    EffectsState { builtin_dir }
}

/// The name the interface shows for a preset: the built-in effect's option id (it has a text),
/// else the file name.
fn effect_name(preset: &Path) -> String {
    let stem = preset
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    if effects::BUILTIN.contains(&stem.as_str())
        && preset.extension().is_some_and(|e| e == "slangp")
    {
        stem
    } else {
        preset
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// The `fx.*` keys: a change to any of them calls [`apply`].
pub fn is_effect_key(key: &str) -> bool {
    key.starts_with("fx.")
}

/// Sends the effect the settings ask for (seen from the open game, else from «Tutti i giochi»)
/// to the native renderer. Without one (webview video) there is nothing to do.
pub fn apply(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let Some(renderer) = &state.renderer else {
        return;
    };
    let Some(settings) = app.try_state::<SettingsState>() else {
        return;
    };
    let values = {
        let mut settings = settings.0.lock();
        let scope = if settings.session_scope().is_some() {
            Scope::Session
        } else {
            Scope::Global
        };
        let mut read = |key: &str| -> Option<Value> {
            match settings.resolve(key, &scope) {
                Ok(r) => r.value,
                Err(e) => {
                    log::warn!("{key} unavailable: {e:?}");
                    None
                }
            }
        };
        // Read under the lock, decided after it.
        [keys::FX_PICK.as_str(), keys::FX_FILE.as_str()]
            .into_iter()
            .chain([keys::FX_INTENSITY.as_str(), keys::FX_BRIGHTNESS.as_str()])
            .map(|key| (key, read(key)))
            .collect::<Vec<_>>()
    };
    let choice = effects::from_settings(|key| {
        values
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.clone())
    });
    let builtin_dir = app
        .try_state::<EffectsState>()
        .and_then(|s| s.builtin_dir.clone());
    let spec = choice.and_then(|c| {
        let preset = match c.source {
            EffectSource::Builtin(id) => builtin_dir?.join(format!("{id}.slangp")),
            EffectSource::File(path) => path,
        };
        Some(EffectSpec {
            preset,
            intensity: c.intensity,
            brightness: c.brightness,
        })
    });
    renderer.set_effect(spec);
}

/// «Aggiungi un effetto da file»: picks a `.slangp` and stores it in `fx.fxfile` («Tutti i
/// giochi»). Resolves the path, or `None` when the picker was cancelled. Whether the preset
/// works shows when it is drawn (`emu://effect-error` otherwise).
#[tauri::command]
pub async fn effects_choose_file(app: AppHandle) -> Result<Option<String>, ErrorCode> {
    let Some(file) = emudome_platform::pick_file(&app, "RetroArch", &["slangp"]) else {
        return Ok(None);
    };
    let path = file.to_string_lossy().into_owned();
    let level = {
        let state = app.state::<SettingsState>();
        let mut settings = state.0.lock();
        settings.set(
            keys::FX_FILE.as_str(),
            &Scope::Global,
            Value::Text(path.clone()),
        )?
    };
    crate::settings::emit_changed(&app, &[keys::FX_FILE.as_str()], level);
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::{effect_name, write_builtins, BUILTIN_FILES};
    use emudome_core::effects::BUILTIN;
    use std::path::Path;

    #[test]
    fn every_built_in_effect_has_its_preset() {
        for id in BUILTIN {
            let name = format!("{id}.slangp");
            assert!(
                BUILTIN_FILES.iter().any(|(rel, _)| *rel == name),
                "{name} missing"
            );
        }
    }

    #[test]
    fn built_in_names_are_ids_and_files_keep_their_name() {
        assert_eq!(effect_name(Path::new("/x/0.0.1/crt.slangp")), "crt");
        assert_eq!(
            effect_name(Path::new("C:/mine/crt-royale.slangp")),
            "crt-royale.slangp"
        );
    }

    #[test]
    fn the_built_in_presets_are_written_and_loadable() {
        let base = std::env::temp_dir().join(format!("emudome-fx-{}", std::process::id()));
        let dir = write_builtins(&base).expect("written");
        // A second time finds them equal and writes nothing.
        write_builtins(&base).expect("written again");
        for id in BUILTIN {
            let preset = dir.join(format!("{id}.slangp"));
            librashader::presets::ShaderPreset::try_parse(
                &preset,
                librashader::presets::ShaderFeatures::NONE,
            )
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
