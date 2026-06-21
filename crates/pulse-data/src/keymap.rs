use std::collections::HashMap;
use std::path::Path;

use pulse_keymap::{KeymapAction, PulseKeymap};

use crate::{DataError, PulsePaths};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KeymapFile {
    #[serde(default)]
    pub bindings: HashMap<KeymapAction, Vec<String>>,
}

impl KeymapFile {
    /// Loads keymap bindings from the default path under `paths`.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when the file cannot be read or parsed.
    pub fn load(paths: &PulsePaths) -> Result<Self, DataError> {
        Self::load_from(paths.keymap_path())
    }

    /// Loads keymap bindings from `path`, or defaults when the file is missing.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when the file cannot be read or parsed.
    pub fn load_from(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = std::fs::read_to_string(path).map_err(|source| DataError::read(path, source))?;
        serde_json::from_str(&text).map_err(|source| DataError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })
    }

    /// Saves keymap bindings to the default path under `paths`.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when the file cannot be written.
    pub fn save(&self, paths: &PulsePaths) -> Result<(), DataError> {
        self.save_to(paths.keymap_path())
    }

    /// Saves keymap bindings to `path`.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when parent directories cannot be created or the file cannot be written.
    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), DataError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| DataError::write(parent, source))?;
        }

        let text = serde_json::to_string_pretty(self).map_err(|source| DataError::Serialize {
            path: path.to_path_buf(),
            source: Box::new(source),
        })?;
        std::fs::write(path, text).map_err(|source| DataError::write(path, source))
    }

    #[must_use]
    pub fn into_keymap(self) -> PulseKeymap {
        let mut keymap = PulseKeymap::default();
        keymap.apply_overrides(&self.bindings);
        keymap
    }

    #[must_use]
    pub fn from_keymap(keymap: &PulseKeymap) -> Self {
        Self {
            bindings: keymap.bindings().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_keymap_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("keymap.json");
        let file = KeymapFile {
            bindings: HashMap::from([(
                KeymapAction::Quit,
                vec!["ctrl-shift-q".into()],
            )]),
        };

        file.save_to(&path)?;
        let loaded = KeymapFile::load_from(&path)?;

        if loaded.bindings != file.bindings {
            return Err("bindings mismatch after round trip".into());
        }
        Ok(())
    }
}
