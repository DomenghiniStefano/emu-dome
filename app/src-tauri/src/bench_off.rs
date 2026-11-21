//! Bench mode left out of this build (the `bench` cargo feature is off, as in every package):
//! `EMUDOME_BENCH` is ignored with a warning, and the interface's `bench_frames` command does not
//! exist. `bench_active` stays, answering `false`, because the canvas asks it at every start.
//! The bench itself: `bench.rs`, built with `cargo build -p emudome --features bench`.

use std::sync::Arc;
use std::time::{Instant, SystemTime};

use crate::tuning::Tuning;

/// Never built: there is no bench in this build.
pub enum Bench {}

impl Bench {
    /// Always `None`; warns when `EMUDOME_BENCH` asked for a bench.
    pub fn new(tuning: &Tuning, _main_start: Instant, _main_wall: SystemTime) -> Option<Arc<Self>> {
        if tuning.bench.is_some() {
            log::warn!("EMUDOME_BENCH ignored: this build has no bench (cargo feature `bench`)");
        }
        None
    }

    pub fn tone(&self) -> Option<f32> {
        match *self {}
    }
}

/// Whether the interface should report its canvas rounds: never in this build.
#[tauri::command]
pub fn bench_active() -> bool {
    false
}

pub fn spawn(_app: tauri::AppHandle, bench: Arc<Bench>) {
    match *bench {}
}
