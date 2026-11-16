//! Audio queue between the emulation thread (producer) and the audio output (consumer).

use std::collections::VecDeque;

/// Interleaved stereo samples at the core's rate.
pub struct AudioQueue {
    pub samples: VecDeque<i16>,
    pub sample_rate: f64,
    /// Most samples (stereo) to keep queued: beyond it the oldest are dropped
    /// so latency does not build up.
    pub max_len: usize,
}

impl Default for AudioQueue {
    fn default() -> Self {
        Self {
            samples: VecDeque::new(),
            sample_rate: 32768.0,
            max_len: 32768 / 4 * 2,
        }
    }
}

impl AudioQueue {
    pub fn set_rate(&mut self, rate: f64) {
        self.sample_rate = rate;
        // ~150 ms of headroom.
        self.max_len = ((rate * 0.15) as usize) * 2;
    }

    pub fn push(&mut self, data: &[i16]) {
        self.samples.extend(data.iter().copied());
        let excess = self.samples.len().saturating_sub(self.max_len);
        if excess > 0 {
            self.samples.drain(..excess + (excess & 1));
        }
    }
}
