use gpui::{AppContext, Application, UpdateGlobal, WindowOptions};

pub mod actions;
pub mod components;
pub mod config;
pub mod error;

use actions::ToggleFullscreen;
use components::pulse::Pulse;
use pulse_keymap::{PulseActionBindings, PulseKeymap};

use crate::config::PulseConfig;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello Pulse!");

    Application::new()
        .with_assets(pulse_assets::PulseAssetSource)
        .run(|mut cx| {
            if let Err(e) = cx
                .text_system()
                .add_fonts(vec![pulse_assets::fonts::INTER.into()])
            {
                tracing::error!("failed to load fonts: {e}");
                cx.quit();
            }

            PulseConfig::set_global(
                &mut cx,
                PulseConfig {
                    theme: pulse_theme::themes::pulse_dark(),
                },
            );

            let window_options = WindowOptions {
                titlebar: None,
                ..Default::default()
            };

            if let Err(e) = cx.open_window(window_options, |window, cx| {
                let pulse = cx.new(Pulse::new);
                window.focus(&pulse.read(cx).focus_handle);
                pulse
            }) {
                tracing::error!("failed to open window: {e}");
                cx.quit();
            }

            PulseKeymap::default().bind(
                &mut cx,
                PulseActionBindings {
                    toggle_fullscreen: ToggleFullscreen,
                },
            );
        });
}
