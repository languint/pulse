use std::ops::Deref;

use gpui::{App, Global, UpdateGlobal};
use pulse_data::{PulsePaths, UserOverrides};

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

/// Application-global user metadata overrides.
#[derive(Clone, Debug, Default)]
pub struct DataOverrides(pub UserOverrides);

impl Deref for DataOverrides {
    type Target = UserOverrides;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Global for DataOverrides {}

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
        data.0.set_album_user_labels(override_key.to_string(), labels);

        if let Err(error) = persist_overrides(&paths, &data.0) {
            tracing::error!(%error, "failed to save metadata overrides");
        }
    });

    cx.refresh_windows();
}
