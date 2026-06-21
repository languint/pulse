use pulse_library::LibraryConfig;

use crate::{DataError, PulsePaths};

pub const DEFAULT_THEME: &str = "Pulse Dark";
pub const THEME_PULSE_DARK: &str = "Pulse Dark";
pub const THEME_PULSE_LIGHT: &str = "Pulse Light";

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
}

impl Default for PulseSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            library: LibraryConfig::default(),
            interface: InterfaceSettings::default(),
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

const fn default_aggressively_prefetch_artwork() -> bool {
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
        };

        settings.save(&paths)?;
        let loaded = PulseSettings::load(&paths)?;

        if loaded != settings {
            return Err("settings mismatch after round trip".into());
        }
        Ok(())
    }
}
