//! RetroPad button state, written by keyboard, controllers and touch,
//! read by the core. One bitmask per source, merged on read.

use std::sync::atomic::{AtomicU16, Ordering};

/// RetroPad buttons, each with the bit of its libretro id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Button {
    B = 0,
    Y = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
    A = 8,
    X = 9,
    L = 10,
    R = 11,
}

impl Button {
    pub fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// Where a pressed button comes from.
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Keyboard = 0,
    Gamepad = 1,
    Touch = 2,
}

#[derive(Default)]
pub struct InputState {
    sources: [AtomicU16; 3],
}

impl InputState {
    pub fn set(&self, source: Source, button: Button, pressed: bool) {
        let slot = &self.sources[source as usize];
        if pressed {
            slot.fetch_or(button.bit(), Ordering::Relaxed);
        } else {
            slot.fetch_and(!button.bit(), Ordering::Relaxed);
        }
    }

    /// Replaces a source's whole mask at once (handy for controllers).
    pub fn set_mask(&self, source: Source, mask: u16) {
        self.sources[source as usize].store(mask, Ordering::Relaxed);
    }

    pub fn mask(&self) -> u16 {
        self.sources
            .iter()
            .fold(0, |m, s| m | s.load(Ordering::Relaxed))
    }
}
