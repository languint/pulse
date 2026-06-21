use std::{path::PathBuf, time::Instant};

use crate::{
    ArtworkCache,
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
    artwork_cache: ArtworkCache,
    watcher: Option<LibraryWatcher>,
}

impl MusicLibrary {
    #[must_use]
    pub fn new(config: LibraryConfig, artwork_cache_dir: impl Into<PathBuf>) -> Self {
        let roots = resolve_roots(&config);
        Self {
            config,
            roots,
            store: LibraryStore::default(),
            artwork_cache: ArtworkCache::new(artwork_cache_dir),
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

    #[must_use]
    pub const fn artwork_cache(&self) -> &ArtworkCache {
        &self.artwork_cache
    }

    /// Rescan configured library roots and rebuild the in-memory index.
    ///
    /// # Errors
    ///
    /// Returns [`LibraryError::RootMissing`] when a configured root path does not exist.
    pub fn scan(&mut self) -> Result<ScanSummary, LibraryError> {
        self.roots = resolve_roots_or_error(&self.config)?;
        self.preload_artwork_from_disk()?;

        let start = Instant::now();
        let summary = scan_roots(&mut self.store, &self.roots, &self.artwork_cache);
        let duration = start.elapsed();

        tracing::info!("scan completed in {}ms", duration.as_millis());

        Ok(summary)
    }

    fn preload_artwork_from_disk(&mut self) -> Result<(), LibraryError> {
        let root = self.artwork_cache.root().join("thumbnails");
        let entries = match std::fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(LibraryError::Io {
                    path: root,
                    source,
                });
            }
        };

        for entry in entries.filter_map(Result::ok) {
            if !entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
                continue;
            }

            let hash = entry.file_name().to_string_lossy().into_owned();
            let Some(meta) = self
                .artwork_cache
                .cached_artwork(&hash)
                .map_err(|source| LibraryError::Io {
                    path: self.artwork_cache.meta_path_for(&hash),
                    source,
                })?
            else {
                continue;
            };

            self.store
                .preload_artwork(&self.artwork_cache, &hash, &meta);
        }

        Ok(())
    }

    /// Watch library roots for filesystem changes and invoke `on_rescan` after debouncing.
    ///
    /// # Errors
    ///
    /// Returns [`LibraryError::RootMissing`] or a watcher error when setup fails.
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
        tracing::info!(
            roots = self.roots.len(),
            "library filesystem watcher started"
        );
        Ok(())
    }

    pub fn stop_watching(&mut self) {
        self.watcher = None;
    }

    /// Replace library configuration and optionally trigger a rescan.
    ///
    /// # Errors
    ///
    /// Returns [`LibraryError::RootMissing`] when a configured root path does not exist,
    /// or a scan error when `rescan` is true.
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
        Self::new(LibraryConfig::default(), ArtworkCache::default_dir())
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

        let mut library = MusicLibrary::new(config, temp.path());
        let summary = library.scan()?;

        if summary.files_seen != 0 {
            return Err("expected no files seen".into());
        }
        if !library.store().songs().is_empty() {
            return Err("expected empty song store".into());
        }
        Ok(())
    }
}
