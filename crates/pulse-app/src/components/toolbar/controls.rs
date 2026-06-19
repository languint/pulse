use gpui::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, WindowControlArea, px, white,
};

use crate::components::ui::prelude::*;

use pulse_assets::icons::IconName;

use crate::config::PulseContext;

#[derive(IntoElement)]
pub struct ToolbarControlButton {
    id: SharedString,
    icon: IconName,
    control_area: WindowControlArea,
    hover_bg: Hsla,
    hover_fg: Hsla,
}

impl ToolbarControlButton {
    fn new(
        id: impl Into<SharedString>,
        icon: IconName,
        control_area: WindowControlArea,
        hover_bg: Hsla,
        hover_fg: Hsla,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            control_area,
            hover_bg,
            hover_fg,
        }
    }

    fn standard(
        cx: &App,
        id: impl Into<SharedString>,
        icon: IconName,
        control_area: WindowControlArea,
    ) -> Self {
        let theme = cx.theme();
        Self::new(
            id,
            icon,
            control_area,
            theme.colors.surface_variant,
            theme.colors.text.primary,
        )
    }

    fn close(cx: &App) -> Self {
        let theme = cx.theme();
        Self::new(
            "close",
            IconName::CLOSE,
            WindowControlArea::Close,
            theme.colors.error,
            white(),
        )
    }
}

impl RenderOnce for ToolbarControlButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl gpui::IntoElement {
        let theme = cx.theme();

        Stack::new(StackDirection::Horizontal)
            .center()
            .id(self.id)
            .occlude()
            .w(px(36.))
            .h_full()
            .hover(|style| style.bg(self.hover_bg).text_color(self.hover_fg))
            .window_control_area(self.control_area)
            .child(Icon::new(self.icon, px(16.)).text_color(theme.colors.text.primary))
    }
}

#[derive(IntoElement)]
pub struct ToolbarControls;

impl RenderOnce for ToolbarControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        Stack::new(StackDirection::Horizontal)
            .id("toolbar-controls")
            .h_full()
            .child(ToolbarControlButton::standard(
                cx,
                "minimize",
                IconName::MINIMIZE,
                WindowControlArea::Min,
            ))
            .child(ToolbarControlButton::standard(
                cx,
                if window.is_maximized() {
                    "restore"
                } else {
                    "maximize"
                },
                if window.is_maximized() {
                    IconName::RESTORE
                } else {
                    IconName::MAXIMIZE
                },
                WindowControlArea::Max,
            ))
            .child(ToolbarControlButton::close(cx))
    }
}
