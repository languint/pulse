use gpui::{ParentElement, Styled};

use crate::config::PulseContext;

pub struct SidebarView;

impl gpui::Render for SidebarView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        gpui::div()
            .w(gpui::px(260.))
            .h_full()
            .flex()
            .flex_col()
            .p(theme.spacing.md)
            .bg(theme.colors.surface)
            .border_r_1()
            .border_color(theme.colors.border)
            .child("Pulse")
    }
}
