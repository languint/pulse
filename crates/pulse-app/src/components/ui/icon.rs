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
    pub fn stack(self) -> Stack {
        self.stack_element
    }

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
