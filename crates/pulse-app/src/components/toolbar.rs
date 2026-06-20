use gpui::{InteractiveElement, ParentElement, Pixels, Styled, div, px};
use gpui_component::ActiveTheme;

use crate::components::{
    toolbar::controls::ToolbarControls,
    ui::stack::{ItemAlignment, Stack, StackDirection},
};
pub mod controls;
pub mod menubar;

pub struct Toolbar;

pub const TOOLBAR_HEIGHT: Pixels = px(32.);

impl gpui::Render for Toolbar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let mut empty_filler = div()
            .flex_1()
            .h_full()
            .window_control_area(gpui::WindowControlArea::Drag);

        #[cfg(target_os = "windows")]
        {
            empty_filler = empty_filler.on_mouse_move(|_, window, _| window.refresh());
        }

        let toolbar = Stack::new(StackDirection::Horizontal)
            .align(ItemAlignment::Center)
            .id("toolbar")
            .h(TOOLBAR_HEIGHT)
            .w_full()
            .content_stretch()
            .px(px(8.))
            .pr(px(0.))
            .bg(theme.tokens.title_bar)
            .border_b_1()
            .border_color(theme.title_bar_border)
            .child(empty_filler);

        if !window.is_fullscreen() {
            return toolbar.child(ToolbarControls);
        }

        toolbar
    }
}
