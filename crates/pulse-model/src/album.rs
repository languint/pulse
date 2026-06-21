use crate::{
    artist::AlbumArtists,
    artwork::ArtworkId,
    metadata::EntityMetadata,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AlbumId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    pub id: AlbumId,
    pub title: String,
    pub album_artists: AlbumArtists,
    pub year: Option<u16>,
    pub artwork_id: Option<ArtworkId>,
    pub metadata: EntityMetadata,
}
