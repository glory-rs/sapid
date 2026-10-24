//! Shared current-snapshot shell over eighteen distinct typed geometries.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::charts::*;
use crate::composite::{self, Callback};
use crate::plot_table::{self, PlotTable};
use glory::render::{Event, EventKind, FocusMode, PointerPhase, Role, Semantics};
use glory::style::{AspectRatio, Display, Edges, Length, Overflow, Position, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
use std::{collections::BTreeMap, rc::Rc};
pub const MAX_PLOT_KEY_BYTES: usize = 48;
fn invalid(error: ChartError) -> glory::ui::Error {
    glory::render::Error::InvalidProps(error.message()).into()
}
#[derive(Clone, PartialEq)]
struct Model {
    data: PlotData,
    options: PlotOptions,
    scene: PlotScene,
    tables: Vec<PlotTable>,
    table_revision: u64,
    marks: BTreeMap<String, usize>,
    paths: BTreeMap<String, usize>,
}
pub struct Plot {
    common: Common,
    key: String,
    kind: PlotKind,
    data: Lotus<PlotData>,
    options: Option<Lotus<PlotOptions>>,
    default_options: PlotOptions,
    frame: ChartFrame,
    palette: Option<ChartTheme>,
    selection: Option<Lotus<Option<String>>>,
    changed: Option<Box<dyn FnMut(PlotOptions)>>,
    selected: Option<Box<dyn FnMut(Option<String>)>>,
    show_table: bool,
    show_legend: bool,
    pointer_selection: bool,
}
pub fn plot(key: impl Into<String>, kind: PlotKind) -> Plot {
    let data = match kind {
        PlotKind::Scatter | PlotKind::Bubble => PlotData::Points(Vec::new()),
        PlotKind::Candlestick => PlotData::Candles(Vec::new()),
        PlotKind::Heatmap | PlotKind::Matrix => PlotData::Matrix(MatrixData::default()),
        PlotKind::Bullet | PlotKind::Gauge => PlotData::Targets(Vec::new()),
        PlotKind::Treemap | PlotKind::Sunburst => PlotData::Hierarchy(Vec::new()),
        PlotKind::Sankey => PlotData::Flows(FlowData::default()),
        _ => PlotData::default(),
    };
    Plot {
        common: Common {
            name: Some("Plot".into()),
            ..Common::default()
        },
        key: key.into(),
        kind,
        data: data.into(),
        options: None,
        default_options: PlotOptions::default(),
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
macro_rules! constructors{($($name:ident:$kind:ident),+$(,)?)=>{$(pub fn $name(key:impl Into<String>)->Plot{plot(key,PlotKind::$kind)})+};}
constructors! {pie_chart:Pie,donut_chart:Donut,radial_bar_chart:RadialBar,scatter_chart:Scatter,bubble_chart:Bubble,candlestick_chart:Candlestick,radar_chart:Radar,funnel_chart:Funnel,sparkline:Sparkline,heatmap:Heatmap,matrix_chart:Matrix,waffle_chart:Waffle,bars_list:BarsList,bullet_chart:Bullet,gauge_chart:Gauge,treemap:Treemap,sunburst_chart:Sunburst,sankey_chart:Sankey}
impl Plot {
    common_methods!();
    pub fn data(mut self, value: impl Into<Lotus<PlotData>>) -> Self {
        self.data = value.into();
        self
    }
    pub fn options(mut self, value: impl Into<Lotus<PlotOptions>>) -> Self {
        self.options = Some(value.into());
        self
    }
    pub fn default_options(mut self, value: PlotOptions) -> Self {
        self.default_options = value;
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
    pub fn selection(mut self, value: impl Into<Lotus<Option<String>>>) -> Self {
        self.selection = Some(value.into());
        self
    }
    pub fn on_options_change(mut self, value: impl FnMut(PlotOptions) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_selection_change(mut self, value: impl FnMut(Option<String>) + 'static) -> Self {
        self.selected = Some(Box::new(value));
        self
    }
    pub fn data_table(mut self, value: bool) -> Self {
        self.show_table = value;
        self
    }
    pub fn legend(mut self, value: bool) -> Self {
        self.show_legend = value;
        self
    }
    pub fn pointer_selection(mut self, value: bool) -> Self {
        self.pointer_selection = value;
        self
    }
}
#[derive(Clone)]
struct State {
    owner: glory::core::InstanceId,
    model: Cage<Rc<Model>>,
    options: Cage<PlotOptions>,
    controlled_options: bool,
    selection: Cage<Option<String>>,
    controlled_selection: bool,
    enabled: Lotus<bool>,
    changed: Callback<PlotOptions>,
    selected: Callback<Option<String>>,
}
impl State {
    fn choose(&self, key: Option<String>) {
        if !self.owner.is_alive()
            || !self.enabled.get()
            || key
                .as_ref()
                .is_some_and(|k| self.model.with(|m| !m.marks.contains_key(k)))
            || self.selection.get() == key
        {
            return;
        }
        if !self.controlled_selection {
            self.selection.revise(|old| *old = key.clone());
        }
        self.selected.emit(self.owner, key);
    }
    fn step(&self, forward: bool) {
        let selected = self.selection.get();
        let key = self.model.with(|m| {
            let count = m.scene.marks.len();
            if count == 0 {
                return None;
            }
            let current = selected.as_ref().and_then(|key| m.marks.get(key).copied());
            let i = match current {
                Some(i) => {
                    if forward {
                        (i + 1).min(count - 1)
                    } else {
                        i.saturating_sub(1)
                    }
                }
                None => {
                    if forward {
                        0
                    } else {
                        count - 1
                    }
                }
            };
            Some(m.scene.marks[i].key.clone())
        });
        self.choose(key);
    }
    fn toggle(&self, key: &str) {
        if !self.owner.is_alive() || !self.enabled.get() {
            return;
        }
        let mut next = self.model.with(|m| m.options.clone());
        if !next.hidden.remove(key) {
            next.hidden.insert(key.into());
        }
        self.propose(next);
    }
    fn propose(&self, next: PlotOptions) {
        if !self.owner.is_alive() || !self.enabled.get() {
            return;
        }
        if !self.controlled_options {
            self.options.revise(|old| *old = next.clone());
        }
        self.changed.emit(self.owner, next);
    }
    fn relative(&self, parent: bool) {
        let selected = self.selection.get();
        let key = self.model.with(|m| {
            let selected = selected.as_ref()?;
            let PlotData::Hierarchy(nodes) = &m.data else {
                return None;
            };
            let next = if parent {
                nodes.iter().find(|n| &n.key == selected)?.parent.clone()
            } else {
                nodes
                    .iter()
                    .find(|n| n.parent.as_ref() == Some(selected))
                    .map(|n| n.key.clone())
            };
            next.filter(|k| m.marks.contains_key(k))
        });
        if let Some(key) = key {
            self.choose(Some(key));
        }
    }
}
fn bounded_options(value: &PlotOptions) -> Result<PlotOptions> {
    if value.hidden.len() > MAX_MATRIX_CELLS {
        return Err(invalid(ChartError::Budget));
    }
    if value.hidden.iter().any(|k| !crate::chart_model::key(k)) {
        return Err(invalid(ChartError::Key));
    }
    Ok(value.clone())
}
impl Widget for Plot {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.len() > MAX_PLOT_KEY_BYTES {
            return Err(invalid(ChartError::Key));
        }
        composite::id("plot", &self.key, "plot", "root")?;
        self.frame.validate().map_err(invalid)?;
        let internal = scope.cage(bounded_options(&self.default_options)?);
        let options = self.options.clone().unwrap_or_else(|| internal.into());
        let data = self.data.clone();
        let frame = self.frame;
        let kind = self.kind;
        let palette = self
            .palette
            .clone()
            .unwrap_or_else(|| ChartTheme::from_theme(&self.common.theme));
        let read = move || -> Result<Rc<Model>> {
            let options = options.try_with(bounded_options)??;
            data.try_with(|data| {
                let scene =
                    PlotScene::build(data, kind, frame, &options, &palette).map_err(invalid)?;
                let tables = plot_table::tables(data, &scene);
                let marks = scene
                    .marks
                    .iter()
                    .enumerate()
                    .map(|(i, m)| (m.key.clone(), i))
                    .collect();
                let paths = scene
                    .paths
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (p.key.clone(), i))
                    .collect();
                Ok(Rc::new(Model {
                    data: data.clone(),
                    options,
                    scene,
                    tables,
                    table_revision: 0,
                    marks,
                    paths,
                }))
            })?
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
                    let rev = if previous.tables != next.tables {
                        previous.table_revision.checked_add(1)
                    } else {
                        Some(previous.table_revision)
                    };
                    let Some(rev) = rev else {
                        scope.report_error(invalid(ChartError::Budget));
                        return;
                    };
                    Rc::make_mut(&mut next).table_revision = rev;
                    if *previous != *next {
                        model.revise(|old| *old = next);
                    }
                }
                Err(error) => scope.report_error(error),
            }
        });
        let selection = scope.cage(None);
        if let Some(source) = self.selection.clone() {
            let accepted = composite::validated(scope, &source, |key| {
                if key
                    .as_ref()
                    .is_some_and(|key| !crate::chart_model::key(key))
                {
                    Err(invalid(ChartError::Key))
                } else {
                    Ok(())
                }
            })?;
            scope.structural(move || {
                let value = accepted.get();
                if glory::core::untracked(|| selection.get() != value) {
                    selection.revise(|old| *old = value);
                }
            });
        }
        let state = State {
            owner: scope.id(),
            model,
            options: internal,
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
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-plot",
            composite::column(8.)
        )
        .node_ref(reference);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        build_drawing(&area, &state, frame, self.pointer_selection)?;
        let stops = area
            .bond(move || model.with(|m| (0..m.scene.scale_legend.len()).collect::<Vec<_>>()));
        glory::ui::view()
            .class("sapid-plot-scale-legend")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .gap(12.)
                    .flex_wrap(glory::style::FlexWrap::Wrap),
            )
            .fill(glory::ui::each(
                stops,
                |i| *i,
                move |i, _| {
                    let i = i.get_untracked();
                    glory::ui::widget(move |scope| {
                        glory::ui::graphic()
                            .class("sapid-plot-scale-swatch")
                            .drawing(scope.bond(move || {
                                model.with(|m| {
                                    crate::chart_aux::swatch(m.scene.scale_legend[i].color)
                                })
                            }))
                            .style(Style::new().width(Length::Px(12.)).height(Length::Px(12.)))
                            .show_in(scope)?;
                        glory::ui::text()
                            .content(scope.bond(move || {
                                model.with(|m| m.scene.scale_legend[i].value.to_string())
                            }))
                            .show_in(scope)
                    })
                },
            ))
            .show_in(&area)?;
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
            .class("sapid-plot-controls")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_wrap(glory::style::FlexWrap::Wrap)
                    .gap(8.),
            )
            .node_ref(controls)
            .show_in(&area)?;
        let actions = area.child_in(controls.get().ok_or(glory::ui::Error::Gone)?)?;
        let enabled = self.common.enabled.clone();
        let available =
            area.bond(move || enabled.get() && model.with(|m| !m.scene.marks.is_empty()));
        for (forward, label) in [(false, "Previous data point"), (true, "Next data point")] {
            let state = state.clone();
            let mut button = crate::button()
                .label(label)
                .theme(self.common.theme.clone())
                .enabled(available)
                .on_press(move || state.step(forward));
            if self.common.unstyled {
                button = button.unstyled();
            }
            button.show_in(&actions)?;
        }
        let clear = state.clone();
        let mut button = crate::button()
            .label("Clear data point")
            .theme(self.common.theme.clone())
            .enabled(self.common.enabled.clone())
            .on_press(move || clear.choose(None));
        if self.common.unstyled {
            button = button.unstyled();
        }
        button.show_in(&actions)?;
        if matches!(kind, PlotKind::Treemap | PlotKind::Sunburst) {
            for (parent, label) in [(true, "Parent node"), (false, "First child")] {
                let state = state.clone();
                crate::button()
                    .label(label)
                    .theme(self.common.theme.clone())
                    .enabled(self.common.enabled.clone())
                    .on_press(move || state.relative(parent))
                    .show_in(&actions)?;
            }
        }
        area.retain(actions)?;
        glory::ui::text()
            .class("sapid-plot-detail")
            .content(area.bond(move || {
                model.with(|m| {
                    selection.with(|s| {
                        s.as_ref().and_then(|k| m.marks.get(k)).map_or_else(
                            || "No data point selected".into(),
                            |i| {
                                format!(
                                    "{}: {}",
                                    m.scene.marks[*i].label, m.scene.marks[*i].value_text
                                )
                            },
                        )
                    })
                })
            }))
            .show_in(&area)?;
        glory::ui::text()
            .class("sapid-plot-empty")
            .content(area.bond(move || {
                model.with(|m| {
                    if m.scene.marks.is_empty() {
                        "No visible plot data".into()
                    } else {
                        String::new()
                    }
                })
            }))
            .show_in(&area)?;
        glory::ui::text()
            .class("sapid-plot-clipping")
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
            build_tables(
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
fn build_drawing(
    scope: &Scope,
    state: &State,
    frame: ChartFrame,
    pointer_selection: bool,
) -> Result {
    let reference = scope.cage(None);
    glory::ui::view()
        .class("sapid-plot-drawing")
        .style(
            Style::new()
                .position(Position::Relative)
                .width(Length::Percent(100.))
                .aspect_ratio(AspectRatio::Ratio(frame.width / frame.height)),
        )
        .node_ref(reference)
        .show_in(scope)?;
    let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
    let model = state.model;
    let keys = area.bond(move || {
        model.with(|m| {
            m.scene
                .paths
                .iter()
                .map(|p| p.key.clone())
                .collect::<Vec<_>>()
        })
    });
    glory::ui::each(
        keys,
        |key| key.clone(),
        move |key, _| {
            let key = key.get_untracked();
            glory::ui::widget(move |scope| {
                let key = key.clone();
                glory::ui::graphic()
                    .class("sapid-plot-layer")
                    .drawing(scope.bond(move || {
                        model.with(|m| m.scene.paths[m.paths[&key]].drawing.clone())
                    }))
                    .style(
                        Style::new()
                            .position(Position::Absolute)
                            .top(Length::Px(0.))
                            .left(Length::Px(0.))
                            .width(Length::Percent(100.))
                            .height(Length::Percent(100.)),
                    )
                    .show_in(scope)
            })
        },
    )
    .show_in(&area)?;
    let labels =
        area.bond(move || model.with(|m| (0..m.scene.labels.len()).collect::<Vec<_>>()));
    glory::ui::each(
        labels,
        |i| *i,
        move |index, _| {
            let i = index.get_untracked();
            glory::ui::widget(move |scope| {
                glory::ui::text()
                    .class("sapid-plot-label")
                    .content(scope.bond(move || model.with(|m| m.scene.labels[i].text.clone())))
                    .style(scope.bond(move || {
                        model.with(|m| {
                            let label = &m.scene.labels[i];
                            Style {
                                font_size: Some(11.),
                                ..Style::new()
                                    .position(Position::Absolute)
                                    .left(Length::Percent(
                                        (label.x / frame.width * 100.).clamp(0., 85.),
                                    ))
                                    .top(Length::Percent(
                                        (label.y / frame.height * 100.).clamp(0., 95.),
                                    ))
                                    .width(Length::Percent(15.))
                                    .height(Length::Px(20.))
                                    .overflow_x(Overflow::Hidden)
                                    .overflow_y(Overflow::Hidden)
                            }
                        })
                    }))
                    .show_in(scope)
            })
        },
    )
    .show_in(&area)?;
    let ranges =
        area.bond(move || model.with(|m| (0..m.scene.ranges.len()).collect::<Vec<_>>()));
    glory::ui::each(
        ranges,
        |i| *i,
        move |index, _| {
            let i = index.get_untracked();
            glory::ui::widget(move |scope| {
                glory::ui::view()
                    .class("sapid-plot-range")
                    .role(Role::ProgressBar)
                    .name(scope.bond(move || model.with(|m| m.scene.ranges[i].label.clone())))
                    .semantics(scope.bond(move || {
                        model.with(|m| {
                            Some(Box::new(Semantics {
                                progress: Some(m.scene.ranges[i].value.clone()),
                                ..Default::default()
                            }))
                        })
                    }))
                    .style(scope.bond(move || {
                        model.with(|m| {
                            let r = &m.scene.ranges[i];
                            Style::new()
                                .position(Position::Absolute)
                                .left(Length::Percent(r.x / frame.width * 100.))
                                .top(Length::Percent(r.y / frame.height * 100.))
                                .width(Length::Percent(r.width / frame.width * 100.))
                                .height(Length::Percent(r.height / frame.height * 100.))
                        })
                    }))
                    .show_in(scope)
            })
        },
    )
    .show_in(&area)?;
    let keys = area.bond(move || {
        model.with(|m| {
            m.scene
                .marks
                .iter()
                .map(|m| m.key.clone())
                .collect::<Vec<_>>()
        })
    });
    let state = state.clone();
    glory::ui::each(
        keys,
        |key| key.clone(),
        move |key, _| {
            let key = key.get_untracked();
            let state = state.clone();
            glory::ui::widget(move |scope| {
                let style_key = key.clone();
                let label_key = key.clone();
                let selected = state.selection;
                let activate = key.clone();
                let hover = key.clone();
                let pressed = state.clone();
                let hovered = state.clone();
                let mut button = glory::ui::button()
                    .class("sapid-plot-point")
                    .focus(FocusMode::Programmatic)
                    .enabled(state.enabled.clone())
                    .name(scope.bond(move || {
                        model.with(|m| {
                            let mark = &m.scene.marks[m.marks[&label_key]];
                            format!("{}: {}", mark.label, mark.value_text)
                        })
                    }))
                    .style(scope.bond(move || {
                        model.with(|m| {
                            let mark = &m.scene.marks[m.marks[&style_key]];
                            Style {
                                background: Some(mark.color),
                                position: Some(Position::Absolute),
                                left: Some(Length::Percent(mark.x / frame.width * 100.)),
                                top: Some(Length::Percent(mark.y / frame.height * 100.)),
                                width: Some(Length::Px(10.)),
                                height: Some(Length::Px(10.)),
                                margin: Some(Edges::all(-5.)),
                                padding: Some(Edges::all(0.)),
                                radius: Some(5.),
                                border_width: Some(Edges::all(
                                    if selected.with(|s| s.as_ref() == Some(&style_key)) {
                                        2.
                                    } else {
                                        0.
                                    },
                                )),
                                ..Style::new()
                            }
                        })
                    }))
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
                            hovered.choose(Some(hover.clone()));
                        }
                    });
                }
                button.show_in(scope)
            })
        },
    )
    .show_in(&area)?;
    scope.retain(area)?;
    Ok(())
}
fn build_legend(scope: &Scope, state: &State, theme: crate::Theme, unstyled: bool) -> Result {
    let model = state.model;
    let keys = scope.bond(move || {
        model.with(|m| {
            m.scene
                .legend
                .iter()
                .map(|l| l.key.clone())
                .collect::<Vec<_>>()
        })
    });
    let state = state.clone();
    glory::ui::each(
        keys,
        |k| k.clone(),
        move |key, _| {
            let key = key.get_untracked();
            let state = state.clone();
            let theme = theme.clone();
            glory::ui::widget(move |scope| {
                let label_key = key.clone();
                let color_key = key.clone();
                let toggle = key.clone();
                let state = state.clone();
                let label = scope.bond(move || {
                    model.with(|m| {
                        m.scene
                            .legend
                            .iter()
                            .find(|l| l.key == label_key)
                            .map_or_else(
                                || "Removed plot series".into(),
                                |l| {
                                    format!(
                                        "{} {}",
                                        if m.options.hidden.contains(&label_key) {
                                            "Show"
                                        } else {
                                            "Hide"
                                        },
                                        l.label
                                    )
                                },
                            )
                    })
                });
                let swatch = glory::ui::graphic()
                    .drawing(scope.bond(move || {
                        model.with(|m| {
                            crate::chart_aux::swatch(
                                m.scene
                                    .legend
                                    .iter()
                                    .find(|l| l.key == color_key)
                                    .expect("live plot legend")
                                    .color,
                            )
                        })
                    }))
                    .style(Style::new().width(Length::Px(12.)).height(Length::Px(12.)));
                let mut button = crate::button()
                    .class("sapid-plot-legend-item")
                    .label(label)
                    .left_section(swatch)
                    .theme(theme.clone())
                    .enabled(state.enabled.clone())
                    .on_press(move || state.toggle(&toggle));
                if unstyled {
                    button = button.unstyled();
                }
                button.show_in(scope)
            })
        },
    )
    .show_in(scope)
}
fn build_tables(
    scope: &Scope,
    model: Cage<Rc<Model>>,
    namespace: &str,
    theme: crate::Theme,
    unstyled: bool,
) -> Result {
    let namespace = namespace.to_owned();
    let revisions = scope.bond(move || model.with(|m| vec![m.table_revision]));
    glory::ui::view()
        .class("sapid-plot-tables")
        .style(Style::new().overflow_x(Overflow::Auto))
        .fill(glory::ui::each(
            revisions,
            |r| *r,
            move |_, _| {
                let tables = glory::core::untracked(|| model.with(|m| m.tables.clone()));
                let namespace = namespace.clone();
                let theme = theme.clone();
                glory::ui::widget(move |scope| {
                    for (index, data) in tables.iter().enumerate() {
                        let mut header = crate::table_row("head");
                        for (i, label) in data.columns.iter().enumerate() {
                            header = header
                                .cell(crate::table_header_cell(format!("c{i}")).text(*label));
                        }
                        let mut body = crate::table_body();
                        for (i, values) in data.rows.iter().enumerate() {
                            let mut row = crate::table_row(format!("r{i}"));
                            for (j, value) in values.iter().enumerate() {
                                row = row.cell(if j == 0 {
                                    crate::table_row_header(format!("c{j}")).text(value.clone())
                                } else {
                                    crate::table_cell(format!("c{j}")).text(value.clone())
                                });
                            }
                            body = body.row(row);
                        }
                        let mut table = crate::table(format!("{namespace}-data-{index}"))
                            .caption(data.caption.clone())
                            .theme(theme.clone())
                            .head(crate::table_head().row(header))
                            .body(body);
                        if unstyled {
                            table = table.unstyled();
                        }
                        table.show_in(scope)?;
                    }
                    Ok(())
                })
            },
        ))
        .show_in(scope)
}
