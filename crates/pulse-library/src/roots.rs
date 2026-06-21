use std::path::{Path, PathBuf};

use crate::{config::LibraryConfig, error::LibraryError};

#[must_use]
pub fn resolve_roots(config: &LibraryConfig) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if config.include_xdg_music_dir
        && let Some(music_dir) = dirs::audio_dir()
    {
        push_unique_root(&mut roots, music_dir);
    }

    for path in &config.extra_paths {
        push_unique_root(&mut roots, path.clone());
    }

    roots
}

pub fn resolve_roots_or_error(config: &LibraryConfig) -> Result<Vec<PathBuf>, LibraryError> {
    let roots = resolve_roots(config);
    if roots.is_empty() {
        if let Some(music_dir) = config.include_xdg_music_dir.then(dirs::audio_dir).flatten() {
            return Err(LibraryError::RootMissing(music_dir));
        }

        if let Some(path) = config.extra_paths.first() {
            return Err(LibraryError::RootMissing(path.clone()));
        }
    }

    Ok(roots)
}

fn push_unique_root(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if !path_is_usable(&path) {
        return;
    }

    let canonical = path.canonicalize().unwrap_or(path);
    if roots
        .iter()
        .any(|existing| paths_equal(existing, &canonical))
    {
        return;
    }

    roots.push(canonical);
}

fn path_is_usable(path: &Path) -> bool {
    path.is_dir()
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    a.canonicalize()
        .ok()
        .zip(b.canonicalize().ok())
        .is_some_and(|(a, b)| a == b)
        || a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_extra_paths() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().to_path_buf();

        let config = LibraryConfig {
            include_xdg_music_dir: false,
            extra_paths: vec![path.clone(), path],
            ..LibraryConfig::default()
        };

        let roots = resolve_roots(&config);
        if roots.len() != 1 {
            return Err("expected deduplicated root list".into());
        }
        Ok(())
    }
}
