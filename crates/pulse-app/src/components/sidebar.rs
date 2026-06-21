use gpui::{Context, Entity, ParentElement, Render, Window};
use gpui_component::{
    IconName,
    sidebar::{
        Sidebar as SidebarPanel, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
    },
};

use crate::components::{
    navigation::PulsePage,
    pulse::Pulse,
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
            .header(SidebarHeader::new().child("Pulse"))
            .child(
                SidebarGroup::new("Library").child(
                    SidebarMenu::new()
                        .child(
                            SidebarMenuItem::new(PulsePage::Albums.label())
                                .icon(IconName::GalleryVerticalEnd)
                                .active(page == PulsePage::Albums)
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
                                .active(page == PulsePage::Artists)
                                .on_click({
                                    move |_, _, cx| {
                                        pulse.update(cx, |pulse, cx| {
                                            pulse.set_page(PulsePage::Artists, cx);
                                        });
                                    }
                                }),
                        ),
                ),
            )
    }
}
