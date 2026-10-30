//! Read-only numeric progress, with shared finite line and Bezier views.
use crate::appearance::{Common, configure, merge};
use glory::render::{
    GraphicPoint as Point, GraphicProps, GraphicStroke, NodeId, PathCommand as Path, ProgressValue,
    Role, Semantics, StrokeCap,
};
use glory::style::{Color, Display, Length, Overflow, Position, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::f64::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Line,
    Ring,
    SemiCircle,
}
pub struct Progress {
    common: Common,
    view: View,
    value: Vary<f64>,
    indeterminate: Vary<bool>,
    value_text: Option<Vary<String>>,
    min: f64,
    max: f64,
    size: f64,
    thickness: f64,
    width: Option<Length>,
}
fn create(view: View, value: impl Into<Vary<f64>>) -> Progress {
    let common = Common {
        name: Some("Progress".into()),
        ..Common::default()
    };
    Progress {
        common,
        view,
        value: value.into(),
        indeterminate: false.into(),
        value_text: None,
        min: 0.,
        max: 100.,
        size: 100.,
        thickness: 8.,
        width: None,
    }
}
pub fn progress(value: impl Into<Vary<f64>>) -> Progress {
    create(View::Line, value)
}
pub fn ring_progress(value: impl Into<Vary<f64>>) -> Progress {
    create(View::Ring, value)
}
pub fn semi_circle_progress(value: impl Into<Vary<f64>>) -> Progress {
    create(View::SemiCircle, value)
}
impl Progress {
    pub fn theme(mut self, theme: crate::Theme) -> Self {
        self.common.theme = theme;
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn class(mut self, class: impl Into<Vary<String>>) -> Self {
        self.common.class = class.into();
        self
    }
    pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
        self.common.style = style.into();
        self
    }
    pub fn node_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(reference);
        self
    }
    pub fn name(mut self, name: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(name.into());
        self
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn indeterminate(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.indeterminate = value.into();
        self
    }
    pub fn value_text(mut self, text: impl Into<Vary<String>>) -> Self {
        self.value_text = Some(text.into());
        self
    }
    /// Circular diameter in pixels; for a line this explicitly sets its width.
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self.width = Some(Length::Px(size));
        self
    }
    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }
    pub fn thickness(mut self, thickness: f64) -> Self {
        self.thickness = thickness;
        self
    }
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn validate_name(name: &str) -> Result {
    if name.trim().is_empty() || name.len() > 256 {
        return Err(invalid(
            "progress name must be nonempty and at most 256 UTF-8 bytes",
        ));
    }
    Ok(())
}
fn selection(
    value: &Vary<f64>,
    indeterminate: &Vary<bool>,
    text: Option<&Vary<String>>,
    min: f64,
    max: f64,
) -> Result<ProgressValue> {
    let value = value.try_get()?;
    glory::render::RangeValue { min, max, value }.validate()?;
    let current = if indeterminate.try_get()? {
        None
    } else {
        Some(value)
    };
    let value_text = text
        .map(|text| {
            text.try_with(|text| {
                if text.trim().is_empty() || text.len() > 1024 {
                    return Err(invalid(
                        "progress text must be nonempty and at most 1024 UTF-8 bytes",
                    ));
                }
                Ok(text.clone())
            })
        })
        .transpose()?
        .transpose()?;
    let result = ProgressValue {
        min,
        max,
        current,
        value_text,
    };
    result.validate()?;
    Ok(result)
}
pub(crate) fn arc(
    size: f64,
    thickness: f64,
    semi: bool,
    fraction: f64,
    color: Color,
) -> GraphicProps {
    let radius = (size - thickness) / 2.0;
    let center = size / 2.;
    let start = if semi { PI } else { -PI / 2. };
    let sweep = if semi { PI } else { TAU };
    let mut commands = Vec::with_capacity(6);
    if fraction > 0. {
        let segments = (fraction * sweep / (PI / 2.)).ceil() as usize;
        let step = fraction * sweep / segments as f64;
        let point =
            |angle: f64| Point::new(center + radius * angle.cos(), center + radius * angle.sin());
        commands.push(Path::Move(point(start)));
        for index in 0..segments {
            let a = start + step * index as f64;
            let b = a + step;
            let factor = 4. / 3. * (step / 4.).tan();
            let p = point(a);
            let q = point(b);
            commands.push(Path::Cubic {
                first: Point::new(
                    p.x - factor * radius * a.sin(),
                    p.y + factor * radius * a.cos(),
                ),
                second: Point::new(
                    q.x + factor * radius * b.sin(),
                    q.y - factor * radius * b.cos(),
                ),
                end: q,
            });
        }
        if fraction == 1. && !semi {
            commands.push(Path::Close);
        }
    }
    GraphicProps {
        width: size,
        height: if semi {
            size / 2. + thickness / 2.
        } else {
            size
        },
        commands,
        stroke: Some(GraphicStroke {
            color,
            width: thickness,
            cap: StrokeCap::Round,
        }),
        ..Default::default()
    }
}
impl Widget for Progress {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.size.is_finite()
            || !(16.0..=512.0).contains(&self.size)
            || !self.thickness.is_finite()
            || !(1.0..=128.0).contains(&self.thickness)
            || self.view != View::Line && self.thickness > self.size / 2.
        {
            return Err(invalid(
                "progress size must be 16..=512 and thickness 1..=128, at most half a circular diameter",
            ));
        }
        let min = self.min;
        let max = self.max;
        let initial = selection(
            &self.value,
            &self.indeterminate,
            self.value_text.as_ref(),
            min,
            max,
        )?;
        let source = self.value.clone();
        let indeterminate = self.indeterminate.clone();
        let text = self.value_text.clone();
        let read = move || selection(&source, &indeterminate, text.as_ref(), min, max);
        let name = self
            .common
            .name
            .take()
            .expect("constructor installs a progress name");
        name.try_with(|name| validate_name(name))??;
        let name = crate::composite::validated(scope, &name, |name| validate_name(name))?;
        let value = scope.cage(initial);
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = read()?;
                if value.try_with_untracked(|old| *old != next)? {
                    value.try_revise(|old| *old = next)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        self.common.name = Some(name.into());
        let height = match self.view {
            View::Line => self.thickness,
            View::Ring => self.size,
            View::SemiCircle => self.size / 2. + self.thickness / 2.,
        };
        let required = Style::new()
            .position(Position::Relative)
            .display(Display::Block)
            .width(self.width.unwrap_or(if self.view == View::Line {
                Length::Percent(100.)
            } else {
                Length::Px(self.size)
            }))
            .height(Length::Px(height));
        let custom = self.common.style.clone();
        self.common.style = if custom.is_reactive() {
            scope
                .bond(move || merge(required.clone(), custom.get()))
                .into()
        } else {
            merge(required, custom.get()).into()
        };
        let mut base = Style::new();
        if self.view == View::Line {
            base.background = Some(self.common.theme.border);
            base.radius = Some(self.thickness / 2.);
            base.overflow_x = Some(Overflow::Hidden);
            base.overflow_y = Some(Overflow::Hidden);
        }
        let color = if self.common.unstyled {
            self.common.theme.foreground
        } else {
            self.common.theme.accent
        };
        let class = match self.view {
            View::Line => "sapid-progress sapid-progress-line",
            View::Ring => "sapid-progress sapid-progress-ring",
            View::SemiCircle => "sapid-progress sapid-progress-semicircle",
        };
        let metadata = value.map(|value| {
            Some(Box::new(Semantics {
                progress: Some(value.clone()),
                ..Default::default()
            }))
        });
        let mut root = configure!(self.common, glory::ui::view(), scope, class, base)
            .role(Role::ProgressBar)
            .semantics(metadata);
        if self.view == View::Line {
            root = root.fill(
                glory::ui::view()
                    .class("sapid-progress-fill")
                    .style(value.map(move |value| {
                        let fraction = value.fraction().expect("accepted progress is finite");
                        let mut style = Style::new()
                            .position(Position::Absolute)
                            .top(Length::Px(0.))
                            .left(Length::Percent(if fraction.is_some() { 0. } else { 35. }))
                            .width(Length::Percent(fraction.unwrap_or(0.3) * 100.))
                            .height(Length::Percent(100.));
                        style.background = Some(color);
                        style
                    })),
            );
        } else {
            let semi = self.view == View::SemiCircle;
            let size = self.size;
            let thickness = self.thickness;
            let drawing_style = Style::new()
                .position(Position::Absolute)
                .top(Length::Px(0.))
                .left(Length::Px(0.))
                .width(Length::Percent(100.))
                .height(Length::Percent(100.))
                .display(Display::Block);
            if !self.common.unstyled {
                root = root.fill(
                    glory::ui::graphic()
                        .class("sapid-progress-track")
                        .drawing(arc(size, thickness, semi, 1., self.common.theme.border))
                        .style(drawing_style.clone()),
                );
            }
            root = root.fill(
                glory::ui::graphic()
                    .class("sapid-progress-arc")
                    .drawing(value.map(move |value| {
                        arc(
                            size,
                            thickness,
                            semi,
                            value
                                .fraction()
                                .expect("accepted progress is finite")
                                .unwrap_or(0.25),
                            color,
                        )
                    }))
                    .style(drawing_style),
            );
        }
        root.show_in(scope)
    }
}
