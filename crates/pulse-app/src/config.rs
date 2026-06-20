use gpui::Global;
// use pulse_theme::PulseTheme;

#[derive(Clone, Copy)]
pub struct PulseConfig {
    // pub theme: PulseTheme,
}

impl Global for PulseConfig {}

pub trait PulseContext {}

impl PulseContext for gpui::App {}
