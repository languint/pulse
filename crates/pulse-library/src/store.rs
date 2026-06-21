use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use pulse_model::{Album, AlbumArtists, AlbumId, Artist, ArtistId, EntityMetadata, Song, SongId};

#[derive(Debug, Default)]
struct IdGenerator {
    song: u64,
    album: u64,
    artist: u64,
}

#[allow(clippy::missing_const_for_fn)]
impl IdGenerator {
    fn allocate_song(&mut self) -> SongId {
        self.song = self.song.saturating_add(1);
        SongId(self.song)
    }

    fn allocate_album(&mut self) -> AlbumId {
        self.album = self.album.saturating_add(1);
        AlbumId(self.album)
    }

    fn allocate_artist(&mut self) -> ArtistId {
        self.artist = self.artist.saturating_add(1);
        ArtistId(self.artist)
    }
}

#[derive(Debug, Default)]
pub struct LibraryStore {
    songs: HashMap<SongId, Song>,
    albums: HashMap<AlbumId, Album>,
    artists: HashMap<ArtistId, Artist>,
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
    pub fn song_for_path(&self, path: &Path) -> Option<&Song> {
        self.song_by_path
            .get(path)
            .and_then(|id| self.songs.get(id))
    }

    pub fn clear(&mut self) {
        self.songs.clear();
        self.albums.clear();
        self.artists.clear();
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

    pub fn next_song_id(&mut self) -> SongId {
        self.ids.allocate_song()
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
