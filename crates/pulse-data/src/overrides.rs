use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{DataError, PulsePaths};

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

impl AlbumOverride {
    #[must_use]
    pub const fn is_metadata_empty(&self) -> bool {
        self.title.is_none()
            && self.artwork.is_none()
            && self.genres.is_none()
            && self.tags.is_none()
            && self.comment.is_none()
    }

    /// User-added labels from both `genres` and `tags` override fields.
    #[must_use]
    pub fn user_labels(&self) -> Vec<String> {
        override_entry_labels(self.genres.as_ref(), self.tags.as_ref())
    }
}

impl ArtistOverride {
    #[must_use]
    pub const fn is_metadata_empty(&self) -> bool {
        self.name.is_none()
            && self.artwork.is_none()
            && self.genres.is_none()
            && self.tags.is_none()
            && self.comment.is_none()
    }

    /// User-added labels from both `genres` and `tags` override fields.
    #[must_use]
    pub fn user_labels(&self) -> Vec<String> {
        override_entry_labels(self.genres.as_ref(), self.tags.as_ref())
    }
}

impl SongOverride {
    /// User-added labels from both `genres` and `tags` override fields.
    #[must_use]
    pub fn user_labels(&self) -> Vec<String> {
        override_entry_labels(self.genres.as_ref(), self.tags.as_ref())
    }
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

    /// All user-added labels across albums, artists, and songs.
    #[must_use]
    pub fn all_user_labels(&self) -> Vec<String> {
        let mut labels = Vec::new();
        labels.extend(self.albums.values().flat_map(AlbumOverride::user_labels));
        labels.extend(self.artists.values().flat_map(ArtistOverride::user_labels));
        labels.extend(self.songs.values().flat_map(SongOverride::user_labels));
        dedupe_user_labels(&labels)
    }

    /// Replaces user-added album labels, persisted under `tags` in `overrides.json`.
    pub fn set_album_user_labels(&mut self, key: String, labels: &[String]) {
        let labels = dedupe_user_labels(labels);

        if labels.is_empty() {
            if let Some(entry) = self.albums.get_mut(&key) {
                entry.tags = None;
                entry.genres = None;
                if entry.is_metadata_empty() {
                    self.albums.remove(&key);
                }
            }
            return;
        }

        let entry = self.albums.entry(key).or_default();
        entry.tags = Some(labels);
        entry.genres = None;
    }

    /// Replaces custom album artwork stored under `artwork` in `overrides.json`.
    pub fn set_album_artwork(&mut self, key: String, artwork: Option<PathBuf>) {
        if artwork.is_none() {
            if let Some(entry) = self.albums.get_mut(&key) {
                entry.artwork = None;
                if entry.is_metadata_empty() {
                    self.albums.remove(&key);
                }
            }
            return;
        }

        let entry = self.albums.entry(key).or_default();
        entry.artwork = artwork;
    }

    /// Replaces custom artist artwork stored under `artwork` in `overrides.json`.
    pub fn set_artist_artwork(&mut self, key: String, artwork: Option<PathBuf>) {
        if artwork.is_none() {
            if let Some(entry) = self.artists.get_mut(&key) {
                entry.artwork = None;
                if entry.is_metadata_empty() {
                    self.artists.remove(&key);
                }
            }
            return;
        }

        let entry = self.artists.entry(key).or_default();
        entry.artwork = artwork;
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

fn dedupe_user_labels(labels: &[String]) -> Vec<String> {
    collect_user_labels(labels.iter().map(String::as_str))
}

fn override_entry_labels(genres: Option<&Vec<String>>, tags: Option<&Vec<String>>) -> Vec<String> {
    collect_user_labels(
        genres
            .into_iter()
            .flatten()
            .chain(tags.into_iter().flatten())
            .map(String::as_str),
    )
}

fn collect_user_labels<'a>(labels: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut unique = Vec::new();

    for label in labels {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            continue;
        }

        let key = normalize_key(trimmed);
        if seen.insert(key) {
            unique.push(trimmed.to_string());
        }
    }

    unique.sort_by_key(|label| label.to_ascii_lowercase());
    unique
}

#[must_use]
pub fn album_user_labels(override_entry: Option<&AlbumOverride>) -> Vec<String> {
    override_entry
        .map(AlbumOverride::user_labels)
        .unwrap_or_default()
}

#[must_use]
pub fn artist_user_labels(override_entry: Option<&ArtistOverride>) -> Vec<String> {
    override_entry
        .map(ArtistOverride::user_labels)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn stable_override_keys() {
        assert_eq!(
            album_override_key("Abbey Road", "The Beatles"),
            "abbey road|the beatles"
        );
        assert_eq!(artist_override_key("The Beatles"), "the beatles");
    }

    #[test]
    fn merges_user_genres_and_tags() {
        let entry = AlbumOverride {
            genres: Some(vec!["Rock".into(), "rock".into()]),
            tags: Some(vec!["Favorite".into()]),
            ..AlbumOverride::default()
        };

        assert_eq!(
            entry.user_labels(),
            vec!["Favorite".to_string(), "Rock".to_string()]
        );
    }

    #[test]
    fn all_user_labels_collects_across_entities() {
        let mut overrides = UserOverrides::default();
        overrides.albums.insert(
            "a|b".into(),
            AlbumOverride {
                tags: Some(vec!["Album Tag".into()]),
                ..AlbumOverride::default()
            },
        );
        overrides.artists.insert(
            "artist".into(),
            ArtistOverride {
                tags: Some(vec!["Artist Tag".into(), "album tag".into()]),
                ..ArtistOverride::default()
            },
        );

        let labels = overrides.all_user_labels();
        assert_eq!(labels, vec!["Album Tag", "Artist Tag"]);
    }

    #[test]
    fn set_album_user_labels_clears_empty_entry() {
        let mut overrides = UserOverrides::default();
        let key = album_override_key("Album", "Artist");
        overrides.set_album_user_labels(key.clone(), &["Synthwave".into()]);
        overrides.set_album_user_labels(key.clone(), &[]);

        assert!(overrides.album(&key).is_none());
    }

    #[test]
    fn set_artist_artwork_clears_empty_entry() {
        let mut overrides = UserOverrides::default();
        let key = artist_override_key("Artist");
        overrides.set_artist_artwork(
            key.clone(),
            Some(PathBuf::from("artwork/custom/artist.png")),
        );
        overrides.set_artist_artwork(key.clone(), None);

        assert!(overrides.artist(&key).is_none());
    }

    #[test]
    fn set_album_artwork_clears_empty_entry() {
        let mut overrides = UserOverrides::default();
        let key = album_override_key("Album", "Artist");
        overrides.set_album_artwork(
            key.clone(),
            Some(PathBuf::from("artwork/custom/album.png")),
        );
        overrides.set_album_artwork(key.clone(), None);

        assert!(overrides.album(&key).is_none());
    }
}
