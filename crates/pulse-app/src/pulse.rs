use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use pulse_data::{KeymapFile, PulsePaths, PulseSettings, UserOverrides};

const PULSE_DARK_THEME: &str = include_str!("themes/pulse_dark.json");

pub fn init(cx: &mut App, paths: &PulsePaths) {
    if let Err(e) = ThemeRegistry::global_mut(cx).load_themes_from_str(PULSE_DARK_THEME) {
        tracing::error!("failed to load pulse dark theme: {e}");
    }

    if let Err(error) = ThemeRegistry::watch_dir(paths.themes_dir(), cx, |_| {}) {
        tracing::error!(%error, "failed to watch user themes directory");
    }
}

pub fn apply_theme(cx: &mut App, theme_name: &str) {
    let theme_name = SharedString::from(theme_name);

    if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
        tracing::info!("applying `{theme_name}` theme");
        Theme::global_mut(cx).apply_config(&theme);
        cx.refresh_windows();
    } else {
        tracing::warn!("theme `{theme_name}` not found");
    }
}

pub fn load_paths() -> Result<PulsePaths, pulse_data::DataError> {
    let paths = PulsePaths::platform_default();
    paths.ensure_all()?;
    tracing::info!(
        config = %paths.config_dir().display(),
        data = %paths.data_dir().display(),
        cache = %paths.cache_dir().display(),
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
