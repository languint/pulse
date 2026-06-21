use std::rc::Rc;

use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ObjectFit, ParentElement, Pixels,
    Render, SharedString, Size, StatefulInteractiveElement, Styled, StyledImage, Window, div, img,
    prelude::FluentBuilder, px, rems, size,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt as _, VirtualListScrollHandle,
    button::{Button, ButtonVariants as _},
    h_flex,
    tag::Tag,
    v_flex, v_virtual_list,
};
use pulse_model::{AlbumId, ArtistId, SongId};

use crate::components::pulse::Pulse;
use crate::data::{import_and_save_artist_logo, save_artist_artwork};
use crate::icons::PulseIcon;
use crate::player::PulsePlayer;

use super::common::{
    ArtistDisplay, CatalogFingerprint, TagCount, artwork_tile_content, catalog_fingerprint,
    empty_state, overrides_generation, resolve_artist_display,
};

const DETAIL_PANEL_WIDTH: f32 = 360.;
const DETAIL_ARTWORK_SIZE: f32 = 320.;
const ROW_HEIGHT: f32 = 56.;
const SECTION_HEADER_HEIGHT: f32 = 36.;
const SECTION_GAP: f32 = 12.;

#[derive(Clone, Debug)]
enum ArtistListRowKind {
    SectionHeader {
        label: SharedString,
    },
    Album {
        album_id: AlbumId,
        title: SharedString,
        subtitle: SharedString,
        artwork: Option<std::path::PathBuf>,
        stripe: bool,
    },
    Song {
        song_id: SongId,
        title: SharedString,
        subtitle: SharedString,
        duration: SharedString,
        artwork: Option<std::path::PathBuf>,
        stripe: bool,
    },
    SectionSpacer,
}

#[derive(Clone, Debug)]
struct ArtistListRow {
    kind: ArtistListRowKind,
}

impl ArtistListRow {
    const fn height(&self) -> Pixels {
        match self.kind {
            ArtistListRowKind::SectionHeader { .. } => px(SECTION_HEADER_HEIGHT),
            ArtistListRowKind::Album { .. } | ArtistListRowKind::Song { .. } => px(ROW_HEIGHT),
            ArtistListRowKind::SectionSpacer => px(SECTION_GAP),
        }
    }
}

pub struct ArtistViewerPage {
    pulse: Entity<Pulse>,
    scroll_handle: VirtualListScrollHandle,
    cached_artist_id: Option<ArtistId>,
    catalog_fp: CatalogFingerprint,
    overrides_gen: u32,
    display: Option<ArtistDisplay>,
    list_rows: Rc<[ArtistListRow]>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
}

impl ArtistViewerPage {
    #[must_use]
    pub fn new(pulse: Entity<Pulse>, _: &mut Context<Self>) -> Self {
        Self {
            pulse,
            scroll_handle: VirtualListScrollHandle::new(),
            cached_artist_id: None,
            catalog_fp: CatalogFingerprint::default(),
            overrides_gen: 0,
            display: None,
            list_rows: Rc::from([]),
            item_sizes: Rc::new(Vec::new()),
        }
    }

    const fn invalidate_artist_cache(&mut self) {
        self.cached_artist_id = None;
    }

    fn ensure_artist(&mut self, artist_id: ArtistId, cx: &gpui::App) {
        let fp = catalog_fingerprint(cx);
        let overrides_gen = overrides_generation(cx);
        if self.cached_artist_id == Some(artist_id)
            && self.catalog_fp == fp
            && self.overrides_gen == overrides_gen
        {
            return;
        }

        let Some(display) = resolve_artist_display(cx, artist_id) else {
            self.cached_artist_id = None;
            self.catalog_fp = fp;
            self.overrides_gen = overrides_gen;
            self.display = None;
            self.list_rows = Rc::from([]);
            self.item_sizes = Rc::new(Vec::new());
            return;
        };

        let rows = build_artist_list_rows(&display);
        self.item_sizes = Rc::new(rows.iter().map(|row| size(px(0.), row.height())).collect());
        self.list_rows = rows.into();
        self.display = Some(display);
        self.cached_artist_id = Some(artist_id);
        self.catalog_fp = fp;
        self.overrides_gen = overrides_gen;
    }

    fn pick_artist_logo(&self, window: &Window, cx: &Context<Self>) {
        let Some(display) = self.display.clone() else {
            return;
        };

        let dialog = rfd::AsyncFileDialog::new()
            .set_title("Select artist logo")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
            .set_parent(window)
            .pick_file();

        cx.spawn(async move |this, cx| {
            let Some(handle) = dialog.await else {
                return;
            };

            let source = handle.path().to_path_buf();
            let override_key = display.override_key.clone();

            if let Err(error) =
                cx.update(|cx| import_and_save_artist_logo(cx, &override_key, &source))
            {
                tracing::error!(%error, "failed to import artist logo");
                return;
            }

            this.update(cx, |view, cx| {
                view.invalidate_artist_cache();
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn remove_artist_logo(&mut self, cx: &mut Context<Self>) {
        let Some(override_key) = self
            .display
            .as_ref()
            .map(|display| display.override_key.clone())
        else {
            return;
        };

        save_artist_artwork(cx, &override_key, None);
        self.invalidate_artist_cache();
        cx.notify();
    }

    fn render_list_row(&self, row_ix: usize, cx: &gpui::App) -> AnyElement {
        let Some(row) = self.list_rows.get(row_ix) else {
            return div().into_any_element();
        };

        match &row.kind {
            ArtistListRowKind::SectionHeader { label } => {
                section_header_row(label, cx).into_any_element()
            }
            ArtistListRowKind::Album {
                album_id,
                title,
                subtitle,
                artwork,
                stripe,
            } => album_row(
                *album_id,
                title,
                subtitle,
                artwork.as_deref(),
                *stripe,
                &self.pulse,
                cx,
            )
            .into_any_element(),
            ArtistListRowKind::Song {
                song_id,
                title,
                subtitle,
                duration,
                artwork,
                stripe,
            } => {
                let song_ids: Vec<_> = self
                    .display
                    .as_ref()
                    .map(|display| {
                        display
                            .other_songs
                            .iter()
                            .map(|song| song.song_id)
                            .collect()
                    })
                    .unwrap_or_default();
                song_row(
                    *song_id,
                    title,
                    subtitle,
                    duration,
                    artwork.as_deref(),
                    *stripe,
                    &song_ids,
                    cx,
                )
                .into_any_element()
            }
            ArtistListRowKind::SectionSpacer => div().h(px(SECTION_GAP)).into_any_element(),
        }
    }
}

impl Render for ArtistViewerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.pulse.read(cx).page();
        let Some(artist_id) = page.artist_detail() else {
            return empty_state("No artist selected.", cx).into_any_element();
        };

        self.ensure_artist(artist_id, cx);

        let Some(display) = self.display.clone() else {
            return empty_state("Artist not found.", cx).into_any_element();
        };

        let item_sizes = self.item_sizes.clone();
        let entity = cx.entity();

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(
                        div().flex_1().min_w_0().min_h_0().px_6().pb_6().child(
                            v_virtual_list(
                                entity.clone(),
                                "artist-content-list",
                                item_sizes,
                                |this, visible_range, _, cx| {
                                    visible_range
                                        .map(|row_ix| this.render_list_row(row_ix, cx))
                                        .collect()
                                },
                            )
                            .track_scroll(&self.scroll_handle),
                        ),
                    )
                    .child(artist_detail_panel(&entity, &display, cx)),
            )
            .into_any_element()
    }
}

fn artist_stats_line(display: &ArtistDisplay) -> String {
    let mut parts = Vec::new();
    parts.push(format!("{} albums", display.album_count));
    if display.other_song_count > 0 {
        parts.push(format!("{} other tracks", display.other_song_count));
    }
    parts.join(" · ")
}

fn artist_detail_panel(
    view: &Entity<ArtistViewerPage>,
    display: &ArtistDisplay,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();

    v_flex()
        .id(("artist-detail-panel", display.artist_id.0))
        .w(px(DETAIL_PANEL_WIDTH))
        .flex_shrink_0()
        .h_full()
        .gap_4()
        .px_6()
        .py_6()
        .border_l_1()
        .border_color(theme.border)
        .bg(theme.muted.opacity(0.35))
        .child(artist_artwork_frame(view, display, cx))
        .child(artist_detail_metadata(display, cx))
        .child(artist_tag_breakdown(display, cx))
}

fn artist_artwork_frame(
    view: &Entity<ArtistViewerPage>,
    display: &ArtistDisplay,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let view_for_pick = view.clone();
    let view_for_remove = view.clone();

    v_flex()
        .gap_2()
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
                    .child(artist_artwork_content(display.artwork.as_deref(), cx)),
            ),
        )
        .child(
            h_flex()
                .justify_end()
                .gap_2()
                .child(
                    Button::new(("artist-logo-replace", display.artist_id.0))
                        .icon(PulseIcon::Pencil)
                        .outline()
                        .on_click(move |_, window, cx| {
                            view_for_pick.update(cx, |view, cx| {
                                view.pick_artist_logo(window, cx);
                            });
                        })
                        .tooltip("Replace"),
                )
                .when(display.has_custom_logo, |this| {
                    this.child(
                        Button::new(("artist-logo-remove", display.artist_id.0))
                            .icon(PulseIcon::Trash2)
                            .ghost()
                            .danger()
                            .on_click(move |_, _, cx| {
                                view_for_remove.update(cx, |view, cx| {
                                    view.remove_artist_logo(cx);
                                });
                            })
                            .tooltip("Remove"),
                    )
                }),
        )
}

fn artist_artwork_content(path: Option<&std::path::Path>, cx: &gpui::App) -> AnyElement {
    let theme = cx.theme();

    path.map_or_else(
        || {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::User)
                        .size(rems(4.))
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

fn artist_detail_metadata(display: &ArtistDisplay, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    v_flex()
        .gap_1()
        .child(div().text_xl().font_semibold().child(display.name.clone()))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(artist_stats_line(display)),
        )
}

fn artist_tag_breakdown(display: &ArtistDisplay, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    v_flex()
        .gap_2()
        .child(
            div()
                .text_sm()
                .font_medium()
                .text_color(theme.muted_foreground)
                .child("Tags"),
        )
        .when(display.tag_counts.is_empty(), |this| {
            this.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("No tags yet."),
            )
        })
        .when(!display.tag_counts.is_empty(), |this| {
            this.child(
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .children(display.tag_counts.iter().map(tag_count_chip)),
            )
        })
}

fn tag_count_chip(tag: &TagCount) -> impl IntoElement {
    Tag::new()
        .small()
        .outline()
        .child(format!("{} ({})", tag.label, tag.count))
}

fn section_header_row(label: &SharedString, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .w_full()
        .h(px(SECTION_HEADER_HEIGHT))
        .flex()
        .items_end()
        .pb_1()
        .text_sm()
        .font_semibold()
        .text_color(theme.muted_foreground)
        .child(label.clone())
}

fn row_artwork(path: Option<&std::path::Path>, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .w(px(40.))
        .h(px(40.))
        .flex_shrink_0()
        .rounded(theme.radius)
        .overflow_hidden()
        .bg(theme.muted)
        .border_1()
        .border_color(theme.border)
        .child(artwork_tile_content(path, cx))
}

fn album_row(
    album_id: AlbumId,
    title: &SharedString,
    subtitle: &SharedString,
    artwork: Option<&std::path::Path>,
    stripe: bool,
    pulse: &Entity<Pulse>,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let pulse = pulse.clone();

    h_flex()
        .id(("artist-album", album_id.0))
        .w_full()
        .h(px(ROW_HEIGHT))
        .items_center()
        .gap_3()
        .px_2()
        .cursor_pointer()
        .when(stripe, |this| this.bg(theme.list_even))
        .child(row_artwork(artwork, cx))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_0p5()
                .child(
                    div()
                        .text_sm()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(subtitle.clone()),
                ),
        )
        .on_click(move |_, _, cx| {
            pulse.update(cx, |pulse, cx| {
                pulse.open_album(album_id, cx);
            });
        })
}

#[allow(clippy::too_many_arguments)]
fn song_row(
    song_id: SongId,
    title: &SharedString,
    subtitle: &SharedString,
    duration: &SharedString,
    artwork: Option<&std::path::Path>,
    stripe: bool,
    queue: &[SongId],
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let index = queue.iter().position(|id| *id == song_id).unwrap_or(0);
    let queue = queue.to_vec();

    h_flex()
        .id(("artist-song", song_id.0))
        .w_full()
        .h(px(ROW_HEIGHT))
        .items_center()
        .gap_3()
        .px_2()
        .cursor_pointer()
        .when(stripe, |this| this.bg(theme.list_even))
        .on_click(move |_, _, cx| PulsePlayer::play_songs(cx, &queue, index))
        .child(row_artwork(artwork, cx))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_0p5()
                .child(
                    div()
                        .text_sm()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(subtitle.clone()),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(duration.clone()),
        )
}

fn build_artist_list_rows(display: &ArtistDisplay) -> Vec<ArtistListRow> {
    let mut rows = Vec::new();

    if !display.albums.is_empty() {
        rows.push(ArtistListRow {
            kind: ArtistListRowKind::SectionHeader {
                label: "Albums".into(),
            },
        });

        for (index, album) in display.albums.iter().enumerate() {
            rows.push(ArtistListRow {
                kind: ArtistListRowKind::Album {
                    album_id: album.album_id,
                    title: album.title.clone(),
                    subtitle: album.subtitle.clone(),
                    artwork: album.artwork.clone(),
                    stripe: index.is_multiple_of(2),
                },
            });
        }
    }

    if !display.other_songs.is_empty() {
        if !display.albums.is_empty() {
            rows.push(ArtistListRow {
                kind: ArtistListRowKind::SectionSpacer,
            });
        }

        rows.push(ArtistListRow {
            kind: ArtistListRowKind::SectionHeader {
                label: "Other tracks".into(),
            },
        });

        for (index, song) in display.other_songs.iter().enumerate() {
            rows.push(ArtistListRow {
                kind: ArtistListRowKind::Song {
                    song_id: song.song_id,
                    title: song.title.clone(),
                    subtitle: song.subtitle.clone(),
                    duration: song.duration.clone(),
                    artwork: song.artwork.clone(),
                    stripe: index.is_multiple_of(2),
                },
            });
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_model::SongId;

    #[test]
    fn builds_album_and_other_track_sections() {
        let display = ArtistDisplay {
            artist_id: ArtistId(1),
            override_key: "artist".into(),
            name: "Artist".into(),
            artwork: None,
            has_custom_logo: false,
            album_count: 1,
            other_song_count: 1,
            albums: vec![super::super::common::ArtistAlbumRow {
                album_id: AlbumId(1),
                title: "Album".into(),
                subtitle: "2020 · 10 tracks".into(),
                artwork: None,
            }],
            other_songs: vec![super::super::common::ArtistSongRow {
                song_id: SongId(2),
                title: "Featured".into(),
                subtitle: "Compilation".into(),
                duration: "3:00".into(),
                artwork: None,
            }],
            tag_counts: Vec::new(),
        };

        let rows = build_artist_list_rows(&display);
        assert!(
            rows.iter()
                .any(|row| matches!(row.kind, ArtistListRowKind::SectionHeader { .. }))
        );
        assert!(
            rows.iter()
                .any(|row| matches!(row.kind, ArtistListRowKind::Album { .. }))
        );
        assert!(
            rows.iter()
                .any(|row| matches!(row.kind, ArtistListRowKind::Song { .. }))
        );
    }
}
