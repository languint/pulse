use gpui::{InteractiveElement, ParentElement, Pixels, Styled, div, px};

use crate::{components::toolbar::controls::ToolbarControls, config::PulseContext};

pub mod actions;
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
            .h(TOOLBAR_HEIGHT)
            .w_full()
            .flex()
            .items_center()
            .px(theme.spacing.md)
            .pr(px(0.))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .window_control_area(gpui::WindowControlArea::Drag)
            .child("Pulse")
            .child(div().flex_1())
            .child(ToolbarControls)
    }
}
