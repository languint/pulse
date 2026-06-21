use gpui::{
    AppContext, Entity, FocusHandle, InteractiveElement, IntoElement, MouseMoveEvent,
    ParentElement, Render, Styled, Window, div,
};
use gpui_component::{ActiveTheme, Root, TITLE_BAR_HEIGHT};

use crate::{
    actions::{ManageLibraryRoots, ToggleFullscreen},
    components::{
        library_roots_dialog::open_library_roots_dialog,
        navigation::PulsePage,
        pages::{AlbumViewerPage, AlbumsPage, ArtistsPage},
        sidebar::AppSidebar,
        toolbar::Toolbar,
    },
};

use pulse_model::AlbumId;

pub mod icon;

pub struct Pulse {
    pub focus_handle: FocusHandle,
    page: PulsePage,
    toolbar: Entity<Toolbar>,
    sidebar: Entity<AppSidebar>,
    albums_page: Entity<AlbumsPage>,
    artists_page: Entity<ArtistsPage>,
    album_viewer_page: Entity<AlbumViewerPage>,
}

impl Pulse {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let pulse = cx.entity();

        Self {
            focus_handle: cx.focus_handle(),
            page: PulsePage::Albums,
            toolbar: cx.new(Toolbar::new),
            sidebar: cx.new(|_| AppSidebar::new(pulse.clone())),
            albums_page: cx.new(|cx| AlbumsPage::new(pulse.clone(), cx)),
            artists_page: cx.new(ArtistsPage::new),
            album_viewer_page: cx.new(|cx| AlbumViewerPage::new(pulse.clone(), cx)),
        }
    }

    #[must_use]
    pub const fn page(&self) -> PulsePage {
        self.page
    }

    pub fn set_page(&mut self, page: PulsePage, cx: &mut gpui::Context<Self>) {
        if self.page == page {
            return;
        }

        self.page = page;
        cx.notify();
    }

    pub fn open_album(&mut self, album_id: AlbumId, cx: &mut gpui::Context<Self>) {
        self.page = PulsePage::AlbumDetail(album_id);
        cx.notify();
    }

    pub fn show_albums(&mut self, cx: &mut gpui::Context<Self>) {
        self.page = PulsePage::Albums;
        cx.notify();
    }
}

impl Render for Pulse {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let theme = cx.theme();
        let background = theme.background;
        let foreground = theme.foreground;
        let font_size = theme.font_size;
        let dialog_layer = Root::render_dialog_layer(window, cx);

        let main_page = match self.page {
            PulsePage::Albums => self.albums_page.clone().into_any_element(),
            PulsePage::Artists => self.artists_page.clone().into_any_element(),
            PulsePage::AlbumDetail(_) => self.album_viewer_page.clone().into_any_element(),
        };

        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(background)
            .text_color(foreground)
            .text_size(font_size)
            .child(self.toolbar.clone())
            .child(
                div()
                    .id("content")
                    .flex_1()
                    .flex()
                    .min_h_0()
                    .track_focus(&self.focus_handle)
                    .on_action(cx.listener(|_, _: &ToggleFullscreen, window, _| {
                        window.toggle_fullscreen();
                    }))
                    .on_action(cx.listener(|_, _: &ManageLibraryRoots, window, cx| {
                        open_library_roots_dialog(window, cx);
                    }))
                    .child(self.sidebar.clone())
                    .child(
                        div()
                            .id("main")
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .overflow_hidden()
                            .bg(background)
                            .child(main_page),
                    ),
            )
            .children(dialog_layer);

        // This is needed for now since there is a bug in gpui
        #[cfg(target_os = "windows")]
        {
            root = root.on_mouse_move(refresh_title_bar_hover);
        }

        root
    }
}

#[cfg(target_os = "windows")]
#[allow(clippy::arithmetic_side_effects)]
fn refresh_title_bar_hover(event: &MouseMoveEvent, window: &mut Window, _: &mut gpui::App) {
    if event.position.y > TITLE_BAR_HEIGHT {
        return;
    }

    let caption_width = TITLE_BAR_HEIGHT * 3.0;
    if event.position.x >= window.viewport_size().width - caption_width {
        window.refresh();
    }
}
