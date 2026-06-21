use gpui::{App, Entity, Menu, MenuItem, SharedString};
use gpui_component::{GlobalState, menu::AppMenuBar};

use crate::actions::{ManageLibraryRoots, Quit, ToggleFullscreen};

pub fn init(cx: &mut App) -> Entity<AppMenuBar> {
    let app_menu_bar = AppMenuBar::new(cx);
    update_app_menu(app_menu_bar.clone(), cx);
    app_menu_bar
}

fn update_app_menu(app_menu_bar: Entity<AppMenuBar>, cx: &mut App) {
    cx.set_menus(build_menus());
    let menus = build_menus()
        .into_iter()
        .map(|menu| menu.owned())
        .collect();
    GlobalState::global_mut(cx).set_app_menus(menus);

    app_menu_bar.update(cx, |menu_bar, cx| {
        menu_bar.reload(cx);
    });
}

fn build_menus() -> Vec<Menu> {
    vec![
        Menu {
            name: SharedString::from("Pulse"),
            items: vec![
                MenuItem::action("Toggle Fullscreen", ToggleFullscreen),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: SharedString::from("Library"),
            items: vec![MenuItem::action("Manage Roots...", ManageLibraryRoots)],
            disabled: false,
        },
        Menu {
            name: SharedString::from("View"),
            items: vec![MenuItem::action("Toggle Fullscreen", ToggleFullscreen)],
            disabled: false,
        },
    ]
}
