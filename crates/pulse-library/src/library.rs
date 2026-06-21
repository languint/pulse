use std::path::PathBuf;

use tracing::info;

use crate::{
    config::LibraryConfig,
    error::LibraryError,
    roots::{resolve_roots, resolve_roots_or_error},
    scan::{ScanSummary, scan_roots},
    store::LibraryStore,
    watch::LibraryWatcher,
};

pub struct MusicLibrary {
    config: LibraryConfig,
    roots: Vec<PathBuf>,
    store: LibraryStore,
    watcher: Option<LibraryWatcher>,
}

impl MusicLibrary {
    #[must_use]
    pub fn new(config: LibraryConfig) -> Self {
        let roots = resolve_roots(&config);
        Self {
            config,
            roots,
            store: LibraryStore::default(),
            watcher: None,
        }
    }

    #[must_use]
    pub const fn config(&self) -> &LibraryConfig {
        &self.config
    }

    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    #[must_use]
    pub const fn store(&self) -> &LibraryStore {
        &self.store
    }

    pub fn scan(&mut self) -> Result<ScanSummary, LibraryError> {
        self.roots = resolve_roots_or_error(&self.config)?;
        let summary = scan_roots(&mut self.store, &self.roots);
        info!(
            roots = self.roots.len(),
            songs = summary.songs_imported,
            skipped = summary.skipped,
            "music library scan complete"
        );
        Ok(summary)
    }

    pub fn start_watching(
        &mut self,
        on_rescan: impl FnMut() + Send + 'static,
    ) -> Result<(), LibraryError> {
        self.stop_watching();

        if self.roots.is_empty() {
            self.roots = resolve_roots_or_error(&self.config)?;
        }

        let watcher = LibraryWatcher::new(&self.roots, self.config.watch_debounce(), on_rescan)?;

        self.watcher = Some(watcher);
        info!(
            roots = self.roots.len(),
            "library filesystem watcher started"
        );
        Ok(())
    }

    pub fn stop_watching(&mut self) {
        self.watcher = None;
    }

    pub fn set_config(&mut self, config: LibraryConfig, rescan: bool) -> Result<(), LibraryError> {
        self.config = config;
        self.roots = resolve_roots_or_error(&self.config)?;
        self.watcher = None;

        if rescan {
            self.scan()?;
        }

        Ok(())
    }
}

impl Default for MusicLibrary {
    fn default() -> Self {
        Self::new(LibraryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let config = LibraryConfig {
            include_xdg_music_dir: false,
            extra_paths: vec![temp.path().to_path_buf()],
            ..LibraryConfig::default()
        };

        let mut library = MusicLibrary::new(config);
        let summary = library.scan()?;

        assert_eq!(summary.files_seen, 0);
        assert!(library.store().songs().is_empty());
        Ok(())
    }
}
