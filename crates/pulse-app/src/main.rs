use gpui::{AppContext, Application, UpdateGlobal, WindowOptions};

pub mod components;
pub mod config;
pub mod error;

use components::pulse::Pulse;

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
                let pulse_view = cx.new(Pulse::new);

                let focus_handle = pulse_view.read(cx).focus_handle.clone();
                window.focus(&focus_handle);

                pulse_view
            }) {
                tracing::error!("failed to open window: {e}");
                cx.quit();
            }
        });
}
