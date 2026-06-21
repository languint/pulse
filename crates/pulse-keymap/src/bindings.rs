use gpui::{Action, App, KeyBinding};

use crate::{KeymapAction, PulseKeymap};

pub fn bind_keystrokes<A: Action + Clone>(cx: &mut App, keystrokes: &[String], action: A) {
    if keystrokes.is_empty() {
        return;
    }

    cx.bind_keys(
        keystrokes
            .iter()
            .map(|keystroke| KeyBinding::new(keystroke.as_str(), action.clone(), None)),
    );
}

impl PulseKeymap {
    pub fn bind_action<A: Action + Clone>(&self, cx: &mut App, id: KeymapAction, action: A) {
        bind_keystrokes(cx, self.keystrokes_for(id), action);
    }
}
