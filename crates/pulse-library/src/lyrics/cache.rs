use std::path::{Path, PathBuf};

use pulse_model::Lyrics;

use super::parse::parse_lrc;

/// On-disk cache for fetched lyrics, keyed by audio file path.
#[derive(Debug, Clone)]
pub struct LyricsCache {
    root: PathBuf,
}

impl LyricsCache {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn cache_key(audio_path: &Path) -> String {
        blake3::hash(audio_path.as_os_str().as_encoded_bytes())
            .to_hex()
            .to_string()
    }

    fn entry_path(&self, audio_path: &Path) -> PathBuf {
        self.root.join(format!("{}.lrc", Self::cache_key(audio_path)))
    }

    /// Reads cached lyrics for an audio file, if present.
    #[must_use]
    pub fn read(&self, audio_path: &Path) -> Option<Lyrics> {
        let path = self.entry_path(audio_path);
        let content = std::fs::read_to_string(path).ok()?;
        parse_lrc(&content)
    }

    /// Writes LRC content to the cache when missing.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when directories cannot be created or the file cannot be written.
    pub fn write_if_missing(&self, audio_path: &Path, content: &str) -> std::io::Result<()> {
        let path = self.entry_path(audio_path);
        if path.is_file() {
            return Ok(());
        }

        std::fs::create_dir_all(&self.root)?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_cached_lyrics() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = LyricsCache::new(temp.path());
        let audio = temp.path().join("song.flac");

        cache
            .write_if_missing(&audio, "[00:01.00]Cached line\n")
            .expect("write");

        let lyrics = cache.read(&audio).expect("lyrics");
        assert!(lyrics.is_synced());
    }

    #[test]
    fn write_if_missing_is_idempotent() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = LyricsCache::new(temp.path());
        let audio = temp.path().join("song.flac");

        cache
            .write_if_missing(&audio, "[00:01.00]First\n")
            .expect("write");
        cache
            .write_if_missing(&audio, "[00:02.00]Second\n")
            .expect("write again");

        let lyrics = cache.read(&audio).expect("lyrics");
        let lines = lyrics.synced_lines().expect("synced");
        assert_eq!(lines[0].text, "First");
    }
}
