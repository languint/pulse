use std::collections::HashMap;

use gpui::Global;
use pulse_keymap::{KeymapAction, PulseKeymap};

#[derive(Clone)]
pub struct PulseConfig {
    pub keymap: PulseKeymap,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            keymap: PulseKeymap::default(),
        }
    }
}

impl PulseConfig {
    pub fn with_keymap_overrides(mut self, overrides: HashMap<KeymapAction, Vec<String>>) -> Self {
        self.keymap.apply_overrides(&overrides);
        self
    }
}

impl Global for PulseConfig {}

pub trait PulseContext {}

impl PulseContext for gpui::App {}
