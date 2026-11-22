//! Physical controllers, read through what each system offers (Trello #47): one [`PadSource`]
//! for the app's controller thread, whatever the backend.
//!
//! - Desktop (Windows, Linux): SDL3's gamepad subsystem (`sdl.rs`), with HIDAPI (Steam
//!   Controller without Steam, PS, Switch Pro). SDL already gives every pad its standard layout.
//! - Android: the system's own input events (`android.rs`, Kotlin class
//!   `it.domehub.emudome.platform.GamepadInput`): `KeyEvent` / `MotionEvent` of gamepads and
//!   joysticks, taken in the activity's window before the WebView sees them, and the device list
//!   through `InputManager.InputDeviceListener`. Kotlin only forwards raw key codes and axis
//!   values; the layout (which key is which position, which axes are the right stick and the
//!   triggers) is decided here, in pure tested code (`android_map.rs`).
//!
//! Both give the same thing: devices added / removed with their name, kind and USB ids, and per
//! poll each pad's buttons by **position** ([`PadButton`]) and six axes in SDL's ranges. What the
//! positions mean (game buttons, combos, menu steps) is the app's (`app/src-tauri/src/gamepad.rs`).

#[cfg(target_os = "android")]
pub(crate) mod android;
mod android_map;
#[cfg(not(target_os = "android"))]
mod sdl;

#[cfg(target_os = "android")]
use android as backend;
#[cfg(not(target_os = "android"))]
use sdl as backend;

/// A pad's buttons by position (SDL's standard gamepad layout): `South` is the bottom face
/// button whatever is printed on it. The triggers are axes (`PadReading::axes`), a digital
/// trigger reads as a fully pressed axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    South,
    East,
    West,
    North,
    L1,
    R1,
    /// Left stick click.
    L3,
    R3,
    /// Select / View / Share / −.
    Back,
    /// Start / Menu / Options / +.
    Start,
    /// Guide / Home / PS.
    Guide,
    Up,
    Down,
    Left,
    Right,
}

impl PadButton {
    pub const ALL: [PadButton; 15] = [
        PadButton::South,
        PadButton::East,
        PadButton::West,
        PadButton::North,
        PadButton::L1,
        PadButton::R1,
        PadButton::L3,
        PadButton::R3,
        PadButton::Back,
        PadButton::Start,
        PadButton::Guide,
        PadButton::Up,
        PadButton::Down,
        PadButton::Left,
        PadButton::Right,
    ];

    /// Its bit in [`PadReading::buttons`].
    pub fn bit(self) -> u16 {
        1 << self as u16
    }

    /// For logs and probes.
    pub fn name(self) -> &'static str {
        match self {
            PadButton::South => "south",
            PadButton::East => "east",
            PadButton::West => "west",
            PadButton::North => "north",
            PadButton::L1 => "l1",
            PadButton::R1 => "r1",
            PadButton::L3 => "l3",
            PadButton::R3 => "r3",
            PadButton::Back => "back",
            PadButton::Start => "start",
            PadButton::Guide => "guide",
            PadButton::Up => "up",
            PadButton::Down => "down",
            PadButton::Left => "left",
            PadButton::Right => "right",
        }
    }
}

/// Which maker's layout a pad has, as far as the system can tell (SDL's controller type; on
/// Android the USB vendor id, then the name).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadKind {
    /// Xbox pads, and the Steam Controller (Xbox letters).
    Xbox,
    PlayStation,
    /// Switch Pro, Joy-Con, pads in Switch mode.
    Nintendo,
    Other,
}

/// A connected pad.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PadDevice {
    /// The backend's id while it stays connected (SDL's instance id, Android's device id).
    pub id: u32,
    /// `None` when the system knows none.
    pub name: Option<String>,
    pub kind: PadKind,
    /// The backend's own word for its type, for logs (SDL's type string, `android`).
    pub type_name: String,
    pub vendor: u16,
    pub product: u16,
    /// Battery charge left, 0..=100; `None` when wired or not told.
    pub battery: Option<u8>,
}

/// One pad read at one poll.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadReading {
    /// [`PadButton::bit`]s held.
    pub buttons: u16,
    /// Left x, left y, right x, right y (-32768..=32767, y positive downwards), left trigger,
    /// right trigger (0..=32767): SDL's axes, in this order.
    pub axes: [i16; 6],
}

/// What changed in the list of pads since the last poll.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PadEvent {
    /// A pad to read from now on. `at_start`: it was already there when the source opened.
    Added { device: PadDevice, at_start: bool },
    /// The pad with this id is gone.
    Removed(u32),
    /// The pad's battery charge as the system tells it now (it may be unchanged).
    Battery { id: u32, battery: Option<u8> },
}

/// The controllers of this system. Open it on the thread that polls it, and keep it there (SDL
/// is used from the thread that initialised it).
pub struct PadSource {
    inner: backend::Source,
}

impl PadSource {
    /// What reads the pads here, for logs: `SDL3` or `Android input`.
    pub const BACKEND: &'static str = backend::NAME;

    /// Starts reading the controllers. The pads already connected come as the first
    /// [`PadEvent::Added`]s, with `at_start` set. `Err` with the system's reason when the
    /// controllers cannot be read at all.
    pub fn open() -> Result<PadSource, String> {
        backend::Source::open().map(|inner| PadSource { inner })
    }

    /// Appends to `events` what changed since the last call.
    pub fn poll(&mut self, events: &mut Vec<PadEvent>) {
        self.inner.poll(events);
    }

    /// The pad's state now; `None` for an id that is not connected.
    pub fn read(&mut self, id: u32) -> Option<PadReading> {
        self.inner.read(id)
    }
}
