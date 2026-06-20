use gpui::{AppContext, UpdateGlobal, WindowOptions};
use gpui_component::Root;

pub mod actions;
pub mod components;
pub mod config;
pub mod error;
pub mod pulse;

use actions::ToggleFullscreen;
use components::pulse::Pulse;
use pulse_keymap::{PulseActionBindings, PulseKeymap};

use crate::config::PulseConfig;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello Pulse!");

    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            pulse::init(cx);

            PulseConfig::set_global(cx, PulseConfig {});

            PulseKeymap::default().bind(
                cx,
                PulseActionBindings {
                    toggle_fullscreen: ToggleFullscreen,
                },
            );

            cx.spawn(async move |cx| {
                let window_options = WindowOptions {
                    titlebar: None,
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
