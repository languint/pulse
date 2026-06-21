use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("library root does not exist: {0}")]
    RootMissing(PathBuf),

    #[error("failed to watch library path {path}: {source}")]
    Watch {
        path: PathBuf,
        source: notify::Error,
    },

    #[error("filesystem watcher error: {0}")]
    Notify(#[from] notify::Error),

    #[error("failed to read metadata from {path}: {source}")]
    Metadata {
        path: PathBuf,
        source: lofty::error::LoftyError,
    },

    #[error("failed to decode artwork image: {source}")]
    ArtworkDecode { source: image::ImageError },

    #[error("failed to encode artwork thumbnail: {source}")]
    ArtworkEncode { source: image::ImageError },

    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}
