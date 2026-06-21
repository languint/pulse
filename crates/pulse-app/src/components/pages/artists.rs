use std::sync::Arc;

use gpui::{Context, Entity, Render, Window};
use gpui_component::VirtualListScrollHandle;

use crate::components::pulse::Pulse;

use super::common::{
    CatalogFingerprint, GridItem, GridLayout, catalog_fingerprint, collect_artist_items,
    grid_item_sizes, overrides_generation, page_shell,
};
use super::grid::{VirtualThumbnailGridParams, virtual_thumbnail_grid};

pub struct ArtistsPage {
    pulse: Entity<Pulse>,
    scroll_handle: VirtualListScrollHandle,
    cached_items: Arc<[GridItem]>,
    catalog_fp: CatalogFingerprint,
    overrides_gen: u32,
    cached_layout: GridLayout,
    cached_item_sizes: std::rc::Rc<Vec<gpui::Size<gpui::Pixels>>>,
}

impl ArtistsPage {
    #[must_use]
    pub fn new(pulse: Entity<Pulse>, _: &mut Context<Self>) -> Self {
        let layout = GridLayout::with_columns(5, 160.);
        Self {
            pulse,
            scroll_handle: VirtualListScrollHandle::new(),
            cached_items: Arc::from([]),
            catalog_fp: CatalogFingerprint::default(),
            overrides_gen: 0,
            cached_layout: layout,
            cached_item_sizes: grid_item_sizes(layout, 0),
        }
    }

    fn ensure_items(&mut self, cx: &gpui::App) {
        let fp = catalog_fingerprint(cx);
        let overrides_gen = overrides_generation(cx);
        if fp == self.catalog_fp && overrides_gen == self.overrides_gen {
            return;
        }

        self.cached_items = collect_artist_items(cx).into();
        self.catalog_fp = fp;
        self.overrides_gen = overrides_gen;
    }

    fn ensure_item_sizes(&mut self, layout: GridLayout) {
        if layout == self.cached_layout
            && self.cached_item_sizes.len() == self.cached_items.len().div_ceil(layout.columns)
        {
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

        let pulse = self.pulse.clone();

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
                    on_artist_open: Some(pulse),
                    item_sizes: self.cached_item_sizes.clone(),
                },
                cx,
            ),
        )
    }
}
