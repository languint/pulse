use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

const CAPACITY: usize = 4096;

#[derive(Debug)]
struct CaptureState {
    samples: [f32; CAPACITY],
    len: usize,
    write_ix: usize,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            samples: [0.; CAPACITY],
            len: 0,
            write_ix: 0,
        }
    }
}

/// Ring buffer of recent mono audio samples for analysis.
#[derive(Clone, Debug)]
pub struct SampleCapture {
    state: Arc<Mutex<CaptureState>>,
    sample_rate: Arc<AtomicU32>,
}

impl Default for SampleCapture {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(CaptureState::default())),
            sample_rate: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl SampleCapture {
    pub fn set_sample_rate(&self, sample_rate: u32) {
        if sample_rate > 0 {
            self.sample_rate.store(sample_rate, Ordering::Relaxed);
        }
    }

    #[must_use]
    pub fn sample_rate(&self) -> Option<u32> {
        match self.sample_rate.load(Ordering::Relaxed) {
            0 => None,
            rate => Some(rate),
        }
    }

    pub fn push(&self, sample: f32) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };

        let write_ix = state.write_ix;
        state.samples[write_ix] = sample;
        state.write_ix = (write_ix + 1) % CAPACITY;
        state.len = state.len.saturating_add(1).min(CAPACITY);
    }

    /// Returns the most recent samples in chronological order.
    #[must_use]
    pub fn snapshot(&self) -> Vec<f32> {
        let Ok(state) = self.state.lock() else {
            return Vec::new();
        };

        if state.len == 0 {
            return Vec::new();
        }

        if state.len < CAPACITY {
            return state.samples[..state.len].to_vec();
        }

        (0..CAPACITY)
            .map(|offset| state.samples[(state.write_ix + offset) % CAPACITY])
            .collect()
    }
}
