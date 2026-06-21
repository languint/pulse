use std::ops::Deref;
use std::path::Path;

use gpui::{App, Global, UpdateGlobal};
use pulse_data::{DataError, PulsePaths, UserOverrides};

/// Application-global handle to [`PulsePaths`].
#[derive(Clone, Debug)]
pub struct DataPaths(PulsePaths);

impl DataPaths {
    #[must_use]
    pub const fn new(paths: PulsePaths) -> Self {
        Self(paths)
    }
}

impl Deref for DataPaths {
    type Target = PulsePaths;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Global for DataPaths {}

#[derive(Clone, Debug, Default)]
pub struct DataOverrides(pub UserOverrides);

impl Deref for DataOverrides {
    type Target = UserOverrides;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Global for DataOverrides {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OverridesGeneration(pub u32);

impl Global for OverridesGeneration {}

fn bump_overrides_generation(cx: &mut App) {
    OverridesGeneration::update_global(cx, |generation, _| {
        generation.0 = generation.0.wrapping_add(1);
    });
}

fn remove_stored_artwork(paths: &PulsePaths, relative: &Path) {
    let absolute = paths.resolve_data_path(relative);
    if absolute.starts_with(paths.custom_artwork_dir()) {
        let _ = std::fs::remove_file(absolute);
    }
}

/// Writes application settings to disk.
///
/// # Errors
///
/// Returns [`pulse_data::DataError`] when the settings file cannot be written.
pub fn persist_settings(
    paths: &PulsePaths,
    settings: &pulse_data::PulseSettings,
) -> Result<(), pulse_data::DataError> {
    settings.save(paths)
}

/// Writes keymap bindings to disk.
///
/// # Errors
///
/// Returns [`pulse_data::DataError`] when the keymap file cannot be written.
pub fn persist_keymap(
    paths: &PulsePaths,
    keymap: &pulse_keymap::PulseKeymap,
) -> Result<(), pulse_data::DataError> {
    pulse_data::KeymapFile::from_keymap(keymap).save(paths)
}

/// Writes user metadata overrides to disk.
///
/// # Errors
///
/// Returns [`pulse_data::DataError`] when the overrides file cannot be written.
pub fn persist_overrides(
    paths: &PulsePaths,
    overrides: &UserOverrides,
) -> Result<(), pulse_data::DataError> {
    overrides.save(paths)
}

/// Updates and persists user-added labels for an album.
pub fn save_album_user_labels(cx: &mut App, override_key: &str, labels: &[String]) {
    let paths = cx.global::<DataPaths>().clone();

    DataOverrides::update_global(cx, |data, _| {
        data.0
            .set_album_user_labels(override_key.to_string(), labels);

        if let Err(error) = persist_overrides(&paths, &data.0) {
            tracing::error!(%error, "failed to save metadata overrides");
        }
    });

    bump_overrides_generation(cx);
    cx.refresh_windows();
}

/// Updates and persists custom artwork for an album.
pub fn save_album_artwork(cx: &mut App, override_key: &str, artwork: Option<std::path::PathBuf>) {
    let paths = cx.global::<DataPaths>().clone();

    if artwork.is_none() {
        if let Some(previous) = cx
            .global::<DataOverrides>()
            .album(override_key)
            .and_then(|entry| entry.artwork.clone())
        {
            remove_stored_artwork(&paths, &previous);
        }
    }

    DataOverrides::update_global(cx, |data, _| {
        data.0.set_album_artwork(override_key.to_string(), artwork);

        if let Err(error) = persist_overrides(&paths, &data.0) {
            tracing::error!(%error, "failed to save metadata overrides");
        }
    });

    bump_overrides_generation(cx);
    cx.refresh_windows();
}

/// Updates and persists custom artwork for an artist.
pub fn save_artist_artwork(cx: &mut App, override_key: &str, artwork: Option<std::path::PathBuf>) {
    let paths = cx.global::<DataPaths>().clone();

    DataOverrides::update_global(cx, |data, _| {
        data.0.set_artist_artwork(override_key.to_string(), artwork);

        if let Err(error) = persist_overrides(&paths, &data.0) {
            tracing::error!(%error, "failed to save metadata overrides");
        }
    });

    bump_overrides_generation(cx);
    cx.refresh_windows();
}

/// Imports an image file into Pulse data storage and saves it as an album cover override.
///
/// # Errors
///
/// Returns [`DataError`] when the image cannot be copied or overrides cannot be saved.
pub fn import_and_save_album_cover(
    cx: &mut App,
    override_key: &str,
    source: &Path,
) -> Result<(), DataError> {
    let paths = cx.global::<DataPaths>().clone();
    let previous = cx
        .global::<DataOverrides>()
        .album(override_key)
        .and_then(|entry| entry.artwork.clone());

    if let Some(previous) = previous.as_ref() {
        remove_stored_artwork(&paths, previous);
    }

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    let relative_path = paths.import_custom_artwork(
        &format!("album-{override_key}-{stamp}"),
        source,
    )?;

    DataOverrides::update_global(cx, |data, _| {
        data.0
            .set_album_artwork(override_key.to_string(), Some(relative_path.clone()));

        if let Err(error) = persist_overrides(&paths, &data.0) {
            tracing::error!(%error, "failed to save metadata overrides");
        }
    });

    bump_overrides_generation(cx);
    cx.refresh_windows();
    Ok(())
}

/// Imports an image file into Pulse data storage and saves it as an artist logo override.
///
/// # Errors
///
/// Returns [`DataError`] when the image cannot be copied or overrides cannot be saved.
pub fn import_and_save_artist_logo(
    cx: &mut App,
    override_key: &str,
    source: &Path,
) -> Result<(), DataError> {
    let paths = cx.global::<DataPaths>().clone();
    let relative_path = paths.import_custom_artwork(&format!("artist-{override_key}"), source)?;

    DataOverrides::update_global(cx, |data, _| {
        data.0
            .set_artist_artwork(override_key.to_string(), Some(relative_path.clone()));

        if let Err(error) = persist_overrides(&paths, &data.0) {
            tracing::error!(%error, "failed to save metadata overrides");
        }
    });

    bump_overrides_generation(cx);
    cx.refresh_windows();
    Ok(())
}
