use gpui::{
    AppContext, Entity, FocusHandle, InteractiveElement, MouseMoveEvent, ParentElement, Render,
    Styled, Window, div,
};
use gpui_component::{ActiveTheme, TITLE_BAR_HEIGHT};

use crate::{
    actions::ToggleFullscreen,
    components::{sidebar::Sidebar, toolbar::Toolbar},
};

pub struct Pulse {
    pub focus_handle: FocusHandle,
    toolbar: Entity<Toolbar>,
    sidebar: Entity<Sidebar>,
}

impl Pulse {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            toolbar: cx.new(Toolbar::new),
            sidebar: cx.new(|_| Sidebar),
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

        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .text_size(theme.font_size)
            .child(self.toolbar.clone())
            .child(
                div()
                    .id("content")
                    .flex_1()
                    .flex()
                    .min_h_0()
                    .track_focus(&self.focus_handle)
                    .on_action(cx.listener(|_, _: &ToggleFullscreen, window, _| {
                        window.toggle_fullscreen();
                    }))
                    .child(self.sidebar.clone()),
            );

        // This is needed for now since there is a bug in gpui
        #[cfg(target_os = "windows")]
        {
            root = root.on_mouse_move(refresh_title_bar_hover);
        }

        root
    }
}

#[cfg(target_os = "windows")]
fn refresh_title_bar_hover(event: &MouseMoveEvent, window: &mut Window, _: &mut gpui::App) {
    if event.position.y > TITLE_BAR_HEIGHT {
        return;
    }

    let caption_width = TITLE_BAR_HEIGHT * 3.0;
    if event.position.x >= window.viewport_size().width - caption_width {
        window.refresh();
    }
}
