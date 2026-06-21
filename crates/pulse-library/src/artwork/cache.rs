use std::path::{Path, PathBuf};

use pulse_model::ThumbnailSize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedArtworkMeta {
    pub width: u32,
    pub height: u32,
    pub extension: String,
}

#[derive(Debug, Clone)]
pub struct ArtworkCache {
    root: PathBuf,
}

impl ArtworkCache {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn default_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pulse")
            .join("artwork")
    }

    #[must_use]
    pub fn with_default_dir() -> Self {
        Self::new(Self::default_dir())
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn source_path(&self, content_hash: &str, extension: &str) -> PathBuf {
        self.root
            .join("sources")
            .join(format!("{content_hash}.{extension}"))
    }

    #[must_use]
    pub fn thumbnail_path(&self, content_hash: &str, size: ThumbnailSize) -> PathBuf {
        self.root.join("thumbnails").join(content_hash).join(format!(
            "{}.jpg",
            size.pixels()
        ))
    }

    fn meta_path(&self, content_hash: &str) -> PathBuf {
        self.root
            .join("thumbnails")
            .join(content_hash)
            .join("meta.json")
    }

    pub(crate) fn meta_path_for(&self, content_hash: &str) -> PathBuf {
        self.meta_path(content_hash)
    }

    fn thumbnails_complete(&self, content_hash: &str) -> bool {
        ThumbnailSize::all()
            .into_iter()
            .all(|size| self.thumbnail_path(content_hash, size).is_file())
    }

    fn source_file(&self, content_hash: &str, extension: &str) -> PathBuf {
        self.source_path(content_hash, extension)
    }

    /// Returns metadata when source and all thumbnails are present on disk.
    ///
    /// # Errors
    ///
    /// Returns an I/O or JSON error when metadata cannot be read or inferred.
    pub fn cached_artwork(&self, content_hash: &str) -> Result<Option<CachedArtworkMeta>, std::io::Error> {
        if !self.thumbnails_complete(content_hash) {
            return Ok(None);
        }

        let meta_path = self.meta_path(content_hash);
        if meta_path.is_file() {
            let raw = std::fs::read_to_string(&meta_path)?;
            let meta: CachedArtworkMeta = serde_json::from_str(&raw).map_err(|source| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, source)
            })?;
            if self
                .source_file(content_hash, &meta.extension)
                .is_file()
            {
                return Ok(Some(meta));
            }
            return Ok(None);
        }

        let Some((_, extension)) = self.find_source_extension(content_hash) else {
            return Ok(None);
        };

        let medium = self.thumbnail_path(content_hash, ThumbnailSize::Medium);
        let (width, height) = image::image_dimensions(&medium).map_err(|source| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, source)
        })?;
        Ok(Some(CachedArtworkMeta {
            width,
            height,
            extension,
        }))
    }

    /// Persists artwork dimensions and source extension for future rescans.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the metadata file cannot be written.
    pub fn write_cached_meta(
        &self,
        content_hash: &str,
        meta: &CachedArtworkMeta,
    ) -> Result<(), std::io::Error> {
        let path = self.meta_path(content_hash);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(meta).map_err(|source| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, source)
        })?;
        std::fs::write(path, json)
    }

    fn find_source_extension(&self, content_hash: &str) -> Option<(PathBuf, String)> {
        for extension in ["jpg", "jpeg", "png", "webp", "gif", "bmp", "bin"] {
            let path = self.source_file(content_hash, extension);
            if path.is_file() {
                return Some((path, extension.to_string()));
            }
        }
        None
    }

    /// Creates cache subdirectories for a content hash.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if a directory cannot be created.
    pub fn ensure_dirs(&self, content_hash: &str) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(self.root.join("sources"))?;
        std::fs::create_dir_all(self.root.join("thumbnails").join(content_hash))?;
        Ok(())
    }

    /// Writes `data` to `path` when the file does not already exist.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if parent directories cannot be created or the file cannot be written.
    pub fn write_if_missing(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
        if path.exists() {
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, data)
    }
}
