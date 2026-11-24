//! Android's controllers as positions: the pure part of the Android pad source, compiled and
//! tested on every system (the JNI glue is `android.rs`).
//!
//! Kotlin (`GamepadInput.kt`) forwards, for gamepad and joystick devices only, the raw
//! `KeyEvent` key codes and the values of [`AXES`]; at connection the device's motion ranges for
//! those axes. Here:
//! - [`key_target`]: Android's key codes by position, as SDL's own Android backend reads them
//!   (`KEYCODE_BUTTON_A` the bottom face button, `KEYCODE_BACK` / `KEYCODE_MENU` from a pad are
//!   Select / Start);
//! - [`AxisLayout`]: which axes are the right stick and the triggers. Android normalises most
//!   pads to `AXIS_Z` / `AXIS_RZ` for the right stick and `AXIS_LTRIGGER` / `AXIS_RTRIGGER` (or
//!   `AXIS_BRAKE` / `AXIS_GAS`) for the triggers, but pads without a key layout file of their own
//!   come with the right stick on `AXIS_RX` / `AXIS_RY` and the triggers on `AXIS_Z` / `AXIS_RZ`,
//!   and some old ones the other way round: a pair centred on 0 (min -1) is a stick, a pair
//!   from 0 up is a pair of triggers;
//! - [`Pads`]: the connected pads and their state, fed by the JNI calls, read by the source.
#![cfg_attr(
    not(target_os = "android"),
    allow(dead_code, reason = "used by android.rs; tested on every system")
)]

use super::{PadButton, PadDevice, PadEvent, PadKind, PadReading};

// Android's `MotionEvent.AXIS_*` values.
const AXIS_X: i32 = 0;
const AXIS_Y: i32 = 1;
const AXIS_Z: i32 = 11;
const AXIS_RX: i32 = 12;
const AXIS_RY: i32 = 13;
const AXIS_RZ: i32 = 14;
const AXIS_HAT_X: i32 = 15;
const AXIS_HAT_Y: i32 = 16;
const AXIS_LTRIGGER: i32 = 17;
const AXIS_RTRIGGER: i32 = 18;
const AXIS_GAS: i32 = 22;
const AXIS_BRAKE: i32 = 23;

/// The axes Kotlin reads, in the order of the values it sends (Kotlin gets this list from Rust
/// at attach: one source of truth).
pub(crate) const AXES: [i32; 12] = [
    AXIS_X,
    AXIS_Y,
    AXIS_Z,
    AXIS_RZ,
    AXIS_RX,
    AXIS_RY,
    AXIS_HAT_X,
    AXIS_HAT_Y,
    AXIS_LTRIGGER,
    AXIS_RTRIGGER,
    AXIS_BRAKE,
    AXIS_GAS,
];

// Indices into AXES (and into the values and ranges Kotlin sends).
const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;
const RZ: usize = 3;
const RX: usize = 4;
const RY: usize = 5;
const HAT_X: usize = 6;
const HAT_Y: usize = 7;
const LTRIGGER: usize = 8;
const RTRIGGER: usize = 9;
const BRAKE: usize = 10;
const GAS: usize = 11;

// Android's `KeyEvent.KEYCODE_*` values.
const KEYCODE_BACK: i32 = 4;
const KEYCODE_DPAD_UP: i32 = 19;
const KEYCODE_DPAD_DOWN: i32 = 20;
const KEYCODE_DPAD_LEFT: i32 = 21;
const KEYCODE_DPAD_RIGHT: i32 = 22;
const KEYCODE_MENU: i32 = 82;
const KEYCODE_BUTTON_A: i32 = 96;
const KEYCODE_BUTTON_B: i32 = 97;
const KEYCODE_BUTTON_X: i32 = 99;
const KEYCODE_BUTTON_Y: i32 = 100;
const KEYCODE_BUTTON_L1: i32 = 102;
const KEYCODE_BUTTON_R1: i32 = 103;
const KEYCODE_BUTTON_L2: i32 = 104;
const KEYCODE_BUTTON_R2: i32 = 105;
const KEYCODE_BUTTON_THUMBL: i32 = 106;
const KEYCODE_BUTTON_THUMBR: i32 = 107;
const KEYCODE_BUTTON_START: i32 = 108;
const KEYCODE_BUTTON_SELECT: i32 = 109;
const KEYCODE_BUTTON_MODE: i32 = 110;

/// What a pad's key is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Button(PadButton),
    /// A digital trigger: reads as its axis fully pressed.
    L2,
    R2,
}

/// The position of an Android key code from a pad; `None` for a key EmuDome does not read
/// (left to the system: volume, the C / Z and numbered buttons of some generic pads…).
pub(crate) fn key_target(code: i32) -> Option<Key> {
    let button = match code {
        KEYCODE_BUTTON_A => PadButton::South,
        KEYCODE_BUTTON_B => PadButton::East,
        KEYCODE_BUTTON_X => PadButton::West,
        KEYCODE_BUTTON_Y => PadButton::North,
        KEYCODE_BUTTON_L1 => PadButton::L1,
        KEYCODE_BUTTON_R1 => PadButton::R1,
        KEYCODE_BUTTON_THUMBL => PadButton::L3,
        KEYCODE_BUTTON_THUMBR => PadButton::R3,
        // Some pads send Back for View / Select (the phone's own back key never comes here:
        // Kotlin forwards only gamepad and joystick devices).
        KEYCODE_BUTTON_SELECT | KEYCODE_BACK => PadButton::Back,
        KEYCODE_BUTTON_START | KEYCODE_MENU => PadButton::Start,
        KEYCODE_BUTTON_MODE => PadButton::Guide,
        KEYCODE_DPAD_UP => PadButton::Up,
        KEYCODE_DPAD_DOWN => PadButton::Down,
        KEYCODE_DPAD_LEFT => PadButton::Left,
        KEYCODE_DPAD_RIGHT => PadButton::Right,
        KEYCODE_BUTTON_L2 => return Some(Key::L2),
        KEYCODE_BUTTON_R2 => return Some(Key::R2),
        _ => return None,
    };
    Some(Key::Button(button))
}

/// The maker's layout from the USB vendor id, then the name (pads in another mode report the
/// vendor of the mode: an 8BitDo in Switch mode says Nintendo's).
pub(crate) fn kind(vendor: u16, name: Option<&str>) -> PadKind {
    match vendor {
        0x045e => return PadKind::Xbox,
        // Valve: the Steam Controller has Xbox letters (as SDL's Steam type).
        0x28de => return PadKind::Xbox,
        0x054c => return PadKind::PlayStation,
        0x057e => return PadKind::Nintendo,
        _ => {}
    }
    let name = name.unwrap_or_default().to_ascii_lowercase();
    if name.contains("xbox") {
        PadKind::Xbox
    } else if ["dualsense", "dualshock", "ps4", "ps5", "playstation"]
        .iter()
        .any(|w| name.contains(w))
    {
        PadKind::PlayStation
    } else if ["pro controller", "joy-con", "nintendo", "switch"]
        .iter()
        .any(|w| name.contains(w))
    {
        PadKind::Nintendo
    } else {
        PadKind::Other
    }
}

/// A battery charge as Kotlin sends it: 0..=100, or negative when not told.
pub(crate) fn battery(percent: i32) -> Option<u8> {
    u8::try_from(percent).ok().filter(|p| *p <= 100)
}

/// One axis's motion range on the device, `None` when it has no such axis (Kotlin sends NaN).
pub(crate) type Range = Option<(f32, f32)>;

/// The ranges Kotlin sends: min and max per axis of [`AXES`], NaN for an axis the device lacks.
pub(crate) fn ranges(flat: &[f32]) -> [Range; AXES.len()] {
    let mut out = [None; AXES.len()];
    for (slot, &[min, max]) in out.iter_mut().zip(flat.as_chunks::<2>().0) {
        if min.is_finite() && max.is_finite() && max > min {
            *slot = Some((min, max));
        }
    }
    out
}

/// Which of [`AXES`] give the right stick and the triggers of one device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AxisLayout {
    /// Indices of the right stick's x and y.
    pub right: Option<(usize, usize)>,
    /// Indices of the left and right analogue triggers.
    pub triggers: Option<(usize, usize)>,
}

/// A pair centred on 0 is a stick; a pair from 0 up is a pair of triggers.
fn is_stick(ranges: &[Range], a: usize, b: usize) -> bool {
    matches!((ranges[a], ranges[b]), (Some((amin, _)), Some((bmin, _))) if amin < -0.5 && bmin < -0.5)
}

fn is_triggers(ranges: &[Range], a: usize, b: usize) -> bool {
    matches!((ranges[a], ranges[b]), (Some((amin, _)), Some((bmin, _))) if amin > -0.1 && bmin > -0.1)
}

impl AxisLayout {
    pub(crate) fn from_ranges(ranges: &[Range; AXES.len()]) -> AxisLayout {
        let has = |i: usize| ranges[i].is_some();
        let mut triggers = match (
            has(LTRIGGER)
                .then_some(LTRIGGER)
                .or(has(BRAKE).then_some(BRAKE)),
            has(RTRIGGER)
                .then_some(RTRIGGER)
                .or(has(GAS).then_some(GAS)),
        ) {
            (Some(l), Some(r)) => Some((l, r)),
            _ => None,
        };
        let right = if is_stick(ranges, Z, RZ) {
            if triggers.is_none() && is_triggers(ranges, RX, RY) {
                triggers = Some((RX, RY));
            }
            Some((Z, RZ))
        } else if is_stick(ranges, RX, RY) {
            if triggers.is_none() && is_triggers(ranges, Z, RZ) {
                triggers = Some((Z, RZ));
            }
            Some((RX, RY))
        } else {
            None
        };
        AxisLayout { right, triggers }
    }
}

/// -1.0..=1.0 → SDL's -32768..=32767 (rounded, clamped).
fn stick(value: f32) -> i16 {
    if value.is_nan() {
        return 0;
    }
    // In range after the clamp: the cast cannot overflow.
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// 0.0..=1.0 → 0..=32767 (a trigger's rest below 0 reads as 0).
fn trigger(value: f32) -> i16 {
    if value.is_nan() {
        return 0;
    }
    (value.clamp(0.0, 1.0) * 32767.0).round() as i16
}

/// A hat past half travel is a d-pad direction.
const HAT_PRESSED: f32 = 0.5;

/// One connected pad: what Kotlin last sent.
#[derive(Clone, Debug)]
struct Pad {
    id: u32,
    layout: AxisLayout,
    /// [`PadButton::bit`]s of the keys held.
    keys: u16,
    l2: bool,
    r2: bool,
    axes: [f32; AXES.len()],
    /// The last reading logged ([`Pads::coarse_change`]).
    logged: Option<PadReading>,
}

impl Pad {
    fn reading(&self) -> PadReading {
        let a = &self.axes;
        let mut buttons = self.keys;
        if a[HAT_X] < -HAT_PRESSED {
            buttons |= PadButton::Left.bit();
        }
        if a[HAT_X] > HAT_PRESSED {
            buttons |= PadButton::Right.bit();
        }
        if a[HAT_Y] < -HAT_PRESSED {
            buttons |= PadButton::Up.bit();
        }
        if a[HAT_Y] > HAT_PRESSED {
            buttons |= PadButton::Down.bit();
        }
        let (rx, ry) = self
            .layout
            .right
            .map_or((0, 0), |(x, y)| (stick(a[x]), stick(a[y])));
        let (mut lt, mut rt) = self
            .layout
            .triggers
            .map_or((0, 0), |(l, r)| (trigger(a[l]), trigger(a[r])));
        if self.l2 {
            lt = i16::MAX;
        }
        if self.r2 {
            rt = i16::MAX;
        }
        PadReading {
            buttons,
            axes: [stick(a[X]), stick(a[Y]), rx, ry, lt, rt],
        }
    }
}

/// The connected pads and the events not polled yet. The JNI calls write it on the UI thread,
/// the controller thread polls and reads it.
#[derive(Debug, Default)]
pub(crate) struct Pads {
    pads: Vec<Pad>,
    events: Vec<PadEvent>,
}

impl Pads {
    pub(crate) const fn new() -> Pads {
        Pads {
            pads: Vec::new(),
            events: Vec::new(),
        }
    }

    /// A pad connected (or announced again: the old entry goes first, as a removal).
    pub(crate) fn add(&mut self, device: PadDevice, layout: AxisLayout, at_start: bool) {
        self.remove(device.id);
        self.pads.push(Pad {
            id: device.id,
            layout,
            keys: 0,
            l2: false,
            r2: false,
            axes: [0.0; AXES.len()],
            logged: None,
        });
        self.events.push(PadEvent::Added { device, at_start });
    }

    /// `false` for a pad that was not connected.
    pub(crate) fn remove(&mut self, id: u32) -> bool {
        let Some(i) = self.pads.iter().position(|p| p.id == id) else {
            return false;
        };
        self.pads.remove(i);
        self.events.push(PadEvent::Removed(id));
        true
    }

    /// A key of a pad pressed or released: `true` when EmuDome reads that key (Kotlin then
    /// consumes the event), `false` for an unknown pad or a key left to the system.
    pub(crate) fn key(&mut self, id: u32, code: i32, down: bool) -> bool {
        let Some(target) = key_target(code) else {
            return false;
        };
        let Some(pad) = self.pads.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        match target {
            Key::Button(b) if down => pad.keys |= b.bit(),
            Key::Button(b) => pad.keys &= !b.bit(),
            Key::L2 => pad.l2 = down,
            Key::R2 => pad.r2 = down,
        }
        true
    }

    /// The axes of a pad moved (`values` in [`AXES`] order): `false` for an unknown pad.
    pub(crate) fn motion(&mut self, id: u32, values: &[f32]) -> bool {
        let Some(pad) = self.pads.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        for (slot, value) in pad.axes.iter_mut().zip(values) {
            *slot = *value;
        }
        true
    }

    pub(crate) fn take_events(&mut self, out: &mut Vec<PadEvent>) {
        out.append(&mut self.events);
    }

    pub(crate) fn read(&self, id: u32) -> Option<PadReading> {
        self.pads.iter().find(|p| p.id == id).map(Pad::reading)
    }

    /// The pad's reading when it changed enough to be logged since the last time (buttons, or
    /// an axis by an eighth of its travel): a log line per real change, not per motion event.
    pub(crate) fn coarse_change(&mut self, id: u32) -> Option<PadReading> {
        let pad = self.pads.iter_mut().find(|p| p.id == id)?;
        let now = pad.reading();
        let coarse = |r: &PadReading| (r.buttons, r.axes.map(|a| a / 4096));
        if pad.logged.as_ref().map(coarse) == Some(coarse(&now)) {
            return None;
        }
        pad.logged = Some(now);
        Some(now)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        battery, key_target, kind, ranges, AxisLayout, Key, Pads, Range, AXES, AXIS_BRAKE,
        AXIS_GAS, AXIS_HAT_X, AXIS_HAT_Y, AXIS_LTRIGGER, AXIS_RTRIGGER, AXIS_RX, AXIS_RY, AXIS_RZ,
        AXIS_X, AXIS_Y, AXIS_Z, BRAKE, GAS, LTRIGGER, RTRIGGER, RX, RY, RZ, Z,
    };
    use crate::pads::{PadButton, PadDevice, PadEvent, PadKind, PadReading};

    const NONE: Range = None;
    const STICK: Range = Some((-1.0, 1.0));
    const TRIG: Range = Some((0.0, 1.0));

    fn layout(set: &[(usize, Range)]) -> AxisLayout {
        let mut r = [NONE; AXES.len()];
        r[0] = STICK;
        r[1] = STICK;
        for (i, range) in set {
            r[*i] = *range;
        }
        AxisLayout::from_ranges(&r)
    }

    fn device(id: u32) -> PadDevice {
        PadDevice {
            id,
            name: Some("Pad".into()),
            kind: PadKind::Xbox,
            type_name: "android".into(),
            vendor: 0x045e,
            product: 0x0b13,
            battery: None,
        }
    }

    fn xbox_layout() -> AxisLayout {
        layout(&[
            (Z, STICK),
            (RZ, STICK),
            (LTRIGGER, TRIG),
            (RTRIGGER, TRIG),
            (BRAKE, TRIG),
            (GAS, TRIG),
        ])
    }

    #[test]
    fn the_axes_are_android_s_numbers_in_the_order_kotlin_sends() {
        assert_eq!(
            AXES,
            [
                AXIS_X,
                AXIS_Y,
                AXIS_Z,
                AXIS_RZ,
                AXIS_RX,
                AXIS_RY,
                AXIS_HAT_X,
                AXIS_HAT_Y,
                AXIS_LTRIGGER,
                AXIS_RTRIGGER,
                AXIS_BRAKE,
                AXIS_GAS
            ]
        );
        // MotionEvent's documented values.
        assert_eq!(
            AXES,
            [0, 1, 11, 14, 12, 13, 15, 16, 17, 18, 23, 22],
            "AXIS_X, Y, Z, RZ, RX, RY, HAT_X, HAT_Y, LTRIGGER, RTRIGGER, BRAKE, GAS"
        );
    }

    #[test]
    fn keys_are_read_by_position_like_sdl() {
        let b = |code| {
            key_target(code).map(|k| match k {
                Key::Button(b) => Some(b),
                Key::L2 | Key::R2 => None,
            })
        };
        // KEYCODE_BUTTON_A / B / X / Y: bottom, right, left, top.
        assert_eq!(b(96), Some(Some(PadButton::South)));
        assert_eq!(b(97), Some(Some(PadButton::East)));
        assert_eq!(b(99), Some(Some(PadButton::West)));
        assert_eq!(b(100), Some(Some(PadButton::North)));
        assert_eq!(b(102), Some(Some(PadButton::L1)));
        assert_eq!(b(103), Some(Some(PadButton::R1)));
        assert_eq!(b(106), Some(Some(PadButton::L3)));
        assert_eq!(b(107), Some(Some(PadButton::R3)));
        assert_eq!(b(108), Some(Some(PadButton::Start)));
        assert_eq!(b(109), Some(Some(PadButton::Back)));
        assert_eq!(b(110), Some(Some(PadButton::Guide)));
        // A pad's BACK / MENU keys: Select / Start.
        assert_eq!(b(4), Some(Some(PadButton::Back)));
        assert_eq!(b(82), Some(Some(PadButton::Start)));
        // KEYCODE_DPAD_*.
        assert_eq!(b(19), Some(Some(PadButton::Up)));
        assert_eq!(b(20), Some(Some(PadButton::Down)));
        assert_eq!(b(21), Some(Some(PadButton::Left)));
        assert_eq!(b(22), Some(Some(PadButton::Right)));
        assert_eq!(key_target(104), Some(Key::L2));
        assert_eq!(key_target(105), Some(Key::R2));
        // Left to the system: volume up / down, BUTTON_C / Z, BUTTON_1, DPAD_CENTER.
        for code in [24, 25, 98, 101, 188, 23] {
            assert_eq!(key_target(code), None, "{code}");
        }
    }

    #[test]
    fn every_position_but_the_triggers_has_a_key() {
        for button in PadButton::ALL {
            assert!(
                (0..300).any(|code| key_target(code) == Some(Key::Button(button))),
                "{button:?}"
            );
        }
    }

    #[test]
    fn an_xbox_pad_has_its_right_stick_on_z_rz_and_its_triggers_named() {
        assert_eq!(
            xbox_layout(),
            AxisLayout {
                right: Some((Z, RZ)),
                triggers: Some((LTRIGGER, RTRIGGER)),
            }
        );
    }

    #[test]
    fn brake_and_gas_are_the_triggers_when_ltrigger_rtrigger_are_missing() {
        let l = layout(&[(Z, STICK), (RZ, STICK), (BRAKE, TRIG), (GAS, TRIG)]);
        assert_eq!(l.triggers, Some((BRAKE, GAS)));
    }

    #[test]
    fn a_generic_pad_has_its_right_stick_on_rx_ry_and_its_triggers_on_z_rz() {
        let l = layout(&[(Z, TRIG), (RZ, TRIG), (RX, STICK), (RY, STICK)]);
        assert_eq!(
            l,
            AxisLayout {
                right: Some((RX, RY)),
                triggers: Some((Z, RZ)),
            }
        );
    }

    #[test]
    fn an_old_pad_with_triggers_on_rx_ry_keeps_z_rz_as_its_right_stick() {
        let l = layout(&[(Z, STICK), (RZ, STICK), (RX, TRIG), (RY, TRIG)]);
        assert_eq!(
            l,
            AxisLayout {
                right: Some((Z, RZ)),
                triggers: Some((RX, RY)),
            }
        );
    }

    #[test]
    fn a_pad_with_no_right_stick_or_analogue_triggers_has_neither() {
        assert_eq!(
            layout(&[]),
            AxisLayout {
                right: None,
                triggers: None,
            }
        );
    }

    #[test]
    fn ranges_skip_missing_and_broken_axes() {
        let mut flat = vec![f32::NAN; AXES.len() * 2];
        flat[0] = -1.0;
        flat[1] = 1.0;
        flat[2] = 1.0;
        flat[3] = 1.0; // max == min: not a range
        let r = ranges(&flat);
        assert_eq!(r[0], Some((-1.0, 1.0)));
        assert_eq!(r[1], None);
        assert!(r[2..].iter().all(Option::is_none));
        // A short array leaves the rest missing.
        assert!(ranges(&[-1.0, 1.0]).iter().skip(1).all(Option::is_none));
    }

    #[test]
    fn the_kind_comes_from_the_vendor_then_the_name() {
        assert_eq!(kind(0x045e, None), PadKind::Xbox);
        assert_eq!(
            kind(0x054c, Some("Wireless Controller")),
            PadKind::PlayStation
        );
        assert_eq!(kind(0x057e, Some("Pro Controller")), PadKind::Nintendo);
        assert_eq!(kind(0x28de, Some("Steam Controller")), PadKind::Xbox);
        assert_eq!(kind(0x2dc8, Some("8BitDo Ultimate Xbox")), PadKind::Xbox);
        assert_eq!(
            kind(0, Some("DualSense Wireless Controller")),
            PadKind::PlayStation
        );
        assert_eq!(kind(0x2dc8, Some("8BitDo Pro 2")), PadKind::Other);
        assert_eq!(kind(0, None), PadKind::Other);
    }

    #[test]
    fn battery_is_a_percentage_or_nothing() {
        assert_eq!(battery(80), Some(80));
        assert_eq!(battery(0), Some(0));
        assert_eq!(battery(-1), None);
        assert_eq!(battery(101), None);
    }

    #[test]
    fn a_pad_reads_its_keys_hat_sticks_and_triggers() {
        let mut pads = Pads::new();
        pads.add(device(7), xbox_layout(), false);
        assert!(pads.key(7, 96, true));
        assert!(pads.key(7, 110, true));
        let mut values = [0.0f32; AXES.len()];
        values[0] = 1.0; // left x
        values[1] = -1.0; // left y (up)
        values[2] = -0.5; // right x (Z)
        values[3] = 0.25; // right y (RZ)
        values[6] = -1.0; // hat left
        values[7] = 1.0; // hat down
        values[8] = 1.0; // ltrigger
        values[9] = -0.2; // rtrigger below its rest
        assert!(pads.motion(7, &values));
        let r = pads.read(7);
        assert_eq!(
            r,
            Some(PadReading {
                buttons: PadButton::South.bit()
                    | PadButton::Guide.bit()
                    | PadButton::Left.bit()
                    | PadButton::Down.bit(),
                axes: [32767, -32767, -16384, 8192, 32767, 0],
            })
        );
        assert!(pads.key(7, 96, false));
        assert_eq!(
            pads.read(7).map(|r| r.buttons & PadButton::South.bit()),
            Some(0)
        );
    }

    #[test]
    fn a_digital_trigger_reads_as_its_axis_fully_pressed() {
        let mut pads = Pads::new();
        pads.add(device(1), layout(&[]), false);
        assert!(pads.key(1, 104, true));
        assert_eq!(
            pads.read(1).map(|r| [r.axes[4], r.axes[5]]),
            Some([i16::MAX, 0])
        );
        assert!(pads.key(1, 105, true));
        assert!(pads.key(1, 104, false));
        assert_eq!(
            pads.read(1).map(|r| [r.axes[4], r.axes[5]]),
            Some([0, i16::MAX])
        );
    }

    #[test]
    fn unknown_pads_and_unread_keys_are_left_to_the_system() {
        let mut pads = Pads::new();
        assert!(!pads.key(3, 96, true));
        assert!(!pads.motion(3, &[0.0; 12]));
        pads.add(device(3), xbox_layout(), false);
        assert!(!pads.key(3, 24, true), "volume up");
        assert_eq!(pads.read(4), None);
    }

    #[test]
    fn connections_become_events_once() {
        let mut pads = Pads::new();
        pads.add(device(1), xbox_layout(), true);
        pads.add(device(2), xbox_layout(), false);
        assert!(pads.remove(1));
        assert!(!pads.remove(1));
        let mut events = Vec::new();
        pads.take_events(&mut events);
        assert_eq!(
            events,
            vec![
                PadEvent::Added {
                    device: device(1),
                    at_start: true
                },
                PadEvent::Added {
                    device: device(2),
                    at_start: false
                },
                PadEvent::Removed(1),
            ]
        );
        let mut again = Vec::new();
        pads.take_events(&mut again);
        assert!(again.is_empty());
        assert_eq!(pads.read(1), None);
        assert!(pads.read(2).is_some());
    }

    #[test]
    fn announcing_a_pad_again_replaces_it_and_drops_its_held_keys() {
        let mut pads = Pads::new();
        pads.add(device(5), xbox_layout(), false);
        assert!(pads.key(5, 97, true));
        pads.add(device(5), xbox_layout(), false);
        let mut events = Vec::new();
        pads.take_events(&mut events);
        assert_eq!(events.len(), 3, "added, removed, added: {events:?}");
        assert_eq!(pads.read(5), Some(PadReading::default()));
    }

    #[test]
    fn only_coarse_changes_are_logged() {
        let mut pads = Pads::new();
        pads.add(device(1), xbox_layout(), false);
        assert!(pads.coarse_change(1).is_some(), "the first reading");
        assert!(pads.coarse_change(1).is_none());
        let mut values = [0.0f32; AXES.len()];
        values[0] = 0.05;
        pads.motion(1, &values);
        assert!(pads.coarse_change(1).is_none(), "a tiny move");
        values[0] = 0.5;
        pads.motion(1, &values);
        assert!(pads.coarse_change(1).is_some());
        pads.key(1, 96, true);
        assert!(pads.coarse_change(1).is_some(), "a button");
        assert!(pads.coarse_change(9).is_none(), "unknown pad");
    }
}
