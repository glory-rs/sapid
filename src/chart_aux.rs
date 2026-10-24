//! Bounded standalone chart legend and an actual anchored semantic tooltip.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::charts::{ChartError, ChartPointDetails, MAX_CHART_SERIES, MAX_CHART_VALUE};
use crate::composite::{self, Callback};
use glory::render::{Event, EventKind, GraphicPoint, GraphicProps, PathCommand};
use glory::style::{Color, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Lotus, Scope};
use std::{collections::BTreeSet, rc::Rc};
#[derive(Clone, Debug, PartialEq)]
pub struct LegendItem {
    pub key: String,
    pub label: String,
    pub color: Color,
}
pub(crate) fn swatch(color: Color) -> GraphicProps {
    use PathCommand::{Close, Line, Move};
    GraphicProps {
        width: 12.,
        height: 12.,
        fill: Some(color),
        commands: vec![
            Move(GraphicPoint::new(0., 0.)),
            Line(GraphicPoint::new(12., 0.)),
            Line(GraphicPoint::new(12., 12.)),
            Line(GraphicPoint::new(0., 12.)),
            Close,
        ],
        ..Default::default()
    }
}
#[derive(Clone, Debug, PartialEq)]
struct LegendModel {
    items: Vec<LegendItem>,
    hidden: BTreeSet<String>,
}
fn invalid(error: ChartError) -> glory::ui::Error {
    glory::render::Error::InvalidProps(error.message()).into()
}
fn admit(items: &[LegendItem], hidden: &BTreeSet<String>) -> Result<Rc<LegendModel>> {
    if items.len() > MAX_CHART_SERIES || hidden.len() > MAX_CHART_SERIES {
        return Err(invalid(ChartError::Budget));
    }
    let mut keys = BTreeSet::new();
    for item in items {
        if !crate::chart_model::key(&item.key) || !keys.insert(&item.key) {
            return Err(invalid(ChartError::Key));
        }
        if !crate::chart_model::label(&item.label) {
            return Err(invalid(ChartError::Label));
        }
    }
    if hidden.iter().any(|key| !keys.contains(key)) {
        return Err(invalid(ChartError::Key));
    }
    Ok(Rc::new(LegendModel {
        items: items.to_vec(),
        hidden: hidden.clone(),
    }))
}
pub struct Legend {
    common: Common,
    items: Lotus<Vec<LegendItem>>,
    hidden: Option<Lotus<BTreeSet<String>>>,
    default_hidden: BTreeSet<String>,
    change: Option<Box<dyn FnMut(BTreeSet<String>)>>,
}
pub fn legend() -> Legend {
    Legend {
        common: Common::default(),
        items: Vec::new().into(),
        hidden: None,
        default_hidden: BTreeSet::new(),
        change: None,
    }
}
impl Legend {
    common_methods!();
    pub fn items(mut self, value: impl Into<Lotus<Vec<LegendItem>>>) -> Self {
        self.items = value.into();
        self
    }
    pub fn hidden(mut self, value: impl Into<Lotus<BTreeSet<String>>>) -> Self {
        self.hidden = Some(value.into());
        self
    }
    pub fn default_hidden(mut self, value: BTreeSet<String>) -> Self {
        self.default_hidden = value;
        self
    }
    pub fn on_hidden_change(mut self, value: impl FnMut(BTreeSet<String>) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self
    }
}
impl Widget for Legend {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.default_hidden.len() > MAX_CHART_SERIES
            || self.default_hidden.iter().any(|key| key.len() > 64)
        {
            return Err(invalid(ChartError::Budget));
        }
        let internal = scope.cage(self.default_hidden.clone());
        let controlled = self.hidden.is_some();
        let hidden = self.hidden.clone().unwrap_or_else(|| internal.into());
        let items = self.items.clone();
        let read = move || -> Result<Rc<LegendModel>> {
            items.try_with(|items| hidden.try_with(|hidden| admit(items, hidden)))??
        };
        let model = scope.cage(glory::core::untracked(&read)?);
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else {
                return;
            };
            match read() {
                Ok(next) => {
                    if glory::core::untracked(|| model.with(|old| **old != *next)) {
                        model.revise(|old| *old = next);
                    }
                }
                Err(e) => scope.report_error(e),
            }
        });
        let callback = Callback::new(self.change.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-chart-legend",
            composite::column(4.)
        )
        .node_ref(reference);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        let keys = area.bond(move || {
            model.with(|m| m.items.iter().map(|i| i.key.clone()).collect::<Vec<_>>())
        });
        let enabled = self.common.enabled.clone();
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let owner = scope.id();
        glory::ui::each(
            keys,
            |key| key.clone(),
            move |key, _| {
                let key = key.get_untracked();
                let theme = theme.clone();
                let callback = callback.clone();
                let enabled = enabled.clone();
                glory::ui::widget(move |scope| {
                    let label_key = key.clone();
                    let color_key = key.clone();
                    let toggle = key.clone();
                    let callback = callback.clone();
                    let enabled_event = enabled.clone();
                    let label = scope.bond(move || {
                        model.with(|m| {
                            m.items.iter().find(|i| i.key == label_key).map_or_else(
                                || "Removed chart series".into(),
                                |i| {
                                    format!(
                                        "{} {}",
                                        if m.hidden.contains(&i.key) {
                                            "Show"
                                        } else {
                                            "Hide"
                                        },
                                        i.label
                                    )
                                },
                            )
                        })
                    });
                    let swatch = glory::ui::graphic()
                        .class("sapid-chart-legend-swatch")
                        .drawing(scope.bond(move || {
                            model.with(|m| {
                                self::swatch(
                                    m.items
                                        .iter()
                                        .find(|i| i.key == color_key)
                                        .expect("live legend item")
                                        .color,
                                )
                            })
                        }))
                        .style(Style::new().width(Length::Px(12.)).height(Length::Px(12.)));
                    let mut button = crate::button()
                        .theme(theme.clone())
                        .label(label)
                        .left_section(swatch)
                        .enabled(enabled.clone())
                        .class("sapid-chart-legend-item")
                        .on_press(move || {
                            if !owner.is_alive() || !enabled_event.get() {
                                return;
                            }
                            let next = model.with(|m| {
                                let mut next = m.hidden.clone();
                                if !next.remove(&toggle) {
                                    next.insert(toggle.clone());
                                }
                                next
                            });
                            if !controlled {
                                internal.revise(|old| *old = next.clone());
                            }
                            callback.emit(owner, next);
                        });
                    if unstyled {
                        button = button.unstyled();
                    }
                    button.show_in(scope)
                })
            },
        )
        .show_in(&area)?;
        scope.retain(area)?;
        Ok(())
    }
}

/// Chart-specific content over the existing real tooltip/focus/owner protocol.
/// No point disables the trigger's tooltip relation, rather than displaying an
/// invented zero value. Native hosts retain their explicit Tooltip limitation.
pub struct Tooltip {
    inner: Option<crate::Tooltip>,
    point: Lotus<Option<ChartPointDetails>>,
    enabled: Lotus<bool>,
}
pub fn tooltip(key: impl Into<String>) -> Tooltip {
    Tooltip {
        inner: Some(crate::tooltip(key)),
        point: None.into(),
        enabled: true.into(),
    }
}
impl Tooltip {
    pub fn point(mut self, value: impl Into<Lotus<Option<ChartPointDetails>>>) -> Self {
        self.point = value.into();
        self
    }
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.enabled = value.into();
        self
    }
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.inner = self.inner.map(|inner| inner.theme(value));
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.inner = self.inner.map(crate::Tooltip::unstyled);
        self
    }
    pub fn label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.inner = self.inner.map(|inner| inner.label(value));
        self
    }
    pub fn open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.inner = self.inner.map(|inner| inner.open(value));
        self
    }
    pub fn target<W: Widget + 'static>(
        mut self,
        factory: impl FnMut(crate::FloatingTarget) -> W + 'static,
    ) -> Self {
        self.inner = self.inner.map(|inner| inner.target(factory));
        self
    }
    pub fn on(mut self, kind: EventKind, callback: impl FnMut(Event) + 'static) -> Self {
        self.inner = self.inner.map(|inner| inner.on(kind, callback));
        self
    }
}
impl Widget for Tooltip {
    fn build(&mut self, scope: &Scope) -> Result {
        let accepted = composite::validated(scope, &self.point, |point| {
            if let Some(point) = point {
                if !crate::chart_model::key(&point.selection.series)
                    || !crate::chart_model::key(&point.selection.category)
                {
                    return Err(invalid(ChartError::Key));
                }
                if !crate::chart_model::label(&point.series_label)
                    || !crate::chart_model::label(&point.category_label)
                {
                    return Err(invalid(ChartError::Label));
                }
                if !point.value.is_finite() || point.value.abs() > MAX_CHART_VALUE {
                    return Err(invalid(ChartError::Values));
                }
            }
            Ok(())
        })?;
        let enabled = self.enabled.clone();
        self.inner
            .take()
            .ok_or(glory::ui::Error::Gone)?
            .text(scope.bond(move || {
                accepted.with(|p| {
                    p.as_ref()
                        .map_or_else(|| "No chart data point".into(), ChartPointDetails::text)
                })
            }))
            .enabled(scope.bond(move || enabled.get() && accepted.with(Option::is_some)))
            .show_in(scope)
    }
}
