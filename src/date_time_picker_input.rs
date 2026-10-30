//! One successful civil ISO Input and a separate host-owned Dialog.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, ContentPolicy};
use crate::state::Value;
use crate::{
    DateTimeBounds, DateTimeDraft, DateTimeOutcome, DateTimePolicy, DateTimeStatus, DatesProvider,
    FieldControl, LocalDateTime, TimeFormat, TimeGridSpec,
};
use glory::render::{Direction, FormControl, NodeId};
use glory::style::{Align, Display, Length, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
/// Complete valid edits publish immediately. Done commits/closes; Escape keeps
/// accepted edits. A draft or unresolved timezone gap never replaces the ISO value.
pub struct DateTimePicker {
    common: Common,
    key: String,
    provider: Vary<DatesProvider>,
    policy: DateTimePolicy,
    value: Value<Option<LocalDateTime>>,
    open: Value<bool>,
    bounds: Vary<DateTimeBounds>,
    revision: Vary<u64>,
    format: TimeFormat,
    readonly: Vary<bool>,
    required: Vary<bool>,
    invalid: Vary<bool>,
    field: Option<FieldControl>,
    form_name: Option<Vary<String>>,
    direction: Direction,
    grid: Option<TimeGridSpec>,
    content_policy: ContentPolicy,
    references: [Option<Cage<Option<NodeId>>>; 3],
    trigger_ref: Option<Cage<Option<NodeId>>>,
    panel_ref: Option<Cage<Option<NodeId>>>,
    calendar_ref: Option<Cage<Option<NodeId>>>,
    changed: Option<Box<dyn FnMut(Option<LocalDateTime>)>>,
    committed: Option<Box<dyn FnMut(Option<DateTimeOutcome>)>>,
    drafted: Option<Box<dyn FnMut(DateTimeDraft)>>,
    resolved: Option<Box<dyn FnMut(DateTimeOutcome)>>,
    rejected: Option<Box<dyn FnMut(DateTimeStatus)>>,
    opened: Option<Box<dyn FnMut(bool)>>,
}
pub fn date_time_picker(
    key: impl Into<String>,
    provider: impl Into<Vary<DatesProvider>>,
    policy: DateTimePolicy,
) -> DateTimePicker {
    DateTimePicker {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        policy,
        value: Value::default(),
        open: Value::default(),
        bounds: DateTimeBounds::default().into(),
        revision: 0.into(),
        format: TimeFormat::default(),
        readonly: false.into(),
        required: false.into(),
        invalid: false.into(),
        field: None,
        form_name: None,
        direction: Direction::Ltr,
        grid: None,
        content_policy: ContentPolicy::KeepMounted,
        references: [None; 3],
        trigger_ref: None,
        panel_ref: None,
        calendar_ref: None,
        changed: None,
        committed: None,
        drafted: None,
        resolved: None,
        rejected: None,
        opened: None,
    }
}
impl DateTimePicker {
    common_methods!();
    pub fn value(mut self, v: impl Into<Vary<Option<LocalDateTime>>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: Option<LocalDateTime>) -> Self {
        self.value.default = v;
        self
    }
    pub fn open(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.open.source = Some(v.into());
        self
    }
    pub fn default_open(mut self, v: bool) -> Self {
        self.open.default = v;
        self
    }
    pub fn bounds(mut self, v: impl Into<Vary<DateTimeBounds>>) -> Self {
        self.bounds = v.into();
        self
    }
    pub fn reset_revision(mut self, v: impl Into<Vary<u64>>) -> Self {
        self.revision = v.into();
        self
    }
    pub fn format(mut self, v: TimeFormat) -> Self {
        self.format = v;
        self
    }
    pub fn readonly(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn required(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.required = v.into();
        self
    }
    pub fn invalid(mut self, v: impl Into<Vary<bool>>) -> Self {
        self.invalid = v.into();
        self
    }
    pub fn field(mut self, v: FieldControl) -> Self {
        self.common.theme = v.theme.clone();
        self.common.size = v.size;
        self.common.unstyled = v.unstyled;
        self.field = Some(v);
        self
    }
    pub fn form_name(mut self, v: impl Into<Vary<String>>) -> Self {
        self.form_name = Some(v.into());
        self
    }
    pub fn direction(mut self, v: Direction) -> Self {
        self.direction = v;
        self
    }
    pub fn grid(mut self, v: TimeGridSpec) -> Self {
        self.grid = Some(v);
        self
    }
    pub fn content_policy(mut self, v: ContentPolicy) -> Self {
        self.content_policy = v;
        self
    }
    pub fn trigger_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.trigger_ref = Some(v);
        self
    }
    pub fn panel_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.panel_ref = Some(v);
        self
    }
    pub fn calendar_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.calendar_ref = Some(v);
        self
    }
    pub fn hour_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.references[0] = Some(v);
        self
    }
    pub fn minute_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.references[1] = Some(v);
        self
    }
    pub fn second_ref(mut self, v: Cage<Option<NodeId>>) -> Self {
        self.references[2] = Some(v);
        self
    }
    pub fn on_value_change(mut self, v: impl FnMut(Option<LocalDateTime>) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
    pub fn on_commit(mut self, v: impl FnMut(Option<DateTimeOutcome>) + 'static) -> Self {
        self.committed = Some(Box::new(v));
        self
    }
    pub fn on_draft_change(mut self, v: impl FnMut(DateTimeDraft) + 'static) -> Self {
        self.drafted = Some(Box::new(v));
        self
    }
    pub fn on_resolved(mut self, v: impl FnMut(DateTimeOutcome) + 'static) -> Self {
        self.resolved = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(DateTimeStatus) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
    pub fn on_open_change(mut self, v: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(v));
        self
    }
}
impl Widget for DateTimePicker {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.key.is_empty()
            || self.key.len() > 40
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(glory::render::Error::InvalidProps("invalid date/time dialog key").into());
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let open = std::mem::take(&mut self.open).mount(scope);
        let format = self.format;
        let value_source = control.source.clone();
        let bounds_source = self.bounds.clone();
        let provider_source = self.provider.clone();
        let snapshot = scope.bond(move || {
            (
                value_source.get(),
                bounds_source.get(),
                provider_source.get(),
            )
        });
        let accepted = composite::validated(
            scope,
            &snapshot.into(),
            move |v: &(Option<LocalDateTime>, DateTimeBounds, DatesProvider)| {
                let provider = v
                    .1
                    .provider(v.2)
                    .map_err(|_| glory::render::Error::InvalidProps("empty date/time interval"))?;
                if v.0.is_some_and(|value| {
                    format.validate(value.time()).is_err()
                        || !v.1.contains(value)
                        || !provider.date_bounds().contains(value.date())
                }) {
                    return Err(
                        glory::render::Error::InvalidProps("date/time precision loss").into(),
                    );
                }
                Ok(())
            },
        )?;
        let owner = scope.id();
        let weak = scope.downgrade();
        let changed = Callback::new(self.changed.take());
        let committed = Callback::new(self.committed.take());
        let drafted = Callback::new(self.drafted.take());
        let resolved = Callback::new(self.resolved.take());
        let rejected = Callback::new(self.rejected.take());
        let opened = Callback::new(self.opened.take());
        let cleanup = (
            changed.clone(),
            committed.clone(),
            drafted.clone(),
            resolved.clone(),
            rejected.clone(),
            opened.clone(),
        );
        scope.on_cleanup(move || {
            cleanup.0.clear();
            cleanup.1.clear();
            cleanup.2.clear();
            cleanup.3.clear();
            cleanup.4.clear();
            cleanup.5.clear();
        })?;
        let enabled = self
            .field
            .as_ref()
            .map(|f| f.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let readonly = self
            .field
            .as_ref()
            .map(|f| f.readonly_state(scope, self.readonly.clone()))
            .unwrap_or_else(|| self.readonly.clone());
        let metadata = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            true.into(),
            self.invalid.clone(),
        );
        let form_name = composite::form_name(scope, &self.form_name)?;
        let metadata = scope.bond(move || {
            let mut s = metadata.get().unwrap_or_default();
            if let Some(name) = form_name {
                s.form_control = Some(FormControl {
                    name: name.get(),
                    value: None,
                });
            }
            Some(s)
        });
        let wrapper = scope.cage(None);
        glory::ui::view()
            .class("sapid-date-time-picker")
            .style(Style {
                align: Some(Align::Center),
                ..Style::new().display(Display::Flex).gap(8.)
            })
            .node_ref(wrapper)
            .show_in(scope)?;
        let content = scope.child_in(wrapper.get().ok_or(glory::ui::Error::Gone)?)?;
        let (class, appearance) = self.common.appearance(
            scope,
            "sapid-date-time-display",
            self.common.theme.control(self.common.size),
        );
        let style = scope.bond(move || {
            let mut s = appearance.get();
            s.width.get_or_insert(Length::Px(260.));
            s.min_height.get_or_insert(Length::Px(32.));
            s
        });
        let mut input = glory::ui::input()
            .class(class)
            .style(style)
            .enabled(enabled.clone())
            .focus(self.common.focus)
            .value(accepted.map(|v| v.0.map(crate::format_local_date_time).unwrap_or_default()))
            .semantics(metadata);
        if let Some(v) = self.common.name.take() {
            input = input.name(v);
        }
        if let Some(r) = self.common.node_ref {
            input = input.node_ref(r);
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            input = input.on(kind, move |e| {
                crate::appearance::observe(&mut handlers, owner, e)
            });
        }
        input.show_in(&content)?;
        let key = format!("{}-inline", self.key);
        let provider = self.provider.clone();
        let policy = self.policy.clone();
        let bounds = self.bounds.clone();
        let revision = self.revision.clone();
        let source = control.source.clone();
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let direction = self.direction;
        let grid = self.grid;
        let refs = self.references;
        let calendar_ref = self.calendar_ref;
        let active = enabled.clone();
        let close_open = open.clone();
        let close_opened = opened.clone();
        let changed_control = control.clone();
        let focus = composite::id("dialog", &format!("{}-dialog", self.key), "close", "root")?;
        let mut dialog = crate::dialog(format!("{}-dialog", self.key))
            .label("Choose date and time")
            .title("Date and time")
            .close_label("Close date and time")
            .initial_focus(focus)
            .theme(theme.clone())
            .size(size)
            .enabled(enabled)
            .open(open.source.clone())
            .content_policy(self.content_policy)
            .on_open_change(move |v| {
                if !owner.is_alive() {
                    return;
                }
                if let Err(e) = open.accept(&v) {
                    if let Some(s) = weak.upgrade() {
                        s.report_error(e);
                    }
                    return;
                }
                opened.emit(owner, v);
            })
            .content(move || {
                let c = changed_control.clone();
                let changed = changed.clone();
                let committed = committed.clone();
                let drafted = drafted.clone();
                let resolved = resolved.clone();
                let rejected = rejected.clone();
                let close = close_open.clone();
                let opened = close_opened.clone();
                let accepted_source = source.clone();
                let mut picker =
                    crate::inline_date_time_picker(key.clone(), provider.clone(), policy.clone())
                        .value(source.clone())
                        .bounds(bounds.clone())
                        .reset_revision(revision.clone())
                        .format(format)
                        .theme(theme.clone())
                        .size(size)
                        .enabled(active.clone())
                        .readonly(readonly.clone())
                        .direction(direction)
                        .on_value_change(move |v| {
                            if owner.is_alive() && c.accept(&v).is_ok() {
                                changed.emit(owner, v);
                            }
                        })
                        .on_draft_change(move |v| drafted.emit(owner, v))
                        .on_resolved(move |v| resolved.emit(owner, v))
                        .on_reject(move |v| rejected.emit(owner, v))
                        .on_commit(move |v| {
                            committed.emit(owner, v);
                            if owner.is_alive()
                                && accepted_source.try_get().ok() == Some(v.map(|v| v.local))
                                && close.accept(&false).is_ok()
                            {
                                opened.emit(owner, false);
                            }
                        });
                if let Some(g) = grid {
                    picker = picker.grid(g);
                }
                if let Some(r) = calendar_ref {
                    picker = picker.calendar_ref(r);
                }
                if let Some(r) = refs[0] {
                    picker = picker.hour_ref(r);
                }
                if let Some(r) = refs[1] {
                    picker = picker.minute_ref(r);
                }
                if let Some(r) = refs[2] {
                    picker = picker.second_ref(r);
                }
                if unstyled {
                    picker = picker.unstyled();
                }
                picker
            });
        if let Some(r) = self.trigger_ref {
            dialog = dialog.trigger_ref(r);
        }
        if let Some(r) = self.panel_ref {
            dialog = dialog.panel_ref(r);
        }
        if self.common.unstyled {
            dialog = dialog.unstyled();
        }
        dialog.show_in(&content)?;
        scope.retain(content)
    }
}
