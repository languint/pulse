use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ObjectFit, ParentElement, Pixels,
    Render, SharedString, Size, Styled, StyledImage, Window, div, img, prelude::FluentBuilder, px,
    size,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt as _, VirtualListScrollHandle, button::Button,
    h_flex, tag::Tag, v_flex, v_virtual_list,
};
use pulse_model::AlbumId;

use crate::components::pulse::Pulse;

use super::common::{
    AlbumDisplay, CatalogFingerprint, TrackRow, catalog_fingerprint, empty_state,
    format_album_duration_ms, resolve_album_display,
};

const DETAIL_PANEL_WIDTH: f32 = 360.;
const DETAIL_ARTWORK_SIZE: f32 = 320.;
const TRACK_ROW_HEIGHT: f32 = 40.;
const DISC_HEADER_HEIGHT: f32 = 34.;
const DISC_SECTION_GAP: f32 = 16.;

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
    cached_album_id: Option<AlbumId>,
    catalog_fp: CatalogFingerprint,
    display: Option<AlbumDisplay>,
    list_rows: Rc<[AlbumTrackListRow]>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
}

impl AlbumViewerPage {
    #[must_use]
    pub fn new(pulse: Entity<Pulse>) -> Self {
        Self {
            pulse,
            track_scroll_handle: VirtualListScrollHandle::new(),
            cached_album_id: None,
            catalog_fp: CatalogFingerprint::default(),
            display: None,
            list_rows: Rc::from([]),
            item_sizes: Rc::new(Vec::new()),
        }
    }

    fn ensure_album(&mut self, album_id: AlbumId, cx: &gpui::App) {
        let fp = catalog_fingerprint(cx);
        if self.cached_album_id == Some(album_id) && self.catalog_fp == fp {
            return;
        }

        let Some(display) = resolve_album_display(cx, album_id) else {
            self.cached_album_id = None;
            self.catalog_fp = fp;
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
                track_row(track, *stripe, cx).into_any_element()
            }
            AlbumTrackListRowKind::DiscSpacer => div().h(px(DISC_SECTION_GAP)).into_any_element(),
        }
    }
}

impl Render for AlbumViewerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.pulse.read(cx).page();
        let Some(album_id) = page.album_detail() else {
            return empty_state("No album selected.", cx).into_any_element();
        };

        self.ensure_album(album_id, cx);

        let Some(display) = self.display.as_ref() else {
            return empty_state("Album not found.", cx).into_any_element();
        };

        let pulse = self.pulse.clone();
        let item_sizes = self.item_sizes.clone();
        let entity = cx.entity();

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(h_flex_back_button(pulse))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(
                        div().flex_1().min_w_0().min_h_0().px_6().pb_6().child(
                            v_virtual_list(
                                entity,
                                "album-track-list",
                                item_sizes,
                                |this, visible_range, _, cx| {
                                    visible_range
                                        .map(|row_ix| this.render_track_list_row(row_ix, cx))
                                        .collect()
                                },
                            )
                            .track_scroll(&self.track_scroll_handle),
                        ),
                    )
                    .child(album_detail_panel(display, cx)),
            )
            .into_any_element()
    }
}

fn h_flex_back_button(pulse: Entity<Pulse>) -> impl IntoElement {
    use gpui_component::h_flex;

    h_flex().items_center().gap_2().px_6().pt_6().pb_4().child(
        Button::new("album-back")
            .icon(IconName::ArrowLeft)
            .outline()
            .on_click(move |_, _, cx| {
                pulse.update(cx, |pulse, cx| {
                    pulse.show_albums(cx);
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

fn album_detail_panel(display: &AlbumDisplay, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    v_flex()
        .id(("album-detail-panel", display.album_id.0))
        .w(px(DETAIL_PANEL_WIDTH))
        .flex_shrink_0()
        .h_full()
        .gap_4()
        .px_6()
        .py_6()
        .border_l_1()
        .border_color(theme.border)
        .bg(theme.muted.opacity(0.35))
        .child(
            div().w_full().flex().justify_center().child(
                div()
                    .w(px(DETAIL_ARTWORK_SIZE))
                    .h(px(DETAIL_ARTWORK_SIZE))
                    .rounded(theme.radius)
                    .overflow_hidden()
                    .bg(theme.muted)
                    .border_1()
                    .border_color(theme.border)
                    .child(artwork_content(display.artwork.as_deref(), cx)),
            ),
        )
        .child(
            v_flex()
                .gap_1()
                .child(div().text_xl().font_semibold().child(display.title.clone()))
                .when_some(album_stats_line(display), |this, line| {
                    this.child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(line),
                    )
                })
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(display.artists.clone()),
                ),
        )
        .when(!display.genres.is_empty(), |this| {
            this.child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_medium()
                            .text_color(theme.muted_foreground)
                            .child("Genres"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .children(display.genres.iter().map(|genre| genre_chip(genre))),
                    ),
            )
        })
}

fn genre_chip(genre: &str) -> impl IntoElement {
    Tag::new().small().outline().child(genre.to_string())
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

fn track_row(track: &TrackRow, stripe: bool, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    h_flex()
        .id(("album-track", track.id.0))
        .w_full()
        .h(px(TRACK_ROW_HEIGHT))
        .items_center()
        .gap_3()
        .px_4()
        .rounded(cx.theme().radius)
        .when(stripe, |this| this.bg(theme.muted.opacity(0.35)))
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
}
