use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{DataError, PulsePaths};

/// User-provided metadata overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserOverrides {
    #[serde(default)]
    pub albums: HashMap<String, AlbumOverride>,

    #[serde(default)]
    pub artists: HashMap<String, ArtistOverride>,

    #[serde(default)]
    pub songs: HashMap<String, SongOverride>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AlbumOverride {
    pub title: Option<String>,
    /// Path relative to the Pulse data directory, or absolute.
    pub artwork: Option<PathBuf>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArtistOverride {
    pub name: Option<String>,
    pub artwork: Option<PathBuf>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SongOverride {
    pub title: Option<String>,
    pub artwork: Option<PathBuf>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub comment: Option<String>,
}

impl UserOverrides {
    /// Loads user metadata overrides from disk, or defaults when the file is missing.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when the file cannot be read or parsed.
    pub fn load(paths: &PulsePaths) -> Result<Self, DataError> {
        let path = paths.overrides_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let text =
            std::fs::read_to_string(&path).map_err(|source| DataError::read(&path, source))?;
        serde_json::from_str(&text).map_err(|source| DataError::Parse {
            path,
            source: Box::new(source),
        })
    }

    /// Saves user metadata overrides to disk, creating parent directories as needed.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] when directories cannot be created or the file cannot be written.
    pub fn save(&self, paths: &PulsePaths) -> Result<(), DataError> {
        paths.ensure_all()?;
        let path = paths.overrides_path();
        let text = serde_json::to_string_pretty(self).map_err(|source| DataError::Serialize {
            path: path.clone(),
            source: Box::new(source),
        })?;
        std::fs::write(&path, text).map_err(|source| DataError::write(path, source))
    }

    #[must_use]
    pub fn album(&self, key: &str) -> Option<&AlbumOverride> {
        self.albums.get(key)
    }

    #[must_use]
    pub fn artist(&self, key: &str) -> Option<&ArtistOverride> {
        self.artists.get(key)
    }

    #[must_use]
    pub fn song(&self, key: &str) -> Option<&SongOverride> {
        self.songs.get(key)
    }

    #[must_use]
    pub fn resolve_artwork<'a>(
        paths: &'a PulsePaths,
        override_path: Option<&'a Path>,
    ) -> Option<PathBuf> {
        override_path.map(|path| paths.resolve_data_path(path))
    }
}

#[must_use]
pub fn album_override_key(title: &str, album_artist: &str) -> String {
    format!("{}|{}", normalize_key(title), normalize_key(album_artist))
}

#[must_use]
pub fn artist_override_key(name: &str) -> String {
    normalize_key(name)
}

#[must_use]
pub fn song_override_key(path: &Path) -> String {
    normalize_key(&path.display().to_string())
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_override_keys() {
        assert_eq!(
            album_override_key("Abbey Road", "The Beatles"),
            "abbey road|the beatles"
        );
        assert_eq!(artist_override_key("The Beatles"), "the beatles");
    }

    #[test]
    fn round_trips_overrides() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let paths = PulsePaths::with_roots(temp.path(), temp.path(), temp.path());
        let mut overrides = UserOverrides::default();
        overrides.albums.insert(
            album_override_key("Album", "Artist"),
            AlbumOverride {
                tags: Some(vec!["favorite".into()]),
                ..AlbumOverride::default()
            },
        );

        overrides.save(&paths)?;
        let loaded = UserOverrides::load(&paths)?;

        if loaded.albums.len() != 1 {
            return Err("expected one album override".into());
        }
        if loaded
            .album("album|artist")
            .and_then(|entry| entry.tags.as_ref())
            != Some(&vec!["favorite".into()])
        {
            return Err("expected favorite tag override".into());
        }
        Ok(())
    }
}
