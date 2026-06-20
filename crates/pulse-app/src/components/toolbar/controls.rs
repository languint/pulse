use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, WindowControlArea, div,
};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable};

use crate::components::toolbar::TOOLBAR_HEIGHT;

const BUTTON_WIDTH: gpui::Pixels = TOOLBAR_HEIGHT;

#[derive(IntoElement)]
struct ToolbarControlButton {
    id: SharedString,
    icon: IconName,
    control_area: WindowControlArea,
}

impl ToolbarControlButton {
    fn new(id: impl Into<SharedString>, icon: IconName, control_area: WindowControlArea) -> Self {
        Self {
            id: id.into(),
            icon,
            control_area,
        }
    }

    fn standard(
        id: impl Into<SharedString>,
        icon: IconName,
        control_area: WindowControlArea,
    ) -> Self {
        Self::new(id, icon, control_area)
    }

    fn close() -> Self {
        Self::new("close", IconName::WindowClose, WindowControlArea::Close)
    }
}

impl RenderOnce for ToolbarControlButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let (hover_bg, hover_fg, active_bg) = if self.control_area == WindowControlArea::Close {
            (theme.danger, theme.danger_foreground, theme.danger_active)
        } else {
            (
                theme.secondary_hover,
                theme.secondary_foreground,
                theme.secondary_active,
            )
        };

        let mut button = div()
            .id(self.id)
            .occlude()
            .flex()
            .flex_shrink_0()
            .w(BUTTON_WIDTH)
            .h_full()
            .justify_center()
            .content_center()
            .items_center()
            .text_color(theme.foreground)
            .hover(|style| style.bg(hover_bg).text_color(hover_fg))
            .active(|style| style.bg(active_bg).text_color(hover_fg))
            .window_control_area(self.control_area);

        // Not sure why, but this is required to get the hover effect to work.
        #[cfg(target_os = "windows")]
        {
            button = button.on_mouse_move(|_, window, _| window.refresh());
        }

        button.child(Icon::new(self.icon).small())
    }
}

#[derive(IntoElement)]
pub struct ToolbarControls;

impl RenderOnce for ToolbarControls {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id("toolbar-controls")
            .flex()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .h_full()
            .child(ToolbarControlButton::standard(
                "minimize",
                IconName::WindowMinimize,
                WindowControlArea::Min,
            ))
            .child(ToolbarControlButton::standard(
                if window.is_maximized() {
                    "restore"
                } else {
                    "maximize"
                },
                if window.is_maximized() {
                    IconName::WindowRestore
                } else {
                    IconName::WindowMaximize
                },
                WindowControlArea::Max,
            ))
            .child(ToolbarControlButton::close())
    }
}
