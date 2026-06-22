use std::time::Duration;

use gpui::{
    Background, Bounds, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    Render, Styled, Window, div, linear_color_stop, linear_gradient, px,
};
use gpui_component::{
    ActiveTheme,
    plot::{
        IntoPlot, Plot, StrokeStyle,
        shape::{Area, Line},
    },
};
use pulse_audio::{PlaybackState, SpectrumAnalyzer, SpectrumConfig};
use pulse_data::{ResolvedVisualizerSettings, VisualizerMode};

use crate::config::PulseConfig;
use crate::player::PulsePlayer;

const SPECTRUM_DISPLAY_POINTS: usize = 512;
const OSCILLOSCOPE_POINTS: usize = 1024;
const DEFAULT_SAMPLE_RATE: f32 = 44_100.;

#[derive(IntoPlot)]
struct SpectrumWavePlot {
    wave: Vec<WavePoint>,
    peak: Vec<WavePoint>,
    settings: ResolvedVisualizerSettings,
    fill: Hsla,
    stroke: Hsla,
    peak_stroke: Hsla,
}

impl SpectrumWavePlot {
    fn new(
        bars: &[f32],
        peaks: &[f32],
        settings: &ResolvedVisualizerSettings,
        fill: Hsla,
        stroke: Hsla,
        peak_stroke: Hsla,
    ) -> Self {
        let emphasized = emphasize_spectrum(bars);
        let wave = build_spectrum_wave(&emphasized, settings.mirror);
        let peak_wave = if settings.peak_hold {
            build_spectrum_wave(&emphasize_spectrum(peaks), settings.mirror)
        } else {
            Vec::new()
        };

        Self {
            wave,
            peak: peak_wave,
            settings: settings.clone(),
            fill,
            stroke,
            peak_stroke,
        }
    }
}

#[derive(IntoPlot)]
struct OscilloscopePlot {
    wave: Vec<WavePoint>,
    fill: Hsla,
    stroke: Hsla,
    gradient: bool,
}

impl OscilloscopePlot {
    fn new(samples: &[f32], fill: Hsla, stroke: Hsla, gradient: bool) -> Self {
        Self {
            wave: build_oscilloscope_wave(samples),
            fill,
            stroke,
            gradient,
        }
    }
}

fn mirror_layout(values: &[f32]) -> Vec<f32> {
    let count = values.len();
    let mut mirrored = Vec::with_capacity(count * 2);
    mirrored.extend(values.iter().rev());
    mirrored.extend_from_slice(values);
    mirrored
}

/// Push quiet bins down relative to the frame peak so dominant frequencies read clearly.
fn emphasize_spectrum(values: &[f32]) -> Vec<f32> {
    if values.is_empty() {
        return Vec::new();
    }

    let peak = values.iter().copied().fold(0_f32, f32::max);
    if peak <= 1e-6 {
        return values.to_vec();
    }

    let smoothed = spatial_smooth(values, 1);
    smoothed
        .iter()
        .map(|value| {
            let relative = (value / peak).clamp(0., 1.);
            (relative.powf(1.55) * peak).clamp(0., 1.)
        })
        .collect()
}

fn spatial_smooth(values: &[f32], radius: usize) -> Vec<f32> {
    if values.is_empty() {
        return Vec::new();
    }

    values
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let start = index.saturating_sub(radius);
            let end = (index + radius + 1).min(values.len());
            values[start..end].iter().sum::<f32>() / (end - start) as f32
        })
        .collect()
}

fn build_spectrum_wave(values: &[f32], mirror: bool) -> Vec<WavePoint> {
    let values = if mirror {
        mirror_layout(values)
    } else {
        values.to_vec()
    };

    if values.len() < 2 {
        return Vec::new();
    }

    let count = values.len();
    (0..SPECTRUM_DISPLAY_POINTS)
        .map(|index| {
            let t = index as f32 / (SPECTRUM_DISPLAY_POINTS - 1) as f32;
            let position = t * (count - 1) as f32;
            let left = position.floor() as usize;
            let fraction = position - left as f32;
            let left = left.min(count - 1);
            let right = (left + 1).min(count - 1);
            let amplitude = values[left] + (values[right] - values[left]) * fraction;
            WavePoint {
                x: t,
                y: amplitude.clamp(0., 1.),
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
struct WavePoint {
    x: f32,
    y: f32,
}

const OSCILLOSCOPE_DISPLAY_POINTS: usize = 512;

fn build_oscilloscope_wave(samples: &[f32]) -> Vec<WavePoint> {
    if samples.len() < 2 {
        return Vec::new();
    }

    let count = OSCILLOSCOPE_POINTS.min(samples.len());
    let slice = &samples[samples.len() - count..];
    let peak = slice
        .iter()
        .map(|sample| sample.abs())
        .fold(0.01_f32, |left, right| {
            if left > right {
                left
            } else {
                right
            }
        });

    (0..OSCILLOSCOPE_DISPLAY_POINTS)
        .map(|index| {
            let t = index as f32 / (OSCILLOSCOPE_DISPLAY_POINTS - 1) as f32;
            let position = t * (count - 1) as f32;
            let left = position.floor() as usize;
            let fraction = position - left as f32;
            let left = left.min(count - 1);
            let right = (left + 1).min(count - 1);
            let sample = slice[left] + (slice[right] - slice[left]) * fraction;
            let normalized = (sample / peak).clamp(-1., 1.);
            WavePoint {
                x: t,
                y: 0.5 + normalized * 0.45,
            }
        })
        .collect()
}

impl Plot for SpectrumWavePlot {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, _: &mut gpui::App) {
        if self.wave.is_empty() {
            return;
        }

        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32();
        let fill = self.fill;
        let stroke = self.stroke;
        let peak_stroke = self.peak_stroke;
        let gradient = self.settings.gradient;

        let to_screen = |point: &WavePoint| WavePoint {
            x: point.x * width,
            y: height - point.y * height,
        };

        let screen_wave: Vec<WavePoint> = self.wave.iter().map(to_screen).collect();
        let area_fill = if gradient {
            linear_gradient(
                0.,
                linear_color_stop(fill, 0.),
                linear_color_stop(fill.opacity(0.), 1.),
            )
        } else {
            Background::from(fill.opacity(0.35))
        };

        Area::new()
            .data(&screen_wave)
            .x(|point: &&WavePoint| Some(point.x))
            .y0(height)
            .y1(|point: &&WavePoint| Some(point.y))
            .fill(area_fill)
            .stroke(Background::from(stroke.opacity(0.85)))
            .stroke_style(StrokeStyle::Natural)
            .paint(&bounds, window);

        Line::new()
            .data(&screen_wave)
            .x(|point: &&WavePoint| Some(point.x))
            .y(|point: &&WavePoint| Some(point.y))
            .stroke(Background::from(stroke.opacity(0.22)))
            .stroke_width(px(5.))
            .stroke_style(StrokeStyle::Natural)
            .paint(&bounds, window);

        Line::new()
            .data(&screen_wave)
            .x(|point: &&WavePoint| Some(point.x))
            .y(|point: &&WavePoint| Some(point.y))
            .stroke(Background::from(stroke))
            .stroke_width(px(1.5))
            .stroke_style(StrokeStyle::Natural)
            .paint(&bounds, window);

        if self.settings.peak_hold && !self.peak.is_empty() {
            let screen_peak: Vec<WavePoint> = self.peak.iter().map(to_screen).collect();
            Line::new()
                .data(&screen_peak)
                .x(|point: &&WavePoint| Some(point.x))
                .y(|point: &&WavePoint| Some(point.y))
                .stroke(Background::from(peak_stroke))
                .stroke_width(px(1.))
                .stroke_style(StrokeStyle::Natural)
                .paint(&bounds, window);
        }
    }
}

impl Plot for OscilloscopePlot {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, _: &mut gpui::App) {
        if self.wave.is_empty() {
            return;
        }

        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32();
        let center = height * 0.5;
        let fill = self.fill;
        let stroke = self.stroke;
        let gradient = self.gradient;

        let screen_wave: Vec<WavePoint> = self
            .wave
            .iter()
            .map(|point| WavePoint {
                x: point.x * width,
                y: height - point.y * height,
            })
            .collect();

        let area_fill = if gradient {
            linear_gradient(
                0.,
                linear_color_stop(fill.opacity(0.85), 0.),
                linear_color_stop(fill.opacity(0.), 1.),
            )
        } else {
            Background::from(fill.opacity(0.35))
        };

        Area::new()
            .data(&screen_wave)
            .x(|point: &&WavePoint| Some(point.x))
            .y0(center)
            .y1(|point: &&WavePoint| Some(point.y))
            .fill(area_fill)
            .stroke(Background::from(stroke.opacity(0.5)))
            .stroke_style(StrokeStyle::Natural)
            .paint(&bounds, window);

        Line::new()
            .data(&screen_wave)
            .x(|point: &&WavePoint| Some(point.x))
            .y(|point: &&WavePoint| Some(point.y))
            .stroke(Background::from(stroke.opacity(0.18)))
            .stroke_width(px(6.))
            .stroke_style(StrokeStyle::Natural)
            .paint(&bounds, window);

        Line::new()
            .data(&screen_wave)
            .x(|point: &&WavePoint| Some(point.x))
            .y(|point: &&WavePoint| Some(point.y))
            .stroke(Background::from(stroke))
            .stroke_width(px(1.5))
            .stroke_style(StrokeStyle::Natural)
            .paint(&bounds, window);
    }
}

pub struct VisualizerPage {
    bars: Vec<f32>,
    peaks: Vec<f32>,
    analyzer: SpectrumAnalyzer,
    runtime: ResolvedVisualizerSettings,
    sample_rate: f32,
}

impl VisualizerPage {
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        let runtime = cx.global::<PulseConfig>().visualizer.resolve();

        cx.spawn(async move |this, cx| {
            loop {
                let refresh_ms = this
                    .read_with(cx, |page, _| page.runtime.refresh_ms)
                    .unwrap_or(16);
                cx.background_executor()
                    .timer(Duration::from_millis(refresh_ms))
                    .await;
                this.update(cx, |page, cx| {
                    page.sync_settings(cx);
                    page.tick(cx);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        let analyzer = SpectrumAnalyzer::with_config(
            SpectrumConfig {
                fft_size: runtime.fft_size,
                bar_count: runtime.bar_count,
            },
            DEFAULT_SAMPLE_RATE,
        );

        Self {
            bars: vec![0.; runtime.bar_count],
            peaks: vec![0.; runtime.bar_count],
            analyzer,
            runtime,
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }

    fn sync_settings(&mut self, cx: &mut Context<Self>) {
        if let Some(rate) = PulsePlayer::sample_rate(cx) {
            self.sample_rate = rate as f32;
        }

        let runtime = cx.global::<PulseConfig>().visualizer.resolve();
        if self.runtime == runtime {
            return;
        }

        self.runtime = runtime.clone();
        self.analyzer.reconfigure(
            SpectrumConfig {
                fft_size: runtime.fft_size,
                bar_count: runtime.bar_count,
            },
            self.sample_rate,
        );
        self.bars.resize(runtime.bar_count, 0.);
        self.peaks.resize(runtime.bar_count, 0.);
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        let playing = PulsePlayer::snapshot(cx).state == PlaybackState::Playing;
        let samples = PulsePlayer::sample_snapshot(cx);
        let sample_rate = PulsePlayer::sample_rate(cx).map(|rate| rate as f32);
        let fft_size = self.runtime.fft_size;

        if let Some(rate) = sample_rate {
            self.sample_rate = rate;
            self.analyzer.reconfigure(
                SpectrumConfig {
                    fft_size: self.runtime.fft_size,
                    bar_count: self.runtime.bar_count,
                },
                rate,
            );
        }

        if self.runtime.mode == VisualizerMode::Spectrum {
            self.tick_spectrum(playing, &samples, sample_rate, fft_size);
        }
    }

    fn tick_spectrum(
        &mut self,
        playing: bool,
        samples: &[f32],
        sample_rate: Option<f32>,
        fft_size: usize,
    ) {
        let target = if playing && samples.len() >= fft_size {
            if let Some(rate) = sample_rate {
                self.analyzer
                    .analyze_overlapped_with_rate(samples, rate, self.runtime.fft_passes)
            } else {
                self.analyzer
                    .analyze_overlapped(samples, self.runtime.fft_passes)
            }
        } else {
            vec![0.; self.runtime.bar_count]
        };

        let attack = self.runtime.attack;
        let decay = self.runtime.decay;
        let idle_decay = self.runtime.decay * 1.5;
        let peak_decay = self.runtime.peak_decay;

        for (bar, target_value) in self.bars.iter_mut().zip(&target) {
            let blend = if playing {
                if *target_value > *bar {
                    attack
                } else {
                    decay
                }
            } else {
                idle_decay
            };
            *bar = (*bar * (1. - blend) + target_value * blend).max(0.);
        }

        if self.runtime.peak_hold {
            for (peak, &value) in self.peaks.iter_mut().zip(&self.bars) {
                if value > *peak {
                    *peak = value;
                } else {
                    *peak *= peak_decay;
                }
            }
        } else {
            self.peaks.fill(0.);
        }
    }
}

impl Render for VisualizerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let samples = PulsePlayer::sample_snapshot(cx);

        if self.runtime.mode == VisualizerMode::Spectrum {
            div()
                .id("visualizer-canvas")
                .size_full()
                .overflow_hidden()
                .bg(theme.background)
                .child(SpectrumWavePlot::new(
                    &self.bars,
                    &self.peaks,
                    &self.runtime,
                    theme.primary,
                    theme.primary,
                    theme.foreground.opacity(0.45),
                ))
                .into_any_element()
        } else {
            div()
                .id("visualizer-canvas")
                .size_full()
                .overflow_hidden()
                .bg(theme.background)
                .child(OscilloscopePlot::new(
                    &samples,
                    theme.primary,
                    theme.primary,
                    self.runtime.gradient,
                ))
                .into_any_element()
        }
    }
}
