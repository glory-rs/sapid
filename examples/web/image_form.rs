use glory::style::{Length, ObjectFit};
use glory::ui::{Widget, component, text};
use sapid::{Theme, avatar, avatar_group, background_image, button, image};
pub fn image_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let source = scope.cage("assets/image-pattern.png".to_string());
        let status = scope.cage("Loading".to_string());
        let fallback_status = scope.cage("Loading fallback".to_string());
        sapid::stack()
            .gap(16.0)
            .fill((
                sapid::group().gap(16.0).fill((
                    image(source)
                        .alt("Color pattern")
                        .theme(theme.clone())
                        .class("image-primary")
                        .width(Length::Px(200.0))
                        .height(Length::Px(120.0))
                        .fit(ObjectFit::Contain)
                        .on_load(move |revision| {
                            status.revise(|value| *value = format!("Loaded {revision}"))
                        })
                        .on_error(move |revision| {
                            status.revise(|value| *value = format!("Failed {revision}"))
                        }),
                    image("assets/image-invalid.png")
                        .alt("Fallback pattern")
                        .fallback_src("assets/image-fallback.png")
                        .theme(theme.clone())
                        .class("image-fallback-demo")
                        .width(Length::Px(120.0))
                        .height(Length::Px(80.0))
                        .fit(ObjectFit::Cover)
                        .on_load(move |revision| {
                            fallback_status
                                .revise(|value| *value = format!("Fallback loaded {revision}"))
                        }),
                )),
                text().class("image-status").content(status),
                text()
                    .class("image-fallback-status")
                    .content(fallback_status),
                sapid::group().gap(12.0).fill((
                    avatar("Ada Lovelace")
                        .fallback("AL")
                        .source(source)
                        .theme(theme.clone())
                        .class("avatar-resource"),
                    avatar_group("image-team")
                        .name("Image team")
                        .theme(theme.clone())
                        .max_visible(2)
                        .avatar(
                            "ada",
                            avatar("Ada Lovelace")
                                .fallback("AL")
                                .source("assets/image-pattern.png"),
                        )
                        .avatar("grace", avatar("Grace Hopper").fallback("GH"))
                        .avatar("katherine", avatar("Katherine Johnson").fallback("KJ")),
                )),
                background_image("assets/image-pattern.png")
                    .theme(theme.clone())
                    .name("Pattern background")
                    .class("image-background")
                    .width(Length::Px(300.0))
                    .height(Length::Px(140.0))
                    .fill((
                        text().content("Independent content over a real image"),
                        button()
                            .theme(theme.clone())
                            .label("Background action")
                            .on_press(move || {
                                status
                                    .revise(|value| *value = "Background action pressed".into())
                            }),
                    )),
                sapid::group().gap(8.0).fill((
                    button()
                        .theme(theme.clone())
                        .label("Break image")
                        .on_press(move || {
                            source.revise(|value| *value = "assets/image-invalid.png".into())
                        }),
                    button()
                        .theme(theme.clone())
                        .label("Restore image")
                        .on_press(move || {
                            source.revise(|value| *value = "assets/image-pattern.png".into())
                        }),
                )),
                sapid::group().gap(8.0).fill((
                    button()
                        .theme(theme.clone())
                        .label("Delayed A")
                        .on_press(move || {
                            source.revise(|value| *value = "assets/image-delayed-a.png".into())
                        }),
                    button()
                        .theme(theme.clone())
                        .label("Delayed B")
                        .on_press(move || {
                            source.revise(|value| *value = "assets/image-delayed-b.png".into())
                        }),
                    button()
                        .theme(theme.clone())
                        .label("Delayed C")
                        .on_press(move || {
                            source.revise(|value| *value = "assets/image-delayed-c.png".into())
                        }),
                )),
            ))
            .show_in(scope)
    })
}
