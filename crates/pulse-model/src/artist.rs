use crate::{artwork::ArtworkId, metadata::EntityMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtistId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub artwork_id: Option<ArtworkId>,
    pub metadata: EntityMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlbumArtists {
    Single(ArtistId),
    Various,
    Multiple(Vec<ArtistId>),
}

impl AlbumArtists {
    #[must_use]
    pub const fn single(id: ArtistId) -> Self {
        Self::Single(id)
    }

    #[must_use]
    pub fn artist_ids(&self) -> Vec<ArtistId> {
        match self {
            Self::Single(id) => vec![*id],
            Self::Various => Vec::new(),
            Self::Multiple(ids) => ids.clone(),
        }
    }

    #[must_use]
    pub const fn is_various(&self) -> bool {
        matches!(self, Self::Various)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn various_artists_has_no_ids() {
        let artists = AlbumArtists::Various;
        assert!(artists.is_various());
        assert!(artists.artist_ids().is_empty());
    }

    #[test]
    fn multiple_album_artists_lists_all() {
        let a = ArtistId(1);
        let b = ArtistId(2);
        let artists = AlbumArtists::Multiple(vec![a, b]);
        assert_eq!(artists.artist_ids(), vec![a, b]);
    }
}
