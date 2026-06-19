use crate::pulse::PulseKeys;

pub mod bindings;
pub mod pulse;

pub use bindings::PulseActionBindings;

#[derive(Debug, Clone)]
pub struct PulseKeymap {
    pub name: &'static str,

    pub pulse: PulseKeys,
}

impl Default for PulseKeymap {
    fn default() -> Self {
        Self {
            name: "Pulse",
            pulse: PulseKeys::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pulse::PulseKeys;
    use super::*;

    #[test]
    fn default_keymap_name() {
        assert_eq!(PulseKeymap::default().name, "Pulse");
    }

    #[test]
    fn default_toggle_fullscreen_binding() {
        assert_eq!(
            PulseKeys::default().toggle_fullscreen,
            vec!["f11"]
        );
    }
}
