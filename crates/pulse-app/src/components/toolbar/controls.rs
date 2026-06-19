use gpui::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    WindowControlArea, div, px,
};

use crate::components::ui::prelude::*;

use pulse_assets::icons::IconName;

use crate::{components::toolbar::TOOLBAR_HEIGHT, config::PulseContext};

#[derive(IntoElement)]
pub struct ToolbarControlButton {
    icon: IconName,
    control_area: WindowControlArea,
    hover: Option<Hsla>,
}

impl ToolbarControlButton {
    #[must_use]
    pub const fn new(icon: IconName, control_area: WindowControlArea) -> Self {
        Self {
            icon,
            control_area,
            hover: None,
        }
    }

    #[must_use]
    pub const fn hover_color(mut self, color: Hsla) -> Self {
        self.hover = Some(color);
        self
    }
}

impl RenderOnce for ToolbarControlButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl gpui::IntoElement {
        let theme = cx.theme();

        Stack::new(StackDirection::Horizontal)
            .center()
            .w(px(36.))
            .h(TOOLBAR_HEIGHT)
            .hover(|style| style.bg(self.hover.unwrap_or(theme.colors.surface_variant)))
            .window_control_area(self.control_area)
            .child(Icon::new(self.icon, px(16.)).text_color(theme.colors.text.primary))
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
                IconName::MINIMIZE,
                WindowControlArea::Min,
            ))
            .child(ToolbarControlButton::new(
                if window.is_maximized() {
                    IconName::RESTORE
                } else {
                    IconName::MAXIMIZE
                },
                WindowControlArea::Max,
            ))
            .child(
                ToolbarControlButton::new(IconName::CLOSE, WindowControlArea::Close)
                    .hover_color(theme.colors.error),
            )
    }
}
