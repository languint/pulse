mod album;
mod artist;
mod artwork;
mod metadata;
mod song;

pub use album::{Album, AlbumId};
pub use artist::{AlbumArtists, Artist, ArtistId};
pub use artwork::{
    Artwork, ArtworkId, ArtworkReference, ArtworkSource, ArtworkThumbnail, ThumbnailSize,
};
pub use metadata::EntityMetadata;
pub use song::{Song, SongId};
