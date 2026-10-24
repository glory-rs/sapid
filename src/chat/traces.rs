//! Display-only trace composition and explicit application action intents.
use super::trace_model::*;
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, ContentPolicy, Observers};
use crate::state::{Control, Value};
use glory::render::{DocumentPart, Event, EventKind, Role, TextKind};
use glory::style::{
    Display, Edges, Family, FlexDirection, Length, Limit, Overflow, Style, WhiteSpace,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
use std::{cell::Cell, collections::BTreeSet, rc::Rc};
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
pub(super) fn validate_presentation(key: &str, height: f64) -> Result {
    if key.len() > 32
        || key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        || !height.is_finite()
        || !(80. ..=640.).contains(&height)
    {
        Err(invalid("invalid bounded trace presentation"))
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TraceExpansion(BTreeSet<TraceKey>);
impl TraceExpansion {
    pub fn new(
        values: impl IntoIterator<Item = TraceKey>,
    ) -> std::result::Result<Self, TraceError> {
        let values: Vec<_> = values.into_iter().take(MAX_TRACE_RECORDS + 1).collect();
        if values.len() > MAX_TRACE_RECORDS {
            return Err(TraceError::TooManyRecords);
        }
        Ok(Self(values.into_iter().collect()))
    }
    pub fn contains(&self, key: &TraceKey) -> bool {
        self.0.contains(key)
    }
    pub fn keys(&self) -> impl Iterator<Item = &TraceKey> {
        self.0.iter()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceToggle {
    pub key: TraceKey,
    pub open: bool,
    pub expanded: TraceExpansion,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceAction {
    pub key: TraceKey,
    pub kind: TraceKind,
}
struct State {
    owner: glory::core::InstanceId,
    weak: glory::ui::WeakScope,
    live: Cell<bool>,
    next: Cell<u64>,
    accepted: Cage<TraceData>,
    expanded: Control<TraceExpansion>,
    enabled: Lotus<bool>,
    changed: Callback<TraceToggle>,
    action: Callback<TraceAction>,
    observers: Observers,
    action_label: Cage<String>,
    has_action: bool,
    key: String,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
    policy: ContentPolicy,
    payload_height: f64,
}
impl State {
    fn ensure(&self) -> Result {
        if self.live.get() && self.owner.is_alive() {
            Ok(())
        } else {
            Err(glory::ui::Error::Gone)
        }
    }
    fn toggle(&self, key: TraceKey, open: bool) -> Result {
        self.ensure()?;
        if !self.enabled.try_get()? || !self.accepted.try_with(|d| d.record(&key).is_some())? {
            return Ok(());
        }
        let mut expanded = self.expanded.source.try_with(|old| {
            self.accepted.with(|d| {
                TraceExpansion(
                    old.keys()
                        .filter(|k| d.record(k).is_some())
                        .cloned()
                        .collect(),
                )
            })
        })?;
        if open {
            expanded.0.insert(key.clone());
        } else {
            expanded.0.remove(&key);
        }
        self.expanded.accept(&expanded)?;
        self.changed.emit(
            self.owner,
            TraceToggle {
                key,
                open,
                expanded,
            },
        );
        Ok(())
    }
    fn act(&self, key: TraceKey) -> Result {
        self.ensure()?;
        if !self.enabled.try_get()? {
            return Ok(());
        }
        if let Some(kind) = self
            .accepted
            .try_with(|d| d.record(&key).map(|r| r.kind()))?
        {
            self.action.emit(self.owner, TraceAction { key, kind });
        }
        Ok(())
    }
    fn retire(&self) {
        self.live.set(false);
        self.changed.clear();
        self.action.clear();
        self.observers.clear()
    }
}
pub struct Trace {
    common: Common,
    key: String,
    data: Lotus<TraceData>,
    expanded: Value<TraceExpansion>,
    changed: Option<Box<dyn FnMut(TraceToggle)>>,
    action: Option<Box<dyn FnMut(TraceAction)>>,
    action_label: Lotus<String>,
    policy: ContentPolicy,
    payload_height: f64,
}
pub fn trace(key: impl Into<String>, data: impl Into<Lotus<TraceData>>) -> Trace {
    Trace {
        common: Common::default(),
        key: key.into(),
        data: data.into(),
        expanded: Value::default(),
        changed: None,
        action: None,
        action_label: "Inspect record".to_owned().into(),
        policy: ContentPolicy::Unmount,
        payload_height: 240.,
    }
}
impl Trace {
    common_methods!();
    pub(super) fn replace_common(&mut self, common: Common) {
        self.common = common;
    }
    pub fn expanded(mut self, value: impl Into<Lotus<TraceExpansion>>) -> Self {
        self.expanded.source = Some(value.into());
        self
    }
    pub fn default_expanded(mut self, value: TraceExpansion) -> Self {
        self.expanded.default = value;
        self
    }
    pub fn on_expand_change(mut self, value: impl FnMut(TraceToggle) + 'static) -> Self {
        self.changed = Some(Box::new(value));
        self
    }
    pub fn on_action(mut self, value: impl FnMut(TraceAction) + 'static) -> Self {
        self.action = Some(Box::new(value));
        self
    }
    pub fn action_label(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.action_label = value.into();
        self
    }
    pub fn content_policy(mut self, value: ContentPolicy) -> Self {
        self.policy = value;
        self
    }
    pub fn payload_height(mut self, value: f64) -> Self {
        self.payload_height = value;
        self
    }
}
impl Widget for Trace {
    fn build(&mut self, scope: &Scope) -> Result {
        validate_presentation(&self.key, self.payload_height)?;
        let accepted = composite::validated(scope, &self.data, |_: &TraceData| Ok(()))?;
        let action_label = composite::validated(scope, &self.action_label, |v| {
            if v.is_empty() || v.len() > 256 || v.contains(['\0', '\r', '\n']) {
                Err(invalid("invalid trace action label"))
            } else {
                Ok(())
            }
        })?;
        let state = Rc::new(State {
            owner: scope.id(),
            weak: scope.downgrade(),
            live: Cell::new(true),
            next: Cell::new(0),
            accepted,
            expanded: std::mem::take(&mut self.expanded).mount(scope),
            enabled: self.common.enabled.clone(),
            changed: Callback::new(self.changed.take()),
            has_action: self.action.is_some(),
            action: Callback::new(self.action.take()),
            observers: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Activate)
                    .unwrap_or_default(),
            ),
            action_label,
            key: self.key.clone(),
            theme: self.common.theme.clone(),
            size: self.common.size,
            unstyled: self.common.unstyled,
            policy: self.policy,
            payload_height: self.payload_height,
        });
        let cleanup = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(s) = cleanup.upgrade() {
                s.retire()
            }
        })?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let (class, appearance) = self.common.appearance(
            scope,
            "sapid-trace",
            Style {
                color: Some(self.common.theme.foreground),
                ..composite::column(8.)
            },
        );
        let style = scope.bond(move || {
            let mut s = appearance.get();
            s.display.get_or_insert(Display::Flex);
            s.flex_direction.get_or_insert(FlexDirection::Column);
            s.min_width.get_or_insert(Length::Px(0.));
            s.max_width.get_or_insert(Limit::Percent(100.));
            s.overflow_x.get_or_insert(Overflow::Auto);
            s
        });
        let mut root = glory::ui::view()
            .role(Role::Group)
            .name(
                self.common
                    .name
                    .take()
                    .unwrap_or_else(|| "Trace".to_owned().into()),
            )
            .class(class)
            .style(style)
            .focus(self.common.focus)
            .node_ref(reference);
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            root = root.on(kind, move |event| {
                crate::appearance::observe(&mut handlers, owner, event)
            });
        }
        root.show_in(scope)?;
        let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        build_list(&area, state.clone(), None)?;
        glory::ui::when(accepted.map(|d| d.records().is_empty()), || {
            glory::ui::text().content("No trace records")
        })
        .show_in(&area)?;
        let weak = Rc::downgrade(&state);
        scope.effect(move || {
            let Some(s) = weak.upgrade() else { return };
            let result = (|| -> Result {
                s.ensure()?;
                let next = s.expanded.source.try_with(|old| {
                    s.accepted.with(|d| {
                        TraceExpansion(
                            old.keys()
                                .filter(|k| d.record(k).is_some())
                                .cloned()
                                .collect(),
                        )
                    })
                })?;
                s.expanded.accept(&next)
            })();
            if let Err(e) = result
                && let Some(scope) = s.weak.upgrade()
            {
                scope.report_error(e)
            }
        });
        scope.retain(area)?;
        scope.retain(state)
    }
}
fn build_list(scope: &Scope, state: Rc<State>, parent: Option<TraceKey>) -> Result {
    let reference = scope.cage(None);
    glory::ui::view()
        .document_part(DocumentPart::OrderedList)
        .class("sapid-trace-list")
        .style(
            Style::new()
                .margin(Edges::all(0.))
                .padding(Edges {
                    left: 20.,
                    ..Edges::all(0.)
                })
                .min_width(Length::Px(0.)),
        )
        .node_ref(reference)
        .show_in(scope)?;
    let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
    let accepted = state.accepted;
    let keys = scope.bond(move || {
        accepted.with(|d| {
            if let Some(parent) = parent.as_ref() {
                d.child_keys(parent).cloned().collect::<Vec<_>>()
            } else {
                d.root_keys().cloned().collect()
            }
        })
    });
    glory::ui::each(
        keys,
        |k| k.clone(),
        move |key, _| TraceRow {
            key: key.get_untracked(),
            state: state.clone(),
        },
    )
    .show_in(&area)?;
    scope.retain(area)
}
struct TraceRow {
    key: TraceKey,
    state: Rc<State>,
}
impl Widget for TraceRow {
    fn build(&mut self, scope: &Scope) -> Result {
        let accepted = self.state.accepted;
        let key = self.key.clone();
        let label_key = key.clone();
        let expanded_key = key.clone();
        let state = self.state.clone();
        let reference = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::ListItem)
            .class("sapid-trace-record")
            .style(
                Style::new()
                    .min_width(Length::Px(0.))
                    .max_width(Limit::Percent(100.)),
            )
            .node_ref(reference)
            .show_in(scope)?;
        let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        let opened =
            scope.bond(move || state.expanded.source.with(|v| v.contains(&expanded_key)));
        let label = scope.bond(move || {
            accepted.with(|d| {
                d.record(&label_key).map_or_else(String::new, |r| {
                    format!(
                        "{} · {} · {}{}",
                        r.kind().label(),
                        r.title(),
                        r.state().label(),
                        r.elapsed()
                            .map_or_else(String::new, |v| format!(" · {v} ms"))
                    )
                })
            })
        });
        // A numeric construction ticket prevents namespace collisions without
        // shortening a business key or retaining a history of visited records.
        let ticket = self
            .state
            .next
            .get()
            .checked_add(1)
            .ok_or_else(|| invalid("trace item identity exhausted"))?;
        self.state.next.set(ticket);
        let namespace = format!("{}-{ticket}", self.state.key);
        let changed = self.state.clone();
        let body = self.state.clone();
        let observer = self.state.clone();
        let weak = scope.downgrade();
        let origin = scope.id();
        let mut widget = crate::collapsible(namespace)
            .class("sapid-trace-disclosure")
            .theme(self.state.theme.clone())
            .size(self.state.size)
            .enabled(self.state.enabled.clone())
            .label(label)
            .open(opened)
            .content_policy(self.state.policy)
            .on_open_change(move |open| {
                if let Err(e) = changed.toggle(key.clone(), open)
                    && let Some(scope) = weak.upgrade()
                {
                    scope.report_error(e)
                }
            })
            .on(EventKind::Activate, move |event| {
                if origin.is_alive() {
                    observer.observers.emit(observer.owner, event)
                }
            })
            .content({
                let key = self.key.clone();
                move || TraceBody {
                    key: key.clone(),
                    state: body.clone(),
                }
            });
        if self.state.unstyled {
            widget = widget.unstyled()
        }
        widget.show_in(&area)?;
        scope.retain(area)
    }
}
struct TraceBody {
    key: TraceKey,
    state: Rc<State>,
}
impl Widget for TraceBody {
    fn build(&mut self, scope: &Scope) -> Result {
        let accepted = self.state.accepted;
        for (input, label) in [(true, "Input"), (false, "Output")] {
            let key = self.key.clone();
            let state = self.state.clone();
            let present = scope.bond(move || {
                accepted.with(|d| {
                    d.record(&key).is_some_and(|r| {
                        if input {
                            r.input_payload().is_some()
                        } else {
                            r.output_payload().is_some()
                        }
                    })
                })
            });
            let key = self.key.clone();
            glory::ui::when(present, move || PayloadWidget {
                accepted,
                key: key.clone(),
                input,
                label,
                state: state.clone(),
            })
            .show_in(scope)?;
        }
        if self.state.has_action {
            let state = self.state.clone();
            let key = self.key.clone();
            let weak = scope.downgrade();
            let label = self.state.action_label;
            let origin = scope.id();
            let mut action = crate::button()
                .theme(self.state.theme.clone())
                .size(self.state.size)
                .class("sapid-trace-action")
                .label(label)
                .enabled(self.state.enabled.clone())
                .on_press(move || {
                    if let Err(e) = state.act(key.clone())
                        && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(e)
                    }
                    if origin.is_alive() {
                        state.observers.emit(state.owner, Event::Activate);
                    }
                });
            if self.state.unstyled {
                action = action.unstyled()
            }
            action.show_in(scope)?;
        }
        build_list(scope, self.state.clone(), Some(self.key.clone()))
    }
}
struct PayloadWidget {
    accepted: Cage<TraceData>,
    key: TraceKey,
    input: bool,
    label: &'static str,
    state: Rc<State>,
}
impl Widget for PayloadWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let mut heading = crate::title(4)
            .theme(self.state.theme.clone())
            .content(self.label);
        if self.state.unstyled {
            heading = heading.unstyled()
        }
        heading.show_in(scope)?;
        let key = self.key.clone();
        let accepted = self.accepted;
        let input = self.input;
        let payload = scope.bond(move || {
            accepted.with(|d| {
                d.record(&key)
                    .and_then(|r| {
                        if input {
                            r.input_payload()
                        } else {
                            r.output_payload()
                        }
                    })
                    .map(payload_text)
                    .unwrap_or_default()
            })
        });
        let mut style = Style::new()
            .display(Display::Block)
            .white_space(WhiteSpace::PreWrap)
            .min_width(Length::Px(0.))
            .max_width(Limit::Percent(100.))
            .max_height(Limit::Px(self.state.payload_height))
            .overflow_x(Overflow::Auto)
            .overflow_y(Overflow::Auto);
        if !self.state.unstyled {
            style = style.padding(Edges::all(8.));
            style.background = Some(self.state.theme.surface);
            style.color = Some(self.state.theme.foreground);
            style.font_family = Some(vec![Family::Mono]);
        }
        glory::ui::text()
            .kind(TextKind::Code)
            .class("sapid-trace-payload")
            .content(payload)
            .style(style)
            .show_in(scope)
    }
}
pub(super) fn payload_text(payload: &TracePayload) -> String {
    match payload {
        TracePayload::Redacted => "[redacted]".into(),
        TracePayload::Public(v) => format!(
            "{}{}",
            v.as_str(),
            if v.is_truncated() {
                "\n[truncated]"
            } else {
                ""
            }
        ),
    }
}
