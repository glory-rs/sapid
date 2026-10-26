use glory::prelude::*;
use sapid::{BadgeVariant, Theme};

pub fn demo(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let visible = scope.cage(true);
        let requests = scope.cage(0usize);
        sapid::stack().theme(theme.clone()).gap(24.0).fill((
            sapid::stack().theme(theme.clone()).gap(12.0).fill((
                sapid::text().theme(theme.clone()).content("Badge variants"),
                sapid::group().theme(theme.clone()).fill((
                    sapid::badge().theme(theme.clone()).content("New release"),
                    sapid::badge().theme(theme.clone()).variant(BadgeVariant::Solid).content("Published"),
                    sapid::badge().theme(theme.clone()).variant(BadgeVariant::Outline).content("Preview"),
                )),
            )),
            sapid::stack().theme(theme.clone()).gap(12.0).fill((
                sapid::text().theme(theme.clone()).content("Removable pills"),
                sapid::group().theme(theme.clone()).fill((
                    sapid::pill().theme(theme.clone()).class("local-pill").content("Rust")
                        .removable("Remove local tag"),
                    sapid::pill().theme(theme.clone()).class("controlled-pill").content("Pinned label")
                        .removable("Request controlled removal").visible(visible)
                        .on_remove(move || requests.revise(|count| *count += 1)),
                    sapid::pill().theme(theme.clone()).content("Disabled label")
                        .removable("Remove disabled tag").enabled(false),
                    sapid::pill().theme(theme.clone()).content("Read only label"),
                )),
                sapid::text().theme(theme.clone()).variant(sapid::TextVariant::Small)
                    .content(requests.map(|count| format!("The pinned label stays until its controlled source changes. Removal requests: {count}"))),
                sapid::group().theme(theme.clone()).fill(
                    sapid::button().theme(theme.clone()).size(sapid::Size::Small)
                        .variant(sapid::ButtonVariant::Secondary).label("Toggle pinned label")
                        .on_press(move || visible.revise(|current| *current = !*current)),
                ),
            )),
        )).show_in(scope)
    })
}
