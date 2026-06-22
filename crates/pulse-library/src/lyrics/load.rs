use std::path::{Path, PathBuf};

use pulse_model::Lyrics;

use super::cache::LyricsCache;
use super::parse::parse_lrc;

/// Load sidecar `.lrc` lyrics for an audio file, if present.
#[must_use]
pub fn load_sidecar_lyrics(audio_path: &Path) -> Option<Lyrics> {
    for candidate in sidecar_candidates(audio_path) {
        let content = std::fs::read_to_string(&candidate).ok()?;
        if let Some(lyrics) = parse_lrc(&content) {
            return Some(lyrics);
        }
    }
    None
}

/// Load sidecar lyrics first, then fall back to the on-disk fetch cache.
#[must_use]
pub fn load_local_lyrics(audio_path: &Path, cache: &LyricsCache) -> Option<Lyrics> {
    load_sidecar_lyrics(audio_path).or_else(|| cache.read(audio_path))
}

#[must_use]
pub fn sidecar_candidates(audio_path: &Path) -> Vec<PathBuf> {
    let Some(parent) = audio_path.parent() else {
        return Vec::new();
    };
    let Some(stem) = audio_path.file_stem().and_then(|value| value.to_str()) else {
        return Vec::new();
    };

    let exact = parent.join(format!("{stem}.lrc"));
    let mut candidates = vec![exact];

    if let Ok(entries) = std::fs::read_dir(parent) {
        let prefix = format!("{stem}.");
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            if name.starts_with(&prefix) && name.ends_with(".lrc") && !candidates.contains(&path) {
                candidates.push(path);
            }
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_exact_sidecar() {
        let temp = tempfile::tempdir().expect("tempdir");
        let audio = temp.path().join("track.flac");
        std::fs::write(&audio, b"audio").expect("audio");
        let lrc = temp.path().join("track.lrc");
        std::fs::File::create(&lrc)
            .expect("lrc")
            .write_all(b"[00:01.00]Hello\n")
            .expect("write");

        let lyrics = load_sidecar_lyrics(&audio).expect("lyrics");
        assert!(lyrics.is_synced());
    }
}
