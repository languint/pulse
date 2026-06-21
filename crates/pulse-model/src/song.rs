use std::path::PathBuf;

use crate::{
    album::AlbumId, artist::ArtistId, artwork::ArtworkReference, metadata::EntityMetadata,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SongId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub id: SongId,
    pub title: String,
    pub album_id: Option<AlbumId>,

    pub track_artists: Vec<ArtistId>,
    pub track_number: Option<u16>,
    pub disc_number: Option<u16>,
    pub duration_ms: u32,
    pub path: PathBuf,
    pub artwork: Option<ArtworkReference>,
    pub metadata: EntityMetadata,
}
