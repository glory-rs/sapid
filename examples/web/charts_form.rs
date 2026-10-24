use glory::prelude::*;
use glory::style::{Display, FlexDirection};
use sapid::charts::*;
pub fn data() -> ChartData {
    ChartData {
        categories: ["January", "February", "March", "April", "May", "June"]
            .into_iter()
            .enumerate()
            .map(|(i, s)| ChartCategory::new(format!("m{i}"), s))
            .collect(),
        series: vec![
            ChartSeries::new(
                "revenue",
                "Revenue",
                [Some(12.), Some(18.), None, Some(15.), Some(-4.), Some(22.)],
            ),
            ChartSeries::new(
                "cost",
                "Cost",
                [
                    Some(8.),
                    Some(10.),
                    Some(6.),
                    Some(9.),
                    Some(-2.),
                    Some(12.),
                ],
            )
            .kind(ChartSeriesKind::Bar),
        ],
    }
}
pub fn charts_form(theme: sapid::Theme, cartesian: bool) -> impl Widget {
    component(move |scope| {
        let data = scope.cage(data());
        let options = scope.cage(CartesianOptions {
            kind: if cartesian {
                CartesianKind::Composite
            } else {
                CartesianKind::Line
            },
            ..Default::default()
        });
        let selection = scope.cage(None);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let mounted = scope.cage(true);
        let proposals = scope.cage(0usize);
        let exports = scope.cage(String::new());
        let root = scope.cage(None);
        view()
            .class("chart-fixture")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(12.)
                    .min_width(Length::Px(0.)),
            )
            .node_ref(root)
            .show_in(scope)?;
        let area = scope.child_in(root.get().unwrap())?;
        sapid::checkbox()
            .label("Accept chart proposals")
            .checked(accept)
            .on_checked_change(move |v| accept.revise(|old| *old = v))
            .theme(theme.clone())
            .show_in(&area)?;
        sapid::checkbox()
            .label("Enable chart controls")
            .checked(enabled)
            .on_checked_change(move |v| enabled.revise(|old| *old = v))
            .theme(theme.clone())
            .show_in(&area)?;
        sapid::button()
            .label("Brush middle categories")
            .theme(theme.clone())
            .on_press(move || {
                options.revise(|o| o.brush = Some(ChartBrush { first: 1, last: 3 }))
            })
            .show_in(&area)?;
        sapid::button()
            .label("Reset chart window")
            .theme(theme.clone())
            .on_press(move || {
                options.revise(|o| {
                    o.brush = None;
                    o.domain = None;
                })
            })
            .show_in(&area)?;
        sapid::button()
            .label("Clip chart scale")
            .theme(theme.clone())
            .on_press(move || {
                options.revise(|o| o.domain = Some(ChartDomain::new(0., 10.).unwrap()))
            })
            .show_in(&area)?;
        sapid::button()
            .label("Replace current chart data")
            .theme(theme.clone())
            .on_press(move || {
                data.revise(|d| {
                    d.categories[0].label = "Updated January".into();
                    d.series[0].values[0] = Some(7.);
                })
            })
            .show_in(&area)?;
        sapid::button()
            .label("Toggle chart owner")
            .theme(theme.clone())
            .on_press(move || mounted.revise(|v| *v = !*v))
            .show_in(&area)?;
        sapid::button()
            .label("Export chart SVG")
            .theme(theme.clone())
            .on_press({
                let theme = theme.clone();
                move || {
                    let value = data
                        .with(|d| {
                            CartesianScene::build(
                                d,
                                ChartFrame::default(),
                                &options.get(),
                                &ChartTheme::from_theme(&theme),
                            )
                        })
                        .unwrap()
                        .to_svg()
                        .unwrap();
                    exports.revise(|old| *old = value);
                }
            })
            .show_in(&area)?;
        let chart_theme = theme.clone();
        glory::ui::when(mounted, move || {
            let theme = chart_theme.clone();
            chart("fixture")
                .class("chart-primary")
                .pointer_selection(true)
                .theme(theme)
                .data(data)
                .options(options)
                .selection(selection)
                .enabled(enabled)
                .kind(if cartesian {
                    CartesianKind::Composite
                } else {
                    CartesianKind::Line
                })
                .on_options_change(move |v| {
                    proposals.revise(|n| *n += 1);
                    if accept.get() {
                        options.revise(|old| *old = v);
                    }
                })
                .on_selection_change(move |v| {
                    proposals.revise(|n| *n += 1);
                    if accept.get() {
                        selection.revise(|old| *old = v);
                    }
                })
        })
        .show_in(&area)?;
        text()
            .class("chart-proposals")
            .content(scope.bond(move || format!("Proposals: {}", proposals.get())))
            .show_in(&area)?;
        text()
            .class("chart-export")
            .content(exports)
            .style(
                Style::new()
                    .height(Length::Px(24.))
                    .overflow_x(glory::style::Overflow::Auto)
                    .overflow_y(glory::style::Overflow::Hidden),
            )
            .show_in(&area)?;
        tooltip("chart-details")
            .theme(theme.clone())
            .label("Inspect chart point")
            .point(scope.bond(move || {
                selection.with(|selected| {
                    selected.as_ref().and_then(|selected| {
                        data.with(|d| {
                            let s = d.series.iter().find(|s| s.key == selected.series)?;
                            let i = d
                                .categories
                                .iter()
                                .position(|c| c.key == selected.category)?;
                            Some(ChartPointDetails {
                                selection: selected.clone(),
                                series_label: s.label.clone(),
                                category_label: d.categories[i].label.clone(),
                                value: s.values[i]?,
                            })
                        })
                    })
                })
            }))
            .show_in(&area)?;
        if cartesian {
            for (kind, label) in [
                (CartesianKind::Line, "Line"),
                (CartesianKind::Area, "Area"),
                (CartesianKind::Bar, "Bar"),
                (CartesianKind::Composite, "Composite"),
            ] {
                text().content(label).show_in(&area)?;
                chart(format!("small-{label}"))
                    .class(format!("chart-mode-{label}"))
                    .theme(theme.clone())
                    .kind(kind)
                    .data(data)
                    .data_table(false)
                    .legend(false)
                    .show_in(&area)?;
            }
            let mut positive = self::data();
            for series in &mut positive.series {
                for value in series.values.iter_mut().flatten() {
                    *value = value.abs();
                }
            }
            area_chart("stacked-area")
                .class("chart-stacked-area")
                .theme(theme.clone())
                .data(positive)
                .default_options(CartesianOptions {
                    stacked: true,
                    ..Default::default()
                })
                .data_table(false)
                .show_in(&area)?;
            bar_chart("stacked-bars")
                .class("chart-stacked-bar")
                .theme(theme.clone())
                .data(data)
                .default_options(CartesianOptions {
                    stacked: true,
                    ..Default::default()
                })
                .data_table(false)
                .show_in(&area)?;
        }
        chart("empty")
            .class("chart-empty-fixture")
            .theme(theme.clone())
            .data_table(false)
            .legend(false)
            .show_in(&area)?;
        scope.retain(area)?;
        Ok(())
    })
}
