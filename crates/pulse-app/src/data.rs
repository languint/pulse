use std::ops::Deref;

use gpui::Global;
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
