use glory::prelude::*;
use sapid::charts::*;
#[path = "../../fixtures/plot_samples.rs"]
pub mod samples;
pub const PAGES: [&str; 6] = [
    "polar-charts",
    "point-charts",
    "shape-charts",
    "matrix-charts",
    "range-charts",
    "hierarchy-charts",
];
pub fn plots_form(theme: sapid::Theme, group: usize) -> impl Widget {
    widget(move |scope| {
        let kinds = &samples::KINDS[group * 3..group * 3 + 3];
        let kind = kinds[0];
        let data = scope.cage(samples::sample(kind));
        let options = scope.cage(PlotOptions::default());
        let selection = scope.cage(None);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let mounted = scope.cage(true);
        let proposals = scope.cage(0usize);
        let root = scope.cage(None);
        view()
            .class("plots-fixture")
            .style(
                Style::new()
                    .display(glory::style::Display::Flex)
                    .flex_direction(glory::style::FlexDirection::Column)
                    .gap(12.)
                    .min_width(Length::Px(0.)),
            )
            .node_ref(root)
            .show_in(scope)?;
        let area = scope.child_in(root.get().unwrap())?;
        sapid::checkbox()
            .label("Accept plot proposals")
            .checked(accept)
            .on_checked_change(move |v| accept.revise(|o| *o = v))
            .theme(theme.clone())
            .show_in(&area)?;
        sapid::checkbox()
            .label("Enable plot controls")
            .checked(enabled)
            .on_checked_change(move |v| enabled.revise(|o| *o = v))
            .theme(theme.clone())
            .show_in(&area)?;
        sapid::button()
            .label("Replace current plot data")
            .theme(theme.clone())
            .on_press(move || data.revise(samples::update))
            .show_in(&area)?;
        sapid::button()
            .label("Empty plot data")
            .theme(theme.clone())
            .on_press(move || {
                options.revise(|o| *o = PlotOptions::default());
                data.revise(|o| *o = samples::empty(kind));
            })
            .show_in(&area)?;
        sapid::button()
            .label("Restore plot data")
            .theme(theme.clone())
            .on_press(move || {
                options.revise(|o| *o = PlotOptions::default());
                data.revise(|o| *o = samples::sample(kind));
            })
            .show_in(&area)?;
        sapid::button()
            .label("Toggle plot owner")
            .theme(theme.clone())
            .on_press(move || mounted.revise(|o| *o = !*o))
            .show_in(&area)?;
        let palette = theme.clone();
        glory::ui::when(mounted, move || {
            plot("primary", kind)
                .class("plot-primary")
                .theme(palette.clone())
                .name(samples::title(kind))
                .pointer_selection(true)
                .data(data)
                .options(options)
                .selection(selection)
                .enabled(enabled)
                .on_options_change(move |v| {
                    proposals.revise(|n| *n += 1);
                    if accept.get() {
                        options.revise(|o| *o = v);
                    }
                })
                .on_selection_change(move |v| {
                    proposals.revise(|n| *n += 1);
                    if accept.get() {
                        selection.revise(|o| *o = v);
                    }
                })
        })
        .show_in(&area)?;
        text()
            .class("plot-proposals")
            .content(scope.bond(move || format!("Proposals: {}", proposals.get())))
            .show_in(&area)?;
        for (i, kind) in kinds[1..].iter().copied().enumerate() {
            text().content(samples::title(kind)).show_in(&area)?;
            plot(format!("secondary{i}"), kind)
                .class("plot-secondary")
                .theme(theme.clone())
                .name(samples::title(kind))
                .data(samples::sample(kind))
                .show_in(&area)?;
        }
        if group == 1 {
            let data = PlotData::sample_points(1_000_000, 16, |i| PointDatum {
                key: format!("s{i}"),
                label: format!("Sample {i}"),
                x: (i % 23) as f64,
                y: (i % 17) as f64,
                size: 1.,
            })
            .unwrap();
            scatter_chart("sample")
                .class("plot-sample")
                .theme(theme.clone())
                .name("Sampled source")
                .data(data)
                .show_in(&area)?;
        }
        scope.retain(area)?;
        Ok(())
    })
}
