use pulse_model::{AlbumId, ArtistId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulsePage {
    #[default]
    Albums,
    Artists,
    Visualizer,
    AlbumDetail(AlbumId),
    ArtistDetail(ArtistId),
}

impl PulsePage {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Albums | Self::AlbumDetail(_) => "Albums",
            Self::Artists | Self::ArtistDetail(_) => "Artists",
            Self::Visualizer => "Visualizer",
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
    pub const fn is_visualizer(self) -> bool {
        matches!(self, Self::Visualizer)
    }

    #[must_use]
    pub const fn album_detail(self) -> Option<AlbumId> {
        match self {
            Self::AlbumDetail(id) => Some(id),
            Self::Albums | Self::Artists | Self::ArtistDetail(_) | Self::Visualizer => None,
        }
    }

    #[must_use]
    pub const fn artist_detail(self) -> Option<ArtistId> {
        match self {
            Self::ArtistDetail(id) => Some(id),
            Self::Albums | Self::Artists | Self::AlbumDetail(_) | Self::Visualizer => None,
        }
    }

    #[must_use]
    pub const fn section_fallback(self) -> Self {
        match self {
            Self::Albums | Self::AlbumDetail(_) => Self::Albums,
            Self::Artists | Self::ArtistDetail(_) => Self::Artists,
            Self::Visualizer => Self::Albums,
        }
    }

    #[must_use]
    pub fn breadcrumb_trail(self) -> Vec<Self> {
        match self {
            Self::Albums => vec![Self::Albums],
            Self::Artists => vec![Self::Artists],
            Self::Visualizer => vec![Self::Visualizer],
            Self::AlbumDetail(id) => vec![Self::Albums, Self::AlbumDetail(id)],
            Self::ArtistDetail(id) => vec![Self::Artists, Self::ArtistDetail(id)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_model::{AlbumId, ArtistId};

    #[test]
    fn breadcrumb_trail_includes_section_and_detail() {
        assert_eq!(
            PulsePage::AlbumDetail(AlbumId(1)).breadcrumb_trail(),
            vec![PulsePage::Albums, PulsePage::AlbumDetail(AlbumId(1))]
        );
        assert_eq!(
            PulsePage::ArtistDetail(ArtistId(2)).breadcrumb_trail(),
            vec![PulsePage::Artists, PulsePage::ArtistDetail(ArtistId(2))]
        );
    }

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
