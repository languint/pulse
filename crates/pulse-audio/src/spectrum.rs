use std::f32::consts::PI;

use num_complex::Complex;
use rustfft::FftPlanner;

pub const DEFAULT_FFT_SIZE: usize = 1024;
pub const DEFAULT_BAR_COUNT: usize = 96;

const DEFAULT_SAMPLE_RATE: f32 = 44_100.;
const MIN_FREQ_HZ: f32 = 80.;
const MAX_FREQ_HZ: f32 = 16_000.;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpectrumConfig {
    pub fft_size: usize,
    pub bar_count: usize,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            fft_size: DEFAULT_FFT_SIZE,
            bar_count: DEFAULT_BAR_COUNT,
        }
    }
}

pub struct SpectrumAnalyzer {
    config: SpectrumConfig,
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    magnitudes: Vec<f32>,
    band_magnitudes: Vec<f32>,
    window: Vec<f32>,
    log_positions: Vec<f32>,
    sample_rate: f32,
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectrumAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(SpectrumConfig::default(), DEFAULT_SAMPLE_RATE)
    }

    #[must_use]
    pub fn with_config(config: SpectrumConfig, sample_rate: f32) -> Self {
        let sample_rate = sample_rate.max(1.);
        let config = normalize_config(config);
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(config.fft_size);
        let window = hann_window(config.fft_size);
        let log_positions = log_sample_grid(
            config.bar_count,
            MIN_FREQ_HZ,
            MAX_FREQ_HZ,
            sample_rate,
            config.fft_size,
        );

        Self {
            config,
            fft,
            scratch: vec![Complex::new(0., 0.); config.fft_size],
            magnitudes: vec![0.; config.fft_size / 2 + 1],
            band_magnitudes: vec![0.; config.bar_count],
            window,
            log_positions,
            sample_rate,
        }
    }

    #[must_use]
    pub const fn config(&self) -> SpectrumConfig {
        self.config
    }

    #[must_use]
    pub const fn fft_size(&self) -> usize {
        self.config.fft_size
    }

    #[must_use]
    pub const fn bar_count(&self) -> usize {
        self.config.bar_count
    }

    pub fn reconfigure(&mut self, config: SpectrumConfig, sample_rate: f32) {
        let config = normalize_config(config);
        let sample_rate = sample_rate.max(1.);
        if self.config == config && (self.sample_rate - sample_rate).abs() < f32::EPSILON {
            return;
        }

        *self = Self::with_config(config, sample_rate);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.reconfigure(self.config, sample_rate);
    }

    #[must_use]
    pub fn analyze(&mut self, samples: &[f32]) -> Vec<f32> {
        self.analyze_overlapped(samples, 1)
    }

    #[must_use]
    pub fn analyze_overlapped(&mut self, samples: &[f32], passes: u32) -> Vec<f32> {
        let passes = passes.max(1);
        let fft_size = self.config.fft_size;
        let hop = (fft_size / passes as usize).max(1);
        let mut merged = vec![0.; self.config.bar_count];
        let mut used = 0u32;

        for pass in 0..passes {
            let end = samples.len().saturating_sub(pass as usize * hop);
            if end < fft_size {
                break;
            }

            let window = &samples[end - fft_size..end];
            let frame = self.analyze_window(window);
            for (accumulator, value) in merged.iter_mut().zip(frame) {
                if value > *accumulator {
                    *accumulator = value;
                }
            }
            used += 1;
        }

        if used == 0 {
            merged
        } else {
            merged
        }
    }

    #[must_use]
    pub fn analyze_with_rate(&mut self, samples: &[f32], sample_rate: f32) -> Vec<f32> {
        if (self.sample_rate - sample_rate).abs() >= f32::EPSILON {
            self.set_sample_rate(sample_rate);
        }
        self.analyze(samples)
    }

    #[must_use]
    pub fn analyze_overlapped_with_rate(
        &mut self,
        samples: &[f32],
        sample_rate: f32,
        passes: u32,
    ) -> Vec<f32> {
        if (self.sample_rate - sample_rate).abs() >= f32::EPSILON {
            self.set_sample_rate(sample_rate);
        }
        self.analyze_overlapped(samples, passes)
    }

    #[must_use]
    fn analyze_window(&mut self, frame: &[f32]) -> Vec<f32> {
        debug_assert_eq!(frame.len(), self.config.fft_size);

        let mean = frame.iter().sum::<f32>() / self.config.fft_size as f32;
        for (index, slot) in self.scratch.iter_mut().enumerate() {
            let centered = frame[index] - mean;
            *slot = Complex::new(centered * self.window[index], 0.);
        }

        self.fft.process(&mut self.scratch);

        let scale = 2.0 / self.config.fft_size as f32;
        self.magnitudes[0] = 0.;
        for (index, magnitude) in self.magnitudes.iter_mut().enumerate().skip(1) {
            *magnitude = self.scratch[index].norm() * scale;
        }

        let max_bin = self.config.fft_size / 2;
        let positions = &self.log_positions;
        for (band_index, band) in self.band_magnitudes.iter_mut().enumerate() {
            let start = positions[band_index].clamp(1., max_bin as f32).floor() as usize;
            let end = if band_index + 1 < positions.len() {
                positions[band_index + 1].ceil() as usize
            } else {
                start + 1
            }
            .clamp(start, max_bin);
            *band = self.magnitudes[start..=end]
                .iter()
                .copied()
                .fold(0_f32, f32::max);
        }

        magnitudes_to_display(&self.band_magnitudes)
    }
}

fn normalize_config(config: SpectrumConfig) -> SpectrumConfig {
    SpectrumConfig {
        fft_size: normalize_fft_size(config.fft_size),
        bar_count: config.bar_count.clamp(16, 512),
    }
}

fn normalize_fft_size(size: usize) -> usize {
    let size = size.clamp(256, 8192);
    size.next_power_of_two()
}

fn hann_window(size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.; size];
    }

    (0..size)
        .map(|index| {
            let phase = index as f32 / (size - 1) as f32;
            0.5 * (1. - (2. * PI * phase).cos())
        })
        .collect()
}

fn log_sample_grid(
    num_bins: usize,
    min_hz: f32,
    max_hz: f32,
    sample_rate: f32,
    fft_size: usize,
) -> Vec<f32> {
    let nyquist = sample_rate * 0.5;
    let max_hz = max_hz.min(nyquist * 0.98);
    let min_hz = min_hz.max(sample_rate / fft_size as f32);
    let bin_hz = sample_rate / fft_size as f32;
    let log_min = min_hz.ln();
    let log_max = max_hz.ln();

    (0..num_bins)
        .map(|index| {
            let t = (index as f32 + 0.5) / num_bins as f32;
            let freq = (log_min + t * (log_max - log_min)).exp();
            freq / bin_hz
        })
        .collect()
}

fn magnitudes_to_display(magnitudes: &[f32]) -> Vec<f32> {
    if magnitudes.iter().all(|value| *value <= 1e-12) {
        return vec![0.; magnitudes.len()];
    }

    magnitudes
        .iter()
        .map(|&magnitude| amplitude_to_display(magnitude))
        .collect()
}

/// Fixed dB window like a hardware LED analyzer — quiet bins stay low, peaks punch up.
fn amplitude_to_display(magnitude: f32) -> f32 {
    const MIN_DB: f32 = -78.;
    const MAX_DB: f32 = -8.;

    let db = 20. * magnitude.max(1e-10).log10();
    let normalized = (db - MIN_DB) / (MAX_DB - MIN_DB);
    normalized.clamp(0., 1.).powf(0.82)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_wave(freq_hz: f32, sample_rate: f32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|index| {
                let t = index as f32 / sample_rate;
                (2. * PI * freq_hz * t).sin() * 0.5
            })
            .collect()
    }

    fn peak_bin(bins: &[f32]) -> usize {
        bins.iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap())
            .map(|(index, _)| index)
            .unwrap_or(0)
    }

    fn mean(values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.;
        }
        values.iter().sum::<f32>() / values.len() as f32
    }

    #[test]
    fn silence_is_flat() {
        let mut analyzer = SpectrumAnalyzer::new();
        let bins = analyzer.analyze(&[0.; DEFAULT_FFT_SIZE]);
        assert_eq!(bins.len(), DEFAULT_BAR_COUNT);
        assert!(bins.iter().all(|value| *value < 0.05));
    }

    #[test]
    fn mid_frequency_tone_is_not_leftmost_bin() {
        let sample_rate = 44_100.;
        let mut analyzer = SpectrumAnalyzer::with_config(SpectrumConfig::default(), sample_rate);
        let samples = sine_wave(1_000., sample_rate, DEFAULT_FFT_SIZE);
        let bins = analyzer.analyze(&samples);
        let peak = peak_bin(&bins);
        assert!(
            peak > 8,
            "expected peak away from bass bins, got peak at {peak}"
        );
    }

    #[test]
    fn overlapped_passes_use_more_samples() {
        let sample_rate = 44_100.;
        let config = SpectrumConfig {
            fft_size: 1024,
            bar_count: 96,
        };
        let mut analyzer = SpectrumAnalyzer::with_config(config, sample_rate);
        let samples = sine_wave(440., sample_rate, 4096);
        let single = analyzer.analyze_overlapped(&samples, 1);
        let multi = analyzer.analyze_overlapped(&samples, 4);
        assert_eq!(single.len(), multi.len());
        assert!(multi.iter().sum::<f32>() >= single.iter().sum::<f32>());
    }

    #[test]
    fn tone_peak_stands_above_neighbors() {
        let sample_rate = 44_100.;
        let mut analyzer = SpectrumAnalyzer::with_config(SpectrumConfig::default(), sample_rate);
        let samples = sine_wave(1_000., sample_rate, DEFAULT_FFT_SIZE);
        let bins = analyzer.analyze(&samples);
        let peak = peak_bin(&bins);
        let peak_value = bins[peak];
        let neighbor_mean = mean(&[
            bins[peak.saturating_sub(2)],
            bins[peak.saturating_sub(1)],
            bins[(peak + 1).min(bins.len() - 1)],
            bins[(peak + 2).min(bins.len() - 1)],
        ]);

        assert!(
            peak_value > neighbor_mean + 0.08,
            "peak should stand out: peak={peak_value:.2}, neighbors={neighbor_mean:.2}"
        );
    }

    #[test]
    fn pink_slope_keeps_bass_emphasis() {
        let frequencies: Vec<f32> = log_sample_grid(
            DEFAULT_BAR_COUNT,
            MIN_FREQ_HZ,
            MAX_FREQ_HZ,
            44_100.,
            DEFAULT_FFT_SIZE,
        )
        .iter()
        .map(|position| position * 44_100. / DEFAULT_FFT_SIZE as f32)
        .collect();
        let magnitudes: Vec<f32> = frequencies
            .iter()
            .map(|frequency| (1_000. / frequency).sqrt() * 0.02)
            .collect();
        let bins = magnitudes_to_display(&magnitudes);

        let left = mean(&bins[..16]);
        let right = mean(&bins[80..]);
        assert!(
            left > right + 0.05,
            "classic analyzer should show stronger bass: left={left:.2}, right={right:.2}"
        );
    }
}
