use gpui::{Context, Entity, ParentElement, Render, Styled, Window, div, px};
use gpui_component::{
    IconName, StyledExt as _, h_flex,
    sidebar::{Sidebar as SidebarPanel, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};

use crate::components::{
    navigation::PulsePage,
    pulse::{Pulse, icon::pulse_logo},
};

pub struct AppSidebar {
    pulse: Entity<Pulse>,
}

impl AppSidebar {
    #[must_use]
    pub const fn new(pulse: Entity<Pulse>) -> Self {
        Self { pulse }
    }
}

impl Render for AppSidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let page = self.pulse.read(cx).page();
        let pulse = self.pulse.clone();

        SidebarPanel::new("pulse-sidebar")
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(pulse_logo(px(24.), cx))
                        .child(div().font_semibold().child("Pulse")),
                ),
            )
            .child(
                SidebarGroup::new("Library").child(
                    SidebarMenu::new()
                        .child(
                            SidebarMenuItem::new(PulsePage::Albums.label())
                                .icon(IconName::GalleryVerticalEnd)
                                .active(page.is_albums_section())
                                .on_click({
                                    let pulse = pulse.clone();
                                    move |_, _, cx| {
                                        pulse.update(cx, |pulse, cx| {
                                            pulse.set_page(PulsePage::Albums, cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            SidebarMenuItem::new(PulsePage::Artists.label())
                                .icon(IconName::User)
                                .active(page.is_artists_section())
                                .on_click({
                                    let pulse = pulse.clone();
                                    move |_, _, cx| {
                                        pulse.update(cx, |pulse, cx| {
                                            pulse.show_artists(cx);
                                        });
                                    }
                                }),
                        ),
                ),
            )
            .child(
                SidebarGroup::new("Now Playing").child(
                    SidebarMenu::new().child(
                        SidebarMenuItem::new(PulsePage::Lyrics.label())
                            .icon(crate::icons::PulseIcon::ScrollText)
                            .active(page.is_lyrics())
                            .on_click({
                                let pulse = pulse.clone();
                                move |_, _, cx| {
                                    pulse.update(cx, |pulse, cx| {
                                        pulse.show_lyrics(cx);
                                    });
                                }
                            }),
                    ),
                ),
            )
    }
}
