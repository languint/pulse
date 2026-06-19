use gpui::{AppContext, Application, UpdateGlobal, WindowOptions};

pub mod config;
pub mod error;
pub mod fonts;
pub mod views;

use views::pulse::PulseView;

use crate::config::PulseConfig;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello Pulse!");

    Application::new().run(|mut cx| {
        if let Err(e) = cx.text_system().add_fonts(vec![fonts::INTER.into()]) {
            tracing::error!("failed to load fonts: {e}");
            cx.quit();
        };

        PulseConfig::set_global(
            &mut cx,
            PulseConfig {
                theme: pulse_theme::themes::pulse_dark(),
            },
        );

        if let Err(e) = cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| PulseView)) {
            tracing::error!("failed to open window: {e}");
            cx.quit();
        }
    });
}
