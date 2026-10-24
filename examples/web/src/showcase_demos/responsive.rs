use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::responsive_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::style::{Align, Length, Style};
    use glory::ui::{Widget, widget};
    use glory::{Lotus, Scope};
    use sapid::{ResponsiveValues, Theme};

    fn five<T>(base: T, small: T, medium: T, large: T, extra_large: T) -> ResponsiveValues<T> {
        ResponsiveValues::new(base, small, medium, large, extra_large)
    }
    pub fn responsive_form(theme: Theme) -> impl Widget {
        widget(move |scope: &Scope| {
            let width = scope.cage(500.);
            let custom: Lotus<Style> = width
                .map(|width| Style::new().width(Length::Px(*width)))
                .into();
            let responsive = sapid::responsive_root(500.)
                .class("responsive-fixture")
                .style(custom)
                .content({
                    let theme = theme.clone();
                    move |context| {
                        let center = context
                            .styles(five(
                                Style {
                                    align: Some(Align::Start),
                                    ..Default::default()
                                },
                                Style {
                                    align: Some(Align::Start),
                                    ..Default::default()
                                },
                                Style {
                                    align: Some(Align::Center),
                                    ..Default::default()
                                },
                                Style {
                                    align: Some(Align::Center),
                                    ..Default::default()
                                },
                                Style {
                                    align: Some(Align::End),
                                    ..Default::default()
                                },
                            ))
                            .unwrap();
                        let container = context
                            .styles(five(
                                Style::new().padding(glory::style::Edges::all(4.)),
                                Style::new().padding(glory::style::Edges::all(8.)),
                                Style::new().padding(glory::style::Edges::all(12.)),
                                Style::new().padding(glory::style::Edges::all(16.)),
                                Style::new().padding(glory::style::Edges::all(20.)),
                            ))
                            .unwrap();
                        let space = context
                            .styles(five(
                                Style::new().height(Length::Px(10.)),
                                Style::new().height(Length::Px(20.)),
                                Style::new().height(Length::Px(30.)),
                                Style::new().height(Length::Px(40.)),
                                Style::new().height(Length::Px(50.)),
                            ))
                            .unwrap();
                        let gaps = context
                            .styles(five(
                                Style::new().gap(4.),
                                Style::new().gap(8.),
                                Style::new().gap(12.),
                                Style::new().gap(16.),
                                Style::new().gap(20.),
                            ))
                            .unwrap();
                        let gap2 = context
                            .styles(five(
                                Style::new().gap(3.),
                                Style::new().gap(6.),
                                Style::new().gap(9.),
                                Style::new().gap(12.),
                                Style::new().gap(15.),
                            ))
                            .unwrap();
                        let label = context
                            .select(five(
                                "Base".to_string(),
                                "Small".to_string(),
                                "Medium".to_string(),
                                "Large".to_string(),
                                "ExtraLarge".to_string(),
                            ))
                            .unwrap();
                        sapid::stack().theme(theme.clone()).fill((
                            sapid::text().class("responsive-tier").content(label),
                            sapid::center()
                                .class("responsive-center")
                                .height(Length::Px(90.))
                                .style(center)
                                .fill(
                                    sapid::box_()
                                        .class("responsive-center-child")
                                        .width(Length::Px(30.))
                                        .height(Length::Px(20.)),
                                ),
                            sapid::container()
                                .class("responsive-container")
                                .max_width(glory::style::Limit::Px(360.))
                                .style(container)
                                .fill(sapid::text().content("Container")),
                            sapid::space().class("responsive-space").style(space),
                            sapid::grid("responsive-grid")
                                .class("responsive-grid")
                                .columns(2)
                                .style(gaps)
                                .items((0..4).map(|index| {
                                    sapid::grid_item(format!("item{index}"))
                                        .fill(sapid::text().content(format!("Grid {index}")))
                                })),
                            sapid::simple_grid("responsive-simple")
                                .class("responsive-simple")
                                .columns(2)
                                .style(gap2)
                                .items((0..4).map(|index| {
                                    sapid::grid_item(format!("item{index}"))
                                        .fill(sapid::text().content(format!("Simple {index}")))
                                })),
                        ))
                    }
                });
            sapid::stack()
                .fill((
                    sapid::group().fill((
                        sapid::button()
                            .label("Narrow container")
                            .on_press(move || width.revise(|value| *value = 500.)),
                        sapid::button()
                            .label("Wide container")
                            .on_press(move || width.revise(|value| *value = 1000.)),
                    )),
                    responsive,
                ))
                .show_in(scope)
        })
    }
}
