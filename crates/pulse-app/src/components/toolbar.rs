use gpui::{Context, Entity, InteractiveElement, MouseButton, ParentElement, Render, Styled, Window, div};
use gpui_component::{TitleBar, menu::AppMenuBar};

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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        TitleBar::new().child(
            div()
                .flex()
                .items_center()
                .h_full()
                .occlude()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(self.app_menu_bar.clone()),
        )
    }
}
