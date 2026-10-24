use glory::ui::{Widget, widget};
use sapid::{AvatarShape, Size, Theme, avatar, avatar_group, button};

pub fn avatar_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let name = scope.cage("Ada Lovelace".to_string());
        let fallback = scope.cage("AL".to_string());
        sapid::stack()
            .gap(16.0)
            .fill((
                sapid::group().gap(12.0).fill((
                    avatar(name)
                        .fallback(fallback)
                        .theme(theme.clone())
                        .class("avatar-dynamic"),
                    avatar("Grace Hopper")
                        .fallback("GH")
                        .theme(theme.clone())
                        .shape(AvatarShape::Square)
                        .size(Size::Large)
                        .class("avatar-square"),
                    avatar("Unknown person")
                        .theme(theme.clone())
                        .size(Size::Small)
                        .class("avatar-default"),
                )),
                avatar_group("project-team")
                    .theme(theme.clone())
                    .name("Project team")
                    .max_visible(2)
                    .class("avatar-team")
                    .avatars([
                        ("ada".into(), avatar("Ada Lovelace").fallback("AL")),
                        ("grace".into(), avatar("Grace Hopper").fallback("GH")),
                        (
                            "margaret".into(),
                            avatar("Margaret Hamilton").fallback("MH"),
                        ),
                        (
                            "katherine".into(),
                            avatar("Katherine Johnson").fallback("KJ"),
                        ),
                    ]),
                avatar("Custom text avatar")
                    .fallback("AB")
                    .unstyled()
                    .class("avatar-unstyled"),
                button()
                    .theme(theme.clone())
                    .label("Change avatar label")
                    .on_press(move || {
                        name.revise(|value| *value = "Katherine Johnson".into());
                        fallback.revise(|value| *value = "KJ".into());
                    }),
            ))
            .show_in(scope)
    })
}
