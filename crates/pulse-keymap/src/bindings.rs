use gpui::{Action, App, KeyBinding};

use crate::PulseKeymap;

pub struct PulseActionBindings<A> {
    pub toggle_fullscreen: A,
}

impl PulseKeymap {
    pub fn bind<A: Action + Clone>(&self, cx: &mut App, actions: PulseActionBindings<A>) {
        if !self.pulse.toggle_fullscreen.is_empty() {
            cx.bind_keys(self.pulse.toggle_fullscreen.iter().map(|keystroke| {
                KeyBinding::new(*keystroke, actions.toggle_fullscreen.clone(), None)
            }));
        }
    }
}
