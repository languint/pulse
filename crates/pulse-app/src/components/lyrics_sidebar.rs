use std::time::Duration;

use gpui::{Context, InteractiveElement, ParentElement, Render, Styled, Window, div, px};
use gpui_component::ActiveTheme;

use crate::{
    components::lyrics_panel::{LyricsLayout, lyrics_body, lyrics_header},
    lyrics::PulseLyrics,
    player::PulsePlayer,
};

const SIDEBAR_WIDTH: f32 = 320.;

pub struct LyricsSidebar {
    tick: u64,
}

impl LyricsSidebar {
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                this.update(cx, |panel, cx| {
                    PulseLyrics::sync_with_player(cx);
                    panel.tick = panel.tick.wrapping_add(1);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self { tick: 0 }
    }
}

impl Render for LyricsSidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _ = self.tick;
        let theme = cx.theme();
        let snapshot = PulsePlayer::snapshot(cx);
        let title = PulsePlayer::current_track_title(cx).unwrap_or_else(|| "Not playing".into());
        let subtitle = PulsePlayer::current_track_subtitle(cx);
        let lyrics = PulseLyrics::current(cx);

        div()
            .id("lyrics-sidebar")
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(lyrics_header(cx, title, subtitle, LyricsLayout::Sidebar))
            .child(lyrics_body(
                cx,
                lyrics.as_ref(),
                snapshot.position_ms,
                LyricsLayout::Sidebar,
            ))
    }
}
