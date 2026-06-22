use gpui::{
    AppContext, Entity, FocusHandle, InteractiveElement, IntoElement, MouseMoveEvent,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_component::{ActiveTheme, Root, TITLE_BAR_HEIGHT};

use crate::{
    actions::{
        ManageLibraryRoots, CommandPaletteTab, OpenSettings, OpenVisualizerSettings,
        ShowOscilloscopeVisualizer, ShowSpectrumVisualizer,
        ToggleCommandPalette, ToggleFullscreen,
    },
    components::{
        breadcrumb::page_breadcrumb,
        command_palette::CommandPalette,
        library_roots_dialog::open_library_roots_dialog,
        settings_dialog::open_settings_dialog,
        visualizer_settings_dialog::{open_visualizer_settings_dialog, set_visualizer_mode},
        navigation::PulsePage,
        pages::{AlbumViewerPage, AlbumsPage, ArtistViewerPage, ArtistsPage, VisualizerPage},
        player_bar::PlayerBar,
        sidebar::AppSidebar,
        toolbar::Toolbar,
    },
};

use pulse_model::{AlbumId, ArtistId};
use pulse_data::VisualizerMode;

pub mod icon;

pub struct ActivePulse(pub Entity<Pulse>);

impl gpui::Global for ActivePulse {}

pub struct Pulse {
    pub focus_handle: FocusHandle,
    page: PulsePage,
    previous_page: Option<PulsePage>,
    toolbar: Entity<Toolbar>,
    sidebar: Entity<AppSidebar>,
    albums_page: Entity<AlbumsPage>,
    artists_page: Entity<ArtistsPage>,
    album_viewer_page: Entity<AlbumViewerPage>,
    artist_viewer_page: Entity<ArtistViewerPage>,
    visualizer_page: Entity<VisualizerPage>,
    player_bar: Entity<PlayerBar>,
    command_palette: Entity<CommandPalette>,
}

impl Pulse {
    pub fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        let pulse = cx.entity();

        let focus_handle = cx.focus_handle();

        Self {
            focus_handle: focus_handle.clone(),
            page: PulsePage::Albums,
            previous_page: None,
            toolbar: cx.new(Toolbar::new),
            sidebar: cx.new(|_| AppSidebar::new(pulse.clone())),
            albums_page: cx.new(|cx| AlbumsPage::new(pulse.clone(), cx)),
            artists_page: cx.new(|cx| ArtistsPage::new(pulse.clone(), cx)),
            album_viewer_page: cx.new(|cx| AlbumViewerPage::new(pulse.clone(), cx)),
            artist_viewer_page: cx.new(|cx| ArtistViewerPage::new(pulse.clone(), cx)),
            visualizer_page: cx.new(VisualizerPage::new),
            player_bar: cx.new(|cx| PlayerBar::new(pulse.clone(), cx)),
            command_palette: cx.new(|cx| CommandPalette::new(window, focus_handle, cx)),
        }
    }

    #[must_use]
    pub const fn page(&self) -> PulsePage {
        self.page
    }

    #[must_use]
    pub const fn previous_page(&self) -> Option<PulsePage> {
        self.previous_page
    }

    #[must_use]
    pub fn back_target(&self) -> PulsePage {
        self.previous_page
            .unwrap_or_else(|| self.page.section_fallback())
    }

    pub fn set_page(&mut self, page: PulsePage, cx: &mut gpui::Context<Self>) {
        if self.page == page {
            return;
        }

        self.previous_page = None;
        self.page = page;
        cx.notify();
    }

    pub fn open_album(&mut self, album_id: AlbumId, cx: &mut gpui::Context<Self>) {
        self.navigate_to(PulsePage::AlbumDetail(album_id), cx);
    }

    pub fn open_artist(&mut self, artist_id: ArtistId, cx: &mut gpui::Context<Self>) {
        self.navigate_to(PulsePage::ArtistDetail(artist_id), cx);
    }

    pub fn go_back(&mut self, cx: &mut gpui::Context<Self>) {
        let destination = self
            .previous_page
            .take()
            .unwrap_or_else(|| self.page.section_fallback());

        if self.page == destination {
            return;
        }

        self.page = destination;
        cx.notify();
    }

    pub fn show_albums(&mut self, cx: &mut gpui::Context<Self>) {
        self.set_page(PulsePage::Albums, cx);
    }

    pub fn show_artists(&mut self, cx: &mut gpui::Context<Self>) {
        self.set_page(PulsePage::Artists, cx);
    }

    pub fn open_visualizer(&mut self, cx: &mut gpui::Context<Self>) {
        if self.page.is_visualizer() {
            return;
        }

        self.previous_page = Some(self.page);
        self.page = PulsePage::Visualizer;
        cx.notify();
    }

    pub fn show_visualizer_mode(&mut self, mode: VisualizerMode, cx: &mut gpui::Context<Self>) {
        set_visualizer_mode(cx, mode);
        self.open_visualizer(cx);
    }

    pub fn show_visualizer(&mut self, cx: &mut gpui::Context<Self>) {
        self.set_page(PulsePage::Visualizer, cx);
    }

    pub fn toggle_visualizer(&mut self, cx: &mut gpui::Context<Self>) {
        if self.page.is_visualizer() {
            let destination = self.previous_page.take().unwrap_or(PulsePage::Albums);
            self.page = destination;
        } else {
            self.previous_page = Some(self.page);
            self.page = PulsePage::Visualizer;
        }
        cx.notify();
    }

    #[must_use]
    pub const fn is_visualizer(&self) -> bool {
        self.page.is_visualizer()
    }

    fn navigate_to(&mut self, page: PulsePage, cx: &mut gpui::Context<Self>) {
        if self.page == page {
            return;
        }

        self.previous_page = Some(self.page);
        self.page = page;
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
        let pulse = cx.entity();

        let main_page = match self.page {
            PulsePage::Albums => self.albums_page.clone().into_any_element(),
            PulsePage::Artists => self.artists_page.clone().into_any_element(),
            PulsePage::Visualizer => self.visualizer_page.clone().into_any_element(),
            PulsePage::AlbumDetail(_) => self.album_viewer_page.clone().into_any_element(),
            PulsePage::ArtistDetail(_) => self.artist_viewer_page.clone().into_any_element(),
        };
        let show_sidebar = !self.page.is_visualizer();

        let mut root = div()
            .relative()
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
                    .on_action(cx.listener(|_, _: &OpenSettings, window, cx| {
                        open_settings_dialog(window, cx);
                    }))
                    .on_action(cx.listener(|_, _: &OpenVisualizerSettings, window, cx| {
                        open_visualizer_settings_dialog(window, cx);
                    }))
                    .on_action(cx.listener(|this, _: &ShowSpectrumVisualizer, _, cx| {
                        this.show_visualizer_mode(VisualizerMode::Spectrum, cx);
                    }))
                    .on_action(cx.listener(|this, _: &ShowOscilloscopeVisualizer, _, cx| {
                        this.show_visualizer_mode(VisualizerMode::Oscilloscope, cx);
                    }))
                    .on_action(cx.listener(|this, _: &ToggleCommandPalette, window, cx| {
                        this.command_palette.update(cx, |palette, cx| {
                            palette.toggle(window, cx);
                        });
                    }))
                    .on_action(cx.listener(|this, _: &CommandPaletteTab, window, cx| {
                        this.command_palette.update(cx, |palette, cx| {
                            if !palette.is_open() {
                                palette.handle_tab(window, cx);
                            }
                        });
                    }))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .min_w_0()
                            .when(show_sidebar, |this| this.child(self.sidebar.clone()))
                            .child(
                                div()
                                    .id("main")
                                    .flex_1()
                                    .min_w_0()
                                    .min_h_0()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .bg(background)
                                    .when(show_sidebar, |this| {
                                        this.child(
                                            div()
                                                .flex_shrink_0()
                                                .px_6()
                                                .pt_4()
                                                .pb_2()
                                                .child(page_breadcrumb(self.page, &pulse, cx)),
                                        )
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_h_0()
                                            .min_w_0()
                                            .overflow_hidden()
                                            .child(main_page),
                                    ),
                            ),
                    ),
            )
            .child(self.player_bar.clone())
            .children(dialog_layer)
            .child(self.command_palette.clone());

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
