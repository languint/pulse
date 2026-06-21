use std::path::Path;

use gpui::{App, ImgResourceLoader, Resource};

use crate::components::pages::{GridItem, GridLayout};
use crate::config::PulseConfig;

const PREFETCH_ROW_RADIUS: usize = 2;
const PREFETCH_COL_RADIUS: usize = 3;

pub fn prefetch_grid_neighbors(
    cx: &mut App,
    cell_ix: usize,
    items: &[GridItem],
    layout: GridLayout,
) {
    if !cx
        .global::<PulseConfig>()
        .interface
        .aggressively_prefetch_artwork
    {
        return;
    }

    for index in neighbor_cell_indices(cell_ix, items.len(), layout.columns) {
        let Some(item) = items.get(index) else {
            continue;
        };

        if let Some(path) = item.thumbnail.as_deref() {
            prefetch_image_path(cx, path);
        }

        if let Some(path) = item.detail_artwork.as_deref() {
            prefetch_image_path(cx, path);
        }
    }
}

fn prefetch_image_path(cx: &mut App, path: &Path) {
    if !path.is_file() {
        return;
    }

    let resource: Resource = path.to_path_buf().into();
    let (_task, _is_first) = cx.fetch_asset::<ImgResourceLoader>(&resource);
}

fn neighbor_cell_indices(cell_ix: usize, item_count: usize, columns: usize) -> Vec<usize> {
    if item_count == 0 || columns == 0 {
        return Vec::new();
    }

    let row = cell_ix.div_euclid(columns);
    let col = cell_ix.rem_euclid(columns);
    let row_count = item_count.div_ceil(columns);
    let min_row = row.saturating_sub(PREFETCH_ROW_RADIUS);
    let max_row = row
        .saturating_add(PREFETCH_ROW_RADIUS)
        .min(row_count.saturating_sub(1));

    let mut indices = Vec::new();

    for row_ix in min_row..=max_row {
        let row_start = row_ix.saturating_mul(columns);
        let row_end = row_start.saturating_add(columns).min(item_count);

        for index in row_start..row_end {
            if row_ix == row {
                let column = index.rem_euclid(columns);
                if col.abs_diff(column) <= PREFETCH_COL_RADIUS {
                    indices.push(index);
                }
            } else {
                indices.push(index);
            }
        }
    }

    indices.sort_unstable();
    indices.dedup();
    indices
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn item_at(index: usize) -> GridItem {
        GridItem {
            album_id: None,
            artist_id: None,
            title: format!("Item {index}").into(),
            subtitle: gpui::SharedString::default(),
            thumbnail: None,
            detail_artwork: None,
        }
    }

    #[test]
    fn neighbor_indices_include_adjacent_rows() {
        let items: Arc<[GridItem]> = (0..30).map(item_at).collect();
        let layout = GridLayout::with_columns(5, 160.);
        let neighbors = neighbor_cell_indices(12, items.len(), layout.columns);

        assert!(neighbors.contains(&12));
        assert!(neighbors.contains(&7));
        assert!(neighbors.contains(&17));
    }

    #[test]
    fn neighbor_indices_respect_column_radius_on_same_row() {
        let items: Arc<[GridItem]> = (0..10).map(item_at).collect();
        let layout = GridLayout::with_columns(5, 160.);
        let neighbors = neighbor_cell_indices(0, items.len(), layout.columns);

        assert!(neighbors.contains(&0));
        assert!(neighbors.contains(&3));
        assert!(!neighbors.contains(&4));
    }
}
