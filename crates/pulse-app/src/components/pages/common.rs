use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, img, InteractiveElement, IntoElement, ObjectFit, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, StyledImage, div, prelude::FluentBuilder, px,
};
use gpui_component::{ActiveTheme, Icon, IconName, StyledExt as _, tooltip::Tooltip, v_flex};
use crate::data::{DataOverrides, DataPaths};
use pulse_data::{UserOverrides, album_override_key, artist_override_key};
use pulse_model::{Album, AlbumArtists, Artist, ArtistId, ArtworkId, ThumbnailSize};

use crate::library::PulseLibrary;

pub const THUMB_SIZE: f32 = 160.;
pub const CARD_WIDTH: f32 = 160.;
pub const GRID_COLUMNS: usize = 5;
pub const GRID_LABEL_LINES: usize = 2;
const GRID_LABEL_LINES_F32: f32 = 2.;
/// Vertical space between virtualized grid rows.
pub const GRID_ROW_GAP: f32 = 12.;
/// `gap_2` between thumb, title, and subtitle inside a card.
pub const GRID_CARD_GAP: f32 = 8.;
/// Approximate single-line heights (`font_size * phi()` at 16px rem).
pub const GRID_TITLE_LINE_HEIGHT: f32 = 22.;
pub const GRID_SUBTITLE_LINE_HEIGHT: f32 = 20.;
/// Thumb + borders + gaps + two-line title + two-line subtitle.
pub const GRID_ROW_HEIGHT: f32 = THUMB_SIZE
    + 2.
    + GRID_CARD_GAP * 2.
    + GRID_TITLE_LINE_HEIGHT * GRID_LABEL_LINES_F32
    + GRID_SUBTITLE_LINE_HEIGHT * GRID_LABEL_LINES_F32;
pub const GRID_GAP: f32 = 16.;

#[derive(Clone, Debug)]
pub struct GridItem {
    pub title: SharedString,
    pub subtitle: SharedString,
    pub thumbnail: Option<PathBuf>,
}

pub fn page_shell(title: &'static str, body: impl IntoElement) -> impl IntoElement {
    v_flex()
        .size_full()
        .child(
            div()
                .w_full()
                .px_6()
                .pt_6()
                .pb_4()
                .text_lg()
                .font_semibold()
                .child(title),
        )
        .child(div().flex_1().min_h_0().px_6().pb_6().child(body))
}

pub fn empty_state(message: &'static str, cx: &gpui::App) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(message)
}

pub fn format_album_artists(
    artists: &HashMap<ArtistId, Artist>,
    album_artists: &AlbumArtists,
) -> String {
    match album_artists {
        AlbumArtists::Single(id) => artists
            .get(id)
            .map_or_else(|| "Unknown Artist".into(), |artist| artist.name.clone()),
        AlbumArtists::Various => "Various Artists".into(),
        AlbumArtists::Multiple(ids) => ids
            .iter()
            .filter_map(|id| artists.get(id).map(|artist| artist.name.as_str()))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

pub fn album_includes_artist(album: &Album, artist_id: ArtistId) -> bool {
    match &album.album_artists {
        AlbumArtists::Single(id) => *id == artist_id,
        AlbumArtists::Various => false,
        AlbumArtists::Multiple(ids) => ids.contains(&artist_id),
    }
}

pub fn album_thumbnail_path(
    store: &pulse_library::LibraryStore,
    artwork_id: Option<ArtworkId>,
) -> Option<PathBuf> {
    artwork_id.and_then(|id| {
        store
            .thumbnail_path(id, ThumbnailSize::Medium)
            .map(std::path::Path::to_path_buf)
    })
}

pub fn artist_thumbnail_path(
    store: &pulse_library::LibraryStore,
    artist_id: ArtistId,
) -> Option<PathBuf> {
    if let Some(artwork_id) = store
        .artists()
        .get(&artist_id)
        .and_then(|artist| artist.artwork_id)
    {
        return album_thumbnail_path(store, Some(artwork_id));
    }

    store
        .albums()
        .values()
        .find(|album| {
            album.artwork_id.is_some() && album_includes_artist(album, artist_id)
        })
        .and_then(|album| album_thumbnail_path(store, album.artwork_id))
}

pub fn collect_album_items(cx: &gpui::App) -> Vec<GridItem> {
    let library = cx.global::<PulseLibrary>().inner();
    let store = library.store();
    let artists = store.artists();

    let mut albums: Vec<&Album> = store.albums().values().collect();
    albums.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then_with(|| {
                format_album_artists(artists, &left.album_artists)
                    .cmp(&format_album_artists(artists, &right.album_artists))
            })
    });

    albums
        .into_iter()
        .map(|album| {
            let artist_label = format_album_artists(artists, &album.album_artists);
            let override_key = album_override_key(&album.title, &artist_label);
            let user_override = cx.global::<DataOverrides>().album(&override_key);
            let paths = cx.global::<DataPaths>();

            let title = user_override
                .and_then(|entry| entry.title.as_deref())
                .unwrap_or(&album.title);

            let thumbnail = user_override
                .and_then(|entry| entry.artwork.as_deref())
                .and_then(|path| UserOverrides::resolve_artwork(paths, Some(path)))
                .or_else(|| album_thumbnail_path(store, album.artwork_id));

            GridItem {
                title: title.to_string().into(),
                subtitle: artist_label.into(),
                thumbnail,
            }
        })
        .collect()
}

pub fn collect_artist_items(cx: &gpui::App) -> Vec<GridItem> {
    let store = cx.global::<PulseLibrary>().inner().store();

    let mut artist_ids: Vec<ArtistId> = store.artists().keys().copied().collect();
    artist_ids.sort_by_key(|id| store.artists().get(id).map(|artist| artist.name.clone()));

    artist_ids
        .into_iter()
        .filter_map(|artist_id| {
            let artist = store.artists().get(&artist_id)?;
            let override_key = artist_override_key(&artist.name);
            let user_override = cx.global::<DataOverrides>().artist(&override_key);
            let paths = cx.global::<DataPaths>();

            let title = user_override
                .and_then(|entry| entry.name.as_deref())
                .unwrap_or(&artist.name);

            let thumbnail = user_override
                .and_then(|entry| entry.artwork.as_deref())
                .and_then(|path| UserOverrides::resolve_artwork(paths, Some(path)))
                .or_else(|| artist_thumbnail_path(store, artist_id));

            Some(GridItem {
                title: title.to_string().into(),
                subtitle: SharedString::default(),
                thumbnail,
            })
        })
        .collect()
}

pub fn media_card(
    grid_id: &'static str,
    row_ix: usize,
    col_ix: usize,
    item: &GridItem,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let cell_ix = row_ix
        .saturating_mul(GRID_COLUMNS)
        .saturating_add(col_ix);

    v_flex()
        .w(px(CARD_WIDTH))
        .flex_shrink_0()
        .gap(px(GRID_CARD_GAP))
        .child(
            div()
                .id((grid_id, cell_ix))
                .w(px(THUMB_SIZE))
                .h(px(THUMB_SIZE))
                .rounded(theme.radius)
                .overflow_hidden()
                .bg(theme.muted)
                .border_1()
                .border_color(theme.border)
                .child(thumbnail_content(item.thumbnail.as_deref(), cx)),
        )
        .child(grid_label(
            (grid_id, cell_ix.saturating_mul(2)),
            item.title.clone(),
            GridLabelStyle::Title,
            cx,
        ))
        .when(!item.subtitle.is_empty(), |this| {
            this.child(grid_label(
                (grid_id, cell_ix.saturating_mul(2).saturating_add(1)),
                item.subtitle.clone(),
                GridLabelStyle::Subtitle,
                cx,
            ))
        })
}

#[derive(Clone, Copy)]
enum GridLabelStyle {
    Title,
    Subtitle,
}

impl GridLabelStyle {
    const fn line_height(self) -> f32 {
        match self {
            Self::Title => GRID_TITLE_LINE_HEIGHT,
            Self::Subtitle => GRID_SUBTITLE_LINE_HEIGHT,
        }
    }
}

fn grid_label(
    element_id: impl Into<gpui::ElementId>,
    text: SharedString,
    style: GridLabelStyle,
    cx: &gpui::App,
) -> impl IntoElement {
    let theme = cx.theme();
    let tooltip_text = text.clone();
    let label_height = px(style.line_height() * GRID_LABEL_LINES_F32);

    let mut label = div()
        .w_full()
        .min_w_0()
        .h(label_height)
        .line_clamp(GRID_LABEL_LINES)
        .text_ellipsis();

    label = match style {
        GridLabelStyle::Title => label.text_sm().font_medium(),
        GridLabelStyle::Subtitle => label.text_xs().text_color(theme.muted_foreground),
    };

    label
        .id(element_id)
        .tooltip(move |window, cx| Tooltip::new(tooltip_text.clone()).build(window, cx))
        .child(text)
}

fn thumbnail_content(path: Option<&Path>, cx: &gpui::App) -> AnyElement {
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
                        .size_8()
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
