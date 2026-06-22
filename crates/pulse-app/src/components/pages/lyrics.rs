use std::time::Duration;

use gpui::{Context, InteractiveElement, ParentElement, Render, Styled, Window, div};
use gpui_component::ActiveTheme;

use crate::{
    components::lyrics_panel::{LyricsLayout, lyrics_body, lyrics_header},
    lyrics::PulseLyrics,
    player::PulsePlayer,
};

pub struct LyricsPage {
    tick: u64,
}

impl LyricsPage {
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                this.update(cx, |page, cx| {
                    PulseLyrics::sync_with_player(cx);
                    page.tick = page.tick.wrapping_add(1);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self { tick: 0 }
    }
}

impl Render for LyricsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _ = self.tick;
        let theme = cx.theme();
        let snapshot = PulsePlayer::snapshot(cx);
        let title = PulsePlayer::current_track_title(cx).unwrap_or_else(|| "Not playing".into());
        let subtitle = PulsePlayer::current_track_subtitle(cx);
        let lyrics = PulseLyrics::current(cx);

        div()
            .id("lyrics-page")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .child(lyrics_header(cx, title, subtitle, LyricsLayout::Page))
            .child(lyrics_body(
                cx,
                lyrics.as_ref(),
                snapshot.position_ms,
                LyricsLayout::Page,
            ))
    }
}
