mod action;
mod bindings;
mod keymap;

pub use action::KeymapAction;
pub use bindings::bind_keystrokes;
pub use keymap::PulseKeymap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keymap_name() {
        assert_eq!(PulseKeymap::default().name, "Pulse");
    }

    #[test]
    fn default_toggle_fullscreen_binding() {
        assert_eq!(
            PulseKeymap::default().keystrokes_for(KeymapAction::ToggleFullscreen),
            &["f11".to_string(), "ctrl-f".to_string()]
        );
    }

    #[test]
    fn with_binding_overrides_defaults() {
        let keymap = PulseKeymap::default().with_binding(
            KeymapAction::ToggleFullscreen,
            vec!["ctrl-shift-f".into()],
        );

        assert_eq!(
            keymap.keystrokes_for(KeymapAction::ToggleFullscreen),
            &["ctrl-shift-f".to_string()]
        );
    }
}
