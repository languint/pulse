use gpui::{ParentElement, RenderOnce, Styled, div};

pub enum StackDirection {
    Horizontal,
    Vertical,
}

pub enum ItemAlignment {
    Start,
    Center,
    End,
}

pub enum JustifyContent {
    Start,
    Center,
    End,
    Between,
}

#[derive(gpui::IntoElement)]
pub struct Stack {
    pub direction: StackDirection,

    pub gap: Option<gpui::Pixels>,
    pub align: ItemAlignment,
    pub justify: JustifyContent,

    pub children: Vec<gpui::AnyElement>,

    pub base_div: gpui::Div,
}

impl Stack {
    #[must_use]
    pub fn new(direction: StackDirection) -> Self {
        Self {
            direction,

            gap: None,
            align: ItemAlignment::Start,
            justify: JustifyContent::Start,

            children: Vec::new(),

            base_div: div().flex(),
        }
    }
}

impl Stack {
    #[must_use]
    pub fn gap(mut self, distance: impl Into<gpui::Pixels>) -> Self {
        self.gap = Some(distance.into());
        self
    }

    #[must_use]
    pub const fn align(mut self, alignment: ItemAlignment) -> Self {
        self.align = alignment;
        self
    }

    #[must_use]
    pub const fn justify(mut self, justification: JustifyContent) -> Self {
        self.justify = justification;
        self
    }

    #[must_use]
    pub const fn center(mut self) -> Self {
        self.align = ItemAlignment::Center;
        self.justify = JustifyContent::Center;
        self
    }
}

impl Styled for Stack {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base_div.style()
    }
}

impl gpui::InteractiveElement for Stack {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base_div.interactivity()
    }
}

impl gpui::ParentElement for Stack {
    fn child(mut self, child: impl gpui::prelude::IntoElement) -> Self
    where
        Self: Sized,
    {
        self.children.push(child.into_any_element());
        self
    }

    fn children(
        mut self,
        children: impl IntoIterator<Item = impl gpui::prelude::IntoElement>,
    ) -> Self
    where
        Self: Sized,
    {
        self.children.extend(
            children
                .into_iter()
                .map(gpui::IntoElement::into_any_element),
        );

        self
    }

    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(
            elements
                .into_iter()
                .map(gpui::IntoElement::into_any_element),
        );
    }
}

impl RenderOnce for Stack {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        let mut stack_div = self.base_div;

        stack_div = if matches!(self.direction, StackDirection::Horizontal) {
            stack_div.flex_row()
        } else {
            stack_div.flex_col()
        };

        stack_div = match self.align {
            ItemAlignment::Start => stack_div.items_start(),
            ItemAlignment::Center => stack_div.items_center(),
            ItemAlignment::End => stack_div.items_end(),
        };

        stack_div = match self.justify {
            JustifyContent::Start => stack_div.justify_start(),
            JustifyContent::Center => stack_div.justify_center(),
            JustifyContent::End => stack_div.justify_end(),
            JustifyContent::Between => stack_div.justify_between(),
        };

        if let Some(gap_size) = self.gap {
            stack_div = stack_div.gap(gap_size);
        }

        stack_div.children(self.children)
    }
}
