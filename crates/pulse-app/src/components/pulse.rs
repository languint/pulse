use gpui::{
    AppContext, FocusHandle, InteractiveElement, ParentElement, Render, Styled, Window, div,
};

use crate::{
    actions::ToggleFullscreen,
    components::{sidebar::Sidebar, toolbar::Toolbar},
    config::PulseContext,
};

pub struct Pulse {
    pub focus_handle: FocusHandle,
}

impl Pulse {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for Pulse {
    fn render(
        &mut self,
        _window: &mut Window,
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
            .child(
                div()
                    .id("content")
                    .flex_1()
                    .flex()
                    .min_h_0()
                    .track_focus(&self.focus_handle)
                    .on_action(cx.listener(|_, _: &ToggleFullscreen, window, _| {
                        window.toggle_fullscreen();
                        window.refresh();
                    }))
                    .child(cx.new(|_| Sidebar)),
            )
    }
}
