//! Bounded Drawer snap configuration. The host owns pointer capture and pixels;
//! this module owns accepted application state and accessible snap buttons.
use glory::render::{Event, EventKind, Semantics, SheetConfig};
use glory::style::{Align, Color, Display, FlexWrap, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

use crate::composite::{self, Callback};
use crate::state::Control;

#[derive(Clone, Debug, PartialEq)]
pub struct DrawerSnap {
    pub fraction: f64,
    pub label: String,
}
impl DrawerSnap {
    pub fn new(fraction: f64, label: impl Into<String>) -> Self {
        Self {
            fraction,
            label: label.into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct DrawerSnaps {
    pub points: Vec<DrawerSnap>,
    pub max_extent: f64,
    pub settle_millis: u16,
    pub dismissible: bool,
}
impl DrawerSnaps {
    /// Consumes at most nine values; the ninth preserves a build-time budget
    /// error without scanning or retaining the rest of a business iterator.
    pub fn new(points: impl IntoIterator<Item = DrawerSnap>) -> Self {
        Self {
            points: points.into_iter().take(9).collect(),
            max_extent: 640.0,
            settle_millis: 180,
            dismissible: true,
        }
    }
    pub fn max_extent(mut self, value: f64) -> Self {
        self.max_extent = value;
        self
    }
    pub fn settle_millis(mut self, value: u16) -> Self {
        self.settle_millis = value;
        self
    }
    pub fn dismissible(mut self, value: bool) -> Self {
        self.dismissible = value;
        self
    }
    pub fn validate(&self, current: usize) -> Result {
        // Check cardinality before constructing the finite protocol Vec.
        if self.points.is_empty()
            || self.points.len() > glory::render::MAX_SHEET_SNAPS
            || self
                .points
                .iter()
                .any(|p| p.label.trim().is_empty() || p.label.len() > 256)
        {
            return Err(glory::render::Error::InvalidProps(
                "sheet requires 1..=8 nonempty snap labels within 256 bytes",
            )
            .into());
        }
        self.protocol(current, 0).validate()?;
        Ok(())
    }
    fn protocol(&self, current: usize, revision: u64) -> SheetConfig {
        SheetConfig {
            fractions: self.points.iter().map(|p| p.fraction).collect(),
            current,
            max_extent: self.max_extent,
            settle_millis: self.settle_millis,
            dismissible: self.dismissible,
            revision,
        }
    }
}
#[derive(Clone, PartialEq)]
pub(crate) struct Accepted {
    pub options: DrawerSnaps,
    pub protocol: SheetConfig,
}
pub(crate) fn mount(
    scope: &Scope,
    options: &Vary<DrawerSnaps>,
    control: &Control<usize>,
) -> Result<Cage<Accepted>> {
    let initial = control.source.try_get()?;
    let initial = options.try_with(|value| {
        value.validate(initial)?;
        Ok::<_, glory::ui::Error>(Accepted {
            options: value.clone(),
            protocol: value.protocol(initial, 0),
        })
    })??;
    let accepted = scope.cage(initial);
    let options = options.clone();
    let index = control.source.clone();
    let weak = scope.downgrade();
    scope.structural(move || {
        let Some(scope) = weak.upgrade() else { return };
        let result = (|| -> Result {
            let current = index.try_get()?;
            let next = options.try_with(|value| {
                value.validate(current)?;
                Ok::<_, glory::ui::Error>(value.clone())
            })??;
            if accepted.try_with(|old| old.options == next && old.protocol.current == current)? {
                return Ok(());
            }
            let revision = accepted
                .try_with(|old| old.protocol.revision.checked_add(1))?
                .ok_or(glory::render::Error::SheetRevisionExhausted)?;
            accepted.try_revise(|value| {
                *value = Accepted {
                    protocol: next.protocol(current, revision),
                    options: next,
                }
            })?;
            Ok(())
        })();
        if let Err(error) = result {
            scope.report_error(error);
        }
    });
    Ok(accepted)
}

pub(crate) fn controls(
    scope: &Scope,
    accepted: Cage<Accepted>,
    panel: Cage<Option<glory::render::NodeId>>,
    handler: Callback<Event>,
    color: Color,
    unstyled: bool,
) -> Result {
    let mut grip = glory::ui::view()
        .class("sapid-sheet-grip")
        .semantics(composite::metadata(Semantics {
            sheet_grip: true,
            ..Default::default()
        }));
    let mut line = Style::new().width(Length::Px(48.0)).height(Length::Px(4.0));
    if !unstyled {
        line.background = Some(color);
        line.radius = Some(2.0);
    }
    grip = grip
        .style(Style {
            display: Some(Display::Flex),
            align: Some(Align::Center),
            justify: Some(glory::style::Justify::Center),
            min_height: Some(Length::Px(24.0)),
            ..Style::new()
        })
        .fill(glory::ui::view().style(line));
    grip.show_in(scope)?;
    let mut buttons = glory::ui::view().class("sapid-sheet-snaps").style(
        Style::new()
            .display(Display::Flex)
            .flex_wrap(FlexWrap::Wrap)
            .gap(8.0),
    );
    for index in 0..glory::render::MAX_SHEET_SNAPS {
        let owner = scope.id();
        let callback = handler.clone();
        let weak = scope.downgrade();
        let button = glory::ui::button()
            .class("sapid-sheet-snap")
            .label(scope.bond(move || {
                accepted.with(|state| {
                    state
                        .options
                        .points
                        .get(index)
                        .map(|p| p.label.clone())
                        .unwrap_or_default()
                })
            }))
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    pressed: Some(accepted.with(|state| state.protocol.current == index)),
                    ..Default::default()
                })
            }))
            .on(EventKind::Activate, move |_| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    let Some(from) = panel.try_get()? else {
                        return Ok(());
                    };
                    let revision = accepted.try_with(|state| state.protocol.revision)?;
                    callback.emit(
                        owner,
                        Event::Sheet {
                            from,
                            revision,
                            command: glory::render::SheetCommand::Snap(index),
                        },
                    );
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        buttons = buttons.fill(
            glory::ui::view()
                .semantics(scope.bond(move || {
                    composite::metadata(Semantics {
                        hidden: accepted.with(|state| index >= state.options.points.len()),
                        ..Default::default()
                    })
                }))
                .fill(button),
        );
    }
    buttons.show_in(scope)
}
