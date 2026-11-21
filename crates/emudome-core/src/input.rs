//! RetroPad button state, written by keyboard, controllers and touch,
//! read by the core. One bitmask per source, merged on read.
//! Also the touch-screen pointer (NDS, 3DS), read as `RETRO_DEVICE_POINTER`, and the two analog
//! sticks (3DS Circle Pad and C-stick, read as `RETRO_DEVICE_ANALOG`).
//!
//! The core never reads these atomics directly: the session takes a [`Snapshot`] before each
//! frame and at the core's `input_poll`, so every `input_state` of one poll sees the same input.

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};

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
    /// Left trigger (RetroPad L2).
    L2 = 12,
    /// Right trigger (RetroPad R2).
    R2 = 13,
    /// Left stick click (RetroPad L3): EmuDome's combo modifier, never sent to the core.
    L3 = 14,
    /// Right stick click (RetroPad R3).
    R3 = 15,
}

impl Button {
    /// Every button, in libretro id order.
    pub const ALL: [Button; 16] = [
        Button::B,
        Button::Y,
        Button::Select,
        Button::Start,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::A,
        Button::X,
        Button::L,
        Button::R,
        Button::L2,
        Button::R2,
        Button::L3,
        Button::R3,
    ];

    pub fn bit(self) -> u16 {
        1 << (self as u16)
    }

    /// The stable id (same as the serde name): settings values and the interface use it.
    pub fn id(self) -> &'static str {
        match self {
            Button::B => "b",
            Button::Y => "y",
            Button::Select => "select",
            Button::Start => "start",
            Button::Up => "up",
            Button::Down => "down",
            Button::Left => "left",
            Button::Right => "right",
            Button::A => "a",
            Button::X => "x",
            Button::L => "l",
            Button::R => "r",
            Button::L2 => "l2",
            Button::R2 => "r2",
            Button::L3 => "l3",
            Button::R3 => "r3",
        }
    }

    pub fn from_id(id: &str) -> Option<Button> {
        Button::ALL.into_iter().find(|b| b.id() == id)
    }
}

/// Where a pressed button comes from.
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Keyboard = 0,
    Gamepad = 1,
    Touch = 2,
    /// The shell itself: a core's own chord pressed for a moment (the lid,
    /// `core_profile::Lid::Chord`).
    Shell = 3,
}

#[derive(Default)]
pub struct InputState {
    sources: [AtomicU16; 4],
    /// The touch-screen pointer in `RETRO_DEVICE_POINTER` values
    /// ([`crate::screens::pointer_value`]): x in bits 0-15, y in 16-31, pressed in bit 32.
    /// One pointer whatever moves it (mouse, finger): the last writer wins.
    pointer: AtomicU64,
    /// Both sticks, `-0x8000..=0x7FFF` each: left x, left y, right x, right y in bits 0-15,
    /// 16-31, 32-47, 48-63. Written by the controllers only.
    sticks: AtomicU64,
}

/// The analog sticks as the core reads them (`RETRO_DEVICE_ANALOG`; y grows downwards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sticks {
    pub left: (i16, i16),
    pub right: (i16, i16),
}

/// Everything the core can read, frozen at one moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Snapshot {
    pub mask: u16,
    pub pointer: Pointer,
    pub sticks: Sticks,
}

/// Where the touch-screen pointer is, as the core reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pointer {
    pub x: i16,
    pub y: i16,
    pub pressed: bool,
}

const POINTER_PRESSED: u64 = 1 << 32;

impl InputState {
    pub fn set(&self, source: Source, button: Button, pressed: bool) {
        let slot = &self.sources[source as usize];
        if pressed {
            slot.fetch_or(button.bit(), Ordering::Relaxed);
        } else {
            slot.fetch_and(!button.bit(), Ordering::Relaxed);
        }
    }

    /// Lets go of every button of `source`: the keyboard when the quick menu opens or the window
    /// loses the focus, so a key released elsewhere is never held in the game.
    pub fn release(&self, source: Source) {
        self.set_mask(source, 0);
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

    /// Touches the screen at `at` (pointer values), or lets go with `None`: the last position
    /// stays, as a finger lifted where it was.
    pub fn set_pointer(&self, at: Option<(i16, i16)>) {
        let packed = match at {
            Some((x, y)) => u64::from(x as u16) | (u64::from(y as u16) << 16) | POINTER_PRESSED,
            None => self.pointer.load(Ordering::Relaxed) & !POINTER_PRESSED,
        };
        self.pointer.store(packed, Ordering::Relaxed);
    }

    pub fn pointer(&self) -> Pointer {
        let p = self.pointer.load(Ordering::Relaxed);
        Pointer {
            x: p as u16 as i16,
            y: (p >> 16) as u16 as i16,
            pressed: p & POINTER_PRESSED != 0,
        }
    }

    /// Both sticks at once (centred: `Sticks::default()`).
    pub fn set_sticks(&self, sticks: Sticks) {
        let part = |v: i16, shift: u32| u64::from(v as u16) << shift;
        let packed = part(sticks.left.0, 0)
            | part(sticks.left.1, 16)
            | part(sticks.right.0, 32)
            | part(sticks.right.1, 48);
        self.sticks.store(packed, Ordering::Relaxed);
    }

    pub fn sticks(&self) -> Sticks {
        let p = self.sticks.load(Ordering::Relaxed);
        let part = |shift: u32| (p >> shift) as u16 as i16;
        Sticks {
            left: (part(0), part(16)),
            right: (part(32), part(48)),
        }
    }

    /// What the core reads until the next snapshot.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            mask: self.mask(),
            pointer: self.pointer(),
            sticks: self.sticks(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, InputState, Pointer, Snapshot, Source, Sticks};

    #[test]
    fn sticks_keep_both_axes_and_their_signs() {
        let input = InputState::default();
        assert_eq!(input.sticks(), Sticks::default());
        let s = Sticks {
            left: (-32768, 32767),
            right: (-1, 12345),
        };
        input.set_sticks(s);
        assert_eq!(input.sticks(), s);
        input.set(Source::Gamepad, Button::A, true);
        let snap = input.snapshot();
        assert_eq!(
            snap,
            Snapshot {
                mask: Button::A.bit(),
                pointer: Pointer::default(),
                sticks: s
            }
        );
        // A snapshot does not move with later input.
        input.set_sticks(Sticks::default());
        assert_eq!(snap.sticks, s);
    }

    #[test]
    fn releasing_a_source_lets_go_of_its_buttons_only() {
        let input = InputState::default();
        input.set(Source::Keyboard, Button::Right, true);
        input.set(Source::Keyboard, Button::A, true);
        input.set(Source::Gamepad, Button::B, true);
        input.release(Source::Keyboard);
        assert_eq!(input.mask(), Button::B.bit());
        input.release(Source::Gamepad);
        assert_eq!(input.mask(), 0);
    }

    #[test]
    fn the_pointer_keeps_its_place_when_released() {
        let input = InputState::default();
        assert_eq!(input.pointer(), Pointer::default());
        input.set_pointer(Some((-32639, 12345)));
        assert_eq!(
            input.pointer(),
            Pointer {
                x: -32639,
                y: 12345,
                pressed: true
            }
        );
        input.set_pointer(None);
        assert_eq!(
            input.pointer(),
            Pointer {
                x: -32639,
                y: 12345,
                pressed: false
            }
        );
    }
}
