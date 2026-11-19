//! RetroPad button state, written by keyboard, controllers and touch,
//! read by the core. One bitmask per source, merged on read.
//! Also the touch-screen pointer (NDS, 3DS), read as `RETRO_DEVICE_POINTER`.

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
}

#[derive(Default)]
pub struct InputState {
    sources: [AtomicU16; 3],
    /// The touch-screen pointer in `RETRO_DEVICE_POINTER` values
    /// ([`crate::screens::pointer_value`]): x in bits 0-15, y in 16-31, pressed in bit 32.
    /// One pointer whatever moves it (mouse, finger): the last writer wins.
    pointer: AtomicU64,
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
}

#[cfg(test)]
mod tests {
    use super::{InputState, Pointer};

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
