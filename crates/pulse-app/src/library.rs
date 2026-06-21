use std::{
    path::PathBuf,
    sync::mpsc::{self, SyncSender, TryRecvError},
    time::Duration,
};

use gpui::{App, AsyncApp, UpdateGlobal};
use pulse_library::{LibraryConfig, MusicLibrary, ScanSummary};
use pulse_runtime::Tokio;

use crate::config::PulseConfig;
use crate::data::{DataPaths, persist_settings};

pub struct PulseLibrary(MusicLibrary);

pub struct LibraryScanCoordinator {
    tx: SyncSender<()>,
}

impl LibraryScanCoordinator {
    fn install(cx: &App) -> Self {
        let (tx, rx) = mpsc::sync_channel(64);

        cx.spawn(async move |async_cx| {
            loop {
                async_cx
                    .background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                if rx.try_recv() == Err(TryRecvError::Empty) {
                    continue;
                }

                while rx.try_recv().is_ok() {}

                tracing::info!("library files changed, rescanning");
                PulseLibrary::schedule_background_scan(async_cx);
            }
        })
        .detach();

        Self { tx }
    }
}

impl PulseLibrary {
    #[must_use]
    pub fn new(config: LibraryConfig, artwork_cache_dir: PathBuf) -> Self {
        Self(MusicLibrary::new(config, artwork_cache_dir))
    }

    pub fn init(cx: &mut App, config: LibraryConfig, artwork_cache_dir: PathBuf) {
        let roots: Vec<_> = pulse_library::resolve_roots(&config);

        LibraryScanCoordinator::set_global(cx, LibraryScanCoordinator::install(cx));
        Self::set_global(cx, Self::new(config, artwork_cache_dir));

        tracing::info!(roots = roots.len(), ?roots, "initializing music library");

        Self::spawn_scan(cx);
    }

    pub fn apply_config(cx: &mut App, config: LibraryConfig) {
        let paths = cx.global::<DataPaths>().clone();

        PulseConfig::update_global(cx, |pulse_config, _| {
            pulse_config.library = config.clone();
        });

        if let Err(error) = persist_settings(&paths, &cx.global::<PulseConfig>().to_settings()) {
            tracing::error!(%error, "failed to save library settings");
        }

        tracing::info!(?config, "updating music library roots");

        if let Err(error) =
            Self::update_global(cx, |library, _| library.0.set_config(config, false))
        {
            tracing::error!(%error, "failed to update library configuration");
        }

        Self::spawn_scan(cx);
    }

    fn spawn_scan(cx: &App) {
        let (config, artwork_cache_dir) = Self::scan_inputs(cx);
        let handle = Tokio::handle(cx);

        cx.spawn(async move |async_cx| {
            let scanned = handle
                .spawn_blocking(move || {
                    let mut library = MusicLibrary::new(config, artwork_cache_dir);
                    let result = library.scan();
                    (library, result)
                })
                .await;

            Self::apply_scan(async_cx, scanned);
        })
        .detach();
    }

    fn schedule_background_scan(async_cx: &AsyncApp) {
        let (config, artwork_cache_dir) = async_cx.update(|app| Self::scan_inputs(app));
        let handle = Tokio::handle(async_cx);

        async_cx
            .spawn(async move |async_cx| {
                let scanned = handle
                    .spawn_blocking(move || {
                        let mut library = MusicLibrary::new(config, artwork_cache_dir);
                        let result = library.scan();
                        (library, result)
                    })
                    .await;

                Self::apply_scan(async_cx, scanned);
            })
            .detach();
    }

    fn scan_inputs(cx: &App) -> (LibraryConfig, PathBuf) {
        (
            cx.global::<PulseConfig>().library.clone(),
            cx.global::<DataPaths>().artwork_cache_dir(),
        )
    }

    fn apply_scan(
        async_cx: &AsyncApp,
        scanned: Result<
            (
                MusicLibrary,
                Result<ScanSummary, pulse_library::LibraryError>,
            ),
            pulse_runtime::JoinError,
        >,
    ) {
        let Ok((library, result)) = scanned else {
            tracing::error!("music library scan task failed");
            return;
        };

        async_cx.update(|app| {
            Self::set_global(app, Self(library));
            match result {
                Ok(summary) => log_scan_complete(&summary),
                Err(error) => log_scan_error(&error),
            }
            Self::ensure_watcher(app);
            app.refresh_windows();
        });
    }

    fn ensure_watcher(app: &mut App) {
        let rescan = app.global::<LibraryScanCoordinator>().tx.clone();

        Self::update_global(app, |library, _| {
            match library.0.start_watching(move || {
                let _ = rescan.send(());
            }) {
                Ok(()) => {}
                Err(error) => {
                    tracing::warn!(%error, "failed to start library filesystem watcher");
                }
            }
        });
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

impl gpui::Global for LibraryScanCoordinator {}

fn log_scan_complete(summary: &ScanSummary) {
    tracing::info!(
        roots = summary.roots.len(),
        files_seen = summary.files_seen,
        songs = summary.songs_imported,
        skipped = summary.skipped,
        "music library scan complete"
    );
}

fn log_scan_error(error: &pulse_library::LibraryError) {
    if let pulse_library::LibraryError::RootMissing(path) = error {
        tracing::warn!(?path, "music library root is missing; skipping scan");
    } else {
        tracing::error!(%error, "music library scan failed");
    }
}
