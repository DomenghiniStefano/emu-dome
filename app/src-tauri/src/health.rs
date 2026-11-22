//! What is not working right now, for the interface (Trello #79): failures that used to reach
//! only the log (the native video falling back, no audio output, a surface that cannot be drawn
//! on, SDL not starting, a thread that stopped) are kept here as [`ErrorCode`]s, one per
//! [`Part`], and sent whole as `emu://health` each time the list changes; `diagnostics` answers the
//! same list (for what happened before the page listened). A part that recovers clears its code
//! and the list is sent again.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use emudome_core::ErrorCode;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};

/// The pieces of EmuDome that report here, each with at most one code at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    /// The graphics API of `video.api` has no GPU here (wgpu chose another), set at startup.
    GraphicsApi,
    /// The native renderer could not start (the webview canvas is used).
    NativeVideo,
    /// The surface the renderer draws on.
    Surface,
    /// A game whose core draws on the GPU (3DS) and the Vulkan device it renders on.
    HwRender,
    /// The audio output.
    Audio,
    /// The controllers (SDL3 on the desktop, Android's input events on Android).
    Controllers,
    /// The one-time copy of the data to the new app identifier's folders (`data_move.rs`).
    DataFolder,
    /// A core that wrote into its own user folder instead of the game's (Azahar, Trello #97):
    /// set once seen, kept until EmuDome restarts.
    CoreUserFolder,
}

impl Part {
    /// The name `threadStopped` carries for a thread of this part.
    pub fn thread(self) -> &'static str {
        match self {
            Part::GraphicsApi | Part::NativeVideo | Part::Surface | Part::HwRender => "video",
            Part::Audio => "audio",
            Part::Controllers => "controllers",
            // Run at start on the main thread, or by the close flow's worker: never in a
            // guarded thread.
            Part::DataFolder | Part::CoreUserFolder => "data",
        }
    }
}

/// Sends the whole list to the interface.
type Sink = Box<dyn Fn(Vec<ErrorCode>) + Send + Sync>;

#[derive(Default)]
pub struct Health {
    issues: Mutex<BTreeMap<Part, ErrorCode>>,
    /// Set once the app exists; codes reported before are sent when it is. A closure, not the
    /// `AppHandle`: the unit tests then never reach Tauri's runtime (whose Windows imports a
    /// test binary cannot load).
    sink: OnceLock<Sink>,
}

pub type SharedHealth = Arc<Health>;

impl Health {
    /// Sets (`Some`) or clears (`None`) the code of `part`; the interface hears of a change.
    pub fn set(&self, part: Part, code: Option<ErrorCode>) {
        let changed = {
            let mut issues = self.issues.lock();
            let before = issues.get(&part).cloned();
            match &code {
                Some(c) => {
                    log::warn!("health: {part:?}: {c:?}");
                    issues.insert(part, c.clone());
                }
                None => {
                    issues.remove(&part);
                }
            }
            before != code
        };
        if changed {
            self.emit();
        }
    }

    /// Every code now, in the order of [`Part`].
    pub fn list(&self) -> Vec<ErrorCode> {
        self.issues.lock().values().cloned().collect()
    }

    /// The app is there: what was reported before is sent.
    pub fn attach(&self, app: AppHandle) {
        let sink: Sink = Box::new(move |list| {
            if let Err(e) = app.emit(crate::events::HEALTH, list) {
                log::warn!("emu://health not sent: {e}");
            }
        });
        if self.sink.set(sink).is_ok() && !self.issues.lock().is_empty() {
            self.emit();
        }
    }

    fn emit(&self) {
        if let Some(sink) = self.sink.get() {
            sink(self.list());
        }
    }

    /// Runs a thread's body; a panic (in a build that unwinds: the release build aborts) is
    /// reported as `threadStopped` for `part` instead of leaving a frozen picture or silence.
    pub fn guard(&self, part: Part, body: impl FnOnce()) {
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).is_err() {
            log::error!("the {} thread panicked", part.thread());
            self.set(
                part,
                Some(ErrorCode::ThreadStopped {
                    thread: part.thread().into(),
                }),
            );
        }
    }
}

/// What is not working right now (`emu://health` sends the same list when it changes).
#[tauri::command]
pub fn diagnostics(health: tauri::State<'_, SharedHealth>) -> Vec<ErrorCode> {
    health.list()
}

#[cfg(test)]
mod tests {
    use super::{Health, Part};
    use emudome_core::ErrorCode;

    #[test]
    fn one_code_per_part_set_and_cleared() {
        let health = Health::default();
        health.set(
            Part::Audio,
            Some(ErrorCode::AudioUnavailable {
                detail: "no device".into(),
            }),
        );
        health.set(
            Part::NativeVideo,
            Some(ErrorCode::NativeVideoFallback {
                detail: "no GPU".into(),
            }),
        );
        health.set(
            Part::Audio,
            Some(ErrorCode::AudioUnavailable {
                detail: "unplugged".into(),
            }),
        );
        assert_eq!(
            health.list(),
            vec![
                ErrorCode::NativeVideoFallback {
                    detail: "no GPU".into()
                },
                ErrorCode::AudioUnavailable {
                    detail: "unplugged".into()
                },
            ]
        );
        health.set(Part::Audio, None);
        assert_eq!(health.list().len(), 1);
    }

    #[test]
    fn a_panicking_thread_is_reported() {
        let health = Health::default();
        health.guard(Part::Controllers, || panic!("SDL went away"));
        assert_eq!(
            health.list(),
            vec![ErrorCode::ThreadStopped {
                thread: "controllers".into()
            }]
        );
        health.guard(Part::Audio, || {});
        assert_eq!(health.list().len(), 1);
    }
}
