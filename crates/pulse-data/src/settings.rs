use pulse_library::LibraryConfig;

use crate::{DataError, PulsePaths};

pub const DEFAULT_THEME: &str = "Pulse Dark";
pub const THEME_PULSE_DARK: &str = "Pulse Dark";
pub const THEME_PULSE_LIGHT: &str = "Pulse Light";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisualizerMode {
    #[default]
    Spectrum,
    Oscilloscope,
}

impl VisualizerMode {
    pub const ALL: [Self; 2] = [Self::Spectrum, Self::Oscilloscope];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Spectrum => "Spectrum",
            Self::Oscilloscope => "Oscilloscope",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Spectrum => "Smooth frequency wave",
            Self::Oscilloscope => "Time-domain waveform",
        }
    }
}

impl<'de> serde::Deserialize<'de> for VisualizerMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "oscilloscope" => Self::Oscilloscope,
            "spectrum" => Self::Spectrum,
            // Migrate saved spectrogram preference to spectrum.
            "spectrogram" => Self::Spectrum,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisualizerQuality {
    #[default]
    Efficient,
    Balanced,
    High,
    Ultra,
}

impl VisualizerQuality {
    pub const ALL: [Self; 4] = [Self::Efficient, Self::Balanced, Self::High, Self::Ultra];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Efficient => "Efficient",
            Self::Balanced => "Balanced",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Efficient => "Lightweight — minimal CPU use",
            Self::Balanced => "Smoother motion and finer detail",
            Self::High => "Large FFT with multi-pass analysis",
            Self::Ultra => "Maximum detail — highest CPU use",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VisualizerSettings {
    #[serde(default)]
    pub mode: VisualizerMode,

    #[serde(default)]
    pub quality: VisualizerQuality,

    #[serde(default = "default_true")]
    pub peak_hold: bool,

    #[serde(default = "default_true")]
    pub mirror: bool,

    #[serde(default = "default_true")]
    pub gradient: bool,
}

impl Default for VisualizerSettings {
    fn default() -> Self {
        Self {
            mode: VisualizerMode::default(),
            quality: VisualizerQuality::default(),
            peak_hold: default_true(),
            mirror: default_true(),
            gradient: default_true(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedVisualizerSettings {
    pub mode: VisualizerMode,
    pub fft_size: usize,
    pub bar_count: usize,
    pub refresh_ms: u64,
    pub fft_passes: u32,
    pub attack: f32,
    pub decay: f32,
    pub peak_decay: f32,
    pub peak_hold: bool,
    pub mirror: bool,
    pub gradient: bool,
}

impl VisualizerSettings {
    #[must_use]
    pub fn resolve(&self) -> ResolvedVisualizerSettings {
        let (fft_size, bar_count, refresh_ms, fft_passes, attack, decay, peak_decay) =
            match self.quality {
                VisualizerQuality::Efficient => (1024, 96, 16, 1, 0.50, 0.12, 0.94),
                VisualizerQuality::Balanced => (2048, 128, 8, 2, 0.55, 0.10, 0.95),
                VisualizerQuality::High => (4096, 192, 8, 4, 0.60, 0.08, 0.96),
                VisualizerQuality::Ultra => (4096, 256, 8, 4, 0.65, 0.06, 0.97),
            };

        ResolvedVisualizerSettings {
            mode: self.mode,
            fft_size,
            bar_count,
            refresh_ms,
            fft_passes,
            attack,
            decay,
            peak_decay,
            peak_hold: self.peak_hold,
            mirror: self.mirror,
            gradient: self.gradient,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LyricsSettings {
    #[serde(default = "default_auto_fetch_lyrics")]
    pub auto_fetch_lyrics: bool,
}

impl Default for LyricsSettings {
    fn default() -> Self {
        Self {
            auto_fetch_lyrics: default_auto_fetch_lyrics(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InterfaceSettings {
    #[serde(default = "default_aggressively_prefetch_artwork")]
    pub aggressively_prefetch_artwork: bool,
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            aggressively_prefetch_artwork: default_aggressively_prefetch_artwork(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PulseSettings {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub library: LibraryConfig,

    #[serde(default)]
    pub interface: InterfaceSettings,

    #[serde(default)]
    pub lyrics: LyricsSettings,

    #[serde(default)]
    pub visualizer: VisualizerSettings,
}

impl Default for PulseSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            library: LibraryConfig::default(),
            interface: InterfaceSettings::default(),
            lyrics: LyricsSettings::default(),
            visualizer: VisualizerSettings::default(),
        }
    }
}

impl PulseSettings {
    /// Loads settings from disk, or defaults when the file is missing.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when the file cannot be read or parsed.
    pub fn load(paths: &PulsePaths) -> Result<Self, DataError> {
        let path = paths.settings_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let text =
            std::fs::read_to_string(&path).map_err(|source| DataError::read(&path, source))?;
        toml::from_str(&text).map_err(|source| DataError::Parse {
            path,
            source: Box::new(source),
        })
    }

    /// Saves settings to disk, creating parent directories as needed.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when directories cannot be created or the file cannot be written.
    pub fn save(&self, paths: &PulsePaths) -> Result<(), DataError> {
        paths.ensure_all()?;
        let path = paths.settings_path();
        let text = toml::to_string_pretty(self).map_err(|source| DataError::Serialize {
            path: path.clone(),
            source: Box::new(source),
        })?;
        std::fs::write(&path, text).map_err(|source| DataError::write(path, source))
    }
}

fn default_theme() -> String {
    DEFAULT_THEME.to_string()
}

const fn default_auto_fetch_lyrics() -> bool {
    true
}

const fn default_aggressively_prefetch_artwork() -> bool {
    true
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_settings() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let paths = PulsePaths::with_roots(temp.path(), temp.path(), temp.path());
        let settings = PulseSettings {
            theme: "Custom".into(),
            library: LibraryConfig {
                include_xdg_music_dir: false,
                extra_paths: vec!["/music".into()],
                watch_debounce_ms: 500,
            },
            interface: InterfaceSettings {
                aggressively_prefetch_artwork: false,
            },
            visualizer: VisualizerSettings {
                mode: VisualizerMode::Oscilloscope,
                quality: VisualizerQuality::Ultra,
                peak_hold: false,
                mirror: false,
                gradient: true,
            },
        };

        settings.save(&paths)?;
        let loaded = PulseSettings::load(&paths)?;

        if loaded != settings {
            return Err("settings mismatch after round trip".into());
        }
        Ok(())
    }

    #[test]
    fn ultra_quality_is_heavier_than_efficient() {
        let efficient = VisualizerSettings::default().resolve();
        let ultra = VisualizerSettings {
            quality: VisualizerQuality::Ultra,
            ..VisualizerSettings::default()
        }
        .resolve();

        assert!(ultra.fft_size >= efficient.fft_size);
        assert!(ultra.bar_count > efficient.bar_count);
        assert!(ultra.fft_passes > efficient.fft_passes);
        assert!(ultra.refresh_ms <= efficient.refresh_ms);
    }
}
