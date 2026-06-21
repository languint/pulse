use std::collections::HashMap;
use std::rc::Rc;

use gpui::SharedString;
use gpui_component::{ThemeConfig, ThemeRegistry};

use crate::bundled_themes::{bundled_sort_key, bundled_themes, resolve_theme};

/// Themes shown in Pulse's picker: bundled Pulse themes and user themes, not gpui defaults.
pub fn selectable_themes(cx: &gpui::App) -> Vec<Rc<ThemeConfig>> {
    let mut themes: HashMap<_, Rc<ThemeConfig>> = HashMap::new();

    for theme in bundled_themes() {
        themes.insert(theme.name.clone(), Rc::new(theme.clone()));
    }

    for theme in ThemeRegistry::global(cx).sorted_themes() {
        if theme.is_default {
            continue;
        }
        themes.insert(theme.name.clone(), theme.clone());
    }

    let mut themes: Vec<_> = themes.into_values().collect();
    themes.sort_by(|left, right| {
        bundled_sort_key(&left.name)
            .cmp(&bundled_sort_key(&right.name))
            .then(
                left.name
                    .to_lowercase()
                    .cmp(&right.name.to_lowercase()),
            )
    });
    themes
}

/// Resolves a theme by name from the registry or bundled sources.
#[must_use]
pub fn theme_by_name(cx: &gpui::App, name: &str) -> Option<Rc<ThemeConfig>> {
    let name = SharedString::from(name);
    ThemeRegistry::global(cx)
        .themes()
        .get(&name)
        .cloned()
        .or_else(|| resolve_theme(name.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_themes_are_always_listed_without_registry() {
        let names: Vec<_> = bundled_themes()
            .iter()
            .map(|theme| theme.name.to_string())
            .collect();

        assert!(names.contains(&"Pulse Dark".to_string()));
        assert!(names.contains(&"Pulse Light".to_string()));
    }
}
