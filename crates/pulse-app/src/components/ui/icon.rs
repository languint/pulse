use gpui::{ParentElement, RenderOnce, Styled, svg};
use pulse_assets::icons::IconName;

use crate::components::ui::stack::{Stack, StackDirection};

#[derive(gpui::IntoElement)]
pub struct Icon {
    pub icon: IconName,
    pub size: gpui::Pixels,

    pub stack_element: Stack,

    pub color: Option<gpui::Hsla>,
}

impl Icon {
    pub fn new(icon: IconName, size: impl Into<gpui::Pixels>) -> Self {
        Self {
            icon,
            size: size.into(),

            stack_element: Stack::new(StackDirection::Horizontal).center(),
            color: None,
        }
    }
}

impl Icon {
    #[must_use]
    pub const fn text_color(mut self, color: gpui::Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl gpui::Styled for Icon {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.stack_element.style()
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        let mut svg_element = svg().path(self.icon.path()).size(self.size);

        if let Some(text_color) = self.color {
            svg_element = svg_element.text_color(text_color);
        }

        self.stack_element.child(svg_element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_defaults() {
        let icon = Icon::new(IconName::CLOSE, gpui::px(16.));

        assert_eq!(icon.size, gpui::px(16.));
        assert!(icon.color.is_none());
    }

    #[test]
    fn text_color_sets_color() {
        let color = gpui::red();

        let icon = Icon::new(IconName::CLOSE, gpui::px(16.)).text_color(color);

        assert_eq!(icon.color, Some(color));
    }
}

#[cfg(test)]
mod render_tests {
    use gpui::{AvailableSpace, IntoElement, point, px, size};

    use super::*;

    #[gpui::test]
    async fn icon_draws(cx: &mut gpui::TestAppContext) {
        let cx = cx.add_empty_window();

        cx.draw(
            point(px(0.), px(0.)),
            size(AvailableSpace::MinContent, AvailableSpace::MinContent),
            |_window, _cx| Icon::new(IconName::CLOSE, px(16.)).into_element(),
        );
    }
}
