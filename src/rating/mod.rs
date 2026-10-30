//! Finite whole/half-star ratings backed by native radio controls.
mod model;
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use crate::state::Value;
use glory::render::{
    Direction, Event, EventKind, FocusMode, GraphicPoint, GraphicProps, GraphicStroke, Orientation,
    PathCommand,
};
use glory::style::{BorderStyle, Edges, Length, Overflow, PointerEvents, Position, Style};
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};
pub use model::{MAX_RATING_COUNT, RatingPrecision};

pub struct Rating {
    key: String,
    common: Common,
    count: u8,
    precision: RatingPrecision,
    clearable: bool,
    value: Value<f64>,
    form_name: Option<String>,
    field: Option<crate::FieldControl>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
    direction: Direction,
    on_change: Option<Box<dyn FnMut(f64)>>,
    display_only: bool,
}
pub fn rating(key: impl Into<String>) -> Rating {
    Rating {
        key: key.into(),
        common: Common::default(),
        count: 5,
        precision: RatingPrecision::Whole,
        clearable: false,
        value: Value::default(),
        form_name: None,
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        direction: Direction::Ltr,
        on_change: None,
        display_only: false,
    }
}
impl Rating {
    common_methods!();
    /// Render a named passive group and graphics without radio or form controls.
    pub fn display_only(mut self) -> Self {
        self.display_only = true;
        self
    }
    pub fn count(mut self, value: u8) -> Self {
        self.count = value;
        self
    }
    pub fn precision(mut self, value: RatingPrecision) -> Self {
        self.precision = value;
        self
    }
    /// Add an explicit, named zero-value radio; selecting a checked star never clears implicitly.
    pub fn clearable(mut self, value: bool) -> Self {
        self.clearable = value;
        self
    }
    pub fn value(mut self, value: impl Into<Vary<f64>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: f64) -> Self {
        self.value.default = value;
        self
    }
    pub fn form_name(mut self, value: impl Into<String>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn required(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn field(mut self, value: crate::FieldControl) -> Self {
        self.common.theme = value.theme.clone();
        self.common.size = value.size;
        self.common.unstyled = value.unstyled;
        self.field = Some(value);
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(value));
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn star(fill: glory::style::Color, stroke: glory::style::Color) -> GraphicProps {
    let mut commands = Vec::with_capacity(11);
    for index in 0..10 {
        let theta = std::f64::consts::TAU * f64::from(index) / 10.;
        let radius = if index % 2 == 0 { 43. } else { 19. };
        let point = GraphicPoint::new(50. + radius * theta.sin(), 50. - radius * theta.cos());
        commands.push(if index == 0 {
            PathCommand::Move(point)
        } else {
            PathCommand::Line(point)
        });
    }
    commands.push(PathCommand::Close);
    GraphicProps {
        width: 100.,
        height: 100.,
        commands,
        fill: Some(fill),
        stroke: Some(GraphicStroke {
            color: stroke,
            width: 2.,
            cap: Default::default(),
        }),
        ..Default::default()
    }
}
impl Widget for Rating {
    fn build(&mut self, scope: &Scope) -> Result {
        let domain = model::Domain {
            count: self.count,
            precision: self.precision,
        };
        domain.validate().map_err(invalid)?;
        if self.display_only
            && (self.field.is_some() || self.form_name.is_some() || self.on_change.is_some())
        {
            return Err(invalid(
                "display-only ratings do not register Field/form controls or change callbacks",
            ));
        }
        composite::id("rating", &self.key, "root", "root")?;
        if self.common.focus != FocusMode::Auto {
            return Err(invalid("rating owns its radio tab stops"));
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let accepted = composite::validated(scope, &control.source, move |value| {
            domain.units(*value).map(|_| ()).map_err(invalid)
        })?;
        let clearable = self.clearable;
        let key_source = scope.bond(move || {
            let value = accepted.get();
            if value == 0. {
                if clearable {
                    String::from("rating-0")
                } else {
                    String::new()
                }
            } else {
                format!("rating-{}", domain.units(value).expect("accepted rating"))
            }
        });
        let named = self.common.name.is_some();
        let name = self.common.name.take().unwrap_or_else(|| "Rating".into());
        let label = composite::validated(scope, &name, |value| {
            if value.trim().is_empty() || value.len() > 512 || value.contains('\0') {
                Err(invalid("rating name must be 1..=512 bytes without NUL"))
            } else {
                Ok(())
            }
        })?;
        if self.display_only {
            return self.build_display(scope, accepted, label, domain);
        }
        let custom = self.common.class.clone();
        let classes = scope.bond(move || format!("sapid-rating {}", custom.get()));
        let callback = Callback::new(self.on_change.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let owner = scope.id();
        let weak = scope.downgrade();
        let mut group = crate::radio_group(self.key.clone())
            .label(label)
            .class(classes)
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .enabled(self.common.enabled.clone())
            .style(self.common.style.clone())
            .value(key_source)
            .orientation(Orientation::Horizontal)
            .direction(self.direction)
            .required(self.required.clone())
            .readonly(self.readonly.clone())
            .invalid(self.invalid.clone())
            .member_gap(0.)
            .on_value_change(move |key| {
                if let Some(unit) = key
                    .strip_prefix("rating-")
                    .and_then(|v| v.parse::<u8>().ok())
                    && let Some(value) = domain.value(unit)
                {
                    match control.accept(&value) {
                        Ok(()) => callback.emit(owner, value),
                        Err(error) => {
                            if let Some(scope) = weak.upgrade() {
                                scope.report_error(error);
                            }
                        }
                    }
                }
            });
        if self.field.is_some() && !named {
            group = group.label("");
        } else if named {
            group = group.name(label);
        }
        if self.common.unstyled {
            group = group.unstyled();
        }
        if let Some(field) = self.field.clone() {
            group = group.field(field);
        }
        if let Some(name) = self.form_name.take() {
            group = group.form_name(name);
        }
        if let Some(reference) = self.common.node_ref {
            group = group.node_ref(reference);
        }
        let checked_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::CheckedChanged)
                .unwrap_or_default(),
        );
        let cleanup = checked_observers.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            for handler in handlers {
                group = group.on(kind, handler);
            }
        }
        let side = match self.common.size {
            crate::Size::ExtraSmall => 20.,
            crate::Size::Small => 24.,
            crate::Size::Medium => 30.,
            crate::Size::Large => 36.,
            crate::Size::ExtraLarge => 44.,
        };
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let root_enabled = self
            .field
            .as_ref()
            .map(|field| field.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let denominator = self.precision.denominator();
        let count = domain.option_count(clearable).map_err(invalid)?;
        for index in 0..count {
            let unit = if clearable {
                index as u8
            } else {
                index as u8 + 1
            };
            let value = domain
                .value(unit)
                .ok_or_else(|| invalid("invalid rating unit"))?;
            let clear = unit == 0;
            let width = if denominator == 2 && !clear {
                side / 2.
            } else {
                side
            };
            let offset = if denominator == 2 && unit % 2 == 0 && !clear {
                -side / 2.
            } else {
                0.
            };
            let theme = theme.clone();
            let enabled = root_enabled.clone();
            let foreground = theme.foreground;
            let observers = checked_observers.clone();
            let item = crate::radio(format!("rating-{unit}"))
                .form_value(value.to_string())
                .on(EventKind::CheckedChanged, move |event| {
                    observers.emit(owner, event)
                })
                .name(if clear {
                    String::from("Clear rating")
                } else {
                    format!("{value} of {}", domain.count)
                })
                .visual(move |control| {
                    let mut control = Some(control);
                    Box::new(glory::ui::widget(move |scope| {
                        let focused = scope.cage(false);
                        let enabled = enabled.clone();
                        let mut wrapper = glory::ui::view()
                            .class(if clear {
                                "sapid-rating-clear"
                            } else {
                                "sapid-rating-part"
                            })
                            .style(scope.bond(move || {
                                Style::new()
                                    .position(Position::Relative)
                                    .width(Length::Px(width))
                                    .height(Length::Px(side))
                                    .overflow_x(Overflow::Clip)
                                    .overflow_y(Overflow::Clip)
                                    .opacity(if enabled.get() { 1. } else { 0.45 })
                            }))
                            .on(EventKind::FocusWithin, move |event| {
                                if let Event::FocusWithin { focused: next, .. } = event {
                                    focused.revise(|v| *v = next)
                                }
                            });
                        if clear {
                            wrapper = wrapper.fill(glory::ui::text().content("×").style(Style {
                                color: Some(foreground),
                                font_size: Some(side * 0.8),
                                ..Default::default()
                            }));
                        } else {
                            let theme = theme.clone();
                            wrapper = wrapper.fill(
                                glory::ui::graphic()
                                    .class("sapid-rating-star")
                                    .style(
                                        Style::new()
                                            .position(Position::Absolute)
                                            .left(Length::Px(offset))
                                            .top(Length::Px(0.))
                                            .width(Length::Px(side))
                                            .height(Length::Px(side))
                                            .pointer_events(PointerEvents::None),
                                    )
                                    .drawing(scope.bond(move || {
                                        star(
                                            if accepted.get() >= value {
                                                if unstyled {
                                                    theme.foreground
                                                } else {
                                                    theme.accent
                                                }
                                            } else {
                                                if unstyled {
                                                    glory::style::Color(0, 0, 0, 0)
                                                } else {
                                                    theme.surface
                                                }
                                            },
                                            if unstyled {
                                                theme.foreground
                                            } else {
                                                theme.border
                                            },
                                        )
                                    })),
                            );
                        }
                        let indicator =
                            glory::ui::view()
                                .class("sapid-rating-focus")
                                .style(scope.bond(move || {
                                    Style::new()
                                        .position(Position::Absolute)
                                        .left(Length::Px(0.))
                                        .top(Length::Px(0.))
                                        .width(Length::Px(width))
                                        .height(Length::Px(side))
                                        .pointer_events(PointerEvents::None)
                                        .border_width(Edges::all(if focused.get() {
                                            2.
                                        } else {
                                            0.
                                        }))
                                        .border_style(BorderStyle::Solid)
                                        .border_color(foreground)
                                }));
                        wrapper
                            .fill(indicator)
                            .fill(
                                control.take().ok_or(glory::ui::Error::Gone)?.style(
                                    Style::new()
                                        .position(Position::Absolute)
                                        .left(Length::Px(0.))
                                        .top(Length::Px(0.))
                                        .width(Length::Px(width))
                                        .height(Length::Px(side))
                                        .opacity(0.),
                                ),
                            )
                            .show_in(scope)
                    }))
                });
            group = group.radio(item);
        }
        group.build(scope)
    }
}

impl Rating {
    fn build_display(
        &mut self,
        scope: &Scope,
        accepted: glory::Cage<f64>,
        label: glory::Cage<String>,
        domain: model::Domain,
    ) -> Result {
        use glory::style::{Display, FlexDirection};
        let side = match self.common.size {
            crate::Size::ExtraSmall => 20.,
            crate::Size::Small => 24.,
            crate::Size::Medium => 30.,
            crate::Size::Large => 36.,
            crate::Size::ExtraLarge => 44.,
        };
        let (classes, styles) = self.common.appearance(
            scope,
            "sapid-rating sapid-rating-display",
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .gap(0.),
        );
        let enabled = self.common.enabled.clone();
        let geometry = scope.bond(move || {
            let mut style = styles.get();
            style.display.get_or_insert(Display::Flex);
            style.flex_direction.get_or_insert(FlexDirection::Row);
            style.gap.get_or_insert(0.);
            style
                .opacity
                .get_or_insert(if enabled.get() { 1. } else { 0.45 });
            style
        });
        let mut children: Vec<Box<dyn Widget>> =
            Vec::with_capacity(domain.option_count(false).map_err(invalid)?);
        for unit in 1..=domain.count * domain.precision.denominator() {
            let half = domain.precision == RatingPrecision::Half;
            let width = if half { side / 2. } else { side };
            let offset = if half && unit % 2 == 0 {
                -side / 2.
            } else {
                0.
            };
            let theme = self.common.theme.clone();
            let unstyled = self.common.unstyled;
            let threshold = domain
                .value(unit)
                .ok_or_else(|| invalid("invalid display rating unit"))?;
            let graphic = glory::ui::graphic()
                .class("sapid-rating-star")
                .style(
                    Style::new()
                        .position(Position::Absolute)
                        .left(Length::Px(offset))
                        .top(Length::Px(0.))
                        .width(Length::Px(side))
                        .height(Length::Px(side)),
                )
                .drawing(scope.bond(move || {
                    star(
                        if accepted.get() >= threshold {
                            if unstyled {
                                theme.foreground
                            } else {
                                theme.accent
                            }
                        } else if unstyled {
                            glory::style::Color(0, 0, 0, 0)
                        } else {
                            theme.surface
                        },
                        if unstyled {
                            theme.foreground
                        } else {
                            theme.border
                        },
                    )
                }));
            children.push(Box::new(
                glory::ui::view()
                    .class("sapid-rating-display-part")
                    .style(
                        Style::new()
                            .position(Position::Relative)
                            .width(Length::Px(width))
                            .height(Length::Px(side))
                            .overflow_x(Overflow::Clip)
                            .overflow_y(Overflow::Clip),
                    )
                    .fill(graphic),
            ));
        }
        let count = domain.count;
        let mut root = glory::ui::view()
            .role(glory::render::Role::Group)
            .class(classes)
            .style(geometry)
            .name(scope.bond(move || {
                let value = accepted.get();
                format!(
                    "{}: {} of {count}",
                    label.get(),
                    if value == 0. { 0. } else { value }
                )
            }))
            .fill(children);
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference)
        }
        let owner = scope.id();
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            })
        }
        root.show_in(scope)
    }
}
