use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use pulse_model::{
    Album, AlbumArtists, AlbumId, Artist, ArtistId, Artwork, ArtworkId, ArtworkReference,
    ArtworkSource, ArtworkThumbnail, EntityMetadata, Song, SongId, ThumbnailSize,
};

use crate::artwork::{ArtworkCache, CachedArtworkMeta};

#[derive(Debug, Default)]
struct IdGenerator {
    song: u64,
    album: u64,
    artist: u64,
    artwork: u64,
}

impl IdGenerator {
    const fn allocate_song(&mut self) -> SongId {
        self.song = self.song.saturating_add(1);
        SongId(self.song)
    }

    const fn allocate_album(&mut self) -> AlbumId {
        self.album = self.album.saturating_add(1);
        AlbumId(self.album)
    }

    const fn allocate_artist(&mut self) -> ArtistId {
        self.artist = self.artist.saturating_add(1);
        ArtistId(self.artist)
    }

    const fn allocate_artwork(&mut self) -> ArtworkId {
        self.artwork = self.artwork.saturating_add(1);
        ArtworkId(self.artwork)
    }

    const fn reset_catalog(&mut self) {
        self.song = 0;
        self.album = 0;
        self.artist = 0;
    }
}

#[derive(Debug, Default)]
pub struct LibraryStore {
    songs: HashMap<SongId, Song>,
    albums: HashMap<AlbumId, Album>,
    artists: HashMap<ArtistId, Artist>,
    artworks: HashMap<ArtworkId, Artwork>,
    artwork_by_hash: HashMap<String, ArtworkId>,
    thumbnails: HashMap<(ArtworkId, ThumbnailSize), PathBuf>,
    song_by_path: HashMap<PathBuf, SongId>,
    ids: IdGenerator,
}

impl LibraryStore {
    #[must_use]
    pub const fn songs(&self) -> &HashMap<SongId, Song> {
        &self.songs
    }

    #[must_use]
    pub const fn albums(&self) -> &HashMap<AlbumId, Album> {
        &self.albums
    }

    #[must_use]
    pub const fn artists(&self) -> &HashMap<ArtistId, Artist> {
        &self.artists
    }

    #[must_use]
    pub const fn artworks(&self) -> &HashMap<ArtworkId, Artwork> {
        &self.artworks
    }

    #[must_use]
    pub fn artwork(&self, id: ArtworkId) -> Option<&Artwork> {
        self.artworks.get(&id)
    }

    #[must_use]
    pub fn thumbnail_path(&self, artwork_id: ArtworkId, size: ThumbnailSize) -> Option<&Path> {
        self.thumbnails
            .get(&(artwork_id, size))
            .map(PathBuf::as_path)
    }

    #[must_use]
    pub fn artwork_id_for_hash(&self, content_hash: &str) -> Option<ArtworkId> {
        self.artwork_by_hash.get(content_hash).copied()
    }

    #[must_use]
    pub fn song_for_path(&self, path: &Path) -> Option<&Song> {
        self.song_by_path
            .get(path)
            .and_then(|id| self.songs.get(id))
    }

    pub fn clear_catalog(&mut self) {
        self.songs.clear();
        self.albums.clear();
        self.artists.clear();
        self.song_by_path.clear();
        self.ids.reset_catalog();
    }

    pub fn clear(&mut self) {
        self.songs.clear();
        self.albums.clear();
        self.artists.clear();
        self.artworks.clear();
        self.artwork_by_hash.clear();
        self.thumbnails.clear();
        self.song_by_path.clear();
        self.ids = IdGenerator::default();
    }

    pub fn intern_artist(&mut self, name: &str) -> ArtistId {
        let normalized = normalize_key(name);
        if let Some((id, _)) = self
            .artists
            .iter()
            .find(|(_, artist)| normalize_key(&artist.name) == normalized)
        {
            return *id;
        }

        let id = self.ids.allocate_artist();
        self.artists.insert(
            id,
            Artist {
                id,
                name: name.to_string(),
                artwork_id: None,
                metadata: EntityMetadata::new(),
            },
        );
        id
    }

    pub fn intern_album(
        &mut self,
        title: &str,
        album_artists: AlbumArtists,
        year: Option<u16>,
    ) -> AlbumId {
        let key = album_key(title, &album_artists);
        if let Some((id, _)) = self
            .albums
            .iter()
            .find(|(_, album)| album_key(&album.title, &album.album_artists) == key)
        {
            return *id;
        }

        let id = self.ids.allocate_album();
        self.albums.insert(
            id,
            Album {
                id,
                title: title.to_string(),
                album_artists,
                year,
                artwork_id: None,
                metadata: EntityMetadata::new(),
            },
        );
        id
    }

    pub fn insert_song(&mut self, song: Song) {
        self.song_by_path.insert(song.path.clone(), song.id);
        self.songs.insert(song.id, song);
    }

    pub const fn next_song_id(&mut self) -> SongId {
        self.ids.allocate_song()
    }

    pub(crate) fn preload_artwork(
        &mut self,
        cache: &ArtworkCache,
        content_hash: &str,
        meta: &CachedArtworkMeta,
    ) -> ArtworkId {
        if let Some(id) = self.artwork_id_for_hash(content_hash) {
            return id;
        }

        let artwork_id = self.insert_artwork(
            Artwork {
                id: ArtworkId(0),
                source: ArtworkSource::Cached {
                    path: cache.source_path(content_hash, &meta.extension),
                },
                width: Some(meta.width),
                height: Some(meta.height),
                dominant_color: None,
            },
            content_hash,
        );

        for size in ThumbnailSize::all() {
            self.insert_thumbnail(artwork_id, size, cache.thumbnail_path(content_hash, size));
        }

        artwork_id
    }

    pub(crate) fn insert_artwork(&mut self, mut artwork: Artwork, content_hash: &str) -> ArtworkId {
        let id = self.ids.allocate_artwork();
        artwork.id = id;
        self.artwork_by_hash.insert(content_hash.to_string(), id);
        self.artworks.insert(id, artwork);
        id
    }

    pub(crate) fn insert_thumbnail(
        &mut self,
        artwork_id: ArtworkId,
        size: ThumbnailSize,
        path: PathBuf,
    ) {
        self.thumbnails.insert((artwork_id, size), path);
    }

    pub(crate) fn link_song_artwork(&mut self, song_id: SongId, artwork_id: ArtworkId) {
        let Some(song) = self.songs.get_mut(&song_id) else {
            return;
        };

        song.artwork = Some(ArtworkReference::Custom(artwork_id));

        if let Some(album_id) = song.album_id
            && let Some(album) = self.albums.get_mut(&album_id)
            && album.artwork_id.is_none()
        {
            album.artwork_id = Some(artwork_id);
        }
    }

    #[must_use]
    pub fn thumbnails_for(&self, artwork_id: ArtworkId) -> Vec<ArtworkThumbnail> {
        ThumbnailSize::all()
            .into_iter()
            .filter_map(|size| {
                self.thumbnail_path(artwork_id, size)
                    .map(|path| ArtworkThumbnail {
                        artwork_id,
                        size,
                        path: path.to_path_buf(),
                    })
            })
            .collect()
    }
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn album_key(title: &str, album_artists: &AlbumArtists) -> String {
    let artists = match album_artists {
        AlbumArtists::Single(id) => format!("single:{id:?}"),
        AlbumArtists::Various => "various".to_string(),
        AlbumArtists::Multiple(ids) => {
            let mut parts: Vec<String> = ids.iter().map(|id| format!("{id:?}")).collect();
            parts.sort();
            format!("multi:{}", parts.join(","))
        }
    };

    format!("{}|{artists}", normalize_key(title))
}
