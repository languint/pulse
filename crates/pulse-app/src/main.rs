use gpui::{AppContext, UpdateGlobal, WindowOptions};
use gpui_component::{Root, TitleBar};

#[allow(clippy::derive_partial_eq_without_eq)]
pub mod actions;
pub mod artwork_prefetch;
pub mod assets;
pub mod bundled_themes;
pub mod components;
pub mod config;
pub mod data;
pub mod error;
pub mod icons;
pub mod library;
pub mod lyrics;
pub mod media_controls;
pub mod player;
pub mod pulse;
pub mod theme_list;

use actions::{
    CommandPaletteTab, ManageLibraryRoots, MediaNextTrack, MediaPlayPause, MediaPreviousTrack,
    OpenSettings, OpenVisualizerSettings, Quit, ShowOscilloscopeVisualizer,
    ShowSpectrumVisualizer, ToggleCommandPalette, ToggleFullscreen,
};
use components::{
    command_palette, library_roots_dialog::open_library_roots_dialog,
    pulse::{ActivePulse, Pulse}, settings_dialog::open_settings_dialog,
    visualizer_settings_dialog::open_visualizer_settings_dialog,
};
use pulse_data::VisualizerMode;
use pulse_keymap::KeymapAction;

use crate::config::PulseConfig;
use crate::data::{DataOverrides, DataPaths, OverridesGeneration};
use crate::library::PulseLibrary;
use crate::media_controls::MediaCommand;
use crate::player::PulsePlayer;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("starting Pulse");

    gpui_platform::application()
        .with_assets(assets::CombinedAssets)
        .run(move |cx| {
            gpui_component::init(cx);
            command_palette::init(cx);
            if let Err(error) = pulse_runtime::init(cx) {
                tracing::error!(%error, "failed to initialize Tokio runtime");
                std::process::exit(1);
            }

            let paths = match pulse::load_paths() {
                Ok(paths) => paths,
                Err(error) => {
                    tracing::error!(%error, "failed to initialize Pulse data directories");
                    std::process::exit(1);
                }
            };

            pulse::init(cx, &paths);

            let settings = pulse::load_settings(&paths);
            let keymap = pulse::load_keymap(&paths);
            let overrides = pulse::load_overrides(&paths);

            let config = PulseConfig::from_settings(settings, keymap);
            config
                .keymap
                .bind_action(cx, KeymapAction::ToggleFullscreen, &ToggleFullscreen);
            config
                .keymap
                .bind_action(cx, KeymapAction::ManageLibraryRoots, &ManageLibraryRoots);
            config.keymap.bind_action(cx, KeymapAction::Quit, &Quit);
            config
                .keymap
                .bind_action(cx, KeymapAction::MediaPlayPause, &MediaPlayPause);
            config
                .keymap
                .bind_action(cx, KeymapAction::MediaNextTrack, &MediaNextTrack);
            config
                .keymap
                .bind_action(cx, KeymapAction::MediaPreviousTrack, &MediaPreviousTrack);
            config
                .keymap
                .bind_action(cx, KeymapAction::ToggleCommandPalette, &ToggleCommandPalette);
            config
                .keymap
                .bind_action(cx, KeymapAction::OpenCommandPalette, &CommandPaletteTab);

            DataPaths::set_global(cx, DataPaths::new(paths.clone()));
            DataOverrides::set_global(cx, DataOverrides(overrides));
            OverridesGeneration::set_global(cx, OverridesGeneration::default());
            PulseConfig::set_global(cx, config.clone());

            pulse::apply_theme(cx, &config.theme);

            PulseLibrary::init(cx, config.library.clone(), paths.artwork_cache_dir());
            PulsePlayer::init(cx);
            lyrics::PulseLyrics::init(cx);

            cx.on_action(|_: &ManageLibraryRoots, cx| {
                if let Some(window) = cx.active_window() {
                    let _ = window.update(cx, |_, window, cx| {
                        open_library_roots_dialog(window, cx);
                    });
                }
            });

            cx.on_action(|_: &OpenSettings, cx| {
                if let Some(window) = cx.active_window() {
                    let _ = window.update(cx, |_, window, cx| {
                        open_settings_dialog(window, cx);
                    });
                }
            });

            cx.on_action(|_: &OpenVisualizerSettings, cx| {
                if let Some(window) = cx.active_window() {
                    let _ = window.update(cx, |_, window, cx| {
                        open_visualizer_settings_dialog(window, cx);
                    });
                }
            });

            cx.on_action(|action: &ShowSpectrumVisualizer, cx| {
                let _ = action;
                open_visualizer_mode_from_menu(VisualizerMode::Spectrum, cx);
            });

            cx.on_action(|action: &ShowOscilloscopeVisualizer, cx| {
                let _ = action;
                open_visualizer_mode_from_menu(VisualizerMode::Oscilloscope, cx);
            });

            cx.on_action(|_: &Quit, cx| {
                cx.quit();
            });

            cx.on_action(|_: &ToggleFullscreen, cx| {
                if let Some(window) = cx.active_window() {
                    let _ = window.update(cx, |_, window, _| window.toggle_fullscreen());
                }
            });

            cx.on_action(|_: &MediaPlayPause, cx| {
                media_controls::dispatch(MediaCommand::TogglePlayback, cx);
            });

            cx.on_action(|_: &MediaNextTrack, cx| {
                media_controls::dispatch(MediaCommand::Next, cx);
            });

            cx.on_action(|_: &MediaPreviousTrack, cx| {
                media_controls::dispatch(MediaCommand::Previous, cx);
            });

            cx.spawn(async move |cx| {
                let window_options = WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    ..Default::default()
                };

                if let Err(e) = cx.open_window(window_options, |window, cx| {
                    media_controls::init(window, cx);
                    let pulse = cx.new(|cx| Pulse::new(window, cx));
                    ActivePulse::set_global(cx, ActivePulse(pulse.clone()));
                    let focus_handle = pulse.read(cx).focus_handle.clone();
                    window.focus(&focus_handle, cx);
                    cx.new(|cx| Root::new(pulse, window, cx))
                }) {
                    tracing::error!("failed to open window: {e}");
                    std::process::exit(1);
                }
            })
            .detach();
        });
}

fn open_visualizer_mode_from_menu(mode: VisualizerMode, cx: &mut gpui::App) {
    if let Some(active) = cx.try_global::<ActivePulse>() {
        let pulse = active.0.clone();
        pulse.update(cx, |pulse, cx| {
            pulse.show_visualizer_mode(mode, cx);
        });
    }
}
