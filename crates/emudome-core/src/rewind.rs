//! Rewind (`speed.rew`, `speed.rewlen`): a ring buffer of save states taken while the game runs,
//! played back newest first while the rewind button is held.
//!
//! Pure: the session hands in the states ([`Rewind::record`]) and loads what
//! [`Rewind::step_back`] gives back; nothing here touches the core, so it is unit-tested.
//!
//! Memory: a GBA state is a few hundred KB and a second holds 60 of them, so the buffer keeps
//! only the newest state whole and every older one as the difference from the next newer
//! ([`encode_delta`]: XOR, then runs of zero bytes skipped). Most of a console's memory does not
//! change between two frames, so a difference is a small fraction of a state. The oldest
//! difference is dropped first: it is never needed to rebuild a newer state.

use std::collections::VecDeque;

use crate::settings::Value;

/// `speed.rewlen`'s default and range (catalog.rs), for a value of the wrong shape.
const DEFAULT_SECONDS: u32 = 30;
const SECONDS_RANGE: (u32, u32) = (10, 120);

/// The seconds to keep from the resolved `speed.rew` and `speed.rewlen` (`None` = the schema
/// default): `None` when rewind is off.
pub fn seconds_from_settings(rew: Option<Value>, rewlen: Option<Value>) -> Option<u32> {
    match rew {
        Some(Value::Bool(false)) => return None,
        Some(Value::Bool(true) | Value::Int(_) | Value::Text(_) | Value::List(_)) | None => {}
    }
    Some(match rewlen {
        Some(Value::Int(v)) => u32::try_from(v)
            .unwrap_or(DEFAULT_SECONDS)
            .clamp(SECONDS_RANGE.0, SECONDS_RANGE.1),
        Some(Value::Bool(_) | Value::Text(_) | Value::List(_)) | None => DEFAULT_SECONDS,
    })
}

/// A state is taken every this many emulated frames (more for very large states, see
/// [`interval_for`]). Played back one state per frame, rewind runs this many times faster
/// than the game, minus the one frame run after each load to show a picture.
pub const INTERVAL: u32 = 2;

/// States larger than this are taken less often ([`interval_for`]): the 3DS («Sperimentale»).
pub const LARGE_STATE: usize = 8 * 1024 * 1024;

/// Whatever `speed.rewlen` asks, the buffer never holds more than this.
pub const MAX_BYTES: usize = 512 * 1024 * 1024;

/// How many frames apart the states are taken, from the size of one state: [`INTERVAL`], or
/// one more frame per [`LARGE_STATE`] for the very large states, up to a state a second.
pub fn interval_for(state_size: usize) -> u32 {
    let extra = u32::try_from(state_size / LARGE_STATE).unwrap_or(u32::MAX);
    INTERVAL
        .saturating_add(extra.saturating_mul(INTERVAL))
        .min(60)
}

/// How many states cover `seconds` of play at `fps`, one every `interval` frames (at least 1).
pub fn capacity(seconds: u32, fps: f64, interval: u32) -> usize {
    let fps = if fps.is_finite() && fps > 0.0 {
        fps
    } else {
        60.0
    };
    let states = (f64::from(seconds) * fps / f64::from(interval.max(1))).ceil();
    // `as` saturates on a float too large; the cap keeps it sane anyway.
    (states as usize).clamp(1, 1 << 20)
}

/// Rough memory of a full buffer: the newest state whole, the others as differences of
/// `delta_size` bytes each (measured: see the report of Trello #42 and the headless example's
/// `--rewind`).
pub fn estimate_bytes(capacity: usize, state_size: usize, delta_size: usize) -> usize {
    state_size.saturating_add(capacity.saturating_sub(1).saturating_mul(delta_size))
}

/// Tag of an encoded difference: the bytes follow as runs (see [`encode_delta`]).
const TAG_DELTA: u8 = 0;
/// Tag of an entry kept whole: the two states have different sizes.
const TAG_RAW: u8 = 1;

/// A zero run shorter than this stays inside the literal bytes (a run costs two varints).
const MIN_ZERO_RUN: usize = 8;

/// `older` as a difference from `newer`, so that `decode_delta(&delta, newer) == older`.
///
/// Format: a tag byte, then pairs of (zero bytes skipped, literal count) as LEB128 varints,
/// each pair followed by its literal bytes (`older ^ newer`). When the two have different
/// sizes the entry is `older` whole.
pub fn encode_delta(older: &[u8], newer: &[u8]) -> Vec<u8> {
    if older.len() != newer.len() {
        let mut out = Vec::with_capacity(older.len() + 1);
        out.push(TAG_RAW);
        out.extend_from_slice(older);
        return out;
    }
    let mut out = vec![TAG_DELTA];
    let len = older.len();
    let xor = |i: usize| older[i] ^ newer[i];
    let mut i = 0;
    while i < len {
        let zeros_from = i;
        while i < len && xor(i) == 0 {
            i += 1;
        }
        let zeros = i - zeros_from;
        if i == len {
            break;
        }
        // Literal bytes until a zero run long enough to be worth a pair.
        let lit_from = i;
        let mut run = 0;
        while i < len {
            if xor(i) == 0 {
                run += 1;
                if run >= MIN_ZERO_RUN {
                    break;
                }
            } else {
                run = 0;
            }
            i += 1;
        }
        // The zero bytes that ended the literal belong to the next skip.
        let lit_to = if run >= MIN_ZERO_RUN { i + 1 - run } else { i };
        i = lit_to;
        write_varint(&mut out, zeros);
        write_varint(&mut out, lit_to - lit_from);
        out.extend((lit_from..lit_to).map(xor));
    }
    out
}

/// The older state from a difference made by [`encode_delta`] and the newer state.
/// `None` when the difference is broken or was made against a state of another size.
pub fn decode_delta(delta: &[u8], newer: &[u8]) -> Option<Vec<u8>> {
    let (&tag, body) = delta.split_first()?;
    if tag == TAG_RAW {
        return Some(body.to_vec());
    }
    if tag != TAG_DELTA {
        return None;
    }
    let mut out = newer.to_vec();
    let mut at = 0usize;
    let mut pos = 0usize;
    while pos < body.len() {
        let zeros = read_varint(body, &mut pos)?;
        let lits = read_varint(body, &mut pos)?;
        at = at.checked_add(zeros)?;
        let src = body.get(pos..pos.checked_add(lits)?)?;
        let dst = out.get_mut(at..at.checked_add(lits)?)?;
        for (d, s) in dst.iter_mut().zip(src) {
            *d ^= s;
        }
        at += lits;
        pos += lits;
    }
    Some(out)
}

fn write_varint(out: &mut Vec<u8>, mut v: usize) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn read_varint(data: &[u8], pos: &mut usize) -> Option<usize> {
    let mut v = 0usize;
    let mut shift = 0u32;
    loop {
        let byte = *data.get(*pos)?;
        *pos += 1;
        v |= usize::from(byte & 0x7f).checked_shl(shift)?;
        if byte & 0x80 == 0 {
            return Some(v);
        }
        shift += 7;
        if shift >= usize::BITS {
            return None;
        }
    }
}

/// The states of the last seconds, newest last.
#[derive(Debug, Default)]
pub struct RewindBuffer {
    /// Older states, oldest first: entry `i` rebuilds state `i` from state `i + 1`
    /// (the last one from `newest`).
    deltas: VecDeque<Vec<u8>>,
    newest: Option<Vec<u8>>,
    /// States kept at most, `newest` included.
    capacity: usize,
    max_bytes: usize,
    /// Bytes held by `deltas`.
    delta_bytes: usize,
}

impl RewindBuffer {
    pub fn new(capacity: usize, max_bytes: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            max_bytes,
            ..Self::default()
        }
    }

    /// States held, the newest included.
    pub fn len(&self) -> usize {
        self.deltas.len() + usize::from(self.newest.is_some())
    }

    pub fn is_empty(&self) -> bool {
        self.newest.is_none()
    }

    /// Memory held by the states (their bytes, not the allocator's overhead).
    pub fn bytes(&self) -> usize {
        self.delta_bytes + self.newest.as_ref().map_or(0, Vec::len)
    }

    /// Adds the newest state; the oldest go when the buffer is over its capacity or its bytes.
    pub fn push(&mut self, state: Vec<u8>) {
        if let Some(previous) = self.newest.take() {
            let delta = encode_delta(&previous, &state);
            self.delta_bytes += delta.len();
            self.deltas.push_back(delta);
        }
        self.newest = Some(state);
        while self.deltas.len() + 1 > self.capacity || self.bytes() > self.max_bytes {
            match self.deltas.pop_front() {
                Some(old) => self.delta_bytes -= old.len(),
                None => break,
            }
        }
    }

    /// Takes the newest state out; the one before it becomes the newest.
    pub fn pop(&mut self) -> Option<Vec<u8>> {
        let newest = self.newest.take()?;
        if let Some(delta) = self.deltas.pop_back() {
            self.delta_bytes -= delta.len();
            // A difference that does not rebuild (it cannot happen with `push`) ends the
            // buffer: what is older would be wrong too.
            match decode_delta(&delta, &newest) {
                Some(older) => self.newest = Some(older),
                None => self.clear(),
            }
        }
        Some(newest)
    }

    pub fn clear(&mut self) {
        self.deltas.clear();
        self.newest = None;
        self.delta_bytes = 0;
    }

    /// A new capacity (`speed.rewlen` changed): the oldest states go if it is smaller.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        while self.deltas.len() + 1 > self.capacity {
            match self.deltas.pop_front() {
                Some(old) => self.delta_bytes -= old.len(),
                None => break,
            }
        }
    }
}

/// Rewind for one session: when to take a state, and the buffer of them.
#[derive(Debug)]
pub struct Rewind {
    buffer: RewindBuffer,
    /// `speed.rewlen`; `None` = rewind off (`speed.rew`): nothing is recorded.
    seconds: Option<u32>,
    fps: f64,
    /// Frames between two states; set from the first state's size.
    interval: Option<u32>,
    /// Frames run since the last state.
    since: u32,
}

impl Rewind {
    /// `seconds`: `speed.rewlen`, or `None` when `speed.rew` is off.
    pub fn new(seconds: Option<u32>, fps: f64) -> Self {
        Self {
            buffer: RewindBuffer::new(1, MAX_BYTES),
            seconds,
            fps,
            interval: None,
            since: 0,
        }
    }

    pub fn enabled(&self) -> bool {
        self.seconds.is_some()
    }

    /// `speed.rew` / `speed.rewlen` changed while the game runs. Off drops every state.
    pub fn set_seconds(&mut self, seconds: Option<u32>) {
        self.seconds = seconds;
        match (seconds, self.interval) {
            (None, _) => self.buffer.clear(),
            (Some(s), Some(interval)) => self.buffer.set_capacity(capacity(s, self.fps, interval)),
            (Some(_), None) => {}
        }
    }

    /// After every frame run forwards: takes a state when one is due. `take` is called only
    /// then; `None` from it (a core without states) is skipped.
    pub fn record(&mut self, take: impl FnOnce() -> Option<Vec<u8>>) {
        let Some(seconds) = self.seconds else {
            return;
        };
        self.since += 1;
        if self.since < self.interval.unwrap_or(1) {
            return;
        }
        self.since = 0;
        let Some(state) = take() else {
            return;
        };
        if self.interval.is_none() {
            let interval = interval_for(state.len());
            self.interval = Some(interval);
            let capacity = capacity(seconds, self.fps, interval);
            self.buffer.set_capacity(capacity);
            log::info!(
                "rewind: {} bytes a state, one every {interval} frames, {capacity} states for {seconds} s",
                state.len()
            );
        }
        self.buffer.push(state);
    }

    /// One step back while rewinding: the state to load, `None` when the buffer is empty
    /// (the game stays on its picture).
    pub fn step_back(&mut self) -> Option<Vec<u8>> {
        // Recording starts again a full interval after the rewind ends.
        self.since = 0;
        self.buffer.pop()
    }

    /// States held and their bytes (Stats, the headless example's `--rewind`).
    pub fn usage(&self) -> (usize, usize) {
        (self.buffer.len(), self.buffer.bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        capacity, decode_delta, encode_delta, estimate_bytes, interval_for, seconds_from_settings,
        Rewind, RewindBuffer, INTERVAL, LARGE_STATE,
    };
    use crate::settings::Value;

    #[test]
    fn the_seconds_come_from_the_settings() {
        assert_eq!(seconds_from_settings(None, None), Some(30));
        assert_eq!(
            seconds_from_settings(Some(Value::Bool(true)), Some(Value::Int(60))),
            Some(60)
        );
        assert_eq!(
            seconds_from_settings(Some(Value::Bool(false)), Some(Value::Int(60))),
            None
        );
        // Out of the slider's range or of the wrong shape.
        assert_eq!(seconds_from_settings(None, Some(Value::Int(-5))), Some(30));
        assert_eq!(
            seconds_from_settings(None, Some(Value::Int(5000))),
            Some(120)
        );
        assert_eq!(
            seconds_from_settings(None, Some(Value::Text("x".into()))),
            Some(30)
        );
    }

    /// A fake state: mostly constant memory with a few bytes that change with `n`.
    fn state(n: u32, size: usize) -> Vec<u8> {
        let mut s: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        for (k, b) in n.to_le_bytes().iter().enumerate() {
            s[k * 97 % size] = *b;
            s[size - 1 - k] ^= *b;
        }
        s
    }

    #[test]
    fn capacity_covers_the_seconds() {
        assert_eq!(capacity(30, 60.0, 2), 900);
        assert_eq!(capacity(10, 59.7275, 2), 299);
        assert_eq!(capacity(120, 60.0, 1), 7200);
        // Never empty, and a broken fps falls back to 60.
        assert_eq!(capacity(0, 60.0, 2), 1);
        assert_eq!(capacity(1, f64::NAN, 1), 60);
        assert_eq!(capacity(1, 60.0, 0), 60);
    }

    #[test]
    fn large_states_are_taken_less_often() {
        assert_eq!(interval_for(400 * 1024), INTERVAL);
        assert_eq!(interval_for(LARGE_STATE - 1), INTERVAL);
        assert_eq!(interval_for(LARGE_STATE), INTERVAL * 2);
        assert_eq!(interval_for(usize::MAX), 60);
    }

    #[test]
    fn estimate_counts_one_state_whole() {
        assert_eq!(estimate_bytes(1, 1000, 10), 1000);
        assert_eq!(estimate_bytes(900, 1000, 10), 1000 + 899 * 10);
        assert_eq!(estimate_bytes(0, 1000, 10), 1000);
    }

    #[test]
    fn a_delta_rebuilds_the_older_state() {
        let newer = state(7, 5000);
        for older in [
            state(6, 5000),
            newer.clone(),
            vec![0; 5000],
            vec![0xff; 5000],
        ] {
            let delta = encode_delta(&older, &newer);
            assert_eq!(decode_delta(&delta, &newer).as_deref(), Some(&older[..]));
        }
        // Few bytes changed: a few bytes of difference.
        assert!(encode_delta(&state(6, 5000), &newer).len() < 64);
        // Another size: kept whole.
        let small = state(1, 100);
        let delta = encode_delta(&small, &newer);
        assert_eq!(delta.len(), 101);
        assert_eq!(decode_delta(&delta, &newer), Some(small));
        // Long varints (a run over 127 bytes) and empty states.
        let mut far = newer.clone();
        far[4000] ^= 1;
        assert_eq!(decode_delta(&encode_delta(&far, &newer), &newer), Some(far));
        assert_eq!(decode_delta(&encode_delta(&[], &[]), &[]), Some(vec![]));
    }

    #[test]
    fn a_broken_delta_is_refused() {
        let newer = state(2, 100);
        let delta = encode_delta(&state(1, 100), &newer);
        assert_eq!(decode_delta(&[], &newer), None);
        assert_eq!(decode_delta(&[9], &newer), None);
        // Against a shorter state the literals fall outside it.
        assert_eq!(decode_delta(&delta, &newer[..10]), None);
        assert_eq!(decode_delta(&delta[..delta.len() - 1], &newer), None);
    }

    #[test]
    fn the_buffer_gives_states_back_newest_first() {
        let mut buf = RewindBuffer::new(4, usize::MAX);
        for n in 0..6 {
            buf.push(state(n, 1000));
        }
        // Capacity 4: states 2..=5 are left.
        assert_eq!(buf.len(), 4);
        for n in (2..6).rev() {
            assert_eq!(buf.pop(), Some(state(n, 1000)));
        }
        assert_eq!(buf.pop(), None);
        assert!(buf.is_empty());
        assert_eq!(buf.bytes(), 0);
    }

    #[test]
    fn the_buffer_keeps_older_states_as_small_deltas() {
        let mut buf = RewindBuffer::new(100, usize::MAX);
        for n in 0..100 {
            buf.push(state(n, 10_000));
        }
        assert_eq!(buf.len(), 100);
        assert!(buf.bytes() < 10_000 + 99 * 64, "{} bytes", buf.bytes());
    }

    #[test]
    fn the_buffer_stays_under_its_bytes() {
        let mut buf = RewindBuffer::new(1000, 3000);
        for n in 0..50 {
            // Unrelated states: every delta is as large as a state.
            buf.push(vec![n as u8; 1000]);
        }
        assert!(buf.bytes() <= 3000);
        assert_eq!(buf.pop(), Some(vec![49; 1000]));
    }

    #[test]
    fn a_smaller_capacity_drops_the_oldest() {
        let mut buf = RewindBuffer::new(10, usize::MAX);
        for n in 0..10 {
            buf.push(state(n, 500));
        }
        buf.set_capacity(3);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.pop(), Some(state(9, 500)));
        assert_eq!(buf.pop(), Some(state(8, 500)));
        assert_eq!(buf.pop(), Some(state(7, 500)));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn states_are_taken_every_interval_and_only_when_on() {
        let mut taken = 0;
        let mut rw = Rewind::new(Some(1), 60.0);
        for n in 0..10 {
            rw.record(|| {
                taken += 1;
                Some(state(n, 1000))
            });
        }
        // The first state at the first frame, then one every INTERVAL frames.
        assert_eq!(taken, 1 + (9 / INTERVAL) as usize);
        assert_eq!(rw.usage().0, taken);

        let mut off = Rewind::new(None, 60.0);
        off.record(|| panic!("no state is taken with rewind off"));
        assert_eq!(off.usage(), (0, 0));
        assert!(!off.enabled());
    }

    #[test]
    fn rewind_holds_the_seconds_asked() {
        let mut rw = Rewind::new(Some(1), 60.0);
        for n in 0..1000 {
            rw.record(|| Some(state(n, 1000)));
        }
        // One second at 60 fps, a state every INTERVAL frames.
        assert_eq!(rw.usage().0, capacity(1, 60.0, INTERVAL));
        rw.set_seconds(None);
        assert_eq!(rw.usage(), (0, 0));
        rw.record(|| panic!("off"));
        rw.set_seconds(Some(2));
        for _ in 0..INTERVAL {
            rw.record(|| Some(state(0, 1000)));
        }
        assert_eq!(rw.usage().0, 1);
    }

    #[test]
    fn stepping_back_restarts_the_interval() {
        let mut rw = Rewind::new(Some(10), 60.0);
        for n in 0..9 {
            rw.record(|| Some(state(n, 100)));
        }
        let held = rw.usage().0;
        assert_eq!(rw.step_back(), Some(state(8, 100)));
        // Back to playing: the next state comes a full interval later.
        rw.record(|| panic!("too early"));
        rw.record(|| Some(state(99, 100)));
        assert_eq!(rw.usage().0, held);
        let mut empty = Rewind::new(Some(10), 60.0);
        assert_eq!(empty.step_back(), None);
    }
}
