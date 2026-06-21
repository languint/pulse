use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, ParentElement, Render, Styled, div, px, size};
use gpui_component::{VirtualListScrollHandle, v_virtual_list};

use super::common::{self, GridItem, GridLayout, GRID_ROW_GAP, media_card};

pub fn virtual_thumbnail_grid<V: Render>(
    view: Entity<V>,
    grid_id: &'static str,
    items: Arc<[GridItem]>,
    layout: GridLayout,
    scroll_handle: &VirtualListScrollHandle,
    empty_message: &'static str,
    cx: &App,
) -> AnyElement {
    if items.is_empty() {
        return common::empty_state(empty_message, cx).into_any_element();
    }

    let row_count = items.len().div_ceil(layout.columns);
    let row_height = layout.row_height;
    let item_sizes = Rc::new(vec![size(px(0.), px(row_height)); row_count]);

    v_virtual_list(
        view,
        grid_id,
        item_sizes,
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
                            media_card(grid_id, row_ix, col_ix, layout, item, cx)
                        }))
                })
                .collect()
        },
    )
    .gap(px(GRID_ROW_GAP))
    .track_scroll(scroll_handle)
    .into_any_element()
}
