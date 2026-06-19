use gpui::{InteractiveElement, ParentElement, Pixels, Styled, div, px};

use crate::{
    components::{
        toolbar::controls::ToolbarControls,
        ui::stack::{ItemAlignment, Stack, StackDirection},
    },
    config::PulseContext,
};

pub mod controls;

pub struct Toolbar;

pub const TOOLBAR_HEIGHT: Pixels = px(32.);

impl gpui::Render for Toolbar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let empty_filler = div()
            .flex_1()
            .h_full()
            .window_control_area(gpui::WindowControlArea::Drag);

        let toolbar = Stack::new(StackDirection::Horizontal)
            .align(ItemAlignment::Center)
            .id("toolbar")
            .h(TOOLBAR_HEIGHT)
            .w_full()
            .content_stretch()
            .px(theme.spacing.md)
            .pr(px(0.))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(empty_filler);

        if !window.is_fullscreen() {
            return toolbar.child(ToolbarControls);
        }

        toolbar
    }
}
