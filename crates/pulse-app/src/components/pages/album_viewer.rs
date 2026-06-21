use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{
    AnyElement, AppContext, Bounds, Context, Entity, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, Render, SharedString, Size, StatefulInteractiveElement, Styled,
    StyledImage, Window, anchored, deferred, div, hash, img, prelude::FluentBuilder, px, rems,
    size,
};
use gpui_component::{
    ActiveTheme, ElementExt, Icon, IconName, Sizable, StyledExt as _, VirtualListScrollHandle,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    tag::Tag,
    v_flex, v_virtual_list,
};
use pulse_model::AlbumId;

use crate::components::pulse::Pulse;
use crate::data::save_album_user_labels;
use crate::player::PulsePlayer;

use super::common::{
    AlbumArtistEntry, AlbumDisplay, CatalogFingerprint, TrackRow, album_label_is_taken,
    catalog_fingerprint, collect_suggested_labels, empty_state, filter_tag_suggestions,
    format_album_duration_ms, overrides_generation, page_back_label, resolve_album_display,
};

const SIDEBAR_WIDTH: f32 = 255.;
const DETAIL_PANEL_MAX_WIDTH: f32 = 360.;
const DETAIL_PANEL_MIN_WIDTH: f32 = 260.;
const DETAIL_STACK_BREAKPOINT: f32 = 820.;
const MIN_TRACK_LIST_WIDTH: f32 = 320.;
const DETAIL_ARTWORK_MAX: f32 = 320.;
const DETAIL_ARTWORK_COMPACT: f32 = 128.;
const DETAIL_ARTWORK_MIN: f32 = 160.;
const TRACK_ROW_HEIGHT: f32 = 40.;
const DISC_HEADER_HEIGHT: f32 = 34.;
const DISC_SECTION_GAP: f32 = 16.;
const TAG_MENU_MAX_HEIGHT: f32 = 16.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlbumDetailLayoutMode {
    Sidebar,
    Stacked,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AlbumDetailLayout {
    mode: AlbumDetailLayoutMode,
    panel_width: f32,
    artwork_size: f32,
}

impl AlbumDetailLayout {
    fn for_window(window: &Window) -> Self {
        let content_width = (window.viewport_size().width.as_f32() - SIDEBAR_WIDTH).max(0.);

        if content_width < DETAIL_STACK_BREAKPOINT {
            let artwork_size = DETAIL_ARTWORK_COMPACT;
            return Self {
                mode: AlbumDetailLayoutMode::Stacked,
                panel_width: content_width,
                artwork_size,
            };
        }

        let panel_width = DETAIL_PANEL_MAX_WIDTH
            .min((content_width - MIN_TRACK_LIST_WIDTH).max(DETAIL_PANEL_MIN_WIDTH));
        let artwork_size = DETAIL_ARTWORK_MAX
            .min(panel_width - 48.)
            .max(DETAIL_ARTWORK_MIN);

        Self {
            mode: AlbumDetailLayoutMode::Sidebar,
            panel_width,
            artwork_size,
        }
    }
}
#[derive(Clone, Debug)]
enum AlbumTrackListRowKind {
    DiscHeader { label: SharedString },
    Track { track: TrackRow, stripe: bool },
    DiscSpacer,
}

#[derive(Clone, Debug)]
struct AlbumTrackListRow {
    kind: AlbumTrackListRowKind,
}

#[allow(clippy::missing_const_for_fn)]
impl AlbumTrackListRow {
    fn disc_header(label: SharedString) -> Self {
        Self {
            kind: AlbumTrackListRowKind::DiscHeader { label },
        }
    }

    fn track(track: TrackRow, stripe: bool) -> Self {
        Self {
            kind: AlbumTrackListRowKind::Track { track, stripe },
        }
    }

    const fn disc_spacer() -> Self {
        Self {
            kind: AlbumTrackListRowKind::DiscSpacer,
        }
    }

    const fn height(&self) -> Pixels {
        match self.kind {
            AlbumTrackListRowKind::DiscHeader { .. } => px(DISC_HEADER_HEIGHT),
            AlbumTrackListRowKind::Track { .. } => px(TRACK_ROW_HEIGHT),
            AlbumTrackListRowKind::DiscSpacer => px(DISC_SECTION_GAP),
        }
    }
}

pub struct AlbumViewerPage {
    pulse: Entity<Pulse>,
    track_scroll_handle: VirtualListScrollHandle,
    tag_input: Option<Entity<InputState>>,
    tag_menu_open: bool,
    tag_input_bounds: Bounds<Pixels>,
    tag_suggestions: Vec<String>,
    cached_album_id: Option<AlbumId>,
    catalog_fp: CatalogFingerprint,
    overrides_gen: u32,
    display: Option<AlbumDisplay>,
    list_rows: Rc<[AlbumTrackListRow]>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
}

impl AlbumViewerPage {
    #[must_use]
    pub fn new(pulse: Entity<Pulse>, _: &mut Context<Self>) -> Self {
        Self {
            pulse,
            track_scroll_handle: VirtualListScrollHandle::new(),
            tag_input: None,
            tag_menu_open: false,
            tag_input_bounds: Bounds::default(),
            tag_suggestions: Vec::new(),
            cached_album_id: None,
            catalog_fp: CatalogFingerprint::default(),
            overrides_gen: 0,
            display: None,
            list_rows: Rc::from([]),
            item_sizes: Rc::new(Vec::new()),
        }
    }

    fn ensure_tag_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(input) = self.tag_input.clone() {
            return input;
        }

        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search or add tag..."));

        cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
            InputEvent::Change | InputEvent::Focus => {
                this.open_tag_menu(cx);
            }
            InputEvent::PressEnter {
                secondary: false,
                shift: false,
            } => {
                this.commit_new_tag(window, cx);
            }
            InputEvent::PressEnter { .. } | InputEvent::Blur => {}
        })
        .detach();

        self.tag_input = Some(input.clone());
        input
    }

    const fn invalidate_album_cache(&mut self) {
        self.cached_album_id = None;
    }

    fn pick_tag_suggestion(&mut self, label: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.add_user_label(label, cx);
        self.clear_tag_input(window, cx);
    }

    fn clear_tag_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tag_input) = self.tag_input.clone() {
            tag_input.update(cx, |input, cx| {
                input.set_value("", window, cx);
            });
        }
        self.tag_menu_open = false;
        cx.notify();
    }

    fn open_tag_menu(&mut self, cx: &mut Context<Self>) {
        if self.tag_menu_open {
            return;
        }

        self.tag_menu_open = true;
        cx.notify();
    }

    fn toggle_tag_menu(&mut self, cx: &mut Context<Self>) {
        self.tag_menu_open = !self.tag_menu_open;
        cx.notify();
    }

    fn close_tag_menu(&mut self, cx: &mut Context<Self>) {
        if self.tag_menu_open {
            self.tag_menu_open = false;
            cx.notify();
        }
    }

    fn add_user_label(&mut self, label: &str, cx: &mut Context<Self>) {
        let label = label.trim();
        if label.is_empty() {
            return;
        }

        let Some(display) = self.display.clone() else {
            return;
        };

        if album_label_is_taken(&display, label) {
            return;
        }

        let mut user_tags = display.user_tags.clone();
        user_tags.push(label.to_string());
        save_album_user_labels(cx, &display.override_key, &user_tags);
        self.invalidate_album_cache();
        cx.notify();
    }

    fn commit_new_tag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tag_input) = self.tag_input.clone() else {
            return;
        };

        let query = tag_input.read(cx).text().to_string();
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return;
        }

        let label = self
            .tag_suggestions
            .iter()
            .find(|existing| existing.eq_ignore_ascii_case(trimmed))
            .cloned()
            .unwrap_or_else(|| trimmed.to_string());

        self.add_user_label(&label, cx);
        self.clear_tag_input(window, cx);
    }

    fn remove_user_tag(&mut self, tag: &str, cx: &mut Context<Self>) {
        let Some(display) = self.display.clone() else {
            return;
        };

        let user_tags: Vec<String> = display
            .user_tags
            .iter()
            .filter(|existing| !existing.eq_ignore_ascii_case(tag))
            .cloned()
            .collect();

        save_album_user_labels(cx, &display.override_key, &user_tags);
        self.invalidate_album_cache();
        cx.notify();
    }

    fn ensure_album(&mut self, album_id: AlbumId, cx: &gpui::App) {
        let fp = catalog_fingerprint(cx);
        let overrides_gen = overrides_generation(cx);
        if self.cached_album_id == Some(album_id)
            && self.catalog_fp == fp
            && self.overrides_gen == overrides_gen
        {
            return;
        }

        let Some(display) = resolve_album_display(cx, album_id) else {
            self.cached_album_id = None;
            self.catalog_fp = fp;
            self.overrides_gen = overrides_gen;
            self.display = None;
            self.list_rows = Rc::from([]);
            self.item_sizes = Rc::new(Vec::new());
            return;
        };

        let rows = build_album_track_list_rows(&display.tracks);
        self.item_sizes = Rc::new(rows.iter().map(|row| size(px(0.), row.height())).collect());
        self.list_rows = rows.into();
        self.display = Some(display);
        self.cached_album_id = Some(album_id);
        self.catalog_fp = fp;
        self.overrides_gen = overrides_gen;
    }

    fn render_track_list_row(&self, row_ix: usize, cx: &gpui::App) -> AnyElement {
        let Some(row) = self.list_rows.get(row_ix) else {
            return div().into_any_element();
        };

        match &row.kind {
            AlbumTrackListRowKind::DiscHeader { label } => {
                disc_header_row(label, cx).into_any_element()
            }
            AlbumTrackListRowKind::Track { track, stripe } => {
                let song_ids: Vec<_> = self
                    .display
                    .as_ref()
                    .map(|display| display.tracks.iter().map(|row| row.id).collect())
                    .unwrap_or_default();
                track_row(track, *stripe, &song_ids, cx).into_any_element()
            }
            AlbumTrackListRowKind::DiscSpacer => div().h(px(DISC_SECTION_GAP)).into_any_element(),
        }
    }
}

impl Render for AlbumViewerPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.pulse.read(cx).page();
        let Some(album_id) = page.album_detail() else {
            return empty_state("No album selected.", cx).into_any_element();
        };

        self.ensure_album(album_id, cx);

        let Some(display) = self.display.clone() else {
            return empty_state("Album not found.", cx).into_any_element();
        };

        let pulse = self.pulse.clone();
        let item_sizes = self.item_sizes.clone();
        let entity = cx.entity();
        let tag_input = self.ensure_tag_input(window, cx);
        let tag_suggestions = collect_suggested_labels(cx, &display);
        self.tag_suggestions.clone_from(&tag_suggestions);
        let tag_menu_open = self.tag_menu_open;
        let tag_input_bounds = self.tag_input_bounds;
        let layout = AlbumDetailLayout::for_window(window);

        let track_list = div().flex_1().min_w_0().min_h_0().px_6().pb_6().child(
            v_virtual_list(
                entity.clone(),
                "album-track-list",
                item_sizes,
                |this, visible_range, _, cx| {
                    visible_range
                        .map(|row_ix| this.render_track_list_row(row_ix, cx))
                        .collect()
                },
            )
            .track_scroll(&self.track_scroll_handle),
        );

        let detail_panel = album_detail_panel(
            &entity,
            &self.pulse,
            &display,
            &tag_input,
            &tag_suggestions,
            tag_menu_open,
            tag_input_bounds,
            layout,
            cx,
        );

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(h_flex_back_button(pulse, cx))
            .child(match layout.mode {
                AlbumDetailLayoutMode::Sidebar => div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(track_list)
                    .child(detail_panel)
                    .into_any_element(),
                AlbumDetailLayoutMode::Stacked => div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(detail_panel)
                    .child(track_list)
                    .into_any_element(),
            })
            .into_any_element()
    }
}

fn h_flex_back_button(pulse: Entity<Pulse>, cx: &gpui::App) -> impl IntoElement {
    use gpui_component::h_flex;

    let back_label = page_back_label(cx, pulse.read(cx).back_target());

    h_flex().items_center().gap_2().px_6().pt_6().pb_4().child(
        Button::new("album-back")
            .icon(IconName::ArrowLeft)
            .label(back_label)
            .outline()
            .on_click(move |_, _, cx| {
                pulse.update(cx, |pulse, cx| {
                    pulse.go_back(cx);
                });
            }),
    )
}

fn album_stats_line(display: &AlbumDisplay) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(year) = display.year {
        parts.push(year.to_string());
    }

    if display.duration_ms > 0 {
        parts.push(format_album_duration_ms(display.duration_ms));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" - "))
    }
}

#[allow(clippy::too_many_arguments)]
fn album_detail_panel(
    view: &Entity<AlbumViewerPage>,
    pulse: &Entity<Pulse>,
    display: &AlbumDisplay,
    tag_input: &Entity<InputState>,
    tag_suggestions: &[String],
    tag_menu_open: bool,
    tag_input_bounds: Bounds<Pixels>,
    layout: AlbumDetailLayout,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let tags = album_tags_editor(
        view,
        display,
        tag_input,
        tag_suggestions,
        tag_menu_open,
        tag_input_bounds,
        cx,
    );

    match layout.mode {
        AlbumDetailLayoutMode::Sidebar => v_flex()
            .id(("album-detail-panel", display.album_id.0))
            .w(px(layout.panel_width))
            .flex_shrink_0()
            .min_h_0()
            .h_full()
            .gap_4()
            .px_6()
            .py_6()
            .overflow_y_scroll()
            .border_l_1()
            .border_color(theme.border)
            .bg(theme.muted.opacity(0.35))
            .child(album_artwork_frame(
                display,
                layout.artwork_size,
                layout.mode,
                cx,
            ))
            .child(album_detail_metadata(
                pulse,
                display,
                AlbumDetailLayoutMode::Sidebar,
                cx,
            ))
            .child(tags)
            .into_any_element(),
        AlbumDetailLayoutMode::Stacked => v_flex()
            .id(("album-detail-panel", display.album_id.0))
            .w_full()
            .flex_shrink_0()
            .gap_4()
            .px_6()
            .py_4()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.muted.opacity(0.35))
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .items_start()
                    .child(album_artwork_frame(
                        display,
                        layout.artwork_size,
                        layout.mode,
                        cx,
                    ))
                    .child(album_detail_metadata(
                        pulse,
                        display,
                        AlbumDetailLayoutMode::Stacked,
                        cx,
                    )),
            )
            .child(tags)
            .into_any_element(),
    }
}

fn album_artwork_frame(
    display: &AlbumDisplay,
    artwork_size: f32,
    layout: AlbumDetailLayoutMode,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();

    let artwork = div()
        .w(px(artwork_size))
        .h(px(artwork_size))
        .rounded(theme.radius)
        .overflow_hidden()
        .bg(theme.muted)
        .border_1()
        .border_color(theme.border)
        .child(artwork_content(display.artwork.as_deref(), cx));

    match layout {
        AlbumDetailLayoutMode::Sidebar => div()
            .w_full()
            .flex()
            .justify_center()
            .flex_shrink_0()
            .child(artwork)
            .into_any_element(),
        AlbumDetailLayoutMode::Stacked => artwork.flex_shrink_0().into_any_element(),
    }
}

fn album_detail_metadata(
    pulse: &Entity<Pulse>,
    display: &AlbumDisplay,
    layout: AlbumDetailLayoutMode,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();

    let title = match layout {
        AlbumDetailLayoutMode::Sidebar => div().text_xl(),
        AlbumDetailLayoutMode::Stacked => div().text_lg(),
    };

    match layout {
        AlbumDetailLayoutMode::Sidebar => v_flex().min_w_0().flex_shrink_0().gap_1(),
        AlbumDetailLayoutMode::Stacked => v_flex().min_w_0().flex_1().gap_1(),
    }
    .child(
        title
            .font_semibold()
            .overflow_hidden()
            .text_ellipsis()
            .child(display.title.clone()),
    )
    .when_some(album_stats_line(display), |this, line| {
        this.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .overflow_hidden()
                .text_ellipsis()
                .child(line),
        )
    })
    .child(album_artist_line(pulse, display, cx))
}

fn album_artist_line(
    pulse: &Entity<Pulse>,
    display: &AlbumDisplay,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();

    if display.artist_entries.is_empty() {
        return div()
            .text_sm()
            .text_color(theme.muted_foreground)
            .child(display.artists.clone())
            .into_any_element();
    }

    h_flex()
        .min_w_0()
        .flex_wrap()
        .items_center()
        .gap_2()
        .children(artist_line_elements(pulse, &display.artist_entries, cx))
        .into_any_element()
}

fn artist_line_elements(
    pulse: &Entity<Pulse>,
    entries: &[AlbumArtistEntry],
    cx: &gpui::App,
) -> Vec<AnyElement> {
    let theme = cx.theme();
    let mut elements = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            elements.push(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(", ")
                    .into_any_element(),
            );
        }
        elements.push(artist_name_chip(pulse, entry, cx).into_any_element());
    }

    elements
}

fn artist_name_chip(
    pulse: &Entity<Pulse>,
    entry: &AlbumArtistEntry,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let pulse = pulse.clone();
    let artist_id = entry.artist_id;

    h_flex()
        .id(("album-artist-link", artist_id.0))
        .items_center()
        .cursor_pointer()
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(entry.name.clone()),
        )
        .hover(|this| this.text_color(theme.primary))
        .on_click(move |_, _, cx| {
            pulse.update(cx, |pulse, cx| {
                pulse.open_artist(artist_id, cx);
            });
        })
}

fn album_tags_editor(
    view: &Entity<AlbumViewerPage>,
    display: &AlbumDisplay,
    tag_input: &Entity<InputState>,
    tag_suggestions: &[String],
    tag_menu_open: bool,
    tag_input_bounds: Bounds<Pixels>,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let query = tag_input.read(cx).text().to_string();
    let filtered = filter_tag_suggestions(tag_suggestions, query.trim());
    let show_menu = tag_menu_open
        && !tag_suggestions.is_empty()
        && (query.trim().is_empty() || !filtered.is_empty());
    let album_id = display.album_id.0;

    v_flex()
        .gap_2()
        .child(
            div()
                .text_xs()
                .font_medium()
                .text_color(theme.muted_foreground)
                .child("Genres / Tags"),
        )
        .when(!display.library_genres.is_empty(), |this| {
            this.child(
                div().flex().flex_wrap().gap_2().children(
                    display
                        .library_genres
                        .iter()
                        .map(|genre| library_genre_chip(genre)),
                ),
            )
        })
        .when(!display.user_tags.is_empty(), |this| {
            this.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(display.user_tags.iter().map(|tag| user_tag_chip(view, tag))),
            )
        })
        .child(
            h_flex()
                .w_full()
                .gap_1()
                .items_center()
                .child(tag_input_with_suggestions(
                    view,
                    tag_input,
                    &filtered,
                    show_menu,
                    tag_input_bounds,
                    album_id,
                    cx,
                ))
                .child(
                    Button::new(("album-add-tag", album_id))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Plus)
                        .tooltip("Add tag")
                        .on_click({
                            let view = view.clone();
                            move |_, window, cx| {
                                view.update(cx, |view, cx| {
                                    view.commit_new_tag(window, cx);
                                });
                            }
                        }),
                ),
        )
}

fn tag_input_with_suggestions(
    view: &Entity<AlbumViewerPage>,
    tag_input: &Entity<InputState>,
    filtered_suggestions: &[String],
    show_menu: bool,
    menu_bounds: Bounds<Pixels>,
    album_id: u64,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let view_for_capture = view.clone();
    let view_for_toggle = view.clone();
    let view_for_dismiss = view.clone();
    let popup_radius = theme.radius.min(px(8.));

    div()
        .relative()
        .flex_1()
        .min_w_0()
        .child(
            h_flex()
                .w_full()
                .items_center()
                .border_1()
                .border_color(theme.input)
                .rounded(theme.radius)
                .bg(theme.background)
                .overflow_hidden()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .capture_any_mouse_down({
                            move |_, _, cx| {
                                view_for_capture.update(cx, |view, cx| {
                                    view.open_tag_menu(cx);
                                });
                            }
                        })
                        .child(Input::new(tag_input).small().appearance(false)),
                )
                .child(
                    Button::new(("album-tag-menu", album_id))
                        .ghost()
                        .xsmall()
                        .icon(IconName::ChevronDown)
                        .tooltip("Browse tags")
                        .on_click({
                            move |_, _, cx| {
                                view_for_toggle.update(cx, |view, cx| {
                                    view.toggle_tag_menu(cx);
                                });
                            }
                        }),
                ),
        )
        .on_prepaint({
            let view = view.clone();
            move |bounds, _, cx| {
                view.update(cx, |view, _| {
                    view.tag_input_bounds = bounds;
                });
            }
        })
        .when(show_menu, |this| {
            this.child(
                deferred(tag_suggestions_menu(
                    view,
                    filtered_suggestions,
                    menu_bounds,
                    popup_radius,
                    view_for_dismiss,
                    cx,
                ))
                .with_priority(1),
            )
        })
}

fn tag_suggestions_menu(
    view: &Entity<AlbumViewerPage>,
    suggestions: &[String],
    bounds: Bounds<Pixels>,
    popup_radius: Pixels,
    view_for_dismiss: Entity<AlbumViewerPage>,
    cx: &gpui::App,
) -> AnyElement {
    let theme = cx.theme();

    anchored()
        .snap_to_window_with_margin(px(8.))
        .child(
            div().occlude().w(bounds.size.width).child(
                v_flex()
                    .occlude()
                    .mt_1p5()
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(popup_radius)
                    .shadow_md()
                    .max_h(rems(TAG_MENU_MAX_HEIGHT))
                    .overflow_hidden()
                    .py_1()
                    .children(suggestions.iter().enumerate().map(|(index, label)| {
                        let view = view.clone();
                        let label = label.clone();

                        div()
                            .id(("album-tag-suggestion", index))
                            .px_3()
                            .py_1p5()
                            .text_sm()
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.muted))
                            .child(label.clone())
                            .on_click(move |_, window, cx| {
                                view.update(cx, |view, cx| {
                                    view.pick_tag_suggestion(&label, window, cx);
                                });
                            })
                    }))
                    .on_mouse_down_out(move |_, _, cx| {
                        view_for_dismiss.update(cx, |view, cx| {
                            view.close_tag_menu(cx);
                        });
                    }),
            ),
        )
        .into_any_element()
}

fn library_genre_chip(genre: &str) -> impl IntoElement {
    Tag::new().small().outline().child(genre.to_string())
}

fn user_tag_chip(view: &Entity<AlbumViewerPage>, tag: &str) -> impl IntoElement {
    let view = view.clone();
    let tag = tag.to_string();

    let tag_element = Tag::new().small().child(tag.clone()).child(
        Button::new(("remove-tag-button", hash(&tag)))
            .icon(IconName::Close)
            .ghost()
            .xsmall()
            .on_click(move |_, _, cx| {
                view.update(cx, |view, cx| {
                    view.remove_user_tag(&tag, cx);
                });
            }),
    );

    h_flex().items_center().gap_1().child(tag_element)
}

fn build_album_track_list_rows(tracks: &[TrackRow]) -> Vec<AlbumTrackListRow> {
    if tracks.is_empty() {
        return Vec::new();
    }

    let mut discs: BTreeMap<u16, Vec<usize>> = BTreeMap::new();
    for (track_ix, track) in tracks.iter().enumerate() {
        let disc = track.disc_number.unwrap_or(1);
        discs.entry(disc).or_default().push(track_ix);
    }

    let show_header = discs.len() > 1;
    let disc_count = discs.len();
    let mut rows = Vec::new();

    for (disc_ix, (disc, indices)) in discs.into_iter().enumerate() {
        if show_header {
            rows.push(AlbumTrackListRow::disc_header(
                format!("Disc {disc}").into(),
            ));
        }

        for (stripe_ix, track_ix) in indices.into_iter().enumerate() {
            let Some(track) = tracks.get(track_ix) else {
                continue;
            };
            rows.push(AlbumTrackListRow::track(
                track.clone(),
                stripe_ix.is_multiple_of(2),
            ));
        }

        if show_header && disc_ix.saturating_add(1) < disc_count {
            rows.push(AlbumTrackListRow::disc_spacer());
        }
    }

    rows
}

fn disc_header_row(label: &SharedString, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .w_full()
        .h(px(DISC_HEADER_HEIGHT))
        .flex()
        .items_end()
        .pb_1()
        .text_sm()
        .font_semibold()
        .text_color(theme.muted_foreground)
        .child(label.clone())
}

fn track_row(
    track: &TrackRow,
    stripe: bool,
    queue: &[pulse_model::SongId],
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let is_current = PulsePlayer::current_song_id(cx) == Some(track.id);
    let index = queue.iter().position(|id| *id == track.id).unwrap_or(0);
    let queue = queue.to_vec();

    h_flex()
        .id(("album-track", track.id.0))
        .w_full()
        .h(px(TRACK_ROW_HEIGHT))
        .items_center()
        .gap_3()
        .px_4()
        .cursor_pointer()
        .when(is_current, |this| this.bg(theme.primary.opacity(0.15)))
        .when(!is_current && stripe, |this| this.bg(theme.list_even))
        .on_click(move |_, _, cx| PulsePlayer::play_songs(cx, &queue, index))
        .child(
            div()
                .w(px(32.))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(track.number_label.clone()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .overflow_hidden()
                .text_ellipsis()
                .child(track.title.clone()),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(track.duration.clone()),
        )
}

fn artwork_content(path: Option<&std::path::Path>, cx: &gpui::App) -> AnyElement {
    let theme = cx.theme();

    path.map_or_else(
        || {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::GalleryVerticalEnd)
                        .size_10()
                        .text_color(theme.muted_foreground),
                )
                .into_any_element()
        },
        |path| {
            img(path)
                .size_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element()
        },
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn track(id: u64, title: &str, disc: Option<u16>, track_number: Option<u16>) -> TrackRow {
        use pulse_model::SongId;

        TrackRow {
            id: SongId(id),
            title: title.into(),
            number_label: track_number.map_or_else(|| "-".into(), |n| n.to_string().into()),
            duration: "3:00".into(),
            disc_number: disc,
            track_number,
        }
    }

    #[test]
    fn single_disc_hides_disc_header() {
        let tracks = vec![
            track(1, "A", None, Some(1)),
            track(2, "B", Some(1), Some(2)),
        ];
        let rows = build_album_track_list_rows(&tracks);
        assert!(
            rows.iter()
                .all(|row| matches!(row.kind, AlbumTrackListRowKind::Track { .. }))
        );
    }

    #[test]
    fn multiple_discs_show_headers() {
        let tracks = vec![
            track(1, "A", Some(1), Some(1)),
            track(2, "B", Some(2), Some(1)),
        ];
        let rows = build_album_track_list_rows(&tracks);
        assert!(
            rows.iter()
                .any(|row| matches!(row.kind, AlbumTrackListRowKind::DiscHeader { .. }))
        );
    }

    #[test]
    fn track_rows_alternate_stripes() {
        let tracks = vec![
            track(1, "A", None, Some(1)),
            track(2, "B", None, Some(2)),
            track(3, "C", None, Some(3)),
        ];
        let stripes: Vec<bool> = build_album_track_list_rows(&tracks)
            .into_iter()
            .filter_map(|row| match row.kind {
                AlbumTrackListRowKind::Track { stripe, .. } => Some(stripe),
                _ => None,
            })
            .collect();
        assert_eq!(stripes, vec![true, false, true]);
    }
}
