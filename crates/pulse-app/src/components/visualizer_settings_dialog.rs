use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, UpdateGlobal, Window, div, px,
};
use gpui_component::{
    Disableable, IndexPath, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::{DialogClose, DialogFooter, DialogTitle},
    form::{field, v_form},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    select::{Select, SelectEvent, SelectState},
    switch::Switch,
    v_flex,
};
use pulse_data::{VisualizerMode, VisualizerQuality, VisualizerSettings};

use crate::components::toolbar::menus;
use crate::config::PulseConfig;
use crate::data::{DataPaths, persist_settings};

pub struct VisualizerSettingsEditor {
    draft: VisualizerSettings,
    mode_select: Entity<SelectState<Vec<SharedString>>>,
    quality_select: Entity<SelectState<Vec<SharedString>>>,
}

impl VisualizerSettingsEditor {
    fn new(window: &mut Window, cx: &mut Context<Self>, settings: VisualizerSettings) -> Self {
        let mode_labels: Vec<SharedString> = VisualizerMode::ALL
            .iter()
            .map(|mode| mode.label().into())
            .collect();
        let mode_ix = VisualizerMode::ALL
            .iter()
            .position(|mode| *mode == settings.mode);

        let mode_select = cx.new(|cx| {
            SelectState::new(
                mode_labels,
                mode_ix.map(|index| IndexPath::default().row(index)),
                window,
                cx,
            )
        });

        cx.subscribe_in(&mode_select, window, |this, _, event, _, cx| {
            if let SelectEvent::Confirm(Some(label)) = event {
                if let Some(mode) = VisualizerMode::ALL
                    .iter()
                    .find(|mode| mode.label() == label.as_ref())
                {
                    this.draft.mode = *mode;
                    cx.notify();
                }
            }
        })
        .detach();

        let quality_labels: Vec<SharedString> = VisualizerQuality::ALL
            .iter()
            .map(|quality| quality.label().into())
            .collect();
        let quality_ix = VisualizerQuality::ALL
            .iter()
            .position(|quality| *quality == settings.quality);

        let quality_select = cx.new(|cx| {
            SelectState::new(
                quality_labels,
                quality_ix.map(|index| IndexPath::default().row(index)),
                window,
                cx,
            )
        });

        cx.subscribe_in(&quality_select, window, |this, _, event, _, cx| {
            if let SelectEvent::Confirm(Some(label)) = event {
                if let Some(quality) = VisualizerQuality::ALL
                    .iter()
                    .find(|quality| quality.label() == label.as_ref())
                {
                    this.draft.quality = *quality;
                    cx.notify();
                }
            }
        })
        .detach();

        Self {
            draft: settings,
            mode_select,
            quality_select,
        }
    }

    fn set_peak_hold(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.draft.peak_hold = enabled;
        cx.notify();
    }

    fn set_mirror(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.draft.mirror = enabled;
        cx.notify();
    }

    fn set_gradient(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.draft.gradient = enabled;
        cx.notify();
    }
}

impl Render for VisualizerSettingsEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let resolved = self.draft.resolve();
        let quality_detail = SharedString::from(format!(
            "{}-point FFT · {} bins · {}ms refresh · {} passes",
            resolved.fft_size,
            resolved.bar_count,
            resolved.refresh_ms,
            resolved.fft_passes,
        ));
        let mode = self.draft.mode;
        let spectrum_options = matches!(mode, VisualizerMode::Spectrum);

        v_flex()
            .gap_4()
            .child(DialogTitle::new().child("Visualizer Settings"))
            .child(
                div()
                    .id("visualizer-settings-scroll")
                    .max_h(px(460.))
                    .overflow_y_scroll()
                    .child(
                        v_flex()
                            .gap_4()
                            .child(
                                GroupBox::new()
                                    .with_variant(GroupBoxVariant::Outline)
                                    .title("Display")
                                    .child(
                                        v_form()
                                            .label_width(px(180.))
                                            .child(
                                                field()
                                                    .label("Mode")
                                                    .description(
                                                        VisualizerMode::ALL
                                                            .iter()
                                                            .find(|item| **item == mode)
                                                            .map(|item| item.description())
                                                            .unwrap_or(""),
                                                    )
                                                    .child(Select::new(&self.mode_select).w_full()),
                                            )
                                            .child(
                                                field()
                                                    .label("Quality")
                                                    .description(quality_detail)
                                                    .child(
                                                        Select::new(&self.quality_select).w_full(),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(
                                GroupBox::new()
                                    .with_variant(GroupBoxVariant::Outline)
                                    .title("Appearance")
                                    .child(
                                        v_form()
                                            .label_width(px(180.))
                                            .child(
                                                field()
                                                    .label("Peak hold")
                                                    .description(
                                                        "Show falling peak caps on the spectrum wave.",
                                                    )
                                                    .child(
                                                        Switch::new("viz-peak-hold")
                                                            .checked(self.draft.peak_hold)
                                                            .disabled(!spectrum_options)
                                                            .on_click(cx.listener(
                                                                |this, checked: &bool, _, cx| {
                                                                    this.set_peak_hold(
                                                                        *checked, cx,
                                                                    );
                                                                },
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                field()
                                                    .label("Mirror")
                                                    .description(
                                                        "Fold the spectrum into a center-out wave.",
                                                    )
                                                    .child(
                                                        Switch::new("viz-mirror")
                                                            .checked(self.draft.mirror)
                                                            .on_click(cx.listener(
                                                                |this, checked: &bool, _, cx| {
                                                                    this.set_mirror(*checked, cx);
                                                                },
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                field()
                                                    .label("Gradient fill")
                                                    .description(
                                                        "Fade the primary color to transparent.",
                                                    )
                                                    .child(
                                                        Switch::new("viz-gradient")
                                                            .checked(self.draft.gradient)
                                                            .on_click(cx.listener(
                                                                |this, checked: &bool, _, cx| {
                                                                    this.set_gradient(
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

pub fn open_visualizer_settings_dialog(window: &mut Window, cx: &mut App) {
    let initial = cx.global::<PulseConfig>().visualizer.clone();
    let editor = cx.new(|cx| VisualizerSettingsEditor::new(window, cx, initial));

    window.open_dialog(cx, move |dialog, _, _| {
        let editor_apply = editor.clone();
        dialog
            .w(px(560.))
            .keyboard(true)
            .child(editor.clone())
            .footer(
                DialogFooter::new()
                    .gap_2()
                    .child(
                        DialogClose::new().child(
                            Button::new("cancel-visualizer-settings")
                                .label("Cancel")
                                .outline(),
                        ),
                    )
                    .child(
                        Button::new("save-visualizer-settings")
                            .label("Save")
                            .primary()
                            .on_click(move |_, window, cx| {
                                let draft = editor_apply.read(cx).draft.clone();
                                apply_visualizer_settings(cx, &draft);
                                window.close_dialog(cx);
                            }),
                    ),
            )
    });
}

pub fn apply_visualizer_settings(cx: &mut App, draft: &VisualizerSettings) {
    PulseConfig::update_global(cx, |config, _| {
        config.visualizer = draft.clone();
    });

    let paths = cx.global::<DataPaths>().clone();
    if let Err(error) = persist_settings(&paths, &cx.global::<PulseConfig>().to_settings()) {
        tracing::error!(%error, "failed to save visualizer settings");
    }

    menus::refresh(cx);
    cx.refresh_windows();
}

pub fn set_visualizer_mode(cx: &mut App, mode: VisualizerMode) {
    let mut settings = cx.global::<PulseConfig>().visualizer.clone();
    if settings.mode == mode {
        return;
    }

    settings.mode = mode;
    apply_visualizer_settings(cx, &settings);
}
