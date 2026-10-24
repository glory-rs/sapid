//! Readonly successful text control with an independent host-owned Dialog trigger.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, ContentPolicy};
use crate::state::Value;
use crate::{
    CivilDate, DatePickerMode, DateSelection, DateSelectionError, DatesProvider, FieldControl,
    Weekday, YearMonth,
};
use glory::render::{Direction, FormControl, NodeId};
use glory::style::{Align, Display, FlexDirection, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};
#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    value: DateSelection,
    provider: DatesProvider,
}
/// A display Input and a separate real Button open a modal date selection Dialog.
/// The display is readonly in every mode; `readonly(true)` also locks selection.
pub struct DatePickerInput {
    common: Common,
    key: String,
    provider: Lotus<DatesProvider>,
    mode: DatePickerMode,
    value: Value<DateSelection>,
    default_set: bool,
    open: Value<bool>,
    readonly: Lotus<bool>,
    required: Lotus<bool>,
    invalid: Lotus<bool>,
    field: Option<FieldControl>,
    form_name: Option<Lotus<String>>,
    direction: Direction,
    excluded: Vec<CivilDate>,
    weekdays: u8,
    deselect: bool,
    month: Option<Lotus<YearMonth>>,
    default_month: Option<YearMonth>,
    trigger_label: Lotus<String>,
    title: Lotus<String>,
    close_label: Lotus<String>,
    trigger_ref: Option<Cage<Option<NodeId>>>,
    panel_ref: Option<Cage<Option<NodeId>>>,
    calendar_ref: Option<Cage<Option<NodeId>>>,
    changed: Option<Box<dyn FnMut(DateSelection)>>,
    rejected: Option<Box<dyn FnMut(DateSelectionError)>>,
    open_changed: Option<Box<dyn FnMut(bool)>>,
    month_changed: Option<Box<dyn FnMut(YearMonth)>>,
}
pub fn date_picker_input(
    key: impl Into<String>,
    provider: impl Into<Lotus<DatesProvider>>,
) -> DatePickerInput {
    DatePickerInput {
        common: Common::default(),
        key: key.into(),
        provider: provider.into(),
        mode: DatePickerMode::Single,
        value: Value::default(),
        default_set: false,
        open: Value::default(),
        readonly: false.into(),
        required: false.into(),
        invalid: false.into(),
        field: None,
        form_name: None,
        direction: Direction::Ltr,
        excluded: Vec::new(),
        weekdays: 0,
        deselect: false,
        month: None,
        default_month: None,
        trigger_label: "Choose dates".into(),
        title: "Choose dates".into(),
        close_label: "Close date picker".into(),
        trigger_ref: None,
        panel_ref: None,
        calendar_ref: None,
        changed: None,
        rejected: None,
        open_changed: None,
        month_changed: None,
    }
}
impl DatePickerInput {
    common_methods!();
    pub fn mode(mut self, v: DatePickerMode) -> Self {
        self.mode = v;
        self
    }
    pub fn value(mut self, v: impl Into<Lotus<DateSelection>>) -> Self {
        self.value.source = Some(v.into());
        self
    }
    pub fn default_value(mut self, v: DateSelection) -> Self {
        self.value.default = v;
        self.default_set = true;
        self
    }
    pub fn open(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(v.into());
        self
    }
    pub fn default_open(mut self, v: bool) -> Self {
        self.open.default = v;
        self
    }
    pub fn readonly(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.readonly = v.into();
        self
    }
    pub fn required(mut self, v: impl Into<Lotus<bool>>) -> Self {
        self.required = v.into();
        self
    }
    pub fn invalid(mut self, v: impl Into<Lotus<bool>>) -> Self {
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
    /// Explicit ISO successful-control name, independent of the accessible name.
    pub fn form_name(mut self, v: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(v.into());
        self
    }
    pub fn direction(mut self, v: Direction) -> Self {
        self.direction = v;
        self
    }
    pub fn exclude_dates(mut self, v: impl IntoIterator<Item = CivilDate>) -> Self {
        self.excluded = v
            .into_iter()
            .take(crate::MAX_CALENDAR_EXCLUDED_DATES + 1)
            .collect();
        self
    }
    pub fn exclude_date(mut self, v: CivilDate) -> Self {
        if self.excluded.len() <= crate::MAX_CALENDAR_EXCLUDED_DATES {
            self.excluded.push(v);
        }
        self
    }
    pub fn disable_weekday(mut self, v: Weekday) -> Self {
        self.weekdays |= 1 << v.index();
        self
    }
    pub fn allow_deselect(mut self, v: bool) -> Self {
        self.deselect = v;
        self
    }
    pub fn month(mut self, v: impl Into<Lotus<YearMonth>>) -> Self {
        self.month = Some(v.into());
        self
    }
    pub fn default_month(mut self, v: YearMonth) -> Self {
        self.default_month = Some(v);
        self
    }
    pub fn trigger_label(mut self, v: impl Into<Lotus<String>>) -> Self {
        self.trigger_label = v.into();
        self
    }
    pub fn title(mut self, v: impl Into<Lotus<String>>) -> Self {
        self.title = v.into();
        self
    }
    pub fn close_label(mut self, v: impl Into<Lotus<String>>) -> Self {
        self.close_label = v.into();
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
    pub fn on_value_change(mut self, v: impl FnMut(DateSelection) + 'static) -> Self {
        self.changed = Some(Box::new(v));
        self
    }
    pub fn on_reject(mut self, v: impl FnMut(DateSelectionError) + 'static) -> Self {
        self.rejected = Some(Box::new(v));
        self
    }
    pub fn on_open_change(mut self, v: impl FnMut(bool) + 'static) -> Self {
        self.open_changed = Some(Box::new(v));
        self
    }
    pub fn on_month_change(mut self, v: impl FnMut(YearMonth) + 'static) -> Self {
        self.month_changed = Some(Box::new(v));
        self
    }
}
impl Widget for DatePickerInput {
    fn build(&mut self, scope: &Scope) -> Result {
        // Overlay and Calendar both append names. Validate before formatting any derived key.
        if self.key.is_empty()
            || self.key.len() > 57
            || !self
                .key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.excluded.len() > crate::MAX_CALENDAR_EXCLUDED_DATES
        {
            return Err(glory::render::Error::InvalidProps(
                "invalid date picker input key or exclusions",
            )
            .into());
        }
        if self.value.source.is_none() && !self.default_set {
            self.value.default = DateSelection::empty(self.mode);
        }
        let value = std::mem::take(&mut self.value).mount(scope);
        let open = std::mem::take(&mut self.open).mount(scope);
        let mode = self.mode;
        let source = value.source.clone();
        let provider = self.provider.clone();
        let snapshot = scope.bond(move || Snapshot {
            value: source.get(),
            provider: provider.get(),
        });
        let accepted = composite::validated(scope, &snapshot.into(), move |s: &Snapshot| {
            s.value
                .validate(mode, s.provider.date_bounds())
                .map_err(|_| {
                    glory::render::Error::InvalidProps("invalid date picker input selection")
                        .into()
                })
        })?;
        let owner = scope.id();
        let weak = scope.downgrade();
        let changed = Callback::new(self.changed.take());
        let rejected = Callback::new(self.rejected.take());
        let opened = Callback::new(self.open_changed.take());
        let month_changed = Callback::new(self.month_changed.take());
        let callbacks = (
            changed.clone(),
            rejected.clone(),
            opened.clone(),
            month_changed.clone(),
        );
        scope.on_cleanup(move || {
            callbacks.0.clear();
            callbacks.1.clear();
            callbacks.2.clear();
            callbacks.3.clear();
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
            .class("sapid-date-picker-input")
            .style(
                Style {
                    align: Some(Align::Center),
                    ..Style::new()
                }
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .gap(8.),
            )
            .node_ref(wrapper)
            .show_in(scope)?;
        let content = scope.child_in(wrapper.get().ok_or(glory::ui::Error::Gone)?)?;
        let (classes, style) = self.common.appearance(
            scope,
            "sapid-date-picker-display",
            self.common.theme.control(self.common.size),
        );
        let mut input = glory::ui::input()
            .class(classes)
            .style(style)
            .enabled(enabled.clone())
            .focus(self.common.focus)
            .value(scope.bond(move || accepted.with(|s| s.value.format_iso())))
            .semantics(metadata);
        if let Some(name) = self.common.name.take() {
            input = input.name(name);
        }
        if let Some(reference) = self.common.node_ref {
            input = input.node_ref(reference);
        }
        for (kind, mut callbacks) in std::mem::take(&mut self.common.handlers) {
            input = input.on(kind, move |e| {
                crate::appearance::observe(&mut callbacks, owner, e)
            });
        }
        input.show_in(&content)?;
        let key = format!("{}-calendar", self.key);
        let theme = self.common.theme.clone();
        let size = self.common.size;
        let unstyled = self.common.unstyled;
        let direction = self.direction;
        let excluded = std::mem::take(&mut self.excluded);
        let weekdays = self.weekdays;
        let deselect = self.deselect;
        let month = self.month.take();
        let default_month = self.default_month;
        let calendar_ref = self.calendar_ref;
        let calendar_source = value.source.clone();
        let callback_value = value.clone();
        let callback_open = open.clone();
        let callback_opened = opened.clone();
        let active = enabled.clone();
        let initial_focus =
            composite::id("dialog", &format!("{}-dialog", self.key), "close", "root")?;
        let mut dialog = crate::dialog(format!("{}-dialog", self.key))
            .initial_focus(initial_focus)
            .label(self.trigger_label.clone())
            .title(self.title.clone())
            .close_label(self.close_label.clone())
            .theme(theme.clone())
            .size(size)
            .enabled(enabled)
            .open(open.source.clone())
            .content_policy(ContentPolicy::KeepMounted)
            .on_open_change(move |v| {
                if !owner.is_alive() {
                    return;
                }
                let result = open.accept(&v);
                if let Err(e) = result {
                    if let Some(scope) = weak.upgrade() {
                        scope.report_error(e);
                    }
                    return;
                }
                opened.emit(owner, v);
            })
            .content(move || {
                let callback_value = callback_value.clone();
                let callback_open = callback_open.clone();
                let changed = changed.clone();
                let opened = callback_opened.clone();
                let rejected = rejected.clone();
                let month_changed = month_changed.clone();
                let mut picker = crate::date_picker(key.clone(), accepted.map(|s| s.provider))
                    .mode(mode)
                    .value(calendar_source.clone())
                    .enabled(active.clone())
                    .readonly(readonly.clone())
                    .direction(direction)
                    .allow_deselect(deselect)
                    .exclude_dates(excluded.iter().copied())
                    .theme(theme.clone())
                    .size(size)
                    .on_value_change(move |next| {
                        if !owner.is_alive() {
                            return;
                        }
                        if callback_value.accept(&next).is_err() {
                            return;
                        }
                        changed.emit(owner, next);
                        if owner.is_alive()
                            && mode != DatePickerMode::Multiple
                            && next.is_complete()
                            && callback_value.source.try_get().ok() == Some(next)
                            && callback_open.accept(&false).is_ok()
                        {
                            opened.emit(owner, false);
                        }
                    })
                    .on_reject(move |e| rejected.emit(owner, e))
                    .on_month_change(move |m| month_changed.emit(owner, m));
                if let Some(month) = &month {
                    picker = picker.month(month.clone());
                }
                if let Some(month) = default_month {
                    picker = picker.default_month(month);
                }
                if let Some(reference) = calendar_ref {
                    picker = picker.node_ref(reference);
                }
                if unstyled {
                    picker = picker.unstyled();
                }
                for day in Weekday::ALL {
                    if weekdays & (1 << day.index()) != 0 {
                        picker = picker.disable_weekday(day);
                    }
                }
                picker
            });
        if let Some(reference) = self.trigger_ref {
            dialog = dialog.trigger_ref(reference);
        }
        if let Some(reference) = self.panel_ref {
            dialog = dialog.panel_ref(reference);
        }
        if self.common.unstyled {
            dialog = dialog.unstyled();
        }
        dialog.show_in(&content)?;
        scope.retain(content)
    }
}
