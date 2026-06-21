use std::sync::Arc;

use gpui::{Context, Render, Window};
use gpui_component::VirtualListScrollHandle;

use super::common::{collect_artist_items, page_shell, GridLayout};
use super::grid::virtual_thumbnail_grid;

pub struct ArtistsPage {
    scroll_handle: VirtualListScrollHandle,
}

impl ArtistsPage {
    #[must_use]
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
}

impl Render for ArtistsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let layout = GridLayout::for_window(window);
        let items: Arc<[super::common::GridItem]> = collect_artist_items(cx).into();

        page_shell(
            "Artists",
            virtual_thumbnail_grid(
                cx.entity(),
                "artists-grid",
                items,
                layout,
                &self.scroll_handle,
                "No artists in your library yet.",
                cx,
            ),
        )
    }
}
