use gpui::{Context, Entity, InteractiveElement, MouseButton, ParentElement, Render, Styled, Window, div, prelude::FluentBuilder};
use gpui_component::{
    ActiveTheme, Sizable, TitleBar, h_flex, menu::AppMenuBar, spinner::Spinner,
};

use crate::library::LibraryScanState;

pub mod menus;

pub struct Toolbar {
    app_menu_bar: Entity<AppMenuBar>,
}

impl Toolbar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            app_menu_bar: menus::init(cx),
        }
    }
}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let syncing = LibraryScanState::is_in_progress(cx);
        let theme = cx.theme();

        TitleBar::new().child(
            h_flex()
                .w_full()
                .items_center()
                .justify_between()
                .h_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .h_full()
                        .occlude()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(self.app_menu_bar.clone()),
                )
                .when(syncing, |this| {
                    this.child(
                        h_flex()
                            .id("library-sync-status")
                            .items_center()
                            .gap_2()
                            .mr_4()
                            .flex_shrink_0()
                            .child(
                                Spinner::new()
                                    .small()
                                    .color(theme.muted_foreground),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("Sync in progress"),
                            ),
                    )
                }),
        )
    }
}
