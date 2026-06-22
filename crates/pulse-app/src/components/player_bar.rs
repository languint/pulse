use std::time::Duration;

use gpui::{
    AppContext, Bounds, Context, DragMoveEvent, Empty, Entity, FocusHandle, InteractiveElement,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Point, Render,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px, relative,
};
use gpui_component::{
    ActiveTheme, Disableable, ElementExt, IconName, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use crate::components::pulse::Pulse;
use crate::library::LibraryScanState;
use crate::lyrics::PulseLyrics;
use crate::media_controls;
use crate::player::PulsePlayer;
use crate::{
    components::pages::{artwork_tile_content, format_duration_ms},
    icons::PulseIcon,
};

const PLAYER_BAR_HEIGHT: f32 = 88.;
const PLAYER_ARTWORK_SIZE: f32 = 56.;
const PROGRESS_SLIDER_WIDTH: f32 = 420.;
const SEEK_HIT_HEIGHT: f32 = 24.;

#[derive(Clone)]
struct SeekDrag;

impl Render for SeekDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        Empty
    }
}

fn duration_label_ms(ms: u64) -> String {
    let clamped = u32::try_from(ms).unwrap_or(u32::MAX);
    format_duration_ms(clamped)
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
fn progress_fraction(position_ms: u64, duration_ms: u64) -> f32 {
    if duration_ms == 0 {
        return 0.;
    }

    let position = f64::from(u32::try_from(position_ms).unwrap_or(u32::MAX));
    let duration = f64::from(u32::try_from(duration_ms).unwrap_or(u32::MAX));
    (position / duration).clamp(0., 1.) as f32
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn fraction_to_position_ms(fraction: f32, duration_ms: u64) -> u64 {
    let duration = f64::from(u32::try_from(duration_ms).unwrap_or(u32::MAX));
    let position = f64::from(fraction.clamp(0., 1.)).mul_add(duration, 0.);
    u64::try_from(position.round() as i64).unwrap_or(0)
}

#[allow(clippy::arithmetic_side_effects)]
fn fraction_from_position(position: Point<Pixels>, bounds: Bounds<Pixels>) -> f32 {
    let width = bounds.size.width;
    if width <= px(0.) {
        return 0.;
    }

    let inner_pos = position.x - bounds.left();
    (inner_pos / width).clamp(0., 1.)
}

pub struct PlayerBar {
    pulse: Entity<Pulse>,
    focus_handle: FocusHandle,
    seek_bounds: Bounds<Pixels>,
    pointer_down: bool,
    dragging: bool,
    resume_after_seek: bool,
    pending_seek_position: Option<Point<Pixels>>,
    scrub_fraction: Option<f32>,
}

impl PlayerBar {
    pub fn new(pulse: Entity<Pulse>, cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                this.update(cx, |_, cx| {
                    media_controls::poll(cx);
                    PulseLyrics::sync_with_player(cx);
                    if LibraryScanState::is_in_progress(cx) {
                        cx.refresh_windows();
                    }
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self {
            pulse,
            focus_handle: cx.focus_handle(),
            seek_bounds: Bounds::default(),
            pointer_down: false,
            dragging: false,
            resume_after_seek: false,
            pending_seek_position: None,
            scrub_fraction: None,
        }
    }

    fn seek_at(&self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let snapshot = PulsePlayer::snapshot(cx);
        if snapshot.duration_ms == 0 {
            return;
        }

        let fraction = fraction_from_position(position, self.seek_bounds);
        PulsePlayer::seek(cx, fraction_to_position_ms(fraction, snapshot.duration_ms));
    }

    fn pointer_down(&mut self, position: Point<Pixels>, cx: &Context<Self>) {
        self.pointer_down = true;
        self.dragging = false;
        self.resume_after_seek = PulsePlayer::is_playing(cx);
        self.pending_seek_position = Some(position);
    }

    fn pointer_drag(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        if !self.pointer_down {
            return;
        }

        self.pending_seek_position = Some(position);

        if !self.dragging {
            self.dragging = true;
            if self.resume_after_seek {
                PulsePlayer::pause(cx);
            }
        }

        self.scrub_fraction = Some(fraction_from_position(position, self.seek_bounds));
        cx.notify();
    }

    fn pointer_up(&mut self, position: Option<Point<Pixels>>, cx: &mut Context<Self>) {
        if !self.pointer_down {
            return;
        }

        if let Some(position) = position.or(self.pending_seek_position) {
            self.seek_at(position, cx);
        }

        if self.resume_after_seek {
            PulsePlayer::play(cx);
        }

        self.pointer_down = false;
        self.dragging = false;
        self.resume_after_seek = false;
        self.pending_seek_position = None;
        self.scrub_fraction = None;
        cx.notify();
    }
}

impl Render for PlayerBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let snapshot = PulsePlayer::snapshot(cx);
        let has_track = snapshot.current_index.is_some();
        let title = PulsePlayer::current_track_title(cx).unwrap_or_else(|| "Not playing".into());
        let subtitle = PulsePlayer::current_track_subtitle(cx);
        let artwork = PulsePlayer::current_track_artwork(cx);
        let position_ms = self
            .scrub_fraction
            .map(|fraction| fraction_to_position_ms(fraction, snapshot.duration_ms))
            .unwrap_or(snapshot.position_ms);
        let position_label = duration_label_ms(position_ms);
        let duration_label = duration_label_ms(snapshot.duration_ms);
        let is_playing = PulsePlayer::is_playing(cx);
        let progress = self
            .scrub_fraction
            .unwrap_or_else(|| progress_fraction(snapshot.position_ms, snapshot.duration_ms));
        let visualizer_active = self.pulse.read(cx).is_visualizer();
        let lyrics_sidebar_open = PulseLyrics::sidebar_open(cx);
        let (border, background, muted_foreground) = {
            let theme = cx.theme();
            (theme.border, theme.background, theme.muted_foreground)
        };

        div()
            .id("player-bar")
            .h(px(PLAYER_BAR_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .border_t_1()
            .border_color(border)
            .bg(background)
            .track_focus(&self.focus_handle)
            .child(
                h_flex()
                    .size_full()
                    .items_center()
                    .child(track_info_panel(
                        has_track,
                        title,
                        subtitle,
                        artwork.as_deref(),
                        cx,
                    ))
                    .child(
                        v_flex()
                            .flex_shrink_0()
                            .w(px(PROGRESS_SLIDER_WIDTH))
                            .max_w_full()
                            .gap_2()
                            .child(
                                h_flex()
                                    .justify_center()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Button::new("player-previous")
                                            .icon(IconName::ChevronLeft)
                                            .ghost()
                                            .disabled(!has_track)
                                            .on_click(|_, _, cx| PulsePlayer::previous(cx)),
                                    )
                                    .child(
                                        Button::new("player-play-pause")
                                            .icon(if is_playing {
                                                IconName::Pause
                                            } else {
                                                IconName::Play
                                            })
                                            .ghost()
                                            .disabled(!has_track)
                                            .on_click(|_, _, cx| PulsePlayer::toggle_pause(cx)),
                                    )
                                    .child(
                                        Button::new("player-next")
                                            .icon(IconName::ChevronRight)
                                            .ghost()
                                            .disabled(!has_track)
                                            .on_click(|_, _, cx| PulsePlayer::next(cx)),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(muted_foreground)
                                            .child(position_label),
                                    )
                                    .child(seek_bar(has_track, progress, window, cx))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(muted_foreground)
                                            .child(duration_label),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w_0()
                            .justify_end()
                            .items_center()
                            .gap_1()
                            .px_4()
                            .child(
                                Button::new("player-lyrics")
                                    .ghost()
                                    .icon(PulseIcon::ScrollText)
                                    .tooltip("Lyrics sidebar")
                                    .disabled(!has_track)
                                    .map(|button| {
                                        if lyrics_sidebar_open {
                                            button.primary()
                                        } else {
                                            button
                                        }
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.pulse.update(cx, |pulse, cx| {
                                            pulse.toggle_lyrics_sidebar(cx);
                                        });
                                    })),
                            )
                            .child(
                                Button::new("player-visualizer")
                                    .ghost()
                                    .icon(PulseIcon::AudioLines)
                                    .tooltip("Visualizer")
                                    .map(|button| {
                                        if visualizer_active {
                                            button.primary()
                                        } else {
                                            button
                                        }
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.pulse.update(cx, |pulse, cx| {
                                            pulse.toggle_visualizer(cx);
                                        });
                                    })),
                            ),
                    ),
            )
    }
}

fn seek_bar(
    has_track: bool,
    progress: f32,
    window: &Window,
    cx: &Context<PlayerBar>,
) -> impl gpui::IntoElement {
    let bar_color = cx.theme().slider_bar;
    let entity = cx.entity();

    div()
        .id("player-seek-bar")
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .h(px(SEEK_HIT_HEIGHT))
        .w_full()
        .cursor_pointer()
        .when(!has_track, |this| this.opacity(0.5))
        .when(has_track, |this| {
            this.on_mouse_down(
                MouseButton::Left,
                window.listener_for(
                    &entity,
                    |this: &mut PlayerBar, event: &MouseDownEvent, _, cx| {
                        this.pointer_down(event.position, cx);
                    },
                ),
            )
            .on_drag(SeekDrag, |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(window.listener_for(
                &entity,
                |this: &mut PlayerBar, event: &DragMoveEvent<SeekDrag>, _, cx| {
                    this.pointer_drag(event.event.position, cx);
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(
                    &entity,
                    |this: &mut PlayerBar, event: &MouseUpEvent, _, cx| {
                        this.pointer_up(Some(event.position), cx);
                    },
                ),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                window.listener_for(&entity, |this: &mut PlayerBar, _, _, cx| {
                    this.pointer_up(None, cx);
                }),
            )
        })
        .on_prepaint(move |bounds, _, cx| {
            entity.update(cx, |this, _| this.seek_bounds = bounds);
        })
        .child(
            div()
                .id("player-seek-track")
                .relative()
                .w_full()
                .h_1p5()
                .rounded_full()
                .bg(bar_color.opacity(0.2))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .w(relative(progress))
                        .bg(bar_color)
                        .rounded_full(),
                ),
        )
}

fn track_info_panel(
    has_track: bool,
    title: gpui::SharedString,
    subtitle: Option<gpui::SharedString>,
    artwork: Option<&std::path::Path>,
    cx: &gpui::App,
) -> impl gpui::IntoElement {
    let theme = cx.theme();

    h_flex()
        .flex_1()
        .min_w_0()
        .items_center()
        .gap_3()
        .px_4()
        .child(
            div()
                .size(px(PLAYER_ARTWORK_SIZE))
                .flex_shrink_0()
                .rounded(theme.radius)
                .overflow_hidden()
                .bg(theme.muted)
                .border_1()
                .border_color(theme.border)
                .when(!has_track, |this| this.opacity(0.5))
                .child(artwork_tile_content(artwork, cx)),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_0p5()
                .child(
                    div()
                        .text_sm()
                        .font_medium()
                        .overflow_hidden()
                        .text_ellipsis()
                        .when(!has_track, |this| this.text_color(theme.muted_foreground))
                        .child(title),
                )
                .when_some(subtitle, |this, subtitle| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(subtitle),
                    )
                }),
        )
}
