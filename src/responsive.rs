//! Explicit measured-container breakpoints, with deterministic SSR width.
use glory::render::{Event, EventKind, NodeId, Query, QueryToken, QueryValue};
use glory::style::Style;
use glory::ui::{Children, IntoChildren, Result, WeakScope, Widget};
use glory::{Bond, Cage, Scope};
use std::{cell::Cell, rc::Rc};

fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn width(value: f64) -> Result {
    if value.is_finite() && (0.0..=1_000_000.).contains(&value) {
        Ok(())
    } else {
        Err(invalid(
            "responsive width must be finite within 0..=1000000px",
        ))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breakpoint {
    Base,
    Small,
    Medium,
    Large,
    ExtraLarge,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Breakpoints([f64; 4]);
impl Default for Breakpoints {
    fn default() -> Self {
        Self([576., 768., 992., 1200.])
    }
}
impl Breakpoints {
    pub fn new(thresholds: [f64; 4]) -> Result<Self> {
        for value in thresholds {
            width(value)?;
        }
        if thresholds[0] <= 0. || !thresholds.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(invalid(
                "breakpoints must be positive and strictly increasing",
            ));
        }
        Ok(Self(thresholds))
    }
    pub fn thresholds(self) -> [f64; 4] {
        self.0
    }
    pub fn resolve(self, value: f64) -> Result<Breakpoint> {
        width(value)?;
        Ok(
            match self
                .0
                .iter()
                .filter(|threshold| value >= **threshold)
                .count()
            {
                0 => Breakpoint::Base,
                1 => Breakpoint::Small,
                2 => Breakpoint::Medium,
                3 => Breakpoint::Large,
                _ => Breakpoint::ExtraLarge,
            },
        )
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ResponsiveValues<T> {
    pub base: T,
    pub small: T,
    pub medium: T,
    pub large: T,
    pub extra_large: T,
}
impl<T> ResponsiveValues<T> {
    pub fn new(base: T, small: T, medium: T, large: T, extra_large: T) -> Self {
        Self {
            base,
            small,
            medium,
            large,
            extra_large,
        }
    }
    fn at(&self, breakpoint: Breakpoint) -> &T {
        match breakpoint {
            Breakpoint::Base => &self.base,
            Breakpoint::Small => &self.small,
            Breakpoint::Medium => &self.medium,
            Breakpoint::Large => &self.large,
            Breakpoint::ExtraLarge => &self.extra_large,
        }
    }
}
struct Controller {
    weak: WeakScope,
    owner: glory::core::InstanceId,
    live: Cell<bool>,
    width: Cage<f64>,
    breakpoint: Cage<Breakpoint>,
    configuration: Breakpoints,
    requested: Cell<bool>,
    pending: Cell<Option<QueryToken>>,
    task: Cell<Option<glory::core::TaskToken>>,
    resized: Cell<bool>,
}
impl Controller {
    fn scope(&self) -> Result<Scope> {
        if !self.live.get() || !self.owner.is_alive() {
            return Err(glory::ui::Error::Gone);
        }
        self.weak.upgrade().ok_or(glory::ui::Error::Gone)
    }
    fn accept(&self, value: f64) -> Result {
        self.scope()?;
        let breakpoint = self.configuration.resolve(value)?;
        if self.width.try_get()? != value {
            self.width.try_revise(|current| *current = value)?;
        }
        if self.breakpoint.try_get()? != breakpoint {
            self.breakpoint
                .try_revise(|current| *current = breakpoint)?;
        }
        Ok(())
    }
    fn report(&self, error: glory::ui::Error) {
        if let Ok(scope) = self.scope() {
            scope.report_error(error);
        }
    }
    fn resize(&self, value: f64) -> Result {
        width(value)?;
        self.resized.set(true);
        self.accept(value)
    }
    fn initial(self: &Rc<Self>, node: NodeId) -> Result {
        let scope = self.scope()?;
        if self.requested.replace(true) || self.resized.get() || !scope.capabilities()?.measure
        {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let task = scope.spawn(async move {
            if let Some(state) = weak.upgrade() {
                state.task.take();
                if let Err(error) = state.measure(node) {
                    state.report(error);
                }
            }
        })?;
        self.task.set(Some(task));
        Ok(())
    }
    fn measure(self: &Rc<Self>, node: NodeId) -> Result {
        let scope = self.scope()?;
        if self.resized.get() {
            return Ok(());
        }
        let weak = Rc::downgrade(self);
        let token = scope.request(Query::Bounds(node), move |response| {
            let Some(controller) = weak.upgrade() else {
                return;
            };
            controller.pending.take();
            if controller.scope().is_err() || controller.resized.get() {
                return;
            }
            match response {
                Ok(QueryValue::Bounds(bounds)) => {
                    if let Err(error) = controller.accept(bounds.width) {
                        controller.report(error);
                    }
                }
                Ok(_) => controller.report(invalid("responsive Bounds response kind mismatch")),
                Err(error) => controller.report(error.into()),
            }
        })?;
        self.pending.set(Some(token));
        Ok(())
    }
    fn retire(&self) {
        self.live.set(false);
        if let Some(task) = self.task.take() {
            task.cancel();
        }
        if let Some(token) = self.pending.take()
            && let Some(scope) = self.weak.upgrade()
        {
            let _ = scope.cancel_request(token);
        }
    }
}
#[derive(Clone)]
pub struct ResponsiveContext {
    inner: Rc<Controller>,
}
impl ResponsiveContext {
    pub fn width(&self) -> Result<f64> {
        self.inner.scope()?;
        Ok(self.inner.width.try_get()?)
    }
    pub fn breakpoint(&self) -> Result<Breakpoint> {
        self.inner.scope()?;
        Ok(self.inner.breakpoint.try_get()?)
    }
    /// Produces one Owner-owned derived value. No source lookup or visited-width cache.
    pub fn select<T: Clone + 'static>(&self, values: ResponsiveValues<T>) -> Result<Bond<T>> {
        let scope = self.inner.scope()?;
        let current = self.inner.breakpoint;
        Ok(scope.bond(move || values.at(current.get()).clone()))
    }
    pub fn styles(&self, values: ResponsiveValues<Style>) -> Result<Bond<Style>> {
        for style in [
            &values.base,
            &values.small,
            &values.medium,
            &values.large,
            &values.extra_large,
        ] {
            style
                .validate()
                .map_err(glory::render::Error::InvalidProps)?;
        }
        self.select(values)
    }
    pub fn pending_queries(&self) -> Result<usize> {
        self.inner.scope()?;
        Ok(usize::from(self.inner.pending.get().is_some()))
    }
}
type Factory = Box<dyn FnMut(ResponsiveContext) -> Children>;
pub struct ResponsiveRoot {
    initial_width: f64,
    breakpoints: Breakpoints,
    style: glory::Lotus<Style>,
    class: glory::Lotus<String>,
    reference: Option<Cage<Option<NodeId>>>,
    context_ref: Option<Cage<Option<ResponsiveContext>>>,
    content: Option<Factory>,
}
/// `initial_width` is required so SSR and initial adoption use the same tier.
pub fn responsive_root(initial_width: f64) -> ResponsiveRoot {
    ResponsiveRoot {
        initial_width,
        breakpoints: Default::default(),
        style: Style::new().into(),
        class: "".into(),
        reference: None,
        context_ref: None,
        content: None,
    }
}
impl ResponsiveRoot {
    pub fn breakpoints(mut self, value: Breakpoints) -> Self {
        self.breakpoints = value;
        self
    }
    pub fn style(mut self, value: impl Into<glory::Lotus<Style>>) -> Self {
        self.style = value.into();
        self
    }
    pub fn class(mut self, value: impl Into<glory::Lotus<String>>) -> Self {
        self.class = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.reference = Some(value);
        self
    }
    pub fn context_ref(mut self, value: Cage<Option<ResponsiveContext>>) -> Self {
        self.context_ref = Some(value);
        self
    }
    pub fn content<C: IntoChildren>(
        mut self,
        mut content: impl FnMut(ResponsiveContext) -> C + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| content(context).into_children()));
        self
    }
}
impl Widget for ResponsiveRoot {
    fn build(&mut self, scope: &Scope) -> Result {
        let breakpoint = self.breakpoints.resolve(self.initial_width)?;
        let style = crate::composite::validated(scope, &self.style, |style| {
            style
                .validate()
                .map_err(glory::render::Error::InvalidProps)?;
            Ok(())
        })?;
        let state = Rc::new(Controller {
            weak: scope.downgrade(),
            owner: scope.id(),
            live: Cell::new(true),
            width: scope.cage(self.initial_width),
            breakpoint: scope.cage(breakpoint),
            configuration: self.breakpoints,
            requested: Cell::new(false),
            pending: Cell::new(None),
            task: Cell::new(None),
            resized: Cell::new(false),
        });
        let context = ResponsiveContext {
            inner: state.clone(),
        };
        let cleanup = state.clone();
        scope.on_cleanup(move || cleanup.retire())?;
        if let Some(reference) = self.context_ref {
            reference.try_revise(|value| *value = Some(context.clone()))?;
        }
        let reference = self.reference.unwrap_or_else(|| scope.cage(None));
        let custom = self.class.clone();
        let class = scope.bond(move || format!("sapid-responsive-root {}", custom.get()));
        let resize = Rc::downgrade(&state);
        let mut root = glory::ui::view()
            .class(class)
            .style(style)
            .node_ref(reference)
            .on(EventKind::Resize, move |event| {
                if let Event::Resize { size, .. } = event
                    && let Some(state) = resize.upgrade()
                    && let Err(error) = state.resize(size.width)
                {
                    state.report(error);
                }
            });
        if let Some(mut factory) = self.content.take() {
            root = root.fill(factory(context));
        }
        root.show_in(scope)?;
        let initial = Rc::downgrade(&state);
        scope.effect(move || {
            if let Some(node) = reference.get()
                && let Some(state) = initial.upgrade()
                && let Err(error) = state.initial(node)
            {
                state.report(error);
            }
        });
        scope.retain(state)?;
        Ok(())
    }
}
