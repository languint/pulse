use gpui::{AppContext, ParentElement, Styled, div};

use crate::{
    components::{sidebar::Sidebar, toolbar::Toolbar},
    config::PulseContext,
};

pub struct Pulse;

impl gpui::Render for Pulse {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let theme = cx.theme();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.background)
            .text_color(theme.colors.text.primary)
            .text_size(theme.typography.body.size)
            .font(theme.typography.font(theme.typography.body))
            .child(cx.new(|_| Toolbar))
            .child(cx.new(|_| Sidebar))
    }
}
