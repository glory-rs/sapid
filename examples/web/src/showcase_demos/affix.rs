use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::{Event, EventKind};
    use glory::style::{Length, Style};
    use sapid::{
        AffixHorizontal, AffixOffset, AffixVertical, ShellLayout, Theme, affix, affix_container,
    };

    pub struct State {
        pub offset: Cage<Option<AffixOffset>>,
        pub visible: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let layout = ShellLayout::new(scope, 1000.0, 700.0).unwrap();
        let driver = layout.clone();
        let offset = scope.cage(None::<AffixOffset>);
        let visible = scope.cage(true);
        let mobile = layout.mobile();
        let position = scope.bond(move || {
            offset.get().unwrap_or_else(|| {
                let inset = if mobile.get() { 8.0 } else { 24.0 };
                AffixOffset {
                    horizontal: AffixHorizontal::Right(inset),
                    vertical: AffixVertical::Bottom(inset),
                }
            })
        });
        let status =
            scope.bond(move || format!("mobile={};visible={}", mobile.get(), visible.get()));
        let fixed = affix("viewport")
            .theme(theme.clone())
            .name("Viewport affix")
            .class("viewport-affix")
            .offset(position)
            .visible(visible)
            .fill(
                glory::ui::input()
                    .name("Floating draft")
                    .class("affix-draft")
                    .value("Keep floating draft"),
            );
        let container_theme = theme.clone();
        let container = affix_container("container")
            .theme(theme.clone())
            .name("Affix container")
            .class("demo-affix-container")
            .style(
                Style::new()
                    .width(Length::Px(500.0))
                    .height(Length::Px(240.0)),
            )
            .content(move |context| {
                glory::ui::view()
                    .style(Style::new().height(Length::Px(1000.0)))
                    .fill((
                        glory::ui::view().style(Style::new().height(Length::Px(80.0))),
                        affix("sticky")
                            .theme(container_theme.clone())
                            .name("Sticky affix")
                            .class("sticky-affix")
                            .sticky(context)
                            .offset(AffixOffset {
                                horizontal: AffixHorizontal::Left(12.0),
                                vertical: AffixVertical::Top(12.0),
                            })
                            .style(Style {
                                background: Some(container_theme.surface),
                                ..Style::new()
                                    .width(Length::Px(160.0))
                                    .height(Length::Px(34.0))
                            })
                            .fill(
                                sapid::button()
                                    .theme(container_theme.clone())
                                    .label("Sticky action"),
                            ),
                        affix("absolute")
                            .theme(container_theme.clone())
                            .name("Container affix")
                            .class("absolute-affix")
                            .within(context)
                            .style(
                                Style::new()
                                    .width(Length::Px(120.0))
                                    .height(Length::Px(34.0)),
                            )
                            .fill(
                                sapid::button()
                                    .theme(container_theme.clone())
                                    .label("Container action"),
                            ),
                        glory::ui::text().content(
                            "Explicit scroll content; sticky remains within this container.",
                        ),
                    ))
            });
        (
            glory::ui::view()
                .class("affix-demo")
                .on(EventKind::Resize, move |event| {
                    if let Event::Resize { size, .. } = event {
                        driver.observe_width(size.width).unwrap();
                    }
                })
                .fill((
                    sapid::title(1).theme(theme.clone()).content("Affix"),
                    sapid::text()
                        .theme(theme.clone())
                        .class("affix-status")
                        .content(status),
                    container,
                    fixed,
                    glory::ui::view()
                        .style(Style::new().height(Length::Px(1500.0)))
                        .fill(
                            sapid::text()
                                .theme(theme)
                                .content("Scroll the page to verify viewport fixation."),
                        ),
                )),
            State { offset, visible },
        )
    }
}
