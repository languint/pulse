use gpui::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    WindowControlArea, div, px, svg,
};

use pulse_assets::icons::Icon;

use crate::{components::toolbar::TOOLBAR_HEIGHT, config::PulseContext};

#[derive(IntoElement)]
pub struct ToolbarControlButton {
    icon: Icon,
    control_area: WindowControlArea,
    hover: Option<Hsla>,
}

impl ToolbarControlButton {
    pub fn new(icon: Icon, control_area: WindowControlArea) -> Self {
        Self {
            icon,
            control_area,
            hover: None,
        }
    }

    pub fn hover_color(mut self, color: Hsla) -> Self {
        self.hover = Some(color);
        self
    }
}

impl RenderOnce for ToolbarControlButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl gpui::IntoElement {
        let theme = cx.theme();

        div()
            .w(px(36.))
            .h(TOOLBAR_HEIGHT)
            .flex()
            .justify_center()
            .items_center()
            .text_size(px(10.))
            .hover(|style| style.bg(self.hover.unwrap_or(theme.colors.surface_variant)))
            .window_control_area(self.control_area)
            .child(
                svg()
                    .path(self.icon.path())
                    .size(px(16.))
                    .text_color(theme.colors.text.primary),
            )
    }
}

#[derive(IntoElement)]
pub struct ToolbarControls;

impl RenderOnce for ToolbarControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .items_end()
            .child(ToolbarControlButton::new(
                Icon::MINIMIZE,
                WindowControlArea::Min,
            ))
            .child(ToolbarControlButton::new(
                if window.is_maximized() {
                    Icon::RESTORE
                } else {
                    Icon::MAXIMIZE
                },
                WindowControlArea::Max,
            ))
            .child(
                ToolbarControlButton::new(Icon::CLOSE, WindowControlArea::Close)
                    .hover_color(theme.colors.error),
            )
    }
}
