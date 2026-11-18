//! Audio queue between the emulation thread (producer) and the audio output (consumer).

use std::collections::VecDeque;

/// Frequency of the bench's test tone.
pub const TEST_TONE_HZ: f64 = 440.0;

/// Interleaved stereo samples at the core's rate.
pub struct AudioQueue {
    pub samples: VecDeque<i16>,
    pub sample_rate: f64,
    /// Most samples (stereo) to keep queued: beyond it the oldest are dropped
    /// so latency does not build up.
    pub max_len: usize,
    /// Samples dropped so far because the queue was over `max_len`.
    pub dropped: u64,
    tone: Option<TestTone>,
}

/// A sine that replaces the core's samples, same count, same timing (bench mode:
/// the test ROM is silent, and a known signal shows every glitch on the way out).
struct TestTone {
    amplitude: f64,
    phase: f64,
}

impl Default for AudioQueue {
    fn default() -> Self {
        Self {
            samples: VecDeque::new(),
            sample_rate: 32768.0,
            max_len: 32768 / 4 * 2,
            dropped: 0,
            tone: None,
        }
    }
}

impl AudioQueue {
    pub fn set_rate(&mut self, rate: f64) {
        self.sample_rate = rate;
        // ~150 ms of headroom.
        self.max_len = ((rate * 0.15) as usize) * 2;
    }

    /// Replaces the game's audio with a [`TEST_TONE_HZ`] sine of this amplitude (0..1), or
    /// gives the game's audio back with `None`.
    pub fn set_test_tone(&mut self, amplitude: Option<f32>) {
        self.tone = amplitude.map(|a| TestTone {
            amplitude: f64::from(a.clamp(0.0, 1.0)) * f64::from(i16::MAX),
            phase: 0.0,
        });
    }

    pub fn push(&mut self, data: &[i16]) {
        match &mut self.tone {
            Some(tone) => {
                let step = std::f64::consts::TAU * TEST_TONE_HZ / self.sample_rate;
                for _ in data.chunks_exact(2) {
                    let s = (tone.phase.sin() * tone.amplitude) as i16;
                    tone.phase = (tone.phase + step) % std::f64::consts::TAU;
                    self.samples.extend([s, s]);
                }
            }
            None => self.samples.extend(data.iter().copied()),
        }
        let excess = self.samples.len().saturating_sub(self.max_len);
        if excess > 0 {
            let n = excess + (excess & 1);
            self.samples.drain(..n);
            self.dropped += n as u64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tone_keeps_the_sample_count_and_its_amplitude() {
        let mut q = AudioQueue::default();
        q.set_rate(32768.0);
        q.set_test_tone(Some(0.05));
        q.push(&[1000; 1094]);
        assert_eq!(q.samples.len(), 1094);
        let peak = q.samples.iter().map(|s| s.unsigned_abs()).max().unwrap();
        assert!((1630..=1639).contains(&peak), "peak {peak}");
        // Left and right are the same signal.
        assert!(q
            .samples
            .iter()
            .step_by(2)
            .eq(q.samples.iter().skip(1).step_by(2)));
    }

    #[test]
    fn overflow_is_counted() {
        let mut q = AudioQueue::default();
        q.set_rate(1000.0); // max_len = 150 * 2
        q.push(&[0; 400]);
        assert_eq!(q.samples.len(), 300);
        assert_eq!(q.dropped, 100);
    }
}
