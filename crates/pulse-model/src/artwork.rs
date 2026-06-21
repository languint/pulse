use std::path::PathBuf;

use crate::song::SongId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtworkId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artwork {
    pub id: ArtworkId,
    pub source: ArtworkSource,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub dominant_color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtworkSource {
    Embedded { song_id: SongId },
    File { path: PathBuf },
    Cached { path: PathBuf },
    Remote { url: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtworkReference {
    Inherit,
    Custom(ArtworkId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkThumbnail {
    pub artwork_id: ArtworkId,
    pub size: ThumbnailSize,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThumbnailSize {
    /// 64x64 pixels
    Small,
    /// 256x256 pixels
    Medium,
    /// 1024x1024 pixels
    Large,
}

impl ThumbnailSize {
    #[must_use]
    pub const fn pixels(self) -> u32 {
        match self {
            Self::Small => 64,
            Self::Medium => 256,
            Self::Large => 1024,
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Small, Self::Medium, Self::Large]
    }
}
