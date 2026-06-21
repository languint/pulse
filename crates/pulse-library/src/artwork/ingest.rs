use pulse_model::{Artwork, ArtworkId, ArtworkSource, SongId, ThumbnailSize};

use crate::{
    artwork::{
        cache::{ArtworkCache, CachedArtworkMeta},
        thumbnail::{generate_all_thumbnails, image_dimensions, source_extension},
    },
    error::LibraryError,
    store::LibraryStore,
};

pub fn ingest_embedded_art(
    store: &mut LibraryStore,
    cache: &ArtworkCache,
    song_id: SongId,
    data: &[u8],
) -> Result<Option<ArtworkId>, LibraryError> {
    if data.is_empty() {
        return Ok(None);
    }

    let content_hash = blake3::hash(data).to_hex().to_string();

    if let Some(existing) = store.artwork_id_for_hash(&content_hash) {
        store.link_song_artwork(song_id, existing);
        return Ok(Some(existing));
    }

    if let Some(meta) = cache
        .cached_artwork(&content_hash)
        .map_err(|source| LibraryError::Io {
            path: cache.meta_path_for(&content_hash),
            source,
        })?
    {
        let artwork_id = register_cached_artwork(store, cache, song_id, &content_hash, &meta);
        return Ok(Some(artwork_id));
    }

    cache
        .ensure_dirs(&content_hash)
        .map_err(|source| LibraryError::Io {
            path: cache.root().to_path_buf(),
            source,
        })?;

    let extension = source_extension(data);
    let source_path = cache.source_path(&content_hash, extension);
    ArtworkCache::write_if_missing(&source_path, data).map_err(|source| LibraryError::Io {
        path: source_path.clone(),
        source,
    })?;

    let (width, height) = image_dimensions(data)?;
    let thumbnails = generate_all_thumbnails(data, cache, &content_hash)?;

    let meta = CachedArtworkMeta {
        width,
        height,
        extension: extension.to_string(),
    };
    cache
        .write_cached_meta(&content_hash, &meta)
        .map_err(|source| LibraryError::Io {
            path: cache.meta_path_for(&content_hash),
            source,
        })?;

    let artwork_id = store.insert_artwork(
        Artwork {
            id: ArtworkId(0),
            source: ArtworkSource::Embedded { song_id },
            width: Some(width),
            height: Some(height),
            dominant_color: None,
        },
        &content_hash,
    );

    for (size, path) in thumbnails {
        store.insert_thumbnail(artwork_id, size, path);
    }

    store.link_song_artwork(song_id, artwork_id);

    Ok(Some(artwork_id))
}

fn register_cached_artwork(
    store: &mut LibraryStore,
    cache: &ArtworkCache,
    song_id: SongId,
    content_hash: &str,
    meta: &CachedArtworkMeta,
) -> ArtworkId {
    if let Some(existing) = store.artwork_id_for_hash(content_hash) {
        store.link_song_artwork(song_id, existing);
        return existing;
    }

    let artwork_id = store.insert_artwork(
        Artwork {
            id: ArtworkId(0),
            source: ArtworkSource::Embedded { song_id },
            width: Some(meta.width),
            height: Some(meta.height),
            dominant_color: None,
        },
        content_hash,
    );

    for size in ThumbnailSize::all() {
        store.insert_thumbnail(artwork_id, size, cache.thumbnail_path(content_hash, size));
    }

    store.link_song_artwork(song_id, artwork_id);
    artwork_id
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::ImageFormat;
    use pulse_model::{ArtworkReference, ThumbnailSize};

    use crate::{
        artwork::{ingest_embedded_art, ArtworkCache},
        store::LibraryStore,
    };

    fn sample_png() -> Result<Vec<u8>, image::ImageError> {
        let mut buffer = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            128,
            128,
            image::Rgb([40, 120, 200]),
        ))
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
        Ok(buffer)
    }

    #[test]
    fn ingest_embedded_art_creates_thumbnails() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let cache = ArtworkCache::new(temp.path());
        let mut store = LibraryStore::default();

        let artist = store.intern_artist("Artist");
        let album = store.intern_album(
            "Album",
            pulse_model::AlbumArtists::single(artist),
            Some(2024),
        );
        let song_id = store.next_song_id();
        store.insert_song(pulse_model::Song {
            id: song_id,
            title: "Track".into(),
            album_id: Some(album),
            track_artists: vec![artist],
            track_number: Some(1),
            disc_number: Some(1),
            duration_ms: 180_000,
            path: "music/track.mp3".into(),
            artwork: Some(ArtworkReference::Inherit),
            metadata: pulse_model::EntityMetadata::new(),
        });

        let png = sample_png()?;
        let artwork_id = ingest_embedded_art(&mut store, &cache, song_id, &png)?
            .ok_or("expected artwork id")?;

        if store.artworks().len() != 1 {
            return Err("expected one artwork entry".into());
        }
        if !store
            .thumbnail_path(artwork_id, ThumbnailSize::Small)
            .is_some_and(std::path::Path::exists)
        {
            return Err("expected small thumbnail file".into());
        }
        if !store
            .thumbnail_path(artwork_id, ThumbnailSize::Medium)
            .is_some_and(std::path::Path::exists)
        {
            return Err("expected medium thumbnail file".into());
        }
        if store.songs().get(&song_id).and_then(|song| song.artwork)
            != Some(ArtworkReference::Custom(artwork_id))
        {
            return Err("expected song artwork link".into());
        }
        if store.albums().get(&album).and_then(|album| album.artwork_id) != Some(artwork_id) {
            return Err("expected album artwork link".into());
        }

        ingest_embedded_art(&mut store, &cache, song_id, &png)?;
        if store.artworks().len() != 1 {
            return Err("duplicate art should dedupe".into());
        }

        Ok(())
    }

    #[test]
    fn ingest_hits_disk_cache_after_catalog_clear() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let cache = ArtworkCache::new(temp.path());
        let mut store = LibraryStore::default();

        let artist = store.intern_artist("Artist");
        let album = store.intern_album(
            "Album",
            pulse_model::AlbumArtists::single(artist),
            Some(2024),
        );
        let song_id = store.next_song_id();
        store.insert_song(pulse_model::Song {
            id: song_id,
            title: "Track".into(),
            album_id: Some(album),
            track_artists: vec![artist],
            track_number: None,
            disc_number: None,
            duration_ms: 180_000,
            path: "music/track.mp3".into(),
            artwork: Some(ArtworkReference::Inherit),
            metadata: pulse_model::EntityMetadata::new(),
        });

        let png = sample_png()?;
        ingest_embedded_art(&mut store, &cache, song_id, &png)?;

        store.clear_catalog();

        let artist = store.intern_artist("Artist");
        let album = store.intern_album(
            "Album",
            pulse_model::AlbumArtists::single(artist),
            Some(2024),
        );
        let song_id = store.next_song_id();
        store.insert_song(pulse_model::Song {
            id: song_id,
            title: "Track".into(),
            album_id: Some(album),
            track_artists: vec![artist],
            track_number: None,
            disc_number: None,
            duration_ms: 180_000,
            path: "music/track.mp3".into(),
            artwork: Some(ArtworkReference::Inherit),
            metadata: pulse_model::EntityMetadata::new(),
        });

        let artwork_id = ingest_embedded_art(&mut store, &cache, song_id, &png)?
            .ok_or("expected artwork id")?;

        if store.artworks().len() != 1 {
            return Err("expected artwork restored from disk cache".into());
        }
        if store.artwork_id_for_hash(blake3::hash(&png).to_hex().as_ref()) != Some(artwork_id)
        {
            return Err("expected artwork hash lookup".into());
        }

        Ok(())
    }
}
