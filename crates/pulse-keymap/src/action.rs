use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeymapAction {
    ToggleFullscreen,
    Quit,
    ManageLibraryRoots,
    MediaPlayPause,
    MediaNextTrack,
    MediaPreviousTrack,
}

impl KeymapAction {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::ToggleFullscreen => "toggle_fullscreen",
            Self::Quit => "quit",
            Self::ManageLibraryRoots => "manage_library_roots",
            Self::MediaPlayPause => "media_play_pause",
            Self::MediaNextTrack => "media_next_track",
            Self::MediaPreviousTrack => "media_previous_track",
        }
    }
}
