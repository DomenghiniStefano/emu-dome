//! Audio between the emulation thread (producer) and the audio output (consumer).
//!
//! A lock-free single-producer single-consumer ring ([`rtrb`]) of interleaved stereo samples at
//! the core's rate: the emulation thread pushes with its [`AudioProducer`] (held by the libretro
//! callbacks' context), the audio output's real-time callback reads with the [`AudioReader`].
//! Neither ever waits for the other: the slow work (`speed_audio`'s time-stretch) runs on the
//! emulation thread before the push, and every other shared value (rate, clear requests, the
//! bench's test tone, the dropped count) is an atomic.
//!
//! Latency stays bounded as the old queue kept it: the reader drops what is queued beyond
//! about 150 ms ([`AudioRing::max_len`]) before it plays, oldest first; a full ring (no output
//! reading, a stalled device) drops the newest samples instead. Both are counted.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

use parking_lot::{Mutex, MutexGuard};
use rtrb::RingBuffer;

/// Frequency of the bench's test tone.
pub const TEST_TONE_HZ: f64 = 440.0;

/// Samples (interleaved stereo `i16`) the ring holds: about 1 s at 32 kHz, 0.68 s at 48 kHz.
/// The reader keeps far less queued ([`AudioRing::max_len`]); the rest is headroom for a frame of
/// a fast-forwarded core and for an output that starts late.
pub const RING_SAMPLES: usize = 1 << 16;

/// The audio of one session (in [`crate::Shared`]).
pub struct AudioRing {
    /// The core's sample rate (`f64` bits).
    rate: AtomicU64,
    /// Most samples (stereo) kept queued: beyond it the reader drops the oldest.
    max_len: AtomicUsize,
    /// Bumped to ask the reader to drop everything queued (rewind, fast forward muted).
    clear: AtomicU64,
    /// Samples dropped so far: queued beyond `max_len`, or pushed into a full ring.
    dropped: AtomicU64,
    /// The bench's test tone amplitude (`f32` bits; 0 = the game's audio).
    tone: AtomicU32,
    /// Handed once to the libretro callbacks' context ([`AudioRing::take_producer`]).
    producer: Mutex<Option<AudioProducer>>,
    /// Locked only by the audio output (with `try_lock`): never contended by the emulation thread.
    reader: Mutex<AudioReader>,
}

impl Default for AudioRing {
    fn default() -> Self {
        Self::with_capacity(RING_SAMPLES)
    }
}

impl AudioRing {
    pub fn with_capacity(samples: usize) -> Self {
        let (producer, consumer) = RingBuffer::new(samples);
        let ring = Self {
            rate: AtomicU64::new(0),
            max_len: AtomicUsize::new(0),
            clear: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            tone: AtomicU32::new(0),
            producer: Mutex::new(Some(AudioProducer {
                producer,
                phase: 0.0,
            })),
            reader: Mutex::new(AudioReader {
                consumer,
                seen_clear: 0,
            }),
        };
        ring.set_rate(32768.0);
        ring
    }

    /// The core's sample rate.
    pub fn sample_rate(&self) -> f64 {
        f64::from_bits(self.rate.load(Ordering::Relaxed))
    }

    pub fn set_rate(&self, rate: f64) {
        self.rate.store(rate.to_bits(), Ordering::Relaxed);
        // ~150 ms of headroom.
        self.max_len
            .store(((rate * 0.15) as usize) * 2, Ordering::Relaxed);
    }

    /// Most samples (stereo, interleaved) kept queued.
    pub fn max_len(&self) -> usize {
        self.max_len.load(Ordering::Relaxed)
    }

    /// Samples dropped so far (queued beyond [`Self::max_len`], or pushed into a full ring).
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Asks the reader to drop everything queued now (seen at its next callback).
    pub fn clear(&self) {
        self.clear.fetch_add(1, Ordering::Relaxed);
    }

    /// Replaces the game's audio with a [`TEST_TONE_HZ`] sine of this amplitude (0..1), or gives
    /// the game's audio back with `None`.
    pub fn set_test_tone(&self, amplitude: Option<f32>) {
        let a = amplitude.map_or(0.0, |a| a.clamp(0.0, 1.0));
        self.tone.store(a.to_bits(), Ordering::Relaxed);
    }

    /// The producer, for the libretro callbacks' context. `None` once taken.
    pub fn take_producer(&self) -> Option<AudioProducer> {
        self.producer.lock().take()
    }

    /// The reader, for the audio output's callback: `None` while something else holds it (never
    /// the emulation thread), so the callback never waits.
    pub fn reader(&self) -> Option<MutexGuard<'_, AudioReader>> {
        self.reader.try_lock()
    }
}

/// The emulation thread's end of the ring.
pub struct AudioProducer {
    producer: rtrb::Producer<i16>,
    /// The test tone's phase.
    phase: f64,
}

impl AudioProducer {
    /// A producer to a ring nobody reads (a context made after the session's producer was taken).
    pub fn detached() -> Self {
        let (producer, _) = RingBuffer::new(2);
        Self {
            producer,
            phase: 0.0,
        }
    }

    /// Room left in the ring, in samples.
    pub fn free(&self) -> usize {
        self.producer.slots()
    }

    /// Queues interleaved stereo samples; what does not fit in the ring is dropped and counted.
    pub fn push(&mut self, ring: &AudioRing, data: &[i16]) {
        let free = self.producer.slots() & !1;
        let n = data.len().min(free) & !1;
        let lost = data.len() - n;
        if lost > 0 {
            ring.dropped.fetch_add(lost as u64, Ordering::Relaxed);
        }
        if n == 0 {
            return;
        }
        let Ok(chunk) = self.producer.write_chunk_uninit(n) else {
            return;
        };
        let amplitude = f64::from(f32::from_bits(ring.tone.load(Ordering::Relaxed)));
        if amplitude > 0.0 {
            // The same count of samples, at the same time, as the game's audio would be.
            let step = std::f64::consts::TAU * TEST_TONE_HZ / ring.sample_rate();
            let amplitude = amplitude * f64::from(i16::MAX);
            let phase = &mut self.phase;
            chunk.fill_from_iter((0..n / 2).flat_map(|_| {
                let s = (phase.sin() * amplitude) as i16;
                *phase = (*phase + step) % std::f64::consts::TAU;
                [s, s]
            }));
        } else {
            chunk.fill_from_iter(data[..n].iter().copied());
        }
    }
}

/// The audio output's end of the ring.
pub struct AudioReader {
    consumer: rtrb::Consumer<i16>,
    /// The last clear request carried out.
    seen_clear: u64,
}

impl AudioReader {
    /// At the start of a callback: carries out a clear request and drops what is queued beyond
    /// the ring's `max_len` (oldest first). Returns the samples queued after it.
    pub fn begin(&mut self, ring: &AudioRing) -> usize {
        let clear = ring.clear.load(Ordering::Relaxed);
        if clear != self.seen_clear {
            self.seen_clear = clear;
            self.drain();
        }
        let queued = self.consumer.slots();
        let excess = queued.saturating_sub(ring.max_len());
        let excess = excess + (excess & 1);
        if excess > 0 {
            let skipped = self.skip(excess);
            ring.dropped.fetch_add(skipped as u64, Ordering::Relaxed);
        }
        self.consumer.slots()
    }

    /// The next left / right pair, `None` when the ring is empty.
    pub fn pop_frame(&mut self) -> Option<[i16; 2]> {
        let chunk = self.consumer.read_chunk(2).ok()?;
        let (a, b) = chunk.as_slices();
        let pair = match (a, b) {
            ([l, r, ..], _) => [*l, *r],
            ([l], [r, ..]) => [*l, *r],
            _ => return None,
        };
        chunk.commit_all();
        Some(pair)
    }

    /// Drops everything queued; returns how many samples went.
    pub fn drain(&mut self) -> usize {
        self.skip(self.consumer.slots())
    }

    fn skip(&mut self, n: usize) -> usize {
        match self.consumer.read_chunk(n.min(self.consumer.slots())) {
            Ok(chunk) => {
                let len = chunk.len();
                chunk.commit_all();
                len
            }
            Err(_) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_all(ring: &AudioRing) -> Vec<i16> {
        let mut reader = ring.reader().unwrap();
        reader.begin(ring);
        let mut out = Vec::new();
        while let Some([l, r]) = reader.pop_frame() {
            out.extend([l, r]);
        }
        out
    }

    #[test]
    fn the_tone_keeps_the_sample_count_and_its_amplitude() {
        let ring = AudioRing::default();
        ring.set_rate(32768.0);
        ring.set_test_tone(Some(0.05));
        let mut producer = ring.take_producer().unwrap();
        producer.push(&ring, &[1000; 1094]);
        let samples = read_all(&ring);
        assert_eq!(samples.len(), 1094);
        let peak = samples.iter().map(|s| s.unsigned_abs()).max().unwrap();
        assert!((1630..=1639).contains(&peak), "peak {peak}");
        // Left and right are the same signal.
        assert!(samples
            .iter()
            .step_by(2)
            .eq(samples.iter().skip(1).step_by(2)));
    }

    #[test]
    fn what_is_queued_beyond_the_limit_is_dropped_oldest_first_and_counted() {
        let ring = AudioRing::default();
        ring.set_rate(1000.0); // max_len = 150 * 2
        let mut producer = ring.take_producer().unwrap();
        let data: Vec<i16> = (0..400).collect();
        producer.push(&ring, &data);
        let samples = read_all(&ring);
        assert_eq!(samples.len(), 300);
        assert_eq!(samples[0], 100, "the oldest went");
        assert_eq!(ring.dropped(), 100);
    }

    #[test]
    fn a_full_ring_drops_the_newest_and_counts_them() {
        let ring = AudioRing::with_capacity(8);
        let mut producer = ring.take_producer().unwrap();
        producer.push(&ring, &[1, 2, 3, 4, 5, 6]);
        producer.push(&ring, &[7, 8, 9, 10]);
        assert_eq!(ring.dropped(), 2);
        assert_eq!(read_all(&ring), [1, 2, 3, 4, 5, 6, 7, 8]);
        // Taken once.
        assert!(ring.take_producer().is_none());
    }

    #[test]
    fn a_clear_request_drops_what_was_queued_before_it() {
        let ring = AudioRing::default();
        let mut producer = ring.take_producer().unwrap();
        producer.push(&ring, &[1; 100]);
        ring.clear();
        assert!(read_all(&ring).is_empty());
        producer.push(&ring, &[2; 10]);
        assert_eq!(read_all(&ring), [2; 10]);
        assert_eq!(ring.dropped(), 0, "a clear is not a loss");
    }

    /// The producer and the reader run on two threads at once, as the emulation thread and the
    /// audio callback do: every sample arrives, in order, and neither side ever waits.
    #[test]
    fn producer_and_reader_on_two_threads_lose_nothing() {
        let ring = std::sync::Arc::new(AudioRing::with_capacity(4096));
        ring.set_rate(1_000_000.0); // no trimming: max_len far above the capacity
        let mut producer = ring.take_producer().unwrap();
        let writer_ring = ring.clone();
        let total: i32 = 200_000;
        let writer = std::thread::spawn(move || {
            let mut next = 0i32;
            while next < total {
                // The test wants every sample: it waits for room instead of losing them.
                if producer.free() < 64 {
                    std::thread::yield_now();
                    continue;
                }
                let chunk: Vec<i16> = (next..next + 64).map(|v| v as i16).collect();
                producer.push(&writer_ring, &chunk);
                next += 64;
            }
            assert_eq!(writer_ring.dropped(), 0);
        });
        let mut got = 0i32;
        while got < total {
            if let Some(mut reader) = ring.reader() {
                reader.begin(&ring);
                while let Some([l, r]) = reader.pop_frame() {
                    assert_eq!((l, r), (got as i16, (got + 1) as i16));
                    got += 2;
                }
            }
        }
        writer.join().unwrap();
    }

    /// Before / after for Trello #79 (not run by default): the longest a 5 ms audio callback is
    /// held up while the emulation thread spends ~4 ms on each push (a WSOLA search on a slow
    /// phone), with the old design (one mutex around the queue and the work) and the ring.
    /// `cargo test -p emudome-core --release audio::tests::callback_wait -- --ignored --nocapture`
    #[test]
    #[ignore = "a measurement: prints the numbers for docs/fase0-esito.md"]
    fn callback_wait_with_a_mutex_and_with_the_ring() {
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        fn busy(d: Duration) {
            let t = Instant::now();
            while t.elapsed() < d {
                std::hint::spin_loop();
            }
        }
        let run = |work: Box<dyn Fn() + Send>, callback: Box<dyn Fn()>| -> Duration {
            let stop = Arc::new(AtomicBool::new(false));
            let s = stop.clone();
            let producer = std::thread::spawn(move || {
                while !s.load(Ordering::Relaxed) {
                    work();
                    std::thread::sleep(Duration::from_millis(1));
                }
            });
            let mut worst = Duration::ZERO;
            for _ in 0..200 {
                let t = Instant::now();
                callback();
                worst = worst.max(t.elapsed());
                std::thread::sleep(Duration::from_millis(5));
            }
            stop.store(true, Ordering::Relaxed);
            producer.join().unwrap();
            worst
        };

        // The old design: the work runs under the queue's lock, the callback takes it too.
        let queue = Arc::new(Mutex::new(std::collections::VecDeque::<i16>::new()));
        let q = queue.clone();
        let old = run(
            Box::new(move || {
                let mut q = q.lock();
                busy(Duration::from_millis(4));
                q.extend([0i16; 1094]);
            }),
            Box::new(move || {
                let mut q = queue.lock();
                let n = q.len().min(960);
                q.drain(..n);
            }),
        );

        // The ring: the work before the push, the callback reads lock-free.
        let ring = Arc::new(AudioRing::default());
        // Locked by the producer thread only (the closure must be `Fn`).
        let producer = Mutex::new(ring.take_producer().unwrap());
        let r = ring.clone();
        let new = run(
            Box::new(move || {
                busy(Duration::from_millis(4));
                producer.lock().push(&r, &[0; 1094]);
            }),
            Box::new(move || {
                if let Some(mut reader) = ring.reader() {
                    reader.begin(&ring);
                    for _ in 0..480 {
                        reader.pop_frame();
                    }
                }
            }),
        );
        println!(
            "audio callback, longest wait over 200 callbacks: mutex {:.3} ms, ring {:.3} ms",
            old.as_secs_f64() * 1000.0,
            new.as_secs_f64() * 1000.0
        );
        assert!(new < old);
    }
}
