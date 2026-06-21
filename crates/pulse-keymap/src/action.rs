use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeymapAction {
    ToggleFullscreen,
    Quit,
}

impl KeymapAction {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::ToggleFullscreen => "toggle_fullscreen",
            Self::Quit => "quit",
        }
    }
}
