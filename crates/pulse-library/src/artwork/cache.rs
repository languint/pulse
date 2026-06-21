use std::path::{Path, PathBuf};

use pulse_model::ThumbnailSize;

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
