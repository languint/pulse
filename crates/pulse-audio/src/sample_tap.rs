use std::num::NonZero;
use std::sync::Arc;
use std::time::Duration;

use rodio::{ChannelCount, SampleRate, Source};

use crate::sample_capture::SampleCapture;

const MONO: NonZero<u16> = NonZero::new(1).unwrap();

pub struct SampleTap<S> {
    inner: S,
    capture: Arc<SampleCapture>,
    channels: ChannelCount,
    frame: Vec<f32>,
}

impl<S: Source> SampleTap<S> {
    pub fn new(inner: S, capture: Arc<SampleCapture>) -> Self {
        let channels = NonZero::new(inner.channels().get().max(1)).unwrap_or(MONO);
        capture.set_sample_rate(inner.sample_rate().get());
        Self {
            inner,
            capture,
            channels,
            frame: Vec::with_capacity(channels.get() as usize),
        }
    }

    fn push_frame(&self, frame: &[f32]) {
        if frame.is_empty() {
            return;
        }

        let mono = frame.iter().sum::<f32>() / frame.len() as f32;
        self.capture.push(mono);
    }
}

impl<S: Source> Iterator for SampleTap<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        self.frame.push(sample);

        if self.frame.len() == self.channels.get() as usize {
            self.push_frame(&self.frame);
            self.frame.clear();
        }

        Some(sample)
    }
}

impl<S: Source> Source for SampleTap<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
