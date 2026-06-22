use std::collections::HashMap;

use gpui::Global;
use pulse_keymap::{KeymapAction, PulseKeymap};
use pulse_library::LibraryConfig;
use pulse_data::{InterfaceSettings, VisualizerSettings};

#[derive(Clone)]
pub struct PulseConfig {
    pub theme: String,
    pub keymap: PulseKeymap,
    pub library: LibraryConfig,
    pub interface: InterfaceSettings,
    pub visualizer: VisualizerSettings,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            theme: pulse_data::DEFAULT_THEME.to_string(),
            keymap: PulseKeymap::default(),
            library: LibraryConfig::default(),
            interface: pulse_data::InterfaceSettings::default(),
            visualizer: VisualizerSettings::default(),
        }
    }
}

impl PulseConfig {
    #[must_use]
    pub fn from_settings(settings: pulse_data::PulseSettings, keymap: PulseKeymap) -> Self {
        Self {
            theme: settings.theme,
            keymap,
            library: settings.library,
            interface: settings.interface,
            visualizer: settings.visualizer,
        }
    }

    #[must_use]
    pub fn to_settings(&self) -> pulse_data::PulseSettings {
        pulse_data::PulseSettings {
            theme: self.theme.clone(),
            library: self.library.clone(),
            interface: self.interface.clone(),
            visualizer: self.visualizer.clone(),
        }
    }

    #[must_use]
    pub fn with_keymap_overrides(mut self, overrides: &HashMap<KeymapAction, Vec<String>>) -> Self {
        self.keymap.apply_overrides(overrides);
        self
    }
}

impl Global for PulseConfig {}

pub trait PulseContext {}

impl PulseContext for gpui::App {}
