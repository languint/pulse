use std::path::PathBuf;

use gpui::{App, Global, UpdateGlobal};
use pulse_library::{LyricsCache, fetch_lrclib_lyrics, load_local_lyrics, parse_lrc};
use pulse_model::Lyrics;
use pulse_runtime::Tokio;

use crate::config::PulseConfig;
use crate::data::DataPaths;
use crate::player::PulsePlayer;

#[derive(Clone, Debug)]
pub struct PulseLyrics {
    pub sidebar_open: bool,
    cache: LyricsCache,
    track_path: Option<PathBuf>,
    lyrics: Option<Lyrics>,
    loading: bool,
    fetch_not_found: bool,
}

impl Global for PulseLyrics {}

impl PulseLyrics {
    pub fn init(cx: &mut App) {
        let cache = LyricsCache::new(cx.global::<DataPaths>().lyrics_dir());
        Self::set_global(
            cx,
            Self {
                sidebar_open: false,
                cache,
                track_path: None,
                lyrics: None,
                loading: false,
                fetch_not_found: false,
            },
        );
    }

    pub fn toggle_sidebar(cx: &mut App) {
        Self::update_global(cx, |state, _| {
            state.sidebar_open = !state.sidebar_open;
        });
        cx.refresh_windows();
    }

    pub fn open_sidebar(cx: &mut App) {
        Self::update_global(cx, |state, _| {
            state.sidebar_open = true;
        });
        cx.refresh_windows();
    }

    #[must_use]
    pub fn sidebar_open(cx: &App) -> bool {
        cx.global::<Self>().sidebar_open
    }

    #[must_use]
    pub fn loading(cx: &App) -> bool {
        cx.global::<Self>().loading
    }

    #[must_use]
    pub fn fetch_not_found(cx: &App) -> bool {
        cx.global::<Self>().fetch_not_found
    }

    #[must_use]
    pub fn auto_fetch_enabled(cx: &App) -> bool {
        cx.global::<PulseConfig>().lyrics.auto_fetch_lyrics
    }

    pub fn sync_with_player(cx: &mut App) {
        let path = PulsePlayer::current_track_path(cx);
        let auto_fetch = Self::auto_fetch_enabled(cx);

        let (same_track, has_lyrics, loading, fetch_not_found) = {
            let state = cx.global::<Self>();
            (
                state.track_path == path,
                state.lyrics.is_some(),
                state.loading,
                state.fetch_not_found,
            )
        };

        if same_track && (has_lyrics || loading || fetch_not_found) {
            return;
        }

        let cache = cx.global::<Self>().cache.clone();
        let lyrics = path
            .as_ref()
            .and_then(|track| load_local_lyrics(track, &cache));

        Self::update_global(cx, |state, _| {
            state.track_path = path.clone();
            state.lyrics = lyrics.clone();
            state.loading = false;
            state.fetch_not_found = false;
        });

        if lyrics.is_some() || !auto_fetch {
            return;
        }

        let Some(path) = path else {
            return;
        };

        let Some(lookup) = PulsePlayer::current_track_lookup(cx) else {
            Self::update_global(cx, |state, _| {
                state.fetch_not_found = true;
            });
            return;
        };

        Self::update_global(cx, |state, _| {
            state.loading = true;
        });
        cx.refresh_windows();

        let fetch_task = Tokio::spawn(cx, async move { fetch_lrclib_lyrics(&lookup).await });

        cx.spawn(async move |async_cx| {
            let fetch_result = fetch_task.await;

            async_cx.update(|app| {
                let fetch_result = match fetch_result {
                    Ok(result) => result,
                    Err(error) => {
                        tracing::warn!(%error, "lyrics fetch task failed");
                        return;
                    }
                };

                PulseLyrics::update_global(app, |state, _| {
                    if state.track_path.as_ref() != Some(&path) {
                        return;
                    }

                    state.loading = false;

                    match fetch_result {
                        Ok(content) => {
                            if let Err(error) = state.cache.write_if_missing(&path, &content) {
                                tracing::warn!(%error, path = ?path, "failed to cache fetched lyrics");
                            }

                            state.lyrics = parse_lrc(&content);
                            state.fetch_not_found = state.lyrics.is_none();
                        }
                        Err(error) => {
                            tracing::debug!(%error, path = ?path, "online lyrics lookup failed");
                            state.fetch_not_found = true;
                        }
                    }
                });

                app.refresh_windows();
            });
        })
        .detach();
    }

    #[must_use]
    pub fn current(cx: &App) -> Option<Lyrics> {
        cx.global::<Self>().lyrics.clone()
    }

    #[must_use]
    pub fn has_lyrics(cx: &App) -> bool {
        cx.global::<Self>().lyrics.is_some()
    }

    pub fn reset_fetch_state(cx: &mut App) {
        Self::update_global(cx, |state, _| {
            state.loading = false;
            state.fetch_not_found = false;
            if state.lyrics.is_none() {
                state.track_path = None;
            }
        });
    }
}
