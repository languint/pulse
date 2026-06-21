use pulse_model::AlbumId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulsePage {
    #[default]
    Albums,
    Artists,
    AlbumDetail(AlbumId),
}

impl PulsePage {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Albums | Self::AlbumDetail(_) => "Albums",
            Self::Artists => "Artists",
        }
    }

    #[must_use]
    pub const fn is_albums_section(self) -> bool {
        matches!(self, Self::Albums | Self::AlbumDetail(_))
    }

    #[must_use]
    pub const fn album_detail(self) -> Option<AlbumId> {
        match self {
            Self::AlbumDetail(id) => Some(id),
            Self::Albums | Self::Artists => None,
        }
    }
}
