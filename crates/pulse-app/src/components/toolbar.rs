use gpui::{InteractiveElement, ParentElement, Pixels, Styled, div, px};

use crate::{components::toolbar::controls::ToolbarControls, config::PulseContext};

pub mod controls;

pub struct Toolbar;

pub const TOOLBAR_HEIGHT: Pixels = px(32.);

impl gpui::Render for Toolbar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        div()
            .id("toolbar")
            .h(TOOLBAR_HEIGHT)
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .content_stretch()
            .px(theme.spacing.md)
            .pr(px(0.))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .window_control_area(gpui::WindowControlArea::Drag)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h_full()
                    .child("Pulse"),
            )
            .child(div().flex_1().h_full())
            .child(ToolbarControls)
    }
}
