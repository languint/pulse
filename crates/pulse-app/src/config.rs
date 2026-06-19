use gpui::{Global, ReadGlobal};
use pulse_theme::PulseTheme;

#[derive(Clone, Copy)]
pub struct PulseConfig {
    pub theme: PulseTheme,
}

impl Global for PulseConfig {}

pub trait PulseContext {
    fn theme(&self) -> PulseTheme;
}

impl PulseContext for gpui::App {
    fn theme(&self) -> PulseTheme {
        PulseConfig::global(self).theme
    }
}
