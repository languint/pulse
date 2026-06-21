use gpui::{
    AppContext, Entity, FocusHandle, InteractiveElement, MouseMoveEvent, ParentElement, Render,
    Styled, Window, div,
};
use gpui_component::{ActiveTheme, Root, TITLE_BAR_HEIGHT};

use crate::{
    actions::{ManageLibraryRoots, ToggleFullscreen},
    components::{library_roots_dialog::open_library_roots_dialog, sidebar::Sidebar, toolbar::Toolbar},
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
        window: &mut Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let theme = cx.theme();
        let background = theme.background;
        let foreground = theme.foreground;
        let font_size = theme.font_size;
        let dialog_layer = Root::render_dialog_layer(window, cx);

        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(background)
            .text_color(foreground)
            .text_size(font_size)
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
                    .on_action(cx.listener(|_, _: &ManageLibraryRoots, window, cx| {
                        open_library_roots_dialog(window, cx);
                    }))
                    .child(self.sidebar.clone()),
            )
            .children(dialog_layer);

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
