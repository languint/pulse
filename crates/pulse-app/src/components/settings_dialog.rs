use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, UpdateGlobal, Window, div, px,
};
use gpui_component::{
    IndexPath, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::{DialogClose, DialogFooter, DialogTitle},
    form::{field, v_form},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    input::{InputState, NumberInput},
    select::{Select, SelectEvent, SelectState},
    switch::Switch,
    v_flex,
};
use pulse_data::PulseSettings;

use crate::components::library_roots_dialog::open_library_roots_dialog;
use crate::config::PulseConfig;
use crate::data::{DataPaths, persist_settings};
use crate::library::PulseLibrary;
use crate::pulse;
use crate::theme_list::selectable_themes;

pub struct SettingsEditor {
    committed: PulseSettings,
    draft: PulseSettings,
    theme_select: Entity<SelectState<Vec<SharedString>>>,
    debounce_input: Entity<InputState>,
}

impl SettingsEditor {
    fn new(window: &mut Window, cx: &mut Context<Self>, settings: PulseSettings) -> Self {
        let theme_names: Vec<SharedString> = selectable_themes(cx)
            .into_iter()
            .map(|theme| theme.name.clone())
            .collect();
        let selected_ix = theme_names
            .iter()
            .position(|name| name.as_ref() == settings.theme);

        let theme_select = cx.new(|cx| {
            SelectState::new(
                theme_names,
                selected_ix.map(|index| IndexPath::default().row(index)),
                window,
                cx,
            )
            .searchable(true)
        });

        let debounce_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("750")
                .default_value(settings.library.watch_debounce_ms.to_string())
                .min(100.)
                .max(10_000.)
        });

        cx.subscribe_in(&theme_select, window, |this, _, event, _, cx| {
            if let SelectEvent::Confirm(Some(theme)) = event {
                this.draft.theme = theme.to_string();
                pulse::preview_theme(cx, &this.draft.theme);
                cx.notify();
            }
        })
        .detach();

        Self {
            committed: settings.clone(),
            draft: settings,
            theme_select,
            debounce_input,
        }
    }

    fn draft_with_debounce(&self, cx: &App) -> PulseSettings {
        let mut draft = self.draft.clone();
        let debounce_text = self.debounce_input.read(cx).value();
        if let Ok(debounce) = debounce_text.parse::<u64>() {
            draft.library.watch_debounce_ms = debounce.clamp(100, 10_000);
        }
        draft
    }

    fn set_include_system_music(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.draft.library.include_xdg_music_dir = enabled;
        cx.notify();
    }

    fn set_prefetch_artwork(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.draft.interface.aggressively_prefetch_artwork = enabled;
        cx.notify();
    }
}

impl Render for SettingsEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let include_system_music = self.draft.library.include_xdg_music_dir;
        let prefetch_artwork = self.draft.interface.aggressively_prefetch_artwork;

        v_flex()
            .gap_4()
            .child(DialogTitle::new().child("Settings"))
            .child(
                div()
                    .id("settings-scroll")
                    .max_h(px(420.))
                    .overflow_y_scroll()
                    .child(
                        v_flex()
                            .gap_4()
                            .child(
                                GroupBox::new()
                                    .with_variant(GroupBoxVariant::Outline)
                                    .title("Appearance")
                                    .child(
                                        v_form()
                                            .label_width(px(180.))
                                            .child(
                                                field()
                                                    .label("Theme")
                                                    .description(
                                                        "Color theme for the Pulse interface.",
                                                    )
                                                    .child(
                                                        Select::new(&self.theme_select)
                                                            .w_full()
                                                            .menu_max_h(px(280.)),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(
                                GroupBox::new()
                                    .with_variant(GroupBoxVariant::Outline)
                                    .title("Library")
                                    .child(
                                        v_form()
                                            .label_width(px(180.))
                                            .child(
                                                field()
                                                    .label("Include system music folder")
                                                    .description(
                                                        "Scan the platform default music directory.",
                                                    )
                                                    .child(
                                                        Switch::new("include-system-music")
                                                            .checked(include_system_music)
                                                            .on_click(cx.listener(
                                                                |this, checked: &bool, _, cx| {
                                                                    this.set_include_system_music(
                                                                        *checked, cx,
                                                                    );
                                                                },
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                field()
                                                    .label("Watch debounce")
                                                    .description(
                                                        "Delay before rescanning after file changes (milliseconds).",
                                                    )
                                                    .child(
                                                        NumberInput::new(&self.debounce_input)
                                                            .suffix("ms"),
                                                    ),
                                            )
                                            .child(
                                                field()
                                                    .label("Library folders")
                                                    .description(
                                                        "Add or remove folders scanned for music.",
                                                    )
                                                    .child(
                                                        Button::new("manage-library-roots")
                                                            .outline()
                                                            .label("Manage Library Roots…")
                                                            .on_click(
                                                                cx.listener(
                                                                    |_, _, window, cx| {
                                                                        open_library_roots_dialog(
                                                                            window, cx,
                                                                        );
                                                                    },
                                                                ),
                                                            ),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(
                                GroupBox::new()
                                    .with_variant(GroupBoxVariant::Outline)
                                    .title("Interface")
                                    .child(
                                        v_form()
                                            .label_width(px(180.))
                                            .child(
                                                field()
                                                    .label("Prefetch artwork")
                                                    .description(
                                                        "Load album art ahead of time while browsing.",
                                                    )
                                                    .child(
                                                        Switch::new("prefetch-artwork")
                                                            .checked(prefetch_artwork)
                                                            .on_click(cx.listener(
                                                                |this, checked: &bool, _, cx| {
                                                                    this.set_prefetch_artwork(
                                                                        *checked, cx,
                                                                    );
                                                                },
                                                            )),
                                                    ),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}

pub fn open_settings_dialog(window: &mut Window, cx: &mut App) {
    let initial = cx.global::<PulseConfig>().to_settings();

    let editor = cx.new(|cx| SettingsEditor::new(window, cx, initial));

    window.open_dialog(cx, move |dialog, _, _| {
        let editor_apply = editor.clone();
        let editor_revert = editor.clone();
        dialog
            .w(px(560.))
            .keyboard(true)
            .child(editor.clone())
            .on_close({
                let editor_revert = editor_revert.clone();
                move |_, _, cx| {
                    let theme = editor_revert.read(cx).committed.theme.clone();
                    pulse::preview_theme(cx, &theme);
                }
            })
            .footer(
                DialogFooter::new()
                    .gap_2()
                    .child(
                        DialogClose::new().child(
                            Button::new("cancel-settings")
                                .label("Cancel")
                                .outline(),
                        ),
                    )
                    .child(
                        Button::new("save-settings")
                            .label("Save")
                            .primary()
                            .on_click(move |_, window, cx| {
                                let draft = editor_apply.read(cx).draft_with_debounce(cx);
                                apply_settings(cx, &draft);
                                window.close_dialog(cx);
                            }),
                    ),
            )
    });
}

fn apply_settings(cx: &mut App, draft: &PulseSettings) {
    let previous = cx.global::<PulseConfig>().to_settings();

    PulseConfig::update_global(cx, |config, _| {
        config.interface = draft.interface.clone();
    });

    if draft.library != previous.library {
        PulseLibrary::apply_config(cx, draft.library.clone());
    }

    if draft.theme != previous.theme {
        pulse::set_theme(cx, &draft.theme);
    }

    let paths = cx.global::<DataPaths>().clone();
    if let Err(error) = persist_settings(&paths, &cx.global::<PulseConfig>().to_settings()) {
        tracing::error!(%error, "failed to save settings");
    }
    cx.refresh_windows();
}
