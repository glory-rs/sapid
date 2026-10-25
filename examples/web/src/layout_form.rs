use glory::prelude::*;
use glory::style::{FlexWrap, Length, Limit};
pub fn layout_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let padding = scope.cage(false);
        let custom = scope.bond(move || {
            if padding.get() {
                Style::new().padding(Edges::all(20.0))
            } else {
                Style::new()
            }
        });
        sapid::stack()
            .theme(theme.clone())
            .class("layout-fixture")
            .gap(18.0)
            .fill((
                sapid::container()
                    .theme(theme.clone())
                    .class("bounded-container")
                    .max_width(Limit::Px(480.0))
                    .style(custom)
                    .fill(
                        sapid::box_()
                            .theme(theme.clone())
                            .class("container-child")
                            .height(Length::Px(36.0))
                            .style(Style {
                                background: Some(theme.accent_soft),
                                ..Style::new()
                            })
                            .fill(text().content("Centered bounded content")),
                    ),
                sapid::button()
                    .theme(theme.clone())
                    .label("Toggle layout padding")
                    .on_press(move || padding.revise(|value| *value = !*value)),
                sapid::group()
                    .theme(theme.clone())
                    .class("wrapping-group")
                    .width(Length::Px(270.0))
                    .gap(10.0)
                    .wrap(FlexWrap::Wrap)
                    .fill((
                        sapid::box_()
                            .theme(theme.clone())
                            .class("wrap-one")
                            .width(Length::Px(110.0))
                            .height(Length::Px(36.0))
                            .style(Style {
                                background: Some(theme.accent_soft),
                                ..Style::new()
                            }),
                        sapid::box_()
                            .theme(theme.clone())
                            .class("wrap-two")
                            .width(Length::Px(110.0))
                            .height(Length::Px(36.0))
                            .style(Style {
                                background: Some(theme.accent_soft),
                                ..Style::new()
                            }),
                        sapid::box_()
                            .theme(theme.clone())
                            .class("wrap-three")
                            .width(Length::Px(110.0))
                            .height(Length::Px(36.0))
                            .style(Style {
                                background: Some(theme.accent_soft),
                                ..Style::new()
                            }),
                    )),
                sapid::center()
                    .theme(theme.clone())
                    .class("centered-box")
                    .height(Length::Px(80.0))
                    .fill(
                        sapid::box_()
                            .theme(theme.clone())
                            .class("center-child")
                            .width(Length::Px(70.0))
                            .height(Length::Px(28.0))
                            .style(Style {
                                background: Some(theme.accent),
                                ..Style::new()
                            }),
                    ),
                sapid::space()
                    .theme(theme.clone())
                    .class("explicit-space")
                    .height(Length::Px(24.0)),
                sapid::stack()
                    .theme(theme.clone())
                    .class("vertical-stack")
                    .gap(8.0)
                    .fill((text().content("First"), text().content("Second"))),
            ))
            .show_in(scope)
    })
}
