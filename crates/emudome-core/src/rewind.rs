//! Rewind (`speed.rew`, `speed.rewlen`): a ring buffer of save states taken while the game runs,
//! played back newest first while the rewind button is held.
//!
//! Nothing here touches the core: the session serializes into the buffer [`Rewind::record`] hands
//! it and loads what [`Rewind::step_back`] shows it, so it is unit-tested.
//!
//! Memory: a GBA state is a few hundred KB and a second holds 60 of them, so the buffer keeps
//! only the newest state whole and every older one as the difference from the next newer
//! ([`encode_delta`]: XOR, then runs of zero bytes skipped). Most of a console's memory does not
//! change between two frames, so a difference is a small fraction of a state. The oldest
//! difference is dropped first: it is never needed to rebuild a newer state.
//!
//! Cost on the emulation thread (Trello #79): the states are serialized into reused buffers, the
//! differences are encoded on a thread of their own ([`Rewind::with_encoder`]; equal stretches
//! compared eight bytes at a time), and stepping back turns the newest state into the older one
//! in place, with no copy of the state.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use parking_lot::{Condvar, Mutex};

use crate::settings::keys::{Reader, SPEED_REWIND, SPEED_REWIND_LENGTH};
use crate::settings::Value;

/// The seconds to keep from the resolved `speed.rew` and `speed.rewlen` (`read`: a resolved
/// value; missing or mistyped ones are the catalogue's defaults, a length out of the slider's
/// range is held to it): `None` when rewind is off.
pub fn seconds_from_settings(read: impl Fn(&str) -> Option<Value>) -> Option<u32> {
    let settings = Reader::new(read);
    if !settings.bool(SPEED_REWIND) {
        return None;
    }
    u32::try_from(settings.int(SPEED_REWIND_LENGTH)).ok()
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
    let mut out = Vec::new();
    encode_delta_into(older, newer, &mut out);
    out
}

/// The first index from `i` where `a` and `b` differ (`a.len()` when none): equal stretches are
/// compared eight bytes at a time.
fn next_difference(a: &[u8], b: &[u8], mut i: usize) -> usize {
    let len = a.len();
    while i + 8 <= len && a[i..i + 8] == b[i..i + 8] {
        i += 8;
    }
    while i < len && a[i] == b[i] {
        i += 1;
    }
    i
}

/// [`encode_delta`] into `out` (cleared first), reusing its memory.
pub fn encode_delta_into(older: &[u8], newer: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if older.len() != newer.len() {
        out.reserve(older.len() + 1);
        out.push(TAG_RAW);
        out.extend_from_slice(older);
        return;
    }
    out.push(TAG_DELTA);
    let len = older.len();
    let xor = |i: usize| older[i] ^ newer[i];
    let mut i = 0;
    while i < len {
        let zeros_from = i;
        // Most of a state does not change between two frames: skipped in u64 steps.
        i = next_difference(older, newer, i);
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
        write_varint(out, zeros);
        write_varint(out, lit_to - lit_from);
        out.extend(
            older[lit_from..lit_to]
                .iter()
                .zip(&newer[lit_from..lit_to])
                .map(|(a, b)| a ^ b),
        );
    }
}

/// Turns `state` (the newer state) into the older one, in place, with a difference made by
/// [`encode_delta`]. False when the difference is broken or was made against a state of another
/// size; `state` may then be half changed (the caller drops it).
fn apply_delta(delta: &[u8], state: &mut Vec<u8>) -> bool {
    let Some((&tag, body)) = delta.split_first() else {
        return false;
    };
    if tag == TAG_RAW {
        state.clear();
        state.extend_from_slice(body);
        return true;
    }
    if tag != TAG_DELTA {
        return false;
    }
    let mut at = 0usize;
    let mut pos = 0usize;
    while pos < body.len() {
        let (Some(zeros), Some(lits)) = (read_varint(body, &mut pos), read_varint(body, &mut pos))
        else {
            return false;
        };
        let Some(from) = at.checked_add(zeros) else {
            return false;
        };
        let (Some(src_end), Some(dst_end)) = (pos.checked_add(lits), from.checked_add(lits)) else {
            return false;
        };
        let (Some(src), Some(dst)) = (body.get(pos..src_end), state.get_mut(from..dst_end)) else {
            return false;
        };
        for (d, s) in dst.iter_mut().zip(src) {
            *d ^= s;
        }
        at = dst_end;
        pos = src_end;
    }
    true
}

/// The older state from a difference made by [`encode_delta`] and the newer state.
/// `None` when the difference is broken or was made against a state of another size.
pub fn decode_delta(delta: &[u8], newer: &[u8]) -> Option<Vec<u8>> {
    let mut out = newer.to_vec();
    apply_delta(delta, &mut out).then_some(out)
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
    /// Buffers of differences that were dropped, encoded into again.
    pool: Vec<Vec<u8>>,
}

/// Buffers [`RewindBuffer::pool`] keeps at most.
const POOL: usize = 2;

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
    /// Returns the state it replaces as the newest (now kept as a difference): a buffer to
    /// serialize the next state into.
    pub fn push(&mut self, state: Vec<u8>) -> Option<Vec<u8>> {
        let recycled = self.newest.take().inspect(|previous| {
            let mut delta = self.pool.pop().unwrap_or_default();
            encode_delta_into(previous, &state, &mut delta);
            // A reused buffer much larger than this difference would hold memory nobody counts.
            if delta.capacity() > delta.len() * 2 + 4096 {
                delta.shrink_to_fit();
            }
            self.delta_bytes += delta.len();
            self.deltas.push_back(delta);
        });
        self.newest = Some(state);
        while self.deltas.len() + 1 > self.capacity || self.bytes() > self.max_bytes {
            if !self.drop_oldest() {
                break;
            }
        }
        recycled
    }

    /// Drops the oldest difference (its buffer kept for the next encode); false when none.
    fn drop_oldest(&mut self) -> bool {
        let Some(old) = self.deltas.pop_front() else {
            return false;
        };
        self.delta_bytes -= old.len();
        if self.pool.len() < POOL {
            self.pool.push(old);
        }
        true
    }

    /// One step back: `load` sees the newest state, which then becomes the one before it, in
    /// place (no copy of the state). `None` when the buffer is empty.
    pub fn step_back<R>(&mut self, load: impl FnOnce(&[u8]) -> R) -> Option<R> {
        let newest = self.newest.as_mut()?;
        let loaded = load(newest);
        match self.deltas.pop_back() {
            Some(delta) => {
                self.delta_bytes -= delta.len();
                // A difference that does not rebuild (it cannot happen with `push`) ends the
                // buffer: what is older would be wrong too.
                if !apply_delta(&delta, newest) {
                    self.clear();
                }
                if self.pool.len() < POOL {
                    self.pool.push(delta);
                }
            }
            None => self.newest = None,
        }
        Some(loaded)
    }

    /// Takes the newest state out (a copy); the one before it becomes the newest.
    pub fn pop(&mut self) -> Option<Vec<u8>> {
        self.step_back(<[u8]>::to_vec)
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
            if !self.drop_oldest() {
                break;
            }
        }
    }
}

/// States handed to the encoder and not pushed yet, at most: beyond it a state is skipped
/// (the encoder is behind, a state comes a few frames later).
const MAX_PENDING: usize = 2;

/// The buffer when the differences are encoded on their own thread.
struct EncoderShared {
    buffer: Mutex<RewindBuffer>,
    /// Told after each state pushed.
    pushed: Condvar,
    /// States sent and not pushed yet.
    pending: AtomicUsize,
    /// The buffer's `len()` and `bytes()` after the last change: read every frame without the lock.
    states: AtomicUsize,
    bytes: AtomicUsize,
}

impl EncoderShared {
    fn publish(&self, buffer: &RewindBuffer) {
        self.states.store(buffer.len(), Ordering::Relaxed);
        self.bytes.store(buffer.bytes(), Ordering::Relaxed);
    }
}

/// The encoder thread and the way to it.
struct Encoder {
    shared: Arc<EncoderShared>,
    tx: Option<Sender<Vec<u8>>>,
    /// Buffers the encoder is done with, to serialize into again.
    spares: Receiver<Vec<u8>>,
    thread: Option<JoinHandle<()>>,
}

impl Encoder {
    fn spawn() -> std::io::Result<Self> {
        let shared = Arc::new(EncoderShared {
            buffer: Mutex::new(RewindBuffer::new(1, MAX_BYTES)),
            pushed: Condvar::new(),
            pending: AtomicUsize::new(0),
            states: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
        });
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let (spare_tx, spares) = mpsc::channel();
        let worker = shared.clone();
        let thread = std::thread::Builder::new()
            .name("emudome-rewind".into())
            .spawn(move || {
                for state in rx {
                    let mut buffer = worker.buffer.lock();
                    let spare = buffer.push(state);
                    worker.publish(&buffer);
                    worker.pending.fetch_sub(1, Ordering::Release);
                    worker.pushed.notify_all();
                    drop(buffer);
                    if let Some(spare) = spare {
                        let _ = spare_tx.send(spare);
                    }
                }
            })?;
        Ok(Self {
            shared,
            tx: Some(tx),
            spares,
            thread: Some(thread),
        })
    }

    /// The buffer once every state sent is in it.
    fn settled(&self) -> parking_lot::MutexGuard<'_, RewindBuffer> {
        let mut buffer = self.shared.buffer.lock();
        while self.shared.pending.load(Ordering::Acquire) > 0 {
            self.shared.pushed.wait(&mut buffer);
        }
        buffer
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        self.tx = None;
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::error!("the rewind encoder panicked");
            }
        }
    }
}

/// Where the states go: encoded right here (tests, or no thread could start), or on the
/// encoder thread.
enum Store {
    Inline(RewindBuffer),
    Encoder(Encoder),
}

/// Rewind for one session: when to take a state, and the buffer of them.
pub struct Rewind {
    store: Store,
    /// `speed.rewlen`; `None` = rewind off (`speed.rew`): nothing is recorded.
    seconds: Option<u32>,
    fps: f64,
    /// Frames between two states; set from the first state's size.
    interval: Option<u32>,
    /// Frames run since the last state.
    since: u32,
    /// The buffer the next state is serialized into.
    spare: Vec<u8>,
}

impl Rewind {
    /// `seconds`: `speed.rewlen`, or `None` when `speed.rew` is off. The differences are encoded
    /// on the calling thread.
    pub fn new(seconds: Option<u32>, fps: f64) -> Self {
        Self::with_store(Store::Inline(RewindBuffer::new(1, MAX_BYTES)), seconds, fps)
    }

    /// As [`Rewind::new`], the differences encoded on a thread of their own (the session's
    /// choice: the emulation thread only serializes). Encoded here if the thread cannot start.
    pub fn with_encoder(seconds: Option<u32>, fps: f64) -> Self {
        let store = match Encoder::spawn() {
            Ok(encoder) => Store::Encoder(encoder),
            Err(e) => {
                log::warn!("rewind encoder thread not started, encoding inline: {e}");
                Store::Inline(RewindBuffer::new(1, MAX_BYTES))
            }
        };
        Self::with_store(store, seconds, fps)
    }

    fn with_store(store: Store, seconds: Option<u32>, fps: f64) -> Self {
        Self {
            store,
            seconds,
            fps,
            interval: None,
            since: 0,
            spare: Vec::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.seconds.is_some()
    }

    /// Runs `f` on the buffer once every state sent is in it.
    fn with_buffer<R>(&mut self, f: impl FnOnce(&mut RewindBuffer) -> R) -> R {
        match &mut self.store {
            Store::Inline(buffer) => f(buffer),
            Store::Encoder(encoder) => {
                let mut buffer = encoder.settled();
                let r = f(&mut buffer);
                encoder.shared.publish(&buffer);
                r
            }
        }
    }

    /// `speed.rew` / `speed.rewlen` changed while the game runs. Off drops every state.
    pub fn set_seconds(&mut self, seconds: Option<u32>) {
        self.seconds = seconds;
        let fps = self.fps;
        match (seconds, self.interval) {
            (None, _) => self.with_buffer(RewindBuffer::clear),
            (Some(s), Some(interval)) => {
                self.with_buffer(|b| b.set_capacity(capacity(s, fps, interval)));
            }
            (Some(_), None) => {}
        }
    }

    /// The core changed its frame rate (`SET_SYSTEM_AV_INFO`): the buffer keeps covering
    /// `speed.rewlen` seconds at the new rate.
    pub fn set_fps(&mut self, fps: f64) {
        self.fps = fps;
        if let (Some(s), Some(interval)) = (self.seconds, self.interval) {
            self.with_buffer(|b| b.set_capacity(capacity(s, fps, interval)));
        }
    }

    /// After every frame run forwards: takes a state when one is due. `take` serializes into
    /// the buffer it is given (reused from older states) and is called only then; false from it
    /// (a core without states) skips the state.
    pub fn record(&mut self, take: impl FnOnce(&mut Vec<u8>) -> bool) {
        let Some(seconds) = self.seconds else {
            return;
        };
        self.since += 1;
        if self.since < self.interval.unwrap_or(1) {
            return;
        }
        if let Store::Encoder(encoder) = &self.store {
            // The encoder is behind: this state is skipped, the next frame tries again.
            if encoder.shared.pending.load(Ordering::Acquire) >= MAX_PENDING {
                return;
            }
            if let Ok(spare) = encoder.spares.try_recv() {
                self.spare = spare;
            }
        }
        self.since = 0;
        let mut state = std::mem::take(&mut self.spare);
        if !take(&mut state) {
            self.spare = state;
            return;
        }
        if self.interval.is_none() {
            let interval = interval_for(state.len());
            self.interval = Some(interval);
            let capacity = capacity(seconds, self.fps, interval);
            self.with_buffer(|b| b.set_capacity(capacity));
            log::info!(
                "rewind: {} bytes a state, one every {interval} frames, {capacity} states for {seconds} s",
                state.len()
            );
        }
        match &mut self.store {
            Store::Inline(buffer) => {
                if let Some(spare) = buffer.push(state) {
                    self.spare = spare;
                }
            }
            Store::Encoder(encoder) => {
                let Some(tx) = &encoder.tx else {
                    return;
                };
                encoder.shared.pending.fetch_add(1, Ordering::AcqRel);
                if tx.send(state).is_err() {
                    // The encoder is gone (it panicked): the state is lost, nothing waits for it.
                    encoder.shared.pending.fetch_sub(1, Ordering::AcqRel);
                }
            }
        }
    }

    /// One step back while rewinding: `load` sees the state to go back to (`None` when the
    /// buffer is empty: the game stays on its picture). The buffer then holds the one before it.
    pub fn step_back<R>(&mut self, load: impl FnOnce(&[u8]) -> R) -> Option<R> {
        // Recording starts again a full interval after the rewind ends.
        self.since = 0;
        self.with_buffer(|b| b.step_back(load))
    }

    /// Waits until every state taken is in the buffer (tests; the encoder's own pace otherwise).
    pub fn sync(&mut self) {
        self.with_buffer(|_| ());
    }

    /// States held and their bytes (Stats, the headless example's `--rewind`). Never waits for
    /// the encoder: what it published after its last push.
    pub fn usage(&self) -> (usize, usize) {
        match &self.store {
            Store::Inline(buffer) => (buffer.len(), buffer.bytes()),
            Store::Encoder(encoder) => (
                encoder.shared.states.load(Ordering::Relaxed),
                encoder.shared.bytes.load(Ordering::Relaxed),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        capacity, decode_delta, encode_delta, estimate_bytes, interval_for, seconds_from_settings,
        Rewind, RewindBuffer, INTERVAL, LARGE_STATE,
    };
    use crate::settings::Value;

    /// `seconds_from_settings` with `speed.rew` = `rew` and `speed.rewlen` = `len`.
    fn seconds(rew: Option<Value>, len: Option<Value>) -> Option<u32> {
        seconds_from_settings(|key| match key {
            "speed.rew" => rew.clone(),
            "speed.rewlen" => len.clone(),
            _ => None,
        })
    }

    #[test]
    fn the_seconds_come_from_the_settings() {
        // The catalogue's defaults: on, 30 s.
        assert_eq!(seconds(None, None), Some(30));
        assert_eq!(
            seconds(Some(Value::Bool(true)), Some(Value::Int(60))),
            Some(60)
        );
        assert_eq!(
            seconds(Some(Value::Bool(false)), Some(Value::Int(60))),
            None
        );
        // Out of the slider's range or of the wrong shape.
        assert_eq!(seconds(None, Some(Value::Int(-5))), Some(10));
        assert_eq!(seconds(None, Some(Value::Int(5000))), Some(120));
        assert_eq!(seconds(None, Some(Value::Text("x".into()))), Some(30));
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
            rw.record(|buf| {
                taken += 1;
                *buf = state(n, 1000);
                true
            });
        }
        // The first state at the first frame, then one every INTERVAL frames.
        assert_eq!(taken, 1 + (9 / INTERVAL) as usize);
        assert_eq!(rw.usage().0, taken);

        let mut off = Rewind::new(None, 60.0);
        off.record(|_| panic!("no state is taken with rewind off"));
        assert_eq!(off.usage(), (0, 0));
        assert!(!off.enabled());
    }

    /// `take` writing state `n` of `size` bytes into the buffer it is given.
    fn put(n: u32, size: usize) -> impl FnOnce(&mut Vec<u8>) -> bool {
        move |buf| {
            *buf = state(n, size);
            true
        }
    }

    #[test]
    fn rewind_holds_the_seconds_asked() {
        let mut rw = Rewind::new(Some(1), 60.0);
        for n in 0..1000 {
            rw.record(put(n, 1000));
        }
        // One second at 60 fps, a state every INTERVAL frames.
        assert_eq!(rw.usage().0, capacity(1, 60.0, INTERVAL));
        rw.set_seconds(None);
        assert_eq!(rw.usage(), (0, 0));
        rw.record(|_| panic!("off"));
        rw.set_seconds(Some(2));
        for _ in 0..INTERVAL {
            rw.record(put(0, 1000));
        }
        assert_eq!(rw.usage().0, 1);
    }

    #[test]
    fn stepping_back_restarts_the_interval() {
        let mut rw = Rewind::new(Some(10), 60.0);
        for n in 0..9 {
            rw.record(put(n, 100));
        }
        let held = rw.usage().0;
        assert_eq!(rw.step_back(<[u8]>::to_vec), Some(state(8, 100)));
        // Back to playing: the next state comes a full interval later.
        rw.record(|_| panic!("too early"));
        rw.record(put(99, 100));
        assert_eq!(rw.usage().0, held);
        let mut empty = Rewind::new(Some(10), 60.0);
        assert_eq!(empty.step_back(<[u8]>::to_vec), None);
    }

    /// The buffers come back: after the first few states, `take` is handed one that already
    /// holds a state of the right size (serializing into it allocates nothing).
    #[test]
    fn states_are_serialized_into_reused_buffers() {
        let mut rw = Rewind::new(Some(10), 60.0);
        let mut reused = 0;
        for n in 0..20 {
            for _ in 0..INTERVAL {
                rw.record(|buf| {
                    reused += usize::from(buf.capacity() >= 1000);
                    let s = state(n, 1000);
                    buf.clear();
                    buf.extend_from_slice(&s);
                    true
                });
            }
        }
        assert!(reused >= 18, "{reused} of 20 reused");
    }

    /// The encoder thread gives the same buffer as encoding inline, and stepping back waits for
    /// the states still on their way.
    #[test]
    fn the_encoder_thread_keeps_the_same_states() {
        let mut inline = Rewind::new(Some(2), 60.0);
        let mut threaded = Rewind::with_encoder(Some(2), 60.0);
        for n in 0..300 {
            inline.record(put(n, 4000));
            threaded.record(put(n, 4000));
            // Without waiting, the encoder may skip a state when it is behind: a sync per frame
            // keeps the two in step for the comparison.
            threaded.sync();
        }
        assert_eq!(threaded.usage(), inline.usage());
        loop {
            let a = inline.step_back(<[u8]>::to_vec);
            let b = threaded.step_back(<[u8]>::to_vec);
            assert_eq!(a, b);
            if a.is_none() {
                break;
            }
        }
        assert_eq!(threaded.usage(), (0, 0));
    }

    /// Unsynced, the encoder never holds more than its few pending states and every state it
    /// kept rebuilds.
    #[test]
    fn the_encoder_thread_under_load_stays_consistent() {
        let mut rw = Rewind::with_encoder(Some(1), 60.0);
        for n in 0..2000 {
            rw.record(put(n, 20_000));
        }
        let (states, _) = {
            rw.sync();
            rw.usage()
        };
        assert!(states > 0);
        let mut back = 0;
        let mut last = None;
        while let Some(s) = rw.step_back(<[u8]>::to_vec) {
            // Newest first: each state is an older frame than the one before.
            let n = u32::from_le_bytes([s[0], s[97], s[194], s[291]]);
            if let Some(prev) = last {
                assert!(n < prev, "{n} after {prev}");
            }
            last = Some(n);
            back += 1;
        }
        assert_eq!(back, states);
    }
}
