#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityMetadata {
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub comment: Option<String>,
}

impl EntityMetadata {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
