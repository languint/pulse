use std::sync::Arc;

use gpui::{Context, Render, Window};
use gpui_component::VirtualListScrollHandle;

use super::common::{
    CatalogFingerprint, GridItem, GridLayout, catalog_fingerprint, collect_artist_items,
    grid_item_sizes, page_shell,
};
use super::grid::{VirtualThumbnailGridParams, virtual_thumbnail_grid};

pub struct ArtistsPage {
    scroll_handle: VirtualListScrollHandle,
    cached_items: Arc<[GridItem]>,
    catalog_fp: CatalogFingerprint,
    cached_layout: GridLayout,
    cached_item_sizes: std::rc::Rc<Vec<gpui::Size<gpui::Pixels>>>,
}

impl ArtistsPage {
    #[must_use]
    pub fn new(_: &mut Context<Self>) -> Self {
        let layout = GridLayout::with_columns(5, 160.);
        Self {
            scroll_handle: VirtualListScrollHandle::new(),
            cached_items: Arc::from([]),
            catalog_fp: CatalogFingerprint::default(),
            cached_layout: layout,
            cached_item_sizes: grid_item_sizes(layout, 0),
        }
    }

    fn ensure_items(&mut self, cx: &gpui::App) {
        let fp = catalog_fingerprint(cx);
        if fp == self.catalog_fp {
            return;
        }

        self.cached_items = collect_artist_items(cx).into();
        self.catalog_fp = fp;
    }

    fn ensure_item_sizes(&mut self, layout: GridLayout) {
        if layout == self.cached_layout && self.cached_item_sizes.len() == self.cached_items.len().div_ceil(layout.columns) {
            return;
        }

        self.cached_layout = layout;
        self.cached_item_sizes = grid_item_sizes(layout, self.cached_items.len());
    }
}

impl Render for ArtistsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.ensure_items(cx);
        let layout = GridLayout::for_window(window);
        self.ensure_item_sizes(layout);

        page_shell(
            "Artists",
            virtual_thumbnail_grid(
                cx.entity(),
                self.cached_items.clone(),
                layout,
                VirtualThumbnailGridParams {
                    grid_id: "artists-grid",
                    empty_message: "No artists in your library yet.",
                    scroll_handle: &self.scroll_handle,
                    on_album_open: None,
                    item_sizes: self.cached_item_sizes.clone(),
                },
                cx,
            ),
        )
    }
}
