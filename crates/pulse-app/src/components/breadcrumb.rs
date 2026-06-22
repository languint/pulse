use gpui::{App, Entity};
use gpui_component::breadcrumb::{Breadcrumb, BreadcrumbItem};

use crate::components::{
    navigation::PulsePage,
    pages::page_back_label,
    pulse::Pulse,
};

pub fn page_breadcrumb(
    current_page: PulsePage,
    pulse: &Entity<Pulse>,
    cx: &App,
) -> impl gpui::IntoElement {
    let trail = current_page.breadcrumb_trail();
    let last_ix = trail.len().saturating_sub(1);
    let mut breadcrumb = Breadcrumb::new();

    for (index, page) in trail.into_iter().enumerate() {
        let label = page_back_label(cx, page);
        let is_last = index == last_ix;

        if is_last {
            breadcrumb = breadcrumb.child(BreadcrumbItem::new(label));
        } else {
            let pulse = pulse.clone();
            breadcrumb = breadcrumb.child(
                BreadcrumbItem::new(label).on_click(move |_, _, cx| {
                    pulse.update(cx, |pulse, cx| {
                        navigate_to_page(pulse, page, cx);
                    });
                }),
            );
        }
    }

    breadcrumb
}

fn navigate_to_page(pulse: &mut Pulse, page: PulsePage, cx: &mut gpui::Context<Pulse>) {
    match page {
        PulsePage::Albums => pulse.show_albums(cx),
        PulsePage::Artists => pulse.show_artists(cx),
        PulsePage::AlbumDetail(album_id) => pulse.open_album(album_id, cx),
        PulsePage::ArtistDetail(artist_id) => pulse.open_artist(artist_id, cx),
        PulsePage::Visualizer => pulse.show_visualizer(cx),
    }
}
