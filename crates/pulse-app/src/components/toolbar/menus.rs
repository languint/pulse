use gpui::{App, Entity, Global, Menu, MenuItem, SharedString, UpdateGlobal};
use gpui_component::{GlobalState, menu::AppMenuBar};
use pulse_data::VisualizerMode;

use crate::actions::{
    ManageLibraryRoots, OpenSettings, OpenVisualizerSettings, Quit, ShowOscilloscopeVisualizer,
    ShowSpectrumVisualizer, ToggleCommandPalette, ToggleFullscreen,
};
use crate::config::PulseConfig;

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

    let mode = cx.global::<PulseConfig>().visualizer.mode;
    let app_menu_bar = state.app_menu_bar.clone();
    cx.set_menus(build_menus(mode));
    let menus = build_menus(mode).into_iter().map(gpui::Menu::owned).collect();
    GlobalState::global_mut(cx).set_app_menus(menus);

    app_menu_bar.update(cx, |menu_bar, cx| {
        menu_bar.reload(cx);
    });
}

fn build_menus(mode: VisualizerMode) -> Vec<Menu> {
    vec![
        Menu {
            name: SharedString::from("Pulse"),
            items: vec![
                MenuItem::action("Settings...", OpenSettings),
                MenuItem::separator(),
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
        Menu {
            name: SharedString::from("Visualizer"),
            items: vec![
                MenuItem::action("Spectrum", ShowSpectrumVisualizer)
                    .checked(mode == VisualizerMode::Spectrum),
                MenuItem::action("Oscilloscope", ShowOscilloscopeVisualizer)
                    .checked(mode == VisualizerMode::Oscilloscope),
                MenuItem::separator(),
                MenuItem::action("Settings...", OpenVisualizerSettings),
            ],
            disabled: false,
        },
    ]
}
