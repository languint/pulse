use std::collections::HashMap;

use crate::action::KeymapAction;

#[derive(Debug, Clone)]
pub struct PulseKeymap {
    pub name: String,
    bindings: HashMap<KeymapAction, Vec<String>>,
}

impl Default for PulseKeymap {
    fn default() -> Self {
        Self {
            name: "Pulse".into(),
            bindings: default_bindings(),
        }
    }
}

impl PulseKeymap {
    #[must_use]
    pub const fn bindings(&self) -> &HashMap<KeymapAction, Vec<String>> {
        &self.bindings
    }

    #[must_use]
    pub fn keystrokes_for(&self, action: KeymapAction) -> &[String] {
        self.bindings.get(&action).map_or(&[], Vec::as_slice)
    }

    pub fn set_binding(&mut self, action: KeymapAction, keystrokes: Vec<String>) {
        self.bindings.insert(action, keystrokes);
    }

    #[must_use]
    pub fn with_binding(mut self, action: KeymapAction, keystrokes: Vec<String>) -> Self {
        self.set_binding(action, keystrokes);
        self
    }

    pub fn apply_overrides(&mut self, overrides: &HashMap<KeymapAction, Vec<String>>) {
        for (action, keystrokes) in overrides {
            self.bindings.insert(*action, keystrokes.clone());
        }
    }
}

fn default_bindings() -> HashMap<KeymapAction, Vec<String>> {
    HashMap::from([
        (
            KeymapAction::ToggleFullscreen,
            vec!["f11".into(), "ctrl-f".into()],
        ),
        (KeymapAction::Quit, default_quit_keystrokes()),
        (
            KeymapAction::ManageLibraryRoots,
            vec!["ctrl-shift-l".into()],
        ),
        (
            KeymapAction::MediaPlayPause,
            vec![
                "space".into(),
                "mediaplaypause".into(),
                "xf86audioplay".into(),
                "xf86audiopause".into(),
            ],
        ),
        (
            KeymapAction::MediaNextTrack,
            vec!["]".into(), "medianexttrack".into(), "xf86audionext".into()],
        ),
        (
            KeymapAction::MediaPreviousTrack,
            vec!["[".into(), "mediaprevtrack".into(), "xf86audioprev".into()],
        ),
    ])
}

fn default_quit_keystrokes() -> Vec<String> {
    if cfg!(target_os = "macos") {
        vec!["cmd-q".into()]
    } else {
        vec!["ctrl-q".into()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_media_keyboard_bindings() {
        let keymap = PulseKeymap::default();

        assert_eq!(
            keymap.keystrokes_for(KeymapAction::MediaPlayPause),
            &[
                "space".to_string(),
                "mediaplaypause".to_string(),
                "xf86audioplay".to_string(),
                "xf86audiopause".to_string(),
            ]
        );
        assert_eq!(
            keymap.keystrokes_for(KeymapAction::MediaNextTrack),
            &[
                "]".to_string(),
                "medianexttrack".to_string(),
                "xf86audionext".to_string(),
            ]
        );
        assert_eq!(
            keymap.keystrokes_for(KeymapAction::MediaPreviousTrack),
            &[
                "[".to_string(),
                "mediaprevtrack".to_string(),
                "xf86audioprev".to_string(),
            ]
        );
    }

    #[test]
    fn default_quit_uses_ctrl_q_on_non_macos() {
        if cfg!(target_os = "macos") {
            return;
        }

        assert_eq!(
            PulseKeymap::default().keystrokes_for(KeymapAction::Quit),
            &["ctrl-q".to_string()]
        );
    }
}
