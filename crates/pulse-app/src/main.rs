use gpui::{AppContext, UpdateGlobal, WindowOptions};
use gpui_component::{Root, TitleBar};

pub mod actions;
pub mod components;
pub mod config;
pub mod error;
pub mod library;
pub mod pulse;

use actions::{ManageLibraryRoots, Quit, ToggleFullscreen};
use components::{library_roots_dialog::open_library_roots_dialog, pulse::Pulse};
use pulse_keymap::KeymapAction;

use crate::config::PulseConfig;
use crate::library::PulseLibrary;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello Pulse!");

    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            pulse::init(cx);

            let config = PulseConfig::default();
            config
                .keymap
                .bind_action(cx, KeymapAction::ToggleFullscreen, &ToggleFullscreen);
            config
                .keymap
                .bind_action(cx, KeymapAction::ManageLibraryRoots, &ManageLibraryRoots);
            config.keymap.bind_action(cx, KeymapAction::Quit, &Quit);
            PulseConfig::set_global(cx, config.clone());

            PulseLibrary::init(cx, config.library);

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

            cx.spawn(async move |cx| {
                let window_options = WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    ..Default::default()
                };

                if let Err(e) = cx.open_window(window_options, |window, cx| {
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
