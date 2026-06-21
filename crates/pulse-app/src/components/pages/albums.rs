use std::sync::Arc;

use gpui::{Context, Render, Window};
use gpui_component::VirtualListScrollHandle;

use super::common::{collect_album_items, page_shell, GridLayout};
use super::grid::virtual_thumbnail_grid;

pub struct AlbumsPage {
    scroll_handle: VirtualListScrollHandle,
}

impl AlbumsPage {
    #[must_use]
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
}

impl Render for AlbumsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let layout = GridLayout::for_window(window);
        let items: Arc<[super::common::GridItem]> = collect_album_items(cx).into();

        page_shell(
            "Albums",
            virtual_thumbnail_grid(
                cx.entity(),
                "albums-grid",
                items,
                layout,
                &self.scroll_handle,
                "No albums in your library yet.",
                cx,
            ),
        )
    }
}
