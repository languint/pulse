use std::path::PathBuf;

use gpui::{App, Global, SharedString, UpdateGlobal};
use pulse_audio::{PlaybackState, Player, PlayerSnapshot, TrackInfo};
use pulse_model::{Song, SongId};

use crate::components::pages::{format_album_artists, resolve_album_artwork};
use crate::library::PulseLibrary;

pub struct PulsePlayer {
    inner: Option<Player>,
}

impl Global for PulsePlayer {}

impl PulsePlayer {
    pub fn init(cx: &mut App) {
        match Player::new() {
            Ok(player) => {
                Self::set_global(
                    cx,
                    Self {
                        inner: Some(player),
                    },
                );
                tracing::info!("audio player initialized");
            }
            Err(error) => {
                tracing::error!(%error, "failed to initialize audio player");
                Self::set_global(cx, Self { inner: None });
            }
        }
    }

    #[must_use]
    pub fn snapshot(cx: &App) -> PlayerSnapshot {
        cx.global::<Self>()
            .inner
            .as_ref()
            .map(Player::snapshot)
            .unwrap_or_default()
    }

    pub fn play_songs(cx: &mut App, song_ids: &[SongId], start_index: usize) {
        let Some(player) = cx.global::<Self>().inner.as_ref() else {
            tracing::warn!("audio player unavailable");
            return;
        };

        let store = cx.global::<PulseLibrary>().inner().store();
        let tracks: Vec<TrackInfo> = song_ids
            .iter()
            .filter_map(|id| store.songs().get(id))
            .map(|song| TrackInfo {
                path: song.path.clone(),
                duration_ms: song.duration_ms,
            })
            .collect();

        if tracks.is_empty() {
            return;
        }

        let index = start_index.min(tracks.len().saturating_sub(1));
        player.play_queue(tracks, index);
        cx.refresh_windows();
    }

    pub fn toggle_pause(cx: &mut App) {
        if let Some(player) = cx.global::<Self>().inner.as_ref() {
            player.toggle_pause();
            cx.refresh_windows();
        }
    }

    pub fn pause(cx: &mut App) {
        if let Some(player) = cx.global::<Self>().inner.as_ref() {
            player.pause();
            cx.refresh_windows();
        }
    }

    pub fn play(cx: &mut App) {
        if let Some(player) = cx.global::<Self>().inner.as_ref() {
            player.play();
            cx.refresh_windows();
        }
    }

    pub fn next(cx: &mut App) {
        if let Some(player) = cx.global::<Self>().inner.as_ref() {
            player.next();
            cx.refresh_windows();
        }
    }

    pub fn previous(cx: &mut App) {
        if let Some(player) = cx.global::<Self>().inner.as_ref() {
            player.previous();
            cx.refresh_windows();
        }
    }

    pub fn seek(cx: &mut App, position_ms: u64) {
        if let Some(player) = cx.global::<Self>().inner.as_ref() {
            player.seek(position_ms);
            cx.refresh_windows();
        }
    }

    #[must_use]
    pub fn current_track_path(cx: &App) -> Option<PathBuf> {
        let snapshot = Self::snapshot(cx);
        let index = snapshot.current_index?;
        Some(snapshot.queue.get(index)?.path.clone())
    }

    #[must_use]
    pub fn current_song_id(cx: &App) -> Option<SongId> {
        current_song(cx).map(|song| song.id)
    }

    #[must_use]
    pub fn current_track_title(cx: &App) -> Option<SharedString> {
        current_song(cx).map(|song| song.title.clone().into())
    }

    #[must_use]
    pub fn current_track_subtitle(cx: &App) -> Option<SharedString> {
        let song = current_song(cx)?;
        let store = cx.global::<PulseLibrary>().inner().store();
        let artists = store.artists();

        let artist_names: Vec<_> = song
            .track_artists
            .iter()
            .filter_map(|id| artists.get(id))
            .map(|artist| artist.name.as_str())
            .collect();

        if artist_names.is_empty() {
            None
        } else {
            Some(artist_names.join(", ").into())
        }
    }

    #[must_use]
    pub fn current_track_artwork(cx: &App) -> Option<PathBuf> {
        let song = current_song(cx)?;
        let store = cx.global::<PulseLibrary>().inner().store();
        song.album_id
            .and_then(|album_id| store.albums().get(&album_id))
            .and_then(|album| {
                let artist_label = format_album_artists(store.artists(), &album.album_artists);
                resolve_album_artwork(cx, album, &artist_label)
            })
    }

    #[must_use]
    pub fn is_playing(cx: &App) -> bool {
        Self::snapshot(cx).state == PlaybackState::Playing
    }

    #[must_use]
    pub fn sample_snapshot(cx: &App) -> Vec<f32> {
        cx.global::<Self>()
            .inner
            .as_ref()
            .map(|player| player.sample_capture().snapshot())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn sample_rate(cx: &App) -> Option<u32> {
        cx.global::<Self>()
            .inner
            .as_ref()
            .and_then(|player| player.sample_capture().sample_rate())
    }

    #[must_use]
    pub fn current_track_lookup(cx: &App) -> Option<pulse_library::LyricsLookup> {
        let song = current_song(cx)?;
        let store = cx.global::<PulseLibrary>().inner().store();
        let album_name = song
            .album_id
            .and_then(|album_id| store.albums().get(&album_id))
            .map(|album| album.title.to_string())
            .unwrap_or_default();

        let artist_names: Vec<_> = song
            .track_artists
            .iter()
            .filter_map(|id| store.artists().get(id))
            .map(|artist| artist.name.as_str())
            .collect();

        Some(pulse_library::LyricsLookup {
            track_name: song.title.to_string(),
            artist_name: artist_names.join(", "),
            album_name,
            duration_secs: song.duration_ms.div_ceil(1000),
        })
    }
}

fn current_song(cx: &App) -> Option<&Song> {
    let snapshot = PulsePlayer::snapshot(cx);
    let index = snapshot.current_index?;
    let path = snapshot.queue.get(index)?.path.clone();
    cx.global::<PulseLibrary>()
        .inner()
        .store()
        .song_for_path(path.as_path())
}
