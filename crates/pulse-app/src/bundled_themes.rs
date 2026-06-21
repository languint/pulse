use std::rc::Rc;
use std::sync::LazyLock;

use gpui::{App, SharedString};
use gpui_component::{ThemeConfig, ThemeRegistry, ThemeSet};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/themes"]
#[include = "*.json"]
struct BundledThemeSources;

static BUNDLED: LazyLock<Vec<ThemeConfig>> = LazyLock::new(|| {
    let mut themes = Vec::new();

    for (path, source) in bundled_sources() {
        match serde_json::from_str::<ThemeSet>(&source) {
            Ok(theme_set) => themes.extend(theme_set.themes),
            Err(error) => tracing::error!(%path, %error, "failed to parse bundled theme"),
        }
    }

    themes
});

fn bundled_sources() -> Vec<(String, String)> {
    let mut paths: Vec<String> = BundledThemeSources::iter()
        .map(|path| path.into_owned())
        .collect();
    paths.sort_unstable();

    paths
        .into_iter()
        .filter_map(|path| {
            let file = BundledThemeSources::get(&path)?;
            let source = std::str::from_utf8(&file.data)
                .map(str::to_owned)
                .inspect_err(|error| {
                    tracing::error!(%path, %error, "bundled theme is not valid UTF-8");
                })
                .ok()?;
            Some((path, source))
        })
        .collect()
}

/// Bundled Pulse themes compiled into the application binary.
pub fn bundled_themes() -> &'static [ThemeConfig] {
    &BUNDLED
}

/// Loads all bundled themes into the global registry (survives registry reloads).
pub fn inject_bundled_themes(cx: &mut App) {
    for (path, source) in bundled_sources() {
        if let Err(error) = ThemeRegistry::global_mut(cx).load_themes_from_str(&source) {
            tracing::error!(%path, %error, "failed to inject bundled theme");
        }
    }
}

/// Resolves a bundled theme by name.
#[must_use]
pub fn resolve_theme(name: &str) -> Option<Rc<ThemeConfig>> {
    bundled_themes()
        .iter()
        .find(|theme| theme.name.as_ref() == name)
        .map(|theme| Rc::new(theme.clone()))
}

/// Sort key that keeps bundled themes at the top in a stable order.
pub fn bundled_sort_key(name: &SharedString) -> u8 {
    match name.as_ref() {
        "Pulse Dark" => 0,
        "Pulse Light" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_all_theme_files() {
        assert!(
            BundledThemeSources::iter().count() >= 24,
            "expected bundled theme files under src/themes"
        );
    }

    #[test]
    fn includes_pulse_defaults_and_added_themes() {
        let names: Vec<_> = bundled_themes()
            .iter()
            .map(|theme| theme.name.to_string())
            .collect();

        assert!(names.iter().any(|name| name == "Pulse Dark"));
        assert!(names.iter().any(|name| name == "Pulse Light"));
        assert!(names.iter().any(|name| name == "Adventure"));
        assert!(names.len() > 2);
    }
}
