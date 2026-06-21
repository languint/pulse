use std::thread;

use gpui::{App, Global, UpdateGlobal};
use pulse_library::{LibraryConfig, LibraryError, MusicLibrary, ScanSummary};
use tracing::{error, info, warn};

use crate::config::PulseConfig;

pub struct PulseLibrary(MusicLibrary);

impl PulseLibrary {
    #[must_use]
    pub fn new(config: LibraryConfig) -> Self {
        Self(MusicLibrary::new(config))
    }

    pub fn init(cx: &mut App, config: LibraryConfig) {
        let roots: Vec<_> = MusicLibrary::new(config.clone()).roots().to_vec();

        Self::set_global(cx, Self::new(config.clone()));

        info!(roots = roots.len(), ?roots, "initializing music library");

        Self::spawn_scan(cx, config);
    }

    pub fn apply_config(cx: &mut App, config: LibraryConfig) {
        PulseConfig::update_global(cx, |pulse_config, _| {
            pulse_config.library = config.clone();
        });

        Self::set_global(cx, Self::new(config.clone()));

        info!(?config, "updating music library roots");

        Self::spawn_scan(cx, config);
    }

    fn spawn_scan(cx: &mut App, config: LibraryConfig) {
        cx.spawn(async move |cx| {
            let scanned = cx
                .background_executor()
                .spawn(async move {
                    thread::spawn(move || {
                        let mut library = MusicLibrary::new(config);
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

impl Global for PulseLibrary {}

fn log_scan_complete(summary: &ScanSummary) {
    info!(
        roots = summary.roots.len(),
        files_seen = summary.files_seen,
        songs = summary.songs_imported,
        skipped = summary.skipped,
        "music library scan complete"
    );
}

fn log_scan_error(error: &LibraryError) {
    if let LibraryError::RootMissing(path) = error {
        warn!(?path, "music library root is missing; skipping scan");
    } else {
        error!(%error, "music library scan failed");
    }
}
