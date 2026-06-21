use gpui::Global;

#[derive(Clone, Copy)]
pub struct PulseConfig {}

impl Global for PulseConfig {}

pub trait PulseContext {}

impl PulseContext for gpui::App {}
