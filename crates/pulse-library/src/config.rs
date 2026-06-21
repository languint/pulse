use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryConfig {
    #[serde(default = "default_include_xdg_music_dir")]
    pub include_xdg_music_dir: bool,

    #[serde(default)]
    pub extra_paths: Vec<PathBuf>,

    #[serde(default = "default_debounce_ms")]
    pub watch_debounce_ms: u64,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            include_xdg_music_dir: default_include_xdg_music_dir(),
            extra_paths: Vec::new(),
            watch_debounce_ms: default_debounce_ms(),
        }
    }
}

impl LibraryConfig {
    #[must_use]
    pub fn with_extra_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.extra_paths.push(path.into());
        self
    }

    #[must_use]
    pub const fn watch_debounce(&self) -> Duration {
        Duration::from_millis(self.watch_debounce_ms)
    }
}

const fn default_include_xdg_music_dir() -> bool {
    true
}

const fn default_debounce_ms() -> u64 {
    750
}
