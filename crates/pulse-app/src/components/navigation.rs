#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulsePage {
    #[default]
    Albums,
    Artists,
}

impl PulsePage {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Albums => "Albums",
            Self::Artists => "Artists",
        }
    }
}
