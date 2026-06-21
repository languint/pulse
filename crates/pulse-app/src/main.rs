use gpui::{AppContext, UpdateGlobal, WindowOptions};
use gpui_component::{Root, TitleBar};

#[allow(clippy::derive_partial_eq_without_eq)]
pub mod actions;
pub mod artwork_prefetch;
pub mod assets;
pub mod components;
pub mod config;
pub mod data;
pub mod error;
pub mod icons;
pub mod library;
pub mod media_controls;
pub mod player;
pub mod pulse;

use actions::{
    ManageLibraryRoots, MediaNextTrack, MediaPlayPause, MediaPreviousTrack, Quit, ToggleFullscreen,
};
use components::{library_roots_dialog::open_library_roots_dialog, pulse::Pulse};
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

            DataPaths::set_global(cx, DataPaths::new(paths.clone()));
            DataOverrides::set_global(cx, DataOverrides(overrides));
            OverridesGeneration::set_global(cx, OverridesGeneration::default());
            PulseConfig::set_global(cx, config.clone());

            pulse::apply_theme(cx, &config.theme);

            PulseLibrary::init(cx, config.library.clone(), paths.artwork_cache_dir());
            PulsePlayer::init(cx);

            cx.on_action(|_: &ManageLibraryRoots, cx| {
                if let Some(window) = cx.active_window() {
                    let _ = window.update(cx, |_, window, cx| {
                        open_library_roots_dialog(window, cx);
                    });
                }
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
                    let pulse = cx.new(Pulse::new);
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
