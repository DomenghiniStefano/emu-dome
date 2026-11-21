//! A core with no emulator inside, for the tests of the session ([`crate::Session::start_with`])
//! here and of the app's game lifecycle in the shell crate. Not used by the app itself.
//!
//! Its state is a frame counter (8 bytes, little endian); its in-game save holds the same counter
//! as of the last frame run, so a test can tell which frame a file was written at.

use std::sync::Arc;

use libretro_sys::retro_system_av_info;
use parking_lot::{Condvar, Mutex};

use crate::session::{Core, CoreInfo};
use crate::speed_audio::FastAudio;

/// Holds a [`FakeCore`] inside `run` until opened: a core stuck in `retro_run`.
#[derive(Default)]
pub struct Gate {
    /// (open, a `run` is held at the gate)
    state: Mutex<(bool, bool)>,
    changed: Condvar,
}

impl Gate {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Lets every waiting and future `run` through.
    pub fn open(&self) {
        self.state.lock().0 = true;
        self.changed.notify_all();
    }

    /// Waits up to `timeout` until a `run` is held at the gate (the core is stuck now); false if
    /// none came.
    pub fn wait_held(&self, timeout: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        let mut state = self.state.lock();
        while !state.1 {
            if self.changed.wait_until(&mut state, deadline).timed_out() {
                return state.1;
            }
        }
        true
    }

    fn wait(&self) {
        let mut state = self.state.lock();
        if !state.0 {
            state.1 = true;
            self.changed.notify_all();
        }
        while !state.0 {
            self.changed.wait(&mut state);
        }
    }
}

/// What the session told the core's callbacks, for the tests to read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FakeCalls {
    pub audio_enabled: Option<bool>,
    pub audio_speed: Option<Option<f64>>,
    pub fast_audio: Option<FastAudio>,
    pub options: Vec<(String, String)>,
    pub resets: u32,
    /// `unserialize` calls (loads, undo, rewind steps), valid states or not.
    pub unserializes: u32,
    /// `end` was called, and whether the core was let go.
    pub ended: Option<bool>,
}

/// The fake core. `calls` is shared, so a test keeps a handle on it after the core moved into a
/// session.
#[derive(Clone)]
pub struct FakeCore {
    pub frames: u64,
    pub sram: Vec<u8>,
    pub fps: f64,
    /// A rate the next frame reports as the core's new timing (`SET_SYSTEM_AV_INFO`).
    pub next_fps: Option<f64>,
    /// While set and closed, `run` blocks.
    pub gate: Option<Arc<Gate>>,
    /// `serialize` fails (a core without states).
    pub no_states: bool,
    pub calls: Arc<Mutex<FakeCalls>>,
}

impl Default for FakeCore {
    fn default() -> Self {
        Self {
            frames: 0,
            sram: vec![0; 8],
            fps: 60.0,
            next_fps: None,
            gate: None,
            no_states: false,
            calls: Arc::default(),
        }
    }
}

impl FakeCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// A core whose `run` blocks until `gate` opens.
    pub fn stuck(gate: Arc<Gate>) -> Self {
        Self {
            gate: Some(gate),
            ..Self::default()
        }
    }

    /// The frame counter a state or an in-game save holds (`None` for other bytes).
    pub fn counter(bytes: &[u8]) -> Option<u64> {
        let bytes: [u8; 8] = bytes.get(..8)?.try_into().ok()?;
        Some(u64::from_le_bytes(bytes))
    }
}

impl Core for FakeCore {
    fn info(&self) -> CoreInfo {
        let mut av = retro_system_av_info::default();
        av.timing.fps = self.fps;
        av.timing.sample_rate = 32768.0;
        CoreInfo {
            name: "Fake".into(),
            version: "1".into(),
            av,
        }
    }

    fn run(&mut self) {
        if let Some(gate) = &self.gate {
            gate.wait();
        }
        self.frames += 1;
        self.sram = self.frames.to_le_bytes().to_vec();
    }

    fn reset(&mut self) {
        self.frames = 0;
        self.calls.lock().resets += 1;
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.serialize_into(&mut buf)?;
        Ok(buf)
    }

    fn serialize_into(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        if self.no_states {
            anyhow::bail!("this core has no states");
        }
        buf.clear();
        buf.extend_from_slice(&self.frames.to_le_bytes());
        Ok(())
    }

    fn unserialize(&mut self, state: &[u8]) -> anyhow::Result<()> {
        self.calls.lock().unserializes += 1;
        match (state.len(), Self::counter(state)) {
            (8, Some(frames)) => {
                self.frames = frames;
                Ok(())
            }
            _ => anyhow::bail!("not a state of this core"),
        }
    }

    fn sram(&self) -> Option<&[u8]> {
        Some(&self.sram)
    }

    fn sram_mut(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.sram)
    }

    fn take_av_change(&mut self) -> Option<retro_system_av_info> {
        let fps = self.next_fps.take()?;
        self.fps = fps;
        Some(self.info().av)
    }

    fn set_audio_speed(&mut self, speed: Option<f64>) {
        self.calls.lock().audio_speed = Some(speed);
    }

    fn set_audio_enabled(&mut self, enabled: bool) {
        self.calls.lock().audio_enabled = Some(enabled);
    }

    fn set_fast_audio(&mut self, policy: FastAudio) {
        self.calls.lock().fast_audio = Some(policy);
    }

    fn set_core_option(&mut self, key: String, value: String) {
        self.calls.lock().options.push((key, value));
    }

    fn end(self, let_go: bool) {
        self.calls.lock().ended = Some(let_go);
    }
}
