#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    pub start_ms: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lyrics {
    Plain(String),
    Synced(Vec<LyricLine>),
}

impl Lyrics {
    #[must_use]
    pub const fn is_synced(&self) -> bool {
        matches!(self, Self::Synced(_))
    }

    /// Index of the active synced line for `position_ms`, with a small lead-in.
    #[must_use]
    pub fn active_line_index(&self, position_ms: u64) -> Option<usize> {
        let Self::Synced(lines) = self else {
            return None;
        };

        if lines.is_empty() {
            return None;
        }

        const LEAD_MS: u32 = 200;
        let position = u32::try_from(position_ms).unwrap_or(u32::MAX).saturating_add(LEAD_MS);
        lines.iter().rposition(|line| line.start_ms <= position)
    }

    #[must_use]
    pub fn plain_text(&self) -> Option<&str> {
        match self {
            Self::Plain(text) => Some(text.as_str()),
            Self::Synced(_) => None,
        }
    }

    #[must_use]
    pub fn synced_lines(&self) -> Option<&[LyricLine]> {
        match self {
            Self::Synced(lines) => Some(lines.as_slice()),
            Self::Plain(_) => None,
        }
    }

    #[must_use]
    pub fn display_text(&self) -> String {
        match self {
            Self::Plain(text) => text.clone(),
            Self::Synced(lines) => lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}
