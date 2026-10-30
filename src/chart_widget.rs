//! Current-data chart widgets. Geometry, legend proposals and selection are
//! admitted before publishing a new render model; rejected inputs retain it.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::charts::*;
use crate::composite::{self, Callback};
use glory::render::{Event, EventKind, FocusMode, PointerPhase};
use glory::style::{AspectRatio, Display, Edges, Length, Overflow, Position, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartSelection {
    pub series: String,
    pub category: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPointDetails {
    pub selection: ChartSelection,
    pub series_label: String,
    pub category_label: String,
    pub value: f64,
}
impl ChartPointDetails {
    pub fn text(&self) -> String {
        format!(
            "{} · {}: {}",
            self.category_label, self.series_label, self.value
        )
    }
}
#[derive(Clone, PartialEq)]
struct Model {
    table_revision: u64,
    data: ChartData,
    options: CartesianOptions,
    scene: CartesianScene,
    palette: ChartTheme,
}
impl Model {
    fn point(&self, key: &ChartSelection) -> Option<ChartPointDetails> {
        let series = self.data.series.iter().position(|s| s.key == key.series)?;
        let category = self
            .data
            .categories
            .iter()
            .position(|c| c.key == key.category)?;
        let location = self
            .scene
            .locations
            .iter()
            .find(|p| p.series == series && p.category == category)?;
        Some(ChartPointDetails {
            selection: key.clone(),
            series_label: self.data.series[series].label.clone(),
            category_label: self.data.categories[category].label.clone(),
            value: location.value,
        })
    }
    fn selection(&self, point: ChartLocation) -> ChartSelection {
        ChartSelection {
            series: self.data.series[point.series].key.clone(),
            category: self.data.categories[point.category].key.clone(),
        }
    }
}
fn invalid(error: ChartError) -> glory::ui::Error {
    glory::render::Error::InvalidProps(error.message()).into()
}
fn bounded_options(value: &CartesianOptions) -> Result<CartesianOptions> {
    if value.hidden.len() > MAX_CHART_SERIES {
        return Err(invalid(ChartError::Budget));
    }
    if value.hidden.iter().any(|key| key.len() > 64) {
        return Err(invalid(ChartError::Key));
    }
    Ok(value.clone())
}
pub struct Chart {
    common: Common,
    key: String,
    data: Vary<ChartData>,
    options: Option<Vary<CartesianOptions>>,
    default_options: CartesianOptions,
    kind: Option<CartesianKind>,
    frame: ChartFrame,
    palette: Option<ChartTheme>,
    selection: Option<Vary<Option<ChartSelection>>>,
    changed: Option<Box<dyn FnMut(CartesianOptions)>>,
    selected: Option<Box<dyn FnMut(Option<ChartSelection>)>>,
    show_table: bool,
    show_legend: bool,
    pointer_selection: bool,
}
pub fn chart(key: impl Into<String>) -> Chart {
    Chart {
        common: Common {
            name: Some("Chart".into()),
            ..Common::default()
        },
        key: key.into(),
        data: ChartData::default().into(),
        options: None,
        default_options: CartesianOptions::default(),
        kind: None,
        frame: ChartFrame::default(),
        palette: None,
        selection: None,
        changed: None,
        selected: None,
        show_table: true,
        show_legend: true,
        pointer_selection: false,
    }
}
pub fn line_chart(key: impl Into<String>) -> Chart {
    chart(key).kind(CartesianKind::Line)
}
pub fn area_chart(key: impl Into<String>) -> Chart {
    chart(key).kind(CartesianKind::Area)
}
pub fn bar_chart(key: impl Into<String>) -> Chart {
    chart(key).kind(CartesianKind::Bar)
}
pub fn composite_chart(key: impl Into<String>) -> Chart {
    chart(key).kind(CartesianKind::Composite)
}
impl Chart {
    common_methods!();
    pub fn data(mut self, value: impl Into<Vary<ChartData>>) -> Self {
        self.data = value.into();
        self
    }
    pub fn options(mut self, value: impl Into<Vary<CartesianOptions>>) -> Self {
        self.options = Some(value.into());
        self
    }
    pub fn default_options(mut self, value: CartesianOptions) -> Self {
        self.default_options = value;
        self
    }
    pub fn kind(mut self, value: CartesianKind) -> Self {
        self.kind = Some(value);
        self
    }
    pub fn frame(mut self, value: ChartFrame) -> Self {
        self.frame = value;
        self
    }
    pub fn chart_theme(mut self, value: ChartTheme) -> Self {
        self.palette = Some(value);
        self
    }
    pub fn selection(mut self, value: impl Into<Vary<Option<ChartSelection>>>) -> Self {
        self.selection = Some(value.into());
        self
    }
    pub fn on_options_change(mut self, value: impl FnMut(CartesianOptions) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_selection_change(
        mut self,
        value: impl FnMut(Option<ChartSelection>) + 'static,
    ) -> Self {
        self.selected = Some(Box::new(value));
        self
    }
    /// Keep the finite full-value table visible by default. When hiding it,
    /// applications must supply their own equivalent data representation.
    pub fn data_table(mut self, value: bool) -> Self {
        self.show_table = value;
        self
    }
    pub fn legend(mut self, value: bool) -> Self {
        self.show_legend = value;
        self
    }
    /// Opt into host pointer boundary observations. Click and keyboard point
    /// controls work independently; native hosts currently reject this interest.
    pub fn pointer_selection(mut self, value: bool) -> Self {
        self.pointer_selection = value;
        self
    }
}
#[derive(Clone)]
struct State {
    owner: glory::core::InstanceId,
    model: Cage<Rc<Model>>,
    options: Cage<CartesianOptions>,
    controlled_options: bool,
    selection: Cage<Option<ChartSelection>>,
    controlled_selection: bool,
    enabled: Vary<bool>,
    changed: Callback<CartesianOptions>,
    selected: Callback<Option<ChartSelection>>,
}
impl State {
    fn choose(&self, value: Option<ChartSelection>) {
        if !self.owner.is_alive() || !self.enabled.get() {
            return;
        }
        if value
            .as_ref()
            .is_some_and(|v| self.model.with(|m| m.point(v).is_none()))
        {
            return;
        }
        if self.selection.get() == value {
            return;
        }
        if !self.controlled_selection {
            self.selection.revise(|old| *old = value.clone());
        }
        self.selected.emit(self.owner, value);
    }
    fn step(&self, forward: bool) {
        let selected = self.selection.get();
        let next = self.model.with(|m| {
            let count = m.scene.locations.len();
            if count == 0 {
                return None;
            }
            let current = selected
                .as_ref()
                .and_then(|s| m.scene.locations.iter().position(|p| m.selection(*p) == *s));
            let index = match current {
                None => {
                    if forward {
                        0
                    } else {
                        count - 1
                    }
                }
                Some(i) => {
                    if forward {
                        (i + 1).min(count - 1)
                    } else {
                        i.saturating_sub(1)
                    }
                }
            };
            Some(m.selection(m.scene.locations[index]))
        });
        self.choose(next);
    }
    fn toggle(&self, key: &str) {
        if !self.owner.is_alive() || !self.enabled.get() {
            return;
        }
        let next = self.model.with(|m| {
            if !m.data.series.iter().any(|s| s.key == key) {
                return None;
            }
            let mut next = m.options.clone();
            if !next.hidden.remove(key) {
                next.hidden.insert(key.into());
            }
            Some(next)
        });
        if let Some(value) = next {
            if !self.controlled_options {
                self.options.revise(|old| *old = value.clone());
            }
            self.changed.emit(self.owner, value);
        }
    }
}
impl Widget for Chart {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("chart", &self.key, "root", "root")?;
        self.frame.validate().map_err(invalid)?;
        let palette = self
            .palette
            .clone()
            .unwrap_or_else(|| ChartTheme::from_theme(&self.common.theme));
        let internal_options = scope.cage(bounded_options(&self.default_options)?);
        let source = self
            .options
            .clone()
            .unwrap_or_else(|| internal_options.into());
        let kind = self.kind;
        let frame = self.frame;
        let read = {
            let data = self.data.clone();
            let options = source;
            move || -> Result<Rc<Model>> {
                let mut options = options.try_with(bounded_options)??;
                if let Some(kind) = kind {
                    options.kind = kind;
                }
                data.try_with(|data| {
                    let scene =
                        CartesianScene::build(data, frame, &options, &palette).map_err(invalid)?;
                    Ok(Rc::new(Model {
                        table_revision: 0,
                        data: data.clone(),
                        options,
                        scene,
                        palette: palette.clone(),
                    }))
                })?
            }
        };
        let model = scope.cage(glory::core::untracked(&read)?);
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else {
                return;
            };
            match read() {
                Ok(mut next) => {
                    let previous = glory::core::untracked(|| model.get());
                    let table_changed = previous.data != next.data;
                    let revision = if table_changed {
                        previous.table_revision.checked_add(1)
                    } else {
                        Some(previous.table_revision)
                    };
                    let Some(revision) = revision else {
                        scope.report_error(invalid(ChartError::Budget));
                        return;
                    };
                    Rc::make_mut(&mut next).table_revision = revision;
                    if *previous != *next {
                        model.revise(|old| *old = next);
                    }
                }
                Err(error) => scope.report_error(error),
            }
        });
        let selection = scope.cage(None);
        if let Some(source) = self.selection.clone() {
            let accepted = composite::validated(scope, &source, |value| {
                if value.as_ref().is_some_and(|v| {
                    !crate::chart_model::key(&v.series) || !crate::chart_model::key(&v.category)
                }) {
                    Err(invalid(ChartError::Key))
                } else {
                    Ok(())
                }
            })?;
            let weak = scope.downgrade();
            scope.structural(move || {
                if weak.upgrade().is_some() {
                    let value = accepted.get();
                    if selection.get() != value {
                        selection.revise(|old| *old = value);
                    }
                }
            });
        }
        let state = State {
            owner: scope.id(),
            model,
            options: internal_options,
            controlled_options: self.options.is_some(),
            selection,
            controlled_selection: self.selection.is_some(),
            enabled: self.common.enabled.clone(),
            changed: Callback::new(self.changed.take()),
            selected: Callback::new(self.selected.take()),
        };
        let cleanup = state.clone();
        scope.on_cleanup(move || {
            cleanup.changed.clear();
            cleanup.selected.clear();
        })?;
        let root = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut widget = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-chart",
            composite::column(8.)
        )
        .node_ref(root);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            widget = widget.on(kind, move |e| observe(&mut handlers, owner, e));
        }
        widget.show_in(scope)?;
        let area = scope.child_in(root.get().ok_or(glory::ui::Error::Gone)?)?;
        build_plot(&area, &state, frame, self.pointer_selection)?;
        if self.show_legend {
            build_legend(
                &area,
                &state,
                self.common.theme.clone(),
                self.common.unstyled,
            )?;
        }
        let controls = area.cage(None);
        glory::ui::view()
            .class("sapid-chart-point-controls")
            .style(Style::new().display(Display::Flex).gap(8.))
            .node_ref(controls)
            .show_in(&area)?;
        let control_scope = area.child_in(controls.get().ok_or(glory::ui::Error::Gone)?)?;
        let previous = state.clone();
        let next = state.clone();
        let clear = state.clone();
        let enabled = self.common.enabled.clone();
        let has_points =
            area.bond(move || enabled.get() && model.with(|m| !m.scene.locations.is_empty()));
        crate::button()
            .label("Previous data point")
            .theme(self.common.theme.clone())
            .enabled(has_points)
            .on_press(move || previous.step(false))
            .show_in(&control_scope)?;
        crate::button()
            .label("Next data point")
            .theme(self.common.theme.clone())
            .enabled(has_points)
            .on_press(move || next.step(true))
            .show_in(&control_scope)?;
        crate::button()
            .label("Clear data point")
            .theme(self.common.theme.clone())
            .enabled(self.common.enabled.clone())
            .on_press(move || clear.choose(None))
            .show_in(&control_scope)?;
        area.retain(control_scope)?;
        let details = area
            .bond(move || model.with(|m| selection.with(|s| s.as_ref().and_then(|s| m.point(s)))));
        glory::ui::text()
            .class("sapid-chart-point-detail")
            .content(area.bond(move || {
                details
                    .get()
                    .map_or_else(|| "No data point selected".into(), |p| p.text())
            }))
            .show_in(&area)?;
        glory::ui::text()
            .class("sapid-chart-empty")
            .content(area.bond(move || {
                model.with(|m| {
                    if m.scene.locations.is_empty() {
                        "No visible chart data".into()
                    } else {
                        String::new()
                    }
                })
            }))
            .show_in(&area)?;
        glory::ui::text()
            .class("sapid-chart-clipping")
            .content(area.bond(move || {
                model.with(|m| {
                    if m.scene.clipped_values == 0 {
                        String::new()
                    } else {
                        format!(
                            "{} values clipped by the explicit scale",
                            m.scene.clipped_values
                        )
                    }
                })
            }))
            .show_in(&area)?;
        if self.show_table {
            build_table(
                &area,
                model,
                &self.key,
                self.common.theme.clone(),
                self.common.unstyled,
            )?;
        }
        scope.retain(area)?;
        Ok(())
    }
}

fn build_plot(scope: &Scope, state: &State, frame: ChartFrame, pointer_selection: bool) -> Result {
    let reference = scope.cage(None);
    glory::ui::view()
        .class("sapid-chart-plot")
        .node_ref(reference)
        .style(
            Style::new()
                .position(Position::Relative)
                .width(Length::Percent(100.))
                .aspect_ratio(AspectRatio::Ratio(frame.width / frame.height)),
        )
        .show_in(scope)?;
    let plot = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
    let model = state.model;
    for index in 0..5 {
        let style = plot.bond(move || {
            model.with(|m| {
                Style {
                    font_size: Some(11.),
                    ..Style::new()
                }
                .position(Position::Absolute)
                .left(Length::Px(0.))
                .top(Length::Percent(
                    m.scene.ticks[index].y / frame.height * 100.,
                ))
                .width(Length::Px(48.))
                .height(Length::Px(20.))
                .overflow_x(Overflow::Hidden)
            })
        });
        glory::ui::text()
            .class("sapid-chart-axis-tick")
            .style(style)
            .content(plot.bond(move || model.with(|m| m.scene.ticks[index].value.to_string())))
            .show_in(&plot)?;
    }
    let labels = plot.bond(move || model.with(|m| (0..m.scene.labels.len()).collect::<Vec<_>>()));
    glory::ui::each(labels)
        .key(|index| *index)
        .children(move |entry| {
            let index = entry.value;
            let index = index.get_untracked();
            glory::ui::widget(move |scope| {
                let style = scope.bond(move || {
                    model.with(|m| {
                        let label = &m.scene.labels[index];
                        Style {
                            font_size: Some(11.),
                            ..Style::new()
                        }
                        .position(Position::Absolute)
                        .left(Length::Percent(
                            (label.x / frame.width * 100. - 6.).clamp(0., 88.),
                        ))
                        .top(Length::Percent(label.y / frame.height * 100.))
                        .width(Length::Percent(12.))
                        .height(Length::Px(24.))
                        .overflow_y(Overflow::Hidden)
                        .overflow_x(Overflow::Hidden)
                    })
                });
                glory::ui::text()
                    .class("sapid-chart-category-label")
                    .style(style)
                    .content(scope.bond(move || {
                        model.with(|m| {
                            m.data.categories[m.scene.labels[index].category]
                                .label
                                .clone()
                        })
                    }))
                    .show_in(scope)
            })
        })
        .show_in(&plot)?;
    let paths = plot.bond(move || {
        model.with(|m| {
            m.scene
                .paths
                .iter()
                .map(|p| p.key.clone())
                .collect::<Vec<_>>()
        })
    });
    glory::ui::each(paths)
        .key(|key| key.clone())
        .children(move |entry| {
            let key = entry.value;
            let key = key.get_untracked();
            glory::ui::widget(move |scope| {
                let key = key.clone();
                glory::ui::graphic()
                    .drawing(scope.bond(move || {
                        model.with(|m| {
                            m.scene
                                .paths
                                .iter()
                                .find(|p| p.key == key)
                                .expect("live chart path")
                                .drawing
                                .clone()
                        })
                    }))
                    .class("sapid-chart-layer")
                    .style(
                        Style::new()
                            .position(Position::Absolute)
                            .left(Length::Px(0.))
                            .top(Length::Px(0.))
                            .width(Length::Percent(100.))
                            .height(Length::Percent(100.)),
                    )
                    .show_in(scope)
            })
        })
        .show_in(&plot)?;
    let state = state.clone();
    let keys = plot.bond(move || {
        model.with(|m| {
            m.scene
                .locations
                .iter()
                .map(|p| m.selection(*p))
                .collect::<Vec<_>>()
        })
    });
    glory::ui::each(keys)
        .key(|key| (key.series.clone(), key.category.clone()))
        .children(move |entry| {
            let key = entry.value;
            let key = key.get_untracked();
            let state = state.clone();
            glory::ui::widget(move |scope| {
                let location_key = key.clone();
                let label_key = key.clone();
                let activate = key.clone();
                let hover_key = key.clone();
                let selected = state.selection;
                let style = scope.bond(move || {
                    model.with(|m| {
                        let p = m
                            .scene
                            .locations
                            .iter()
                            .find(|p| m.selection(**p) == location_key)
                            .expect("live chart point");
                        let color = m.palette.series[p.series];
                        Style {
                            position: Some(Position::Absolute),
                            left: Some(Length::Percent(p.x / frame.width * 100.)),
                            top: Some(Length::Percent(p.y / frame.height * 100.)),
                            width: Some(Length::Px(10.)),
                            height: Some(Length::Px(10.)),
                            margin: Some(Edges::all(-5.)),
                            padding: Some(Edges::all(0.)),
                            radius: Some(5.),
                            background: Some(color),
                            border_width: Some(Edges::all(
                                if selected.with(|s| s.as_ref() == Some(&location_key)) {
                                    2.
                                } else {
                                    0.
                                },
                            )),
                            ..Style::new()
                        }
                    })
                });
                let pressed = state.clone();
                let hovered = state.clone();
                let mut button = glory::ui::button()
                    .class("sapid-chart-point")
                    .focus(FocusMode::Programmatic)
                    .enabled(state.enabled.clone())
                    .name(scope.bond(move || {
                        model.with(|m| m.point(&label_key).map_or_else(String::new, |p| p.text()))
                    }))
                    .style(style)
                    .on(EventKind::Activate, move |_| {
                        pressed.choose(Some(activate.clone()))
                    });
                if pointer_selection {
                    button = button.on(EventKind::Pointer, move |event| {
                        if matches!(
                            event,
                            Event::Pointer {
                                phase: PointerPhase::Enter,
                                ..
                            }
                        ) {
                            hovered.choose(Some(hover_key.clone()));
                        }
                    });
                }
                button.show_in(scope)
            })
        })
        .show_in(&plot)?;
    scope.retain(plot)?;
    Ok(())
}

fn build_legend(scope: &Scope, state: &State, theme: crate::Theme, unstyled: bool) -> Result {
    let model = state.model;
    let state = state.clone();
    let keys = scope.bond(move || {
        model.with(|m| {
            m.data
                .series
                .iter()
                .map(|s| s.key.clone())
                .collect::<Vec<_>>()
        })
    });
    glory::ui::each(keys)
        .key(|key| key.clone())
        .children(move |entry| {
            let key = entry.value;
            let key = key.get_untracked();
            let state = state.clone();
            let theme = theme.clone();
            glory::ui::widget(move |scope| {
                let label_key = key.clone();
                let toggle = key.clone();
                let color_key = key.clone();
                let state = state.clone();
                let label = scope.bond(move || {
                    model.with(|m| {
                        m.data
                            .series
                            .iter()
                            .find(|s| s.key == label_key)
                            // Its keyed row is retired in this structural turn.
                            // Keep construction validation nonempty until then;
                            // no binding publishes this retiring placeholder.
                            .map_or_else(
                                || "Removed chart series".into(),
                                |s| {
                                    format!(
                                        "{} {}",
                                        if m.options.hidden.contains(&label_key) {
                                            "Show"
                                        } else {
                                            "Hide"
                                        },
                                        s.label
                                    )
                                },
                            )
                    })
                });
                let mut button = crate::button()
                    .label(label)
                    .left_section(
                        glory::ui::graphic()
                            .class("sapid-chart-legend-swatch")
                            .drawing(scope.bond(move || {
                                model.with(|m| {
                                    crate::chart_aux::swatch(
                                        m.palette.series[m
                                            .data
                                            .series
                                            .iter()
                                            .position(|s| s.key == color_key)
                                            .expect("live legend series")],
                                    )
                                })
                            }))
                            .style(Style::new().width(Length::Px(12.)).height(Length::Px(12.))),
                    )
                    .class("sapid-chart-legend-item")
                    .theme(theme.clone())
                    .enabled(state.enabled.clone())
                    .on_press(move || state.toggle(&toggle));
                if unstyled {
                    button = button.unstyled();
                }
                button.show_in(scope)
            })
        })
        .show_in(scope)
}
fn build_table(
    scope: &Scope,
    model: Cage<Rc<Model>>,
    key: &str,
    theme: crate::Theme,
    unstyled: bool,
) -> Result {
    let key = key.to_owned();
    // The read-only table has no focus or editing state. Rebuild it only when
    // the admitted data changes, preserving plot and legend identities.
    let values = scope.bond(move || model.with(|m| vec![m.table_revision]));
    glory::ui::view()
        .class("sapid-chart-data-table")
        .style(Style::new().overflow_x(Overflow::Auto))
        .fill(
            glory::ui::each(values)
                .key(|revision| *revision)
                .children(move |__each_row| {
                    let data = model.with(|m| m.data.clone());
                    let key = key.clone();
                    let theme = theme.clone();
                    glory::ui::widget(move |scope| {
                        let mut header = crate::table_row("head")
                            .cell(crate::table_header_cell("category").text("Category"));
                        for (j, s) in data.series.iter().enumerate() {
                            header = header.cell(
                                crate::table_header_cell(format!("s{j}")).text(s.label.clone()),
                            );
                        }
                        let mut body = crate::table_body();
                        for (i, c) in data.categories.iter().enumerate() {
                            let mut row = crate::table_row(format!("r{i}"))
                                .cell(crate::table_row_header("category").text(c.label.clone()));
                            for (j, s) in data.series.iter().enumerate() {
                                row = row.cell(
                                    crate::table_cell(format!("s{j}")).text(
                                        s.values[i]
                                            .map_or_else(|| "No value".into(), |v| v.to_string()),
                                    ),
                                );
                            }
                            body = body.row(row);
                        }
                        let mut table = crate::table(key.clone())
                            .theme(theme.clone())
                            .caption("Chart data")
                            .head(crate::table_head().row(header))
                            .body(body);
                        if unstyled {
                            table = table.unstyled();
                        }
                        table.show_in(scope)
                    })
                }),
        )
        .show_in(scope)
}
