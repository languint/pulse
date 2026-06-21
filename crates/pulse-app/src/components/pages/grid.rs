use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, ParentElement, Render, Styled, div, px, Pixels, Size};
use gpui_component::{VirtualListScrollHandle, v_virtual_list};

use crate::components::pulse::Pulse;

use super::common::{self, GridItem, GridLayout, GRID_ROW_GAP, media_card, MediaCardParams};

pub struct VirtualThumbnailGridParams<'a> {
    pub grid_id: &'static str,
    pub empty_message: &'static str,
    pub scroll_handle: &'a VirtualListScrollHandle,
    pub on_album_open: Option<Entity<Pulse>>,
    pub on_artist_open: Option<Entity<Pulse>>,
    pub item_sizes: Rc<Vec<Size<Pixels>>>,
}

pub fn virtual_thumbnail_grid<V: Render>(
    view: Entity<V>,
    items: Arc<[GridItem]>,
    layout: GridLayout,
    params: VirtualThumbnailGridParams<'_>,
    cx: &App,
) -> AnyElement {
    if items.is_empty() {
        return common::empty_state(params.empty_message, cx).into_any_element();
    }

    let grid_id = params.grid_id;
    let on_album_open = params.on_album_open;
    let on_artist_open = params.on_artist_open;

    v_virtual_list(
        view,
        grid_id,
        params.item_sizes,
        move |_, visible_range, _, cx| {
            visible_range
                .map(|row_ix| {
                    let start = row_ix.saturating_mul(layout.columns);
                    let end = start.saturating_add(layout.columns).min(items.len());
                    let row_items = items.get(start..end).unwrap_or(&[]);

                    div()
                        .flex()
                        .flex_row()
                        .items_start()
                        .w_full()
                        .gap(px(common::GRID_GAP))
                        .children(row_items.iter().enumerate().map(|(col_ix, item)| {
                            media_card(
                                item,
                                MediaCardParams {
                                    grid_id,
                                    row_ix,
                                    col_ix,
                                    layout,
                                    items: items.clone(),
                                    on_album_open: on_album_open.clone(),
                                    on_artist_open: on_artist_open.clone(),
                                },
                                cx,
                            )
                        }))
                })
                .collect()
        },
    )
    .gap(px(GRID_ROW_GAP))
    .track_scroll(params.scroll_handle)
    .into_any_element()
}
