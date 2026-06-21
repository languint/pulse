use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey},
};
use pulse_model::{AlbumArtists, ArtworkReference, EntityMetadata, Song};

use crate::{
    artwork::{extract_cover_art, ingest_embedded_art, ArtworkCache},
    error::LibraryError,
    store::LibraryStore,
};

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "oga", "opus", "m4a", "aac", "wav", "aiff", "aif", "wma", "mpc", "ape",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanSummary {
    pub roots: Vec<PathBuf>,
    pub files_seen: usize,
    pub songs_imported: usize,
    pub skipped: usize,
}

pub fn scan_roots(
    store: &mut LibraryStore,
    roots: &[PathBuf],
    artwork_cache: &ArtworkCache,
) -> ScanSummary {
    store.clear_catalog();

    let mut files_seen = 0_usize;
    let mut songs_imported = 0_usize;
    let mut skipped = 0_usize;

    for root in roots {
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.into_path();
            if !is_audio_file(&path) {
                continue;
            }

            files_seen = files_seen.saturating_add(1);
            match import_file(store, artwork_cache, &path) {
                Ok(()) => songs_imported = songs_imported.saturating_add(1),
                Err(error) => {
                    skipped = skipped.saturating_add(1);
                    tracing::warn!(?path, %error, "skipping audio file during library scan");
                }
            }
        }
    }

    tracing::debug!(files_seen, songs_imported, skipped, "library scan finished");

    ScanSummary {
        roots: roots.to_vec(),
        files_seen,
        songs_imported,
        skipped,
    }
}

fn import_file(
    store: &mut LibraryStore,
    artwork_cache: &ArtworkCache,
    path: &Path,
) -> Result<(), LibraryError> {
    let tagged = Probe::open(path)
        .map_err(|source| LibraryError::Metadata {
            path: path.to_path_buf(),
            source,
        })?
        .read()
        .map_err(|source| LibraryError::Metadata {
            path: path.to_path_buf(),
            source,
        })?;

    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let properties = tagged.properties();

    let title = tag
        .and_then(|tag| tag.title().map(std::borrow::Cow::into_owned))
        .unwrap_or_else(|| filename_title(path));

    let album_title = tag
        .and_then(|tag| tag.album().map(std::borrow::Cow::into_owned))
        .unwrap_or_else(|| "Unknown Album".to_string());

    let track_artists = tag
        .and_then(|tag| tag.artist().map(|value| split_multi_value(&value)))
        .unwrap_or_default();

    let track_artist_ids: Vec<_> = track_artists
        .iter()
        .map(|name| store.intern_artist(name))
        .collect();

    let album_artists = tag.map_or_else(
        || fallback_album_artists(&track_artist_ids),
        |tag| parse_album_artists(tag, store),
    );

    let year = tag
        .and_then(lofty::tag::Accessor::year)
        .and_then(|year| u16::try_from(year).ok());
    let track_number = tag
        .and_then(lofty::tag::Accessor::track)
        .and_then(|track| u16::try_from(track).ok());
    let disc_number = tag
        .and_then(lofty::tag::Accessor::disk)
        .and_then(|disc| u16::try_from(disc).ok());

    let duration_ms = u32::try_from(properties.duration().as_millis()).unwrap_or(u32::MAX);

    let metadata = tag.map(entity_metadata).unwrap_or_default();

    let album_id = store.intern_album(&album_title, album_artists, year);

    let song_id = store.next_song_id();
    let song = Song {
        id: song_id,
        title,
        album_id: Some(album_id),
        track_artists: track_artist_ids,
        track_number,
        disc_number,
        duration_ms,
        path: path.to_path_buf(),
        artwork: Some(ArtworkReference::Inherit),
        metadata,
    };

    store.insert_song(song);

    if let Some(tag) = tag
        && let Some(cover_data) = extract_cover_art(tag)
        && let Err(error) = ingest_embedded_art(store, artwork_cache, song_id, &cover_data)
    {
        tracing::warn!(?path, %error, "failed to cache embedded artwork");
    }

    Ok(())
}

fn entity_metadata(tag: &lofty::tag::Tag) -> EntityMetadata {
    let genres = tag
        .genre()
        .map(|genre| split_multi_value(&genre))
        .unwrap_or_default();

    EntityMetadata {
        genres,
        tags: Vec::new(),
        comment: tag.comment().map(std::borrow::Cow::into_owned),
    }
}

fn parse_album_artists(tag: &lofty::tag::Tag, store: &mut LibraryStore) -> AlbumArtists {
    if let Some(album_artist) = tag.get_string(&ItemKey::AlbumArtist) {
        return album_artists_from_names(split_multi_value(album_artist), store);
    }

    if let Some(artist) = tag.artist() {
        return album_artists_from_names(split_multi_value(&artist), store);
    }

    AlbumArtists::Various
}

fn album_artists_from_names(names: Vec<String>, store: &mut LibraryStore) -> AlbumArtists {
    if names.is_empty() {
        return AlbumArtists::Various;
    }

    if names.len() == 1 {
        let name = names.first().map_or("", String::as_str);
        if is_various_artists(name) {
            return AlbumArtists::Various;
        }
        return AlbumArtists::Single(store.intern_artist(name));
    }

    let ids: Vec<_> = names
        .into_iter()
        .filter(|name| !is_various_artists(name))
        .map(|name| store.intern_artist(&name))
        .collect();

    match ids.as_slice() {
        [] => AlbumArtists::Various,
        [id] => AlbumArtists::Single(*id),
        _ => AlbumArtists::Multiple(ids),
    }
}

fn fallback_album_artists(track_artist_ids: &[pulse_model::ArtistId]) -> AlbumArtists {
    match track_artist_ids {
        [] => AlbumArtists::Various,
        [id] => AlbumArtists::Single(*id),
        _ => AlbumArtists::Multiple(track_artist_ids.to_vec()),
    }
}

fn is_audio_file(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str).is_some_and(|ext| {
        AUDIO_EXTENSIONS
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
    })
}

fn filename_title(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("Unknown Title")
        .to_string()
}

fn split_multi_value(value: &str) -> Vec<String> {
    value
        .split([';', '/', '\0'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn is_various_artists(name: &str) -> bool {
    matches!(
        normalize_name(name).as_str(),
        "various artists" | "various" | "va" | "unknown"
    )
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_semicolon_separated_artists() {
        let names = split_multi_value("Artist A; Artist B");
        assert_eq!(names, vec!["Artist A", "Artist B"]);
    }

    #[test]
    fn detects_various_artists() {
        assert!(is_various_artists("Various Artists"));
        assert!(!is_various_artists("Pink Floyd"));
    }
}
