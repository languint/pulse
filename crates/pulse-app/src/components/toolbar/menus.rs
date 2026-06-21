use gpui::{App, Entity, Global, Menu, MenuItem, SharedString, UpdateGlobal};
use gpui_component::{GlobalState, menu::AppMenuBar};

use crate::actions::{
    ManageLibraryRoots, Quit, ToggleCommandPalette, ToggleFullscreen,
};

struct MenuState {
    app_menu_bar: Entity<AppMenuBar>,
}

impl Global for MenuState {}

pub fn init(cx: &mut App) -> Entity<AppMenuBar> {
    let app_menu_bar = AppMenuBar::new(cx);
    MenuState::set_global(cx, MenuState {
        app_menu_bar: app_menu_bar.clone(),
    });
    refresh(cx);
    app_menu_bar
}

pub fn refresh(cx: &mut App) {
    let Some(state) = cx.try_global::<MenuState>() else {
        return;
    };

    let app_menu_bar = state.app_menu_bar.clone();
    cx.set_menus(build_menus());
    let menus = build_menus().into_iter().map(gpui::Menu::owned).collect();
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
            items: vec![
                MenuItem::action("Toggle Fullscreen", ToggleFullscreen),
                MenuItem::separator(),
                MenuItem::action("Command Palette...", ToggleCommandPalette),
            ],
            disabled: false,
        },
    ]
}
