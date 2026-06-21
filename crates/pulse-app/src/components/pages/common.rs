use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::data::{DataOverrides, DataPaths};
use gpui::{
    AnyElement, InteractiveElement, IntoElement, ObjectFit, ParentElement, Pixels, SharedString,
    Size, StatefulInteractiveElement, Styled, StyledImage, Window, div, img, prelude::FluentBuilder,
    px, size,
};
use gpui_component::{ActiveTheme, Icon, IconName, StyledExt as _, tooltip::Tooltip, v_flex};
use pulse_data::{UserOverrides, album_override_key, artist_override_key};
use pulse_model::{Album, AlbumArtists, Artist, ArtistId, ArtworkId, Song, SongId, ThumbnailSize};

use crate::artwork_prefetch;
use crate::library::PulseLibrary;

pub const GRID_MIN_CELL_WIDTH: f32 = 140.;
pub const GRID_LABEL_LINES: usize = 2;
const GRID_LABEL_LINES_F32: f32 = 2.;
pub const GRID_ROW_GAP: f32 = 12.;
pub const GRID_CARD_GAP: f32 = 8.;
pub const GRID_TITLE_LINE_HEIGHT: f32 = 22.;
pub const GRID_SUBTITLE_LINE_HEIGHT: f32 = 20.;
pub const GRID_GAP: f32 = 16.;
const SIDEBAR_WIDTH: f32 = 255.;
const PAGE_HORIZONTAL_INSET: f32 = 48.;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLayout {
    pub columns: usize,
    pub cell_width: f32,
    pub thumb_size: f32,
    pub row_height: f32,
}

impl GridLayout {
    #[must_use]
    pub fn for_content_width(content_width: f32) -> Self {
        if content_width <= 0. {
            return Self::with_columns(1, GRID_MIN_CELL_WIDTH);
        }

        let columns = columns_for_width(content_width);
        let cell_width = cell_width_for(content_width, columns);
        Self::with_columns(columns, cell_width)
    }

    #[must_use]
    pub fn for_window(window: &Window) -> Self {
        let viewport_width = window.viewport_size().width.as_f32();
        let content_width = (viewport_width - SIDEBAR_WIDTH).max(0.);
        let content_width = (content_width - PAGE_HORIZONTAL_INSET).max(0.);
        Self::for_content_width(content_width)
    }

    #[must_use]
    pub const fn with_columns(columns: usize, cell_width: f32) -> Self {
        let thumb_size = cell_width;
        let row_height = thumb_size
            + 2.
            + GRID_CARD_GAP * 2.
            + GRID_TITLE_LINE_HEIGHT * GRID_LABEL_LINES_F32
            + GRID_SUBTITLE_LINE_HEIGHT * GRID_LABEL_LINES_F32;

        Self {
            columns,
            cell_width,
            thumb_size,
            row_height,
        }
    }
}

fn columns_for_width(content_width: f32) -> usize {
    let mut columns = 1_usize;
    while columns < 512 {
        let next = columns.saturating_add(1);
        let cell = cell_width_for(content_width, next);
        if cell < GRID_MIN_CELL_WIDTH {
            break;
        }
        columns = next;
    }
    columns.max(1)
}

fn cell_width_for(content_width: f32, columns: usize) -> f32 {
    let gaps = usize_to_f32(columns.saturating_sub(1));
    let cols = usize_to_f32(columns);
    (content_width - GRID_GAP.mul_add(gaps, 0.)) / cols
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn usize_to_f32(value: usize) -> f32 {
    u32::try_from(value).map_or(u32::MAX, |value| value) as f32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CatalogFingerprint {
    pub albums: usize,
    pub songs: usize,
    pub artists: usize,
}

#[must_use]
pub fn catalog_fingerprint(cx: &gpui::App) -> CatalogFingerprint {
    let store = cx.global::<PulseLibrary>().inner().store();
    CatalogFingerprint {
        albums: store.albums().len(),
        songs: store.songs().len(),
        artists: store.artists().len(),
    }
}

#[must_use]
pub fn grid_item_sizes(layout: GridLayout, item_count: usize) -> Rc<Vec<Size<Pixels>>> {
    let row_count = item_count.div_ceil(layout.columns);
    Rc::new(vec![size(px(0.), px(layout.row_height)); row_count])
}

#[derive(Clone, Debug)]
pub struct AlbumDisplay {
    pub album_id: pulse_model::AlbumId,
    pub title: SharedString,
    pub artists: SharedString,
    pub year: Option<u16>,
    pub duration_ms: u64,
    pub artwork: Option<PathBuf>,
    pub genres: Vec<String>,
    pub tracks: Vec<TrackRow>,
}

#[derive(Clone, Debug)]
pub struct TrackRow {
    pub id: SongId,
    pub title: SharedString,
    pub number_label: SharedString,
    pub duration: SharedString,
    pub disc_number: Option<u16>,
    #[allow(dead_code)]
    pub track_number: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct GridItem {
    pub album_id: Option<pulse_model::AlbumId>,
    pub title: SharedString,
    pub subtitle: SharedString,
    pub thumbnail: Option<PathBuf>,
    pub detail_artwork: Option<PathBuf>,
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
        .child(
            div()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .overflow_hidden()
                .px_6()
                .pb_6()
                .child(body),
        )
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

pub fn artwork_path(
    store: &pulse_library::LibraryStore,
    artwork_id: Option<ArtworkId>,
    size: ThumbnailSize,
) -> Option<PathBuf> {
    artwork_id.and_then(|id| {
        store
            .thumbnail_path(id, size)
            .map(std::path::Path::to_path_buf)
    })
}

pub fn album_thumbnail_path(
    store: &pulse_library::LibraryStore,
    artwork_id: Option<ArtworkId>,
) -> Option<PathBuf> {
    artwork_path(store, artwork_id, ThumbnailSize::Medium)
}

pub fn album_detail_artwork_path(
    store: &pulse_library::LibraryStore,
    artwork_id: Option<ArtworkId>,
) -> Option<PathBuf> {
    artwork_path(store, artwork_id, ThumbnailSize::Large)
        .or_else(|| artwork_path(store, artwork_id, ThumbnailSize::Medium))
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
        .find(|album| album.artwork_id.is_some() && album_includes_artist(album, artist_id))
        .and_then(|album| album_thumbnail_path(store, album.artwork_id))
}

pub fn collect_album_items(cx: &gpui::App) -> Vec<GridItem> {
    let library = cx.global::<PulseLibrary>().inner();
    let store = library.store();
    let artists = store.artists();

    let mut albums: Vec<&Album> = store.albums().values().collect();
    albums.sort_by(|left, right| {
        left.title.cmp(&right.title).then_with(|| {
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
                album_id: Some(album.id),
                title: title.to_string().into(),
                subtitle: artist_label.into(),
                thumbnail,
                detail_artwork: album_detail_artwork_path(store, album.artwork_id),
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
                album_id: None,
                title: title.to_string().into(),
                subtitle: SharedString::default(),
                thumbnail,
                detail_artwork: None,
            })
        })
        .collect()
}

pub fn format_duration_ms(duration_ms: u32) -> String {
    let total_seconds = duration_ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

#[must_use]
pub fn format_album_duration_ms(total_ms: u64) -> String {
    let total_seconds = total_ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 {
        if minutes > 0 {
            format!("{hours} hr {minutes} min")
        } else {
            format!("{hours} hr")
        }
    } else if minutes > 0 {
        format!("{minutes} min")
    } else {
        let seconds = total_seconds % 60;
        format!("{seconds} sec")
    }
}

pub fn resolve_album_display(
    cx: &gpui::App,
    album_id: pulse_model::AlbumId,
) -> Option<AlbumDisplay> {
    use std::collections::BTreeSet;

    let library = cx.global::<PulseLibrary>().inner();
    let store = library.store();
    let album = store.albums().get(&album_id)?;
    let artists = store.artists();

    let artist_label = format_album_artists(artists, &album.album_artists);
    let override_key = album_override_key(&album.title, &artist_label);
    let user_override = cx.global::<DataOverrides>().album(&override_key);
    let paths = cx.global::<DataPaths>();

    let title = user_override
        .and_then(|entry| entry.title.as_deref())
        .unwrap_or(&album.title);

    let artwork = user_override
        .and_then(|entry| entry.artwork.as_deref())
        .and_then(|path| UserOverrides::resolve_artwork(paths, Some(path)))
        .or_else(|| album_detail_artwork_path(store, album.artwork_id));

    let mut songs: Vec<&Song> = store
        .songs()
        .values()
        .filter(|song| song.album_id == Some(album_id))
        .collect();
    songs.sort_by(|left, right| compare_album_songs(left, right));

    let mut genres: BTreeSet<String> = album.metadata.genres.iter().cloned().collect();
    for song in &songs {
        for genre in &song.metadata.genres {
            genres.insert(genre.clone());
        }
    }

    let total_duration_ms = songs
        .iter()
        .fold(0_u64, |total, song| total.saturating_add(u64::from(song.duration_ms)));

    let tracks = songs
        .into_iter()
        .map(|song| track_row_from_song(song, store, artists))
        .collect();

    Some(AlbumDisplay {
        album_id,
        title: title.to_string().into(),
        artists: artist_label.into(),
        year: album.year,
        duration_ms: total_duration_ms,
        artwork,
        genres: genres.into_iter().collect(),
        tracks,
    })
}

fn compare_album_songs(left: &Song, right: &Song) -> std::cmp::Ordering {
    left.disc_number
        .cmp(&right.disc_number)
        .then_with(|| left.track_number.cmp(&right.track_number))
        .then_with(|| left.title.cmp(&right.title))
}

fn track_row_from_song(
    song: &Song,
    _store: &pulse_library::LibraryStore,
    _artists: &HashMap<ArtistId, Artist>,
) -> TrackRow {
    let number_label = song
        .track_number
        .map_or_else(|| "-".to_string(), |number| number.to_string());

    TrackRow {
        id: song.id,
        title: song.title.clone().into(),
        number_label: number_label.into(),
        duration: format_duration_ms(song.duration_ms).into(),
        disc_number: song.disc_number,
        track_number: song.track_number,
    }
}

pub struct MediaCardParams {
    pub grid_id: &'static str,
    pub row_ix: usize,
    pub col_ix: usize,
    pub layout: GridLayout,
    pub items: std::sync::Arc<[GridItem]>,
    pub on_album_open: Option<gpui::Entity<crate::components::pulse::Pulse>>,
}

pub fn media_card(item: &GridItem, params: MediaCardParams, cx: &gpui::App) -> impl IntoElement {
    let theme = cx.theme();
    let cell_ix = params
        .row_ix
        .saturating_mul(params.layout.columns)
        .saturating_add(params.col_ix);
    let layout = params.layout;
    let items = params.items;
    let grid_id = params.grid_id;
    let on_album_open = params.on_album_open;

    let card = v_flex()
        .w(px(layout.cell_width))
        .flex_shrink_0()
        .gap(px(GRID_CARD_GAP))
        .child(
            div()
                .id((grid_id, cell_ix))
                .w(px(layout.thumb_size))
                .h(px(layout.thumb_size))
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
        });

    let card_id = (grid_id, cell_ix);

    let attach_hover = |element: gpui::Stateful<gpui::Div>, items: std::sync::Arc<[GridItem]>| {
        element.on_hover(move |hovered, _, cx| {
            if *hovered {
                artwork_prefetch::prefetch_grid_neighbors(cx, cell_ix, &items, layout);
            }
        })
    };

    if let (Some(pulse), Some(album_id)) = (on_album_open, item.album_id) {
        attach_hover(div().id(card_id).cursor_pointer(), items)
            .on_click(move |_, _, cx| {
                pulse.update(cx, |pulse, cx| {
                    pulse.open_album(album_id, cx);
                });
            })
            .child(card)
    } else {
        attach_hover(div().id(card_id), items).child(card)
    }
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

#[cfg(test)]
mod layout_tests {
    use super::*;

    fn row_width_for(layout: GridLayout) -> f32 {
        let gaps = usize_to_f32(layout.columns.saturating_sub(1));
        layout
            .cell_width
            .mul_add(usize_to_f32(layout.columns), GRID_GAP.mul_add(gaps, 0.))
    }

    #[test]
    fn fills_wide_containers() {
        let layout = GridLayout::for_content_width(900.);
        assert_eq!(layout.columns, 5);
        assert!((layout.cell_width - 167.2).abs() < 0.1);
    }

    #[test]
    fn adds_columns_in_narrow_containers() {
        let layout = GridLayout::for_content_width(480.);
        assert_eq!(layout.columns, 3);
    }

    #[test]
    fn never_exceeds_available_row_width() {
        let content_width = 640.;
        let layout = GridLayout::for_content_width(content_width);
        assert!(row_width_for(layout) <= content_width + 0.5);
    }

    #[test]
    fn formats_album_duration_minutes() {
        assert_eq!(format_album_duration_ms(2_520_000), "42 min");
    }

    #[test]
    fn formats_album_duration_hours() {
        assert_eq!(format_album_duration_ms(4_800_000), "1 hr 20 min");
    }

    #[test]
    fn formats_album_duration_hours_only() {
        assert_eq!(format_album_duration_ms(3_600_000), "1 hr");
    }
}
