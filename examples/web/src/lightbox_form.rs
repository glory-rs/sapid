use glory::prelude::*;
use sapid::{LightboxImage, LightboxView, lightbox, lightbox_image};
pub fn images() -> Vec<LightboxImage> {
    vec![
        lightbox_image(
            "one",
            crate::assets::media("image-pattern.png"),
            "Color pattern",
        )
        .caption("First color pattern"),
        lightbox_image(
            "two",
            crate::assets::media("image-fallback.png"),
            "Fallback pattern",
        )
        .caption("Second color pattern"),
        lightbox_image(
            "broken",
            crate::assets::media("image-invalid.png"),
            "Unavailable pattern",
        )
        .caption("Unavailable image with a text alternative"),
    ]
}
pub fn lightbox_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let images = scope.cage(images());
        let view = scope.cage(LightboxView::default());
        let open = scope.cage(false);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let mounted = scope.cage(true);
        let proposals = scope.cage(0usize);
        let root = scope.cage(None);
        glory::ui::view()
            .class("lightbox-fixture")
            .style(
                Style::new()
                    .display(glory::style::Display::Flex)
                    .flex_direction(glory::style::FlexDirection::Column)
                    .gap(12.),
            )
            .node_ref(root)
            .show_in(scope)?;
        let area = scope.child_in(root.get().unwrap())?;
        sapid::checkbox()
            .label("Accept lightbox proposals")
            .checked(accept)
            .on_checked_change(move |v| accept.revise(|o| *o = v))
            .theme(theme.clone())
            .show_in(&area)?;
        sapid::checkbox()
            .label("Enable lightbox controls")
            .checked(enabled)
            .on_checked_change(move |v| enabled.revise(|o| *o = v))
            .theme(theme.clone())
            .show_in(&area)?;
        sapid::button()
            .label("Replace lightbox metadata")
            .theme(theme.clone())
            .on_press(move || images.revise(|v| v[0].caption = "Updated first caption".into()))
            .show_in(&area)?;
        sapid::button()
            .label("Toggle lightbox owner")
            .theme(theme.clone())
            .on_press(move || mounted.revise(|v| *v = !*v))
            .show_in(&area)?;
        let palette = theme.clone();
        glory::ui::when(mounted, move || {
            lightbox("fixture")
                .class("lightbox-primary")
                .theme(palette.clone())
                .images(images)
                .view(view)
                .open(open)
                .enabled(enabled)
                .on_view_change(move |v| {
                    proposals.revise(|n| *n += 1);
                    if accept.get() {
                        view.revise(|o| *o = v);
                    }
                })
                .on_open_change(move |v| {
                    proposals.revise(|n| *n += 1);
                    open.revise(|o| *o = v);
                })
        })
        .show_in(&area)?;
        glory::ui::text()
            .class("lightbox-proposals")
            .content(scope.bond(move || format!("Proposals: {}", proposals.get())))
            .show_in(&area)?;
        scope.retain(area)?;
        Ok(())
    })
}
