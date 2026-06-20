use gpui::SharedString;
use gpui_component::{Theme, ThemeRegistry};

const DEFAULT_THEME: &str = "Pulse Dark";
const PULSE_DARK_THEME: &str = include_str!("themes/pulse_dark.json");

pub fn init(cx: &mut gpui::App) {
    let theme_name = SharedString::from(DEFAULT_THEME);

    if let Err(e) = ThemeRegistry::global_mut(cx).load_themes_from_str(PULSE_DARK_THEME) {
        tracing::error!("failed to load pulse dark theme: {e}");
    }

    apply_theme(cx, &theme_name);
}

fn apply_theme(cx: &mut gpui::App, theme_name: &SharedString) {
    if let Some(theme) = ThemeRegistry::global(cx).themes().get(theme_name).cloned() {
        tracing::info!("applying `{theme_name}` theme");
        Theme::global_mut(cx).apply_config(&theme);
        cx.refresh_windows();
    } else {
        tracing::warn!("theme `{theme_name}` not found");
    }
}
