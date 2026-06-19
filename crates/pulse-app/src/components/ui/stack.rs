use gpui::{ParentElement, RenderOnce, Styled, div};

#[derive(Debug, Clone, Copy)]
pub enum StackDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum ItemAlignment {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy)]
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

#[cfg(test)]
mod tests {
    use gpui::{IntoElement, ParentElement};

    use crate::components::ui::stack::{ItemAlignment, JustifyContent, Stack, StackDirection};

    #[test]
    fn child_builder_adds_children() {
        let stack = Stack::new(StackDirection::Horizontal)
            .child("A")
            .child("B")
            .child("C");

        assert_eq!(stack.children.len(), 3);
    }

    #[test]
    fn children_builder_extends_children() {
        let stack = Stack::new(StackDirection::Horizontal)
            .child("A")
            .children(["B", "C"]);

        assert_eq!(stack.children.len(), 3);
    }

    #[test]
    fn extend_adds_children() {
        let mut stack = Stack::new(StackDirection::Horizontal);

        stack.extend(vec![
            "A".into_any_element(),
            "B".into_any_element(),
            "C".into_any_element(),
        ]);

        assert_eq!(stack.children.len(), 3);
    }

    #[test]
    fn gap_sets_gap() {
        let stack = Stack::new(StackDirection::Horizontal).gap(gpui::px(12.));

        assert_eq!(stack.gap, Some(gpui::px(12.)));
    }

    #[test]
    fn align_sets_alignment() {
        let stack = Stack::new(StackDirection::Horizontal).align(ItemAlignment::End);

        assert!(matches!(stack.align, ItemAlignment::End));
    }

    #[test]
    fn justify_sets_justification() {
        let stack = Stack::new(StackDirection::Horizontal).justify(JustifyContent::Between);

        assert!(matches!(stack.justify, JustifyContent::Between));
    }

    #[test]
    fn center_sets_both_alignment_and_justification() {
        let stack = Stack::new(StackDirection::Horizontal).center();

        assert!(matches!(stack.align, ItemAlignment::Center));
        assert!(matches!(stack.justify, JustifyContent::Center));
    }

    #[test]
    fn new_vertical_stack_has_correct_direction() {
        let stack = Stack::new(StackDirection::Vertical);

        assert!(matches!(stack.direction, StackDirection::Vertical));
    }

    #[test]
    fn new_horizontal_stack_has_correct_direction() {
        let stack = Stack::new(StackDirection::Horizontal);

        assert!(matches!(stack.direction, StackDirection::Horizontal));
    }
}

#[cfg(test)]
mod render_tests {
    use gpui::{AvailableSpace, IntoElement, ParentElement, point, px, size};

    use super::*;

    #[gpui::test]
    async fn horizontal_stack_draws(cx: &mut gpui::TestAppContext) {
        let cx = cx.add_empty_window();

        cx.draw(
            point(px(0.), px(0.)),
            size(AvailableSpace::MinContent, AvailableSpace::MinContent),
            |_window, _cx| {
                Stack::new(StackDirection::Horizontal)
                    .gap(px(8.))
                    .child("A")
                    .child("B")
                    .into_element()
            },
        );
    }

    #[gpui::test]
    async fn vertical_stack_draws(cx: &mut gpui::TestAppContext) {
        let cx = cx.add_empty_window();

        cx.draw(
            point(px(0.), px(0.)),
            size(AvailableSpace::MinContent, AvailableSpace::MinContent),
            |_window, _cx| {
                Stack::new(StackDirection::Vertical)
                    .gap(px(8.))
                    .child("A")
                    .child("B")
                    .into_element()
            },
        );
    }

    #[gpui::test]
    async fn stack_accepts_styling(cx: &mut gpui::TestAppContext) {
        let cx = cx.add_empty_window();

        cx.draw(
            point(px(0.), px(0.)),
            size(AvailableSpace::MinContent, AvailableSpace::MinContent),
            |_window, _cx| {
                Stack::new(StackDirection::Horizontal)
                    .bg(gpui::red())
                    .border_1()
                    .child("Hello")
                    .into_element()
            },
        );
    }
}
