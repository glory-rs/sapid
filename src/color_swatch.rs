//! A finite RGBA preview, optionally an actual toggle/action button.
use crate::appearance::{Common, configure, observe};
use crate::{composite, state::Value};
use glory::render::{
    Event, EventKind, GraphicPoint as Point, GraphicProps, GraphicStroke, PathCommand as Path,
    Semantics, StrokeCap,
};
use glory::style::{BorderStyle, Color, Display, Edges, Length, Overflow, Position, Style};
use glory::ui::{Result, Widget};
use glory::{Scope, Vary};
pub struct ColorSwatch {
    common: Common,
    color: Vary<Color>,
    side: f64,
    round: bool,
    interactive: bool,
    pressed: Option<Value<bool>>,
    change: Option<Box<dyn FnMut(bool)>>,
    presses: Vec<Box<dyn FnMut()>>,
}
pub fn color_swatch(color: impl Into<Vary<Color>>) -> ColorSwatch {
    ColorSwatch {
        common: Common::default(),
        color: color.into(),
        side: 32.,
        round: false,
        interactive: false,
        pressed: None,
        change: None,
        presses: Vec::new(),
    }
}
impl ColorSwatch {
    pub fn theme(mut self, value: crate::Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn side(mut self, value: f64) -> Self {
        self.side = value;
        self
    }
    pub fn round(mut self, value: bool) -> Self {
        self.round = value;
        self
    }
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn interactive(mut self, value: bool) -> Self {
        self.interactive = value;
        self
    }
    pub fn pressed(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.pressed.get_or_insert_with(Value::default).source = Some(value.into());
        self.interactive = true;
        self
    }
    pub fn default_pressed(mut self, value: bool) -> Self {
        self.pressed.get_or_insert_with(Value::default).default = value;
        self.interactive = true;
        self
    }
    pub fn on_pressed_change(mut self, value: impl FnMut(bool) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self.pressed.get_or_insert_with(Value::default);
        self.interactive = true;
        self
    }
    pub fn on_press(mut self, value: impl FnMut() + 'static) -> Self {
        self.presses.push(Box::new(value));
        self.interactive = true;
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: glory::Cage<Option<glory::render::NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on(mut self, kind: EventKind, value: impl FnMut(Event) + 'static) -> Self {
        if kind == EventKind::Activate {
            self.interactive = true
        }
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(value));
        self
    }
}
fn square(commands: &mut Vec<Path>, x: f64, y: f64, side: f64) {
    commands.extend([
        Path::Move(Point::new(x, y)),
        Path::Line(Point::new(x + side, y)),
        Path::Line(Point::new(x + side, y + side)),
        Path::Line(Point::new(x, y + side)),
        Path::Close,
    ]);
}
fn checker(color: Color) -> GraphicProps {
    let mut commands = Vec::with_capacity(40);
    for y in 0..4 {
        for x in 0..4 {
            if (x + y) % 2 == 0 {
                square(&mut commands, x as f64 * 4., y as f64 * 4., 4.)
            }
        }
    }
    GraphicProps {
        width: 16.,
        height: 16.,
        commands,
        fill: Some(color),
        ..Default::default()
    }
}
fn full(color: Color) -> GraphicProps {
    let mut commands = Vec::with_capacity(5);
    square(&mut commands, 0., 0., 16.);
    GraphicProps {
        width: 16.,
        height: 16.,
        commands,
        fill: Some(color),
        ..Default::default()
    }
}
fn mark(color: Color, backdrop: Color, light_mark: Color, dark_mark: Color) -> GraphicProps {
    let Color(r, g, b, a) = color;
    let opaque = |v: u8, b: u8| {
        ((u32::from(v) * u32::from(a) + u32::from(b) * (255 - u32::from(a))) / 255) as u8
    };
    let light = u32::from(opaque(r, backdrop.0)) * 299
        + u32::from(opaque(g, backdrop.1)) * 587
        + u32::from(opaque(b, backdrop.2)) * 114
        >= 128_000;
    GraphicProps {
        width: 16.,
        height: 16.,
        commands: vec![
            Path::Move(Point::new(3., 8.)),
            Path::Line(Point::new(6.5, 11.5)),
            Path::Line(Point::new(13., 4.5)),
        ],
        stroke: Some(GraphicStroke {
            color: if light { light_mark } else { dark_mark },
            width: 2.,
            cap: StrokeCap::Round,
        }),
        ..Default::default()
    }
}
fn canonical_name(value: &str) -> Result {
    if value.trim().is_empty() || value.len() > 256 || value.contains(['\0', '\r', '\n']) {
        Err(glory::render::Error::InvalidProps(
            "swatch names require nonempty canonical single-line text within 256 bytes",
        )
        .into())
    } else {
        Ok(())
    }
}
impl Widget for ColorSwatch {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.side.is_finite() || !(8.0..=128.0).contains(&self.side) {
            return Err(glory::render::Error::InvalidProps(
                "swatch side must be finite in 8..=128 pixels",
            )
            .into());
        }
        if !self.interactive && self.pressed.is_some() {
            return Err(glory::render::Error::InvalidProps(
                "pressed swatches require interactive mode",
            )
            .into());
        }
        let color = self.color.clone();
        let name = if let Some(name) = self.common.name.take() {
            Some(composite::validated(scope, &name, |value| canonical_name(value))?.into())
        } else if self.interactive {
            Some(
                scope
                    .bond(move || {
                        let Color(r, g, b, a) = color.get();
                        format!("Color #{r:02x}{g:02x}{b:02x}{a:02x}")
                    })
                    .into(),
            )
        } else {
            None
        };
        self.common.name = name;
        let pressed = self.pressed.take().map(|value| value.mount(scope));
        let checked = pressed.as_ref().map(|control| control.source.clone());
        let side = self.side;
        let radius = if self.round {
            side / 2.
        } else {
            self.common.theme.radius
        };
        let mut base = Style::new()
            .display(Display::InlineFlex)
            .width(Length::Px(side))
            .height(Length::Px(side))
            .min_width(Length::Px(side))
            .padding(Edges::all(0.))
            .position(Position::Relative)
            .overflow_x(Overflow::Hidden)
            .overflow_y(Overflow::Hidden)
            .radius(radius)
            .border_width(Edges::all(1.))
            .border_style(BorderStyle::Solid)
            .border_color(self.common.theme.border);
        base.background = Some(self.common.theme.checker_light);
        base.shrink = Some(0.);
        // Structural geometry remains in unstyled mode; the preview is the content.
        let mut structural = Style::new()
            .position(Position::Relative)
            .width(Length::Px(side))
            .height(Length::Px(side))
            .min_width(Length::Px(side))
            .padding(Edges::all(0.))
            .overflow_x(Overflow::Hidden)
            .overflow_y(Overflow::Hidden);
        structural.background = Some(self.common.theme.checker_light);
        if self.round {
            structural.radius = Some(side / 2.)
        }
        let custom = self.common.style.clone();
        self.common.style = scope
            .bond(move || crate::appearance::merge(structural.clone(), custom.get()))
            .into();
        let layer = Style::new()
            .position(Position::Absolute)
            .left(Length::Px(0.))
            .top(Length::Px(0.))
            .width(Length::Percent(100.))
            .height(Length::Percent(100.))
            .pointer_events(glory::style::PointerEvents::None);
        let backdrop = self.common.theme.checker_light;
        let light_mark = self.common.theme.swatch_mark_light;
        let dark_mark = self.common.theme.swatch_mark_dark;
        let color = self.color.clone();
        let foreground = glory::ui::graphic()
            .drawing(scope.bond(move || full(color.get())))
            .class("sapid-color-swatch-color")
            .style(layer.clone());
        let check = self.color.clone();
        let checked_mark = glory::ui::graphic()
            .class("sapid-color-swatch-check")
            .style(layer.clone())
            .drawing(scope.bond(move || {
                if checked.as_ref().is_some_and(|source| source.get()) {
                    mark(check.get(), backdrop, light_mark, dark_mark)
                } else {
                    GraphicProps {
                        width: 16.,
                        height: 16.,
                        ..Default::default()
                    }
                }
            }));
        let background = glory::ui::graphic()
            .drawing(checker(self.common.theme.checker_dark))
            .class("sapid-color-swatch-checker")
            .style(layer);
        let owner = scope.id();
        if self.interactive {
            let checked = pressed.as_ref().map(|control| control.source.clone());
            let mut root = configure!(
                self.common,
                glory::ui::button(),
                scope,
                "sapid-color-swatch",
                base
            )
            .enabled(self.common.enabled.clone())
            .label("")
            .semantics(scope.bond(move || {
                Some(Box::new(Semantics {
                    pressed: checked.as_ref().map(|value| value.get()),
                    ..Default::default()
                }))
            }))
            .fill((background, foreground, checked_mark));
            let weak = scope.downgrade();
            let mut change = self.change.take();
            let mut presses = std::mem::take(&mut self.presses);
            let mut observers = self
                .common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default();
            root = root.on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if let Some(control) = &pressed {
                        let next = !control.source.try_get()?;
                        control.accept(&next)?;
                        if let Some(callback) = &mut change {
                            callback(next)
                        }
                    }
                    for callback in &mut presses {
                        if !owner.is_alive() {
                            return Ok(());
                        }
                        callback()
                    }
                    observe(&mut observers, owner, event);
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error)
                }
            });
            for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
                root = root.on(kind, move |event| observe(&mut callbacks, owner, event))
            }
            root.show_in(scope)
        } else {
            let mut root = configure!(
                self.common,
                glory::ui::view(),
                scope,
                "sapid-color-swatch",
                base
            )
            .fill((background, foreground, checked_mark));
            for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
                root = root.on(kind, move |event| observe(&mut callbacks, owner, event))
            }
            root.show_in(scope)
        }
    }
}
