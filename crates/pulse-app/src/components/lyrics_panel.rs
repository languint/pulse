use gpui::{
    App, IntoElement, ParentElement, SharedString, Styled, div, prelude::FluentBuilder, px,
};
use gpui_component::{ActiveTheme, StyledExt as _, scroll::ScrollableElement};
use pulse_model::{LyricLine, Lyrics};

use crate::lyrics::PulseLyrics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LyricsLayout {
    Sidebar,
    Page,
}

pub fn lyrics_body(
    cx: &App,
    lyrics: Option<&Lyrics>,
    position_ms: u64,
    layout: LyricsLayout,
) -> impl IntoElement {
    let theme = cx.theme();
    let muted = theme.muted_foreground;
    let primary = theme.primary;
    let foreground = theme.foreground;

    div()
        .flex_1()
        .min_h_0()
        .overflow_y_scrollbar()
        .child(match lyrics {
            None if PulseLyrics::loading(cx) => div()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .child(
                    div()
                        .text_sm()
                        .text_color(muted)
                        .child("Fetching lyrics…"),
                )
                .into_any_element(),
            None => div()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .child(
                    div()
                        .text_sm()
                        .text_color(muted)
                        .child(empty_lyrics_message(cx)),
                )
                .into_any_element(),
            Some(Lyrics::Plain(text)) => {
                plain_lyrics(text, foreground, layout).into_any_element()
            }
            Some(Lyrics::Synced(lines)) => synced_lyrics(
                lines,
                position_ms,
                foreground,
                primary,
                muted,
                layout,
            )
            .into_any_element(),
        })
}

fn empty_lyrics_message(cx: &App) -> &'static str {
    if PulseLyrics::auto_fetch_enabled(cx) {
        if PulseLyrics::fetch_not_found(cx) {
            "No lyrics found for this track."
        } else {
            "No lyrics available."
        }
    } else {
        "No sidecar .lrc file found. Enable online fetch in Settings."
    }
}

fn plain_lyrics(text: &str, foreground: gpui::Hsla, layout: LyricsLayout) -> impl IntoElement {
    let text_size = match layout {
        LyricsLayout::Sidebar => px(13.),
        LyricsLayout::Page => px(18.),
    };

    div()
        .px_4()
        .py_3()
        .flex()
        .flex_col()
        .gap_1()
        .children(text.lines().map(|line| {
            div()
                .text_size(text_size)
                .text_color(foreground)
                .child(line.to_string())
        }))
}

fn active_line_index(lines: &[LyricLine], position_ms: u64) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }

    const LEAD_MS: u32 = 200;
    let position = u32::try_from(position_ms)
        .unwrap_or(u32::MAX)
        .saturating_add(LEAD_MS);
    lines.iter().rposition(|line| line.start_ms <= position)
}

fn synced_lyrics(
    lines: &[LyricLine],
    position_ms: u64,
    foreground: gpui::Hsla,
    primary: gpui::Hsla,
    muted: gpui::Hsla,
    layout: LyricsLayout,
) -> impl IntoElement {
    let active = active_line_index(lines, position_ms);

    let (visible_start, visible_end) = match layout {
        LyricsLayout::Sidebar => {
            let active = active.unwrap_or(0);
            let padding = 4;
            (
                active.saturating_sub(padding),
                (active + padding + 1).min(lines.len()),
            )
        }
        LyricsLayout::Page => (0, lines.len()),
    };

    let base_size = match layout {
        LyricsLayout::Sidebar => px(13.),
        LyricsLayout::Page => px(20.),
    };
    let active_size = match layout {
        LyricsLayout::Sidebar => px(14.),
        LyricsLayout::Page => px(24.),
    };

    div()
        .px_4()
        .py_3()
        .flex()
        .flex_col()
        .gap_2()
        .children(lines[visible_start..visible_end].iter().enumerate().map(
            |(offset, line)| {
                let index = visible_start + offset;
                let is_active = active == Some(index);
                let is_near = active
                    .map(|active_index| index.abs_diff(active_index) == 1)
                    .unwrap_or(false);

                div()
                    .text_size(if is_active { active_size } else { base_size })
                    .font_weight(if is_active {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .text_color(if is_active {
                        primary
                    } else if is_near {
                        foreground
                    } else {
                        muted
                    })
                    .opacity(if is_active {
                        1.0
                    } else if is_near {
                        0.85
                    } else {
                        0.55
                    })
                    .child(line.text.clone())
            },
        ))
}

pub fn lyrics_header(
    cx: &App,
    title: SharedString,
    subtitle: Option<SharedString>,
    layout: LyricsLayout,
) -> impl IntoElement {
    let theme = cx.theme();
    let page = layout == LyricsLayout::Page;

    div()
        .flex_shrink_0()
        .px_4()
        .pt_3()
        .pb_2()
        .when(page, |this| this.pt_6().px_8())
        .border_b_1()
        .border_color(theme.border)
        .child(
            div()
                .text_sm()
                .when(page, |this| this.text_lg())
                .font_semibold()
                .overflow_hidden()
                .text_ellipsis()
                .child(title),
        )
        .when_some(subtitle, |this, subtitle| {
            this.child(
                div()
                    .text_xs()
                    .when(page, |this| this.text_sm())
                    .text_color(theme.muted_foreground)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(subtitle),
            )
        })
}
