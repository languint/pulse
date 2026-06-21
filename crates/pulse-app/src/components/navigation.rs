use pulse_model::{AlbumId, ArtistId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulsePage {
    #[default]
    Albums,
    Artists,
    AlbumDetail(AlbumId),
    ArtistDetail(ArtistId),
}

impl PulsePage {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Albums | Self::AlbumDetail(_) => "Albums",
            Self::Artists | Self::ArtistDetail(_) => "Artists",
        }
    }

    #[must_use]
    pub const fn is_albums_section(self) -> bool {
        matches!(self, Self::Albums | Self::AlbumDetail(_))
    }

    #[must_use]
    pub const fn is_artists_section(self) -> bool {
        matches!(self, Self::Artists | Self::ArtistDetail(_))
    }

    #[must_use]
    pub const fn album_detail(self) -> Option<AlbumId> {
        match self {
            Self::AlbumDetail(id) => Some(id),
            Self::Albums | Self::Artists | Self::ArtistDetail(_) => None,
        }
    }

    #[must_use]
    pub const fn artist_detail(self) -> Option<ArtistId> {
        match self {
            Self::ArtistDetail(id) => Some(id),
            Self::Albums | Self::Artists | Self::AlbumDetail(_) => None,
        }
    }

    /// Top-level list page for this section, used when there is no navigation history.
    #[must_use]
    pub const fn section_fallback(self) -> Self {
        match self {
            Self::Albums | Self::AlbumDetail(_) => Self::Albums,
            Self::Artists | Self::ArtistDetail(_) => Self::Artists,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_model::{AlbumId, ArtistId};

    #[test]
    fn section_fallback_returns_list_page_for_detail_views() {
        assert_eq!(
            PulsePage::AlbumDetail(AlbumId(1)).section_fallback(),
            PulsePage::Albums
        );
        assert_eq!(
            PulsePage::ArtistDetail(ArtistId(2)).section_fallback(),
            PulsePage::Artists
        );
    }
}
