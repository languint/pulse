#[derive(Debug, Clone)]
pub struct PulseKeys {
    pub toggle_fullscreen: Vec<&'static str>,
}

impl Default for PulseKeys {
    fn default() -> Self {
        Self {
            toggle_fullscreen: vec!["f11", "ctrl-f"],
        }
    }
}
