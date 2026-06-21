use std::path::PathBuf;

use gpui::{App, UpdateGlobal};
use pulse_library::{LibraryConfig, MusicLibrary, ScanSummary};
use crate::data::DataPaths;
use tracing::{error, info, warn};

use crate::config::PulseConfig;
use crate::data::persist_settings;

pub struct PulseLibrary(MusicLibrary);

impl PulseLibrary {
    #[must_use]
    pub fn new(config: LibraryConfig, artwork_cache_dir: PathBuf) -> Self {
        Self(MusicLibrary::new(config, artwork_cache_dir))
    }

    pub fn init(cx: &mut App, config: LibraryConfig, artwork_cache_dir: PathBuf) {
        let roots: Vec<_> = pulse_library::resolve_roots(&config);

        Self::set_global(cx, Self::new(config.clone(), artwork_cache_dir));

        info!(roots = roots.len(), ?roots, "initializing music library");

        Self::spawn_scan(cx, config);
    }

    pub fn apply_config(cx: &mut App, config: LibraryConfig) {
        let paths = cx.global::<DataPaths>().clone();

        PulseConfig::update_global(cx, |pulse_config, _| {
            pulse_config.library = config.clone();
        });

        if let Err(error) = persist_settings(&paths, &cx.global::<PulseConfig>().to_settings()) {
            error!(%error, "failed to save library settings");
        }

        let artwork_cache_dir = paths.artwork_cache_dir();
        Self::set_global(cx, Self::new(config.clone(), artwork_cache_dir));

        info!(?config, "updating music library roots");

        Self::spawn_scan(cx, config);
    }

    fn spawn_scan(cx: &App, config: LibraryConfig) {
        let artwork_cache_dir = cx.global::<DataPaths>().artwork_cache_dir();

        cx.spawn(async move |cx| {
            let scanned = cx
                .background_executor()
                .spawn(async move {
                    std::thread::spawn(move || {
                        let mut library = MusicLibrary::new(config, artwork_cache_dir);
                        let result = library.scan();
                        (library, result)
                    })
                    .join()
                    .ok()
                })
                .await;

            let Some((library, result)) = scanned else {
                error!("music library scan thread failed to join");
                return;
            };

            cx.update(|app| {
                Self::set_global(app, Self(library));
                match result {
                    Ok(summary) => log_scan_complete(&summary),
                    Err(error) => log_scan_error(&error),
                }
                app.refresh_windows();
            });
        })
        .detach();
    }

    #[must_use]
    pub const fn inner(&self) -> &MusicLibrary {
        &self.0
    }

    #[must_use]
    pub const fn inner_mut(&mut self) -> &mut MusicLibrary {
        &mut self.0
    }
}

impl gpui::Global for PulseLibrary {}

fn log_scan_complete(summary: &ScanSummary) {
    info!(
        roots = summary.roots.len(),
        files_seen = summary.files_seen,
        songs = summary.songs_imported,
        skipped = summary.skipped,
        "music library scan complete"
    );
}

fn log_scan_error(error: &pulse_library::LibraryError) {
    if let pulse_library::LibraryError::RootMissing(path) = error {
        warn!(?path, "music library root is missing; skipping scan");
    } else {
        error!(%error, "music library scan failed");
    }
}
