use std::rc::Rc;

use gpui::{App, SharedString, UpdateGlobal};
use gpui_component::{Theme, ThemeConfig, ThemeRegistry};
use pulse_data::{KeymapFile, PulsePaths, PulseSettings, UserOverrides};

use crate::components::toolbar::menus;
use crate::config::PulseConfig;
use crate::data::{DataPaths, persist_settings};
use crate::theme_list::{selectable_themes, theme_by_name};

pub fn init(cx: &mut App, paths: &PulsePaths) {
    crate::bundled_themes::inject_bundled_themes(cx);

    cx.observe_global::<ThemeRegistry>(|cx| {
        crate::bundled_themes::inject_bundled_themes(cx);
    })
    .detach();

    if let Err(error) = ThemeRegistry::watch_dir(paths.themes_dir(), cx, |cx| {
        crate::bundled_themes::inject_bundled_themes(cx);
        refresh_after_theme_reload(cx);
    }) {
        tracing::error!(%error, "failed to watch user themes directory");
    }
}

fn refresh_after_theme_reload(cx: &mut App) {
    if cx.try_global::<PulseConfig>().is_some() {
        let theme = cx.global::<PulseConfig>().theme.clone();
        apply_theme(cx, &theme);
    }

    menus::refresh(cx);
}

fn theme_config(cx: &App, name: &SharedString) -> Option<Rc<ThemeConfig>> {
    theme_by_name(cx, name.as_ref())
}

pub fn apply_theme(cx: &mut App, theme_name: &str) {
    preview_theme(cx, theme_name);
}

/// Applies a theme without persisting it (used for command palette preview).
pub fn preview_theme(cx: &mut App, theme_name: &str) {
    let theme_name = SharedString::from(theme_name);

    if let Some(theme) = theme_config(cx, &theme_name) {
        Theme::global_mut(cx).apply_config(&theme);
        cx.refresh_windows();
    } else if theme_name.as_ref() != cx.global::<PulseConfig>().theme.as_str() {
        tracing::warn!(
            theme = %theme_name,
            available = ?selectable_themes(cx)
                .iter()
                .map(|theme| theme.name.to_string())
                .collect::<Vec<_>>(),
            "theme not found"
        );
    }
}

/// Applies a theme, updates global config, and persists the choice to settings.
pub fn set_theme(cx: &mut App, theme_name: &str) {
    let theme_name = theme_name.to_string();
    if theme_name == cx.global::<PulseConfig>().theme {
        preview_theme(cx, &theme_name);
        return;
    }

    preview_theme(cx, &theme_name);

    tracing::info!(theme = %theme_name, "saved theme preference");

    PulseConfig::update_global(cx, |config, _| {
        config.theme = theme_name;
    });

    let paths = cx.global::<DataPaths>().clone();
    let settings = cx.global::<PulseConfig>().to_settings();
    if let Err(error) = persist_settings(&paths, &settings) {
        tracing::error!(%error, "failed to save theme setting");
    }

    menus::refresh(cx);
}

/// Resolves platform data directories and creates them if needed.
///
/// # Errors
///
/// Returns [`pulse_data::DataError`] when a directory cannot be created.
pub fn load_paths() -> Result<PulsePaths, pulse_data::DataError> {
    let paths = PulsePaths::platform_default();
    paths.ensure_all()?;
    tracing::info!(
        config = %paths.config_dir().display(),
        data = %paths.data_dir().display(),
        cache = %paths.cache_dir().display(),
        themes = %paths.themes_dir().display(),
        "Pulse data directories ready"
    );
    Ok(paths)
}

pub fn load_settings(paths: &PulsePaths) -> PulseSettings {
    match PulseSettings::load(paths) {
        Ok(settings) => settings,
        Err(error) => {
            tracing::error!(%error, "failed to load settings; using defaults");
            PulseSettings::default()
        }
    }
}

pub fn load_keymap(paths: &PulsePaths) -> pulse_keymap::PulseKeymap {
    match KeymapFile::load(paths) {
        Ok(file) => file.into_keymap(),
        Err(error) => {
            tracing::error!(%error, "failed to load keymap; using defaults");
            pulse_keymap::PulseKeymap::default()
        }
    }
}

pub fn load_overrides(paths: &PulsePaths) -> UserOverrides {
    match UserOverrides::load(paths) {
        Ok(overrides) => overrides,
        Err(error) => {
            tracing::error!(%error, "failed to load metadata overrides; using empty set");
            UserOverrides::default()
        }
    }
}
