use gpui::{ParentElement, Styled, div, px};
use gpui_component::ActiveTheme;

pub struct Sidebar;

impl gpui::Render for Sidebar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        div()
            .w(px(260.))
            .h_full()
            .flex()
            .flex_col()
            .p(px(12.))
            .bg(theme.sidebar)
            .border_r_1()
            .border_color(theme.sidebar_border)
            .child("Pulse")
    }
}
