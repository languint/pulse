use gpui::{AppContext, ParentElement, Styled};

use crate::{config::PulseContext, views::sidebar::SidebarView};

pub struct PulseView;

impl gpui::Render for PulseView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let theme = cx.theme();

        gpui::div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.background)
            .text_color(theme.colors.text.primary)
            .text_size(theme.typography.body.size)
            .font(theme.typography.font(theme.typography.body))
            .child(cx.new(|_| SidebarView))
    }
}
