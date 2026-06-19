use gpui::{
    AppContext, FocusHandle, InteractiveElement, ParentElement, Render, Styled, Window, div,
};

use crate::{
    components::{
        sidebar::Sidebar,
        toolbar::{
            Toolbar,
            actions::{Close, Minimize},
        },
    },
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

impl Pulse {
    pub fn register_actions(&self, cx: &mut gpui::Context<Self>, div: gpui::Div) -> gpui::Div {
        div.track_focus(&self.focus_handle)
            .on_action(cx.listener(|_, _: &Minimize, window, cx| {
                window.minimize_window();
                cx.notify();
            }))
            .on_action(cx.listener(|_, _: &Close, _, cx| {
                cx.quit();
            }))
    }
}

impl Render for Pulse {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let theme = cx.theme();

        let div = self.register_actions(cx, div());

        div.size_full()
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
