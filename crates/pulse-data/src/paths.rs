use std::path::{Path, PathBuf};

use crate::DataError;

/// Platform-specific Pulse config, data, and cache directories.
#[derive(Debug, Clone)]
pub struct PulsePaths {
    config: PathBuf,
    data: PathBuf,
    cache: PathBuf,
}

impl PulsePaths {
    #[must_use]
    pub fn platform_default() -> Self {
        Self {
            config: platform_dir(dirs::config_dir, "config"),
            data: platform_dir(dirs::data_local_dir, "data"),
            cache: platform_dir(dirs::cache_dir, "cache"),
        }
    }

    #[must_use]
    pub fn with_roots(
        config_dir: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config: config_dir.into(),
            data: data_dir.into(),
            cache: cache_dir.into(),
        }
    }

    /// Creates config, data, cache, and nested subdirectories.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when a directory cannot be created.
    pub fn ensure_all(&self) -> Result<(), DataError> {
        std::fs::create_dir_all(&self.config)
            .map_err(|source| DataError::write(&self.config, source))?;
        std::fs::create_dir_all(&self.data)
            .map_err(|source| DataError::write(&self.data, source))?;
        std::fs::create_dir_all(&self.cache)
            .map_err(|source| DataError::write(&self.cache, source))?;
        std::fs::create_dir_all(self.themes_dir())
            .map_err(|source| DataError::write(self.themes_dir(), source))?;
        std::fs::create_dir_all(self.custom_artwork_dir())
            .map_err(|source| DataError::write(self.custom_artwork_dir(), source))?;
        std::fs::create_dir_all(self.artwork_cache_dir())
            .map_err(|source| DataError::write(self.artwork_cache_dir(), source))?;
        Ok(())
    }

    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config
    }

    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data
    }

    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache
    }

    #[must_use]
    pub fn settings_path(&self) -> PathBuf {
        self.config.join("settings.toml")
    }

    #[must_use]
    pub fn keymap_path(&self) -> PathBuf {
        self.config.join("keymap.json")
    }

    #[must_use]
    pub fn themes_dir(&self) -> PathBuf {
        self.data.join("themes")
    }

    #[must_use]
    pub fn overrides_path(&self) -> PathBuf {
        self.data.join("overrides.json")
    }

    #[must_use]
    pub fn custom_artwork_dir(&self) -> PathBuf {
        self.data.join("artwork").join("custom")
    }

    #[must_use]
    pub fn artwork_cache_dir(&self) -> PathBuf {
        self.cache.join("artwork")
    }

    /// Resolve a path stored in user data (overrides, etc.) relative to [`Self::data_dir`].
    #[must_use]
    pub fn resolve_data_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.data.join(path)
        }
    }
}

fn platform_dir(lookup: fn() -> Option<PathBuf>, kind: &'static str) -> PathBuf {
    lookup().map_or_else(
        || PathBuf::from(".").join(kind).join("pulse"),
        |dir| dir.join("pulse"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_paths_under_roots() {
        let paths = PulsePaths::with_roots("/cfg", "/data", "/cache");

        assert_eq!(paths.settings_path(), PathBuf::from("/cfg/settings.toml"));
        assert_eq!(paths.keymap_path(), PathBuf::from("/cfg/keymap.json"));
        assert_eq!(paths.themes_dir(), PathBuf::from("/data/themes"));
        assert_eq!(
            paths.overrides_path(),
            PathBuf::from("/data/overrides.json")
        );
        assert_eq!(
            paths.custom_artwork_dir(),
            PathBuf::from("/data/artwork/custom")
        );
        assert_eq!(paths.artwork_cache_dir(), PathBuf::from("/cache/artwork"));
    }

    #[test]
    fn resolves_relative_data_paths() {
        let paths = PulsePaths::with_roots("/cfg", "/data", "/cache");
        assert_eq!(
            paths.resolve_data_path(Path::new("artwork/custom/cover.jpg")),
            PathBuf::from("/data/artwork/custom/cover.jpg")
        );
    }
}
