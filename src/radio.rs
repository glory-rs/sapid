//! Controlled native radio groups. Members are explicit, stable, mounted values.
use std::cell::RefCell;
use std::rc::Rc;

use glory::render::{
    Change, CheckMode, Direction, Event, EventKind, FocusMode, FormControl, Navigation, NodeId,
    Orientation, Patch, Role, SemanticId, Semantics,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

use crate::appearance::{Common, common_methods, configure, observe};
use crate::collection::{
    Collection, CollectionLimits, CollectionMember, RovingFocus, Typeahead, TypeaheadConfig,
};
use crate::composite::{self, Callback};
use crate::state::{Control, Dirty, Value};

pub const MAX_RADIO_GROUP_ITEMS: usize = 1024;
type RadioVisual = Box<dyn FnOnce(glory::ui::Check) -> Box<dyn Widget>>;

/// A named value belonging to exactly one RadioGroup; its owner is the mounted
/// native control. Selection is always controlled by the group's single value.
pub struct Radio {
    key: String,
    common: Common,
    label: Vary<String>,
    visual: Option<RadioVisual>,
    form_value: Option<String>,
}
pub fn radio(value: impl Into<String>) -> Radio {
    Radio {
        key: value.into(),
        common: Common::default(),
        label: "".into(),
        visual: None,
        form_value: None,
    }
}
impl Radio {
    pub(crate) fn form_value(mut self, value: String) -> Self {
        self.form_value = Some(value);
        self
    }
    pub(crate) fn visual(
        mut self,
        value: impl FnOnce(glory::ui::Check) -> Box<dyn Widget> + 'static,
    ) -> Self {
        self.visual = Some(Box::new(value));
        self
    }
    pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Vary<glory::style::Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn unstyled(mut self) -> Self {
        self.common.unstyled = true;
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on(mut self, kind: EventKind, handler: impl FnMut(Event) + 'static) -> Self {
        self.common
            .handlers
            .entry(kind)
            .or_default()
            .push(Box::new(handler));
        self
    }
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
}

pub struct RadioGroup {
    common: Common,
    key: String,
    label: Vary<String>,
    form_name: Option<String>,
    value: Value<String>,
    items: Vec<Radio>,
    field: Option<crate::FieldControl>,
    required: Vary<bool>,
    readonly: Vary<bool>,
    invalid: Vary<bool>,
    orientation: Orientation,
    looping: bool,
    direction: Direction,
    typeahead: Option<TypeaheadConfig>,
    on_change: Option<Box<dyn FnMut(String)>>,
    member_gap: f64,
}
pub fn radio_group(key: impl Into<String>) -> RadioGroup {
    RadioGroup {
        common: Common::default(),
        key: key.into(),
        label: "".into(),
        form_name: None,
        value: Value::default(),
        items: Vec::new(),
        field: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
        orientation: Orientation::Vertical,
        looping: true,
        direction: Direction::Ltr,
        typeahead: None,
        on_change: None,
        member_gap: 8.,
    }
}
impl RadioGroup {
    common_methods!();
    pub(crate) fn member_gap(mut self, value: f64) -> Self {
        self.member_gap = value;
        self
    }
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
    /// Explicit native successful-control name. Native HTML uses this same
    /// token for grouping; it must be a bounded ASCII semantic identifier.
    pub fn form_name(mut self, value: impl Into<String>) -> Self {
        self.form_name = Some(value.into());
        self
    }
    pub fn value(mut self, value: impl Into<Vary<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }
    pub fn radio(mut self, value: Radio) -> Self {
        if self.items.len() <= MAX_RADIO_GROUP_ITEMS {
            self.items.push(value);
        }
        self
    }
    pub fn items(mut self, values: impl IntoIterator<Item = Radio>) -> Self {
        let remaining = (MAX_RADIO_GROUP_ITEMS + 1).saturating_sub(self.items.len());
        self.items.extend(values.into_iter().take(remaining));
        self
    }
    pub fn on_value_change(mut self, handler: impl FnMut(String) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = value;
        self
    }
    pub fn loop_focus(mut self, value: bool) -> Self {
        self.looping = value;
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.direction = value;
        self
    }
    pub fn typeahead(mut self, value: TypeaheadConfig) -> Self {
        self.typeahead = Some(value);
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
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
        self
    }
}

struct Selection {
    control: Control<String>,
    keys: Rc<Vec<String>>,
    enabled: Vec<Vary<bool>>,
    root_enabled: Vary<bool>,
    readonly: Vary<bool>,
    collection: Collection,
    roving: RovingFocus,
    typeahead: RefCell<Option<Typeahead>>,
    dirty: Dirty,
    callback: Callback<String>,
    owner: glory::core::InstanceId,
}
impl Selection {
    fn flags(&self) -> Result<Vec<bool>> {
        let enabled = self.root_enabled.try_get()?;
        self.enabled
            .iter()
            .map(|flag| Ok(enabled && flag.try_get()?))
            .collect()
    }
    fn select(&self, index: usize) -> Result {
        if !self.flags()?[index] || self.readonly.try_get()? {
            return Ok(());
        }
        let value = &self.keys[index];
        self.control.accept(value)?;
        self.dirty.touch()?;
        self.callback.emit(self.owner, value.clone());
        Ok(())
    }
    fn navigate(&self, scope: &Scope, from: NodeId, command: Navigation) -> Result {
        if self.readonly.try_get()? {
            return Ok(());
        }
        if let Some(search) = self.typeahead.borrow_mut().as_mut() {
            search.clear();
        }
        if let Some(next) = self.roving.destination(from, command)? {
            self.roving.accept(&next)?;
            composite::focus_node(scope, Some(next.node))?;
            if let Some(index) = self.keys.iter().position(|key| key == &next.key) {
                self.select(index)?;
            }
        }
        Ok(())
    }
    fn search(&self, scope: &Scope, from: NodeId, text: &str, at_millis: u64) -> Result {
        if self.readonly.try_get()? {
            return Ok(());
        }
        let next = {
            let mut search = self.typeahead.borrow_mut();
            match search.as_mut() {
                Some(search) => search.search(&self.collection, from, text, at_millis)?,
                None => None,
            }
        };
        if let Some(next) = next {
            self.roving.accept(&next)?;
            composite::focus_node(scope, Some(next.node))?;
            if let Some(index) = self.keys.iter().position(|key| key == &next.key) {
                self.select(index)?;
            }
        }
        Ok(())
    }
}

struct RegisteredRadio {
    control: Option<Box<dyn Widget>>,
    collection: Collection,
    key: String,
    enabled: Vary<bool>,
    label: Vary<String>,
    reference: Cage<Option<NodeId>>,
    token: Cage<Option<CollectionMember>>,
}
impl Widget for RegisteredRadio {
    fn build(&mut self, scope: &Scope) -> Result {
        self.control
            .take()
            .ok_or(glory::ui::Error::Gone)?
            .show_in(scope)?;
        let member = self.collection.register(
            scope,
            self.key.clone(),
            self.reference,
            self.enabled.clone(),
            self.label.clone(),
        )?;
        self.token.try_revise(|token| *token = Some(member))?;
        let token = self.token;
        scope.on_cleanup(move || {
            let _ = token.try_revise(|token| *token = None);
        })?;
        Ok(())
    }
}

impl Widget for RadioGroup {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.items.is_empty()
            || self.items.len() > MAX_RADIO_GROUP_ITEMS
            || !self.member_gap.is_finite()
            || !(0. ..=1024.).contains(&self.member_gap)
            || self.common.focus != FocusMode::Auto
            || self
                .items
                .iter()
                .any(|item| item.common.focus != FocusMode::Auto)
        {
            return Err(glory::render::Error::InvalidProps(
                "radio group requires 1..=1024 members and owns their tab stops",
            )
            .into());
        }
        let id = composite::id("radio-group", &self.key, "root", "root")?;
        let label_id = composite::id("radio-group", &self.key, "label", "root")?;
        let native_name = match self.form_name.take() {
            Some(name) => SemanticId::new(name)?,
            None => composite::id("radio-group", &self.key, "native", "root")?,
        };
        let keys = self
            .items
            .iter()
            .map(|item| item.key.clone())
            .collect::<Vec<_>>();
        composite::validate_keys(&keys)?;
        let keys = Rc::new(keys);
        let control = std::mem::take(&mut self.value).mount(scope);
        let valid_keys = keys.clone();
        let selected = composite::validated(scope, &control.source, move |value| {
            if value.is_empty() || valid_keys.contains(value) {
                Ok(())
            } else {
                Err(glory::render::Error::InvalidProps("unknown radio group selection").into())
            }
        })?;
        let initial = selected.try_get()?;
        let enabled = self
            .field
            .as_ref()
            .map(|field| field.enabled(scope, self.common.enabled.clone()))
            .unwrap_or_else(|| self.common.enabled.clone());
        let metadata = crate::field::control_metadata(
            scope,
            self.field.as_ref(),
            self.common.name.is_some(),
            self.required.clone(),
            self.readonly.clone(),
            self.invalid.clone(),
        );
        let readonly_metadata = metadata.clone();
        let readonly = scope.bond(move || {
            readonly_metadata
                .get()
                .is_some_and(|metadata| metadata.readonly)
        });
        let references = Rc::new(
            self.items
                .iter()
                .map(|item| item.common.node_ref.unwrap_or_else(|| scope.cage(None)))
                .collect::<Vec<_>>(),
        );
        let enabled_flags = self
            .items
            .iter()
            .map(|item| item.common.enabled.clone())
            .collect::<Vec<_>>();
        let collection = Collection::new(scope, CollectionLimits::default())?;
        let roving = RovingFocus::new(scope, collection.clone(), self.looping)?;
        let search = self.typeahead.map(Typeahead::new).transpose()?;
        let tokens = self
            .items
            .iter()
            .map(|_| scope.cage(None))
            .collect::<Vec<_>>();
        let dirty = Dirty::new(scope);
        let mut children: Vec<Box<dyn Widget>> = Vec::new();
        let owner = scope.id();
        // Each member registers in its mounted Owner before the root controller
        // handles events. Registration identity is independent of key reuse.
        let mut members = Vec::new();
        for (index, mut item) in self.items.drain(..).enumerate() {
            item.common.theme = self.common.theme.clone();
            item.common.size = self.common.size;
            item.common.unstyled |= self.common.unstyled;
            let native_name = native_name.clone();
            let states = metadata.clone();
            let key = item.key.clone();
            let form_value = item.form_value.take().unwrap_or_else(|| key.clone());
            let flag = item.common.enabled.clone();
            let root = enabled.clone();
            let member_enabled: Vary<bool> = scope.bond(move || root.get() && flag.get()).into();
            let member_label = item.label.clone();
            let member_key = item.key.clone();
            let token = tokens[index];
            let focus_controller = roving.clone();
            let check = configure!(
                item.common,
                glory::ui::check(),
                scope,
                "sapid-checkbox sapid-radio",
                self.common.theme.checkbox(self.common.size)
            )
            .mode(CheckMode::Radio)
            .label(item.label)
            .checked(initial == item.key)
            .enabled(member_enabled.clone())
            .focus(scope.bond(move || {
                if token
                    .get()
                    .is_some_and(|member| focus_controller.is_tab_stop(&member).unwrap_or(false))
                {
                    FocusMode::TabStop
                } else {
                    FocusMode::Programmatic
                }
            }))
            .node_ref(references[index])
            .semantics(scope.bond(move || {
                let metadata = states.get();
                composite::metadata(Semantics {
                    check_group: Some(native_name.clone()),
                    form_control: Some(FormControl {
                        name: native_name.as_str().into(),
                        value: Some(form_value.clone()),
                    }),
                    navigation_item: true,
                    required: metadata.as_ref().is_some_and(|metadata| metadata.required),
                    readonly: metadata.as_ref().is_some_and(|metadata| metadata.readonly),
                    invalid: metadata.as_ref().is_some_and(|metadata| metadata.invalid),
                    ..Default::default()
                })
            }));
            members.push(std::mem::take(&mut item.common.handlers));
            let control = match item.visual.take() {
                Some(wrap) => wrap(check),
                None => Box::new(check),
            };
            children.push(Box::new(RegisteredRadio {
                control: Some(control),
                collection: collection.clone(),
                key: member_key,
                enabled: member_enabled,
                label: member_label,
                reference: references[index],
                token,
            }));
        }
        let root_metadata = metadata.clone();
        let supplied_field = self.field.is_some();
        let axis = self.orientation;
        let direction = self.direction;
        let has_typeahead = self.typeahead.is_some();
        let label = glory::ui::text()
            .content(self.label.clone())
            .semantics(composite::metadata(Semantics {
                id: Some(label_id.clone()),
                ..Default::default()
            }));
        let root_id = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let base = composite::column(8.0);
        let mut root = configure!(
            self.common,
            glory::ui::view().role(Role::RadioGroup),
            scope,
            "sapid-radio-group",
            base
        )
        .node_ref(root_id)
        .semantics(scope.bond(move || {
            let mut metadata = root_metadata.get().unwrap_or_default();
            if metadata.id.is_none() {
                metadata.id = Some(id.clone());
            }
            if !supplied_field && metadata.labelled_by.is_none() {
                metadata.labelled_by = Some(label_id.clone());
            }
            metadata.navigation = Some(axis);
            metadata.navigation_direction = direction;
            metadata.typeahead = has_typeahead;
            Some(metadata)
        }))
        .fill(label)
        .fill(
            glory::ui::view()
                .class("sapid-radio-group-items")
                .style(
                    glory::style::Style::new()
                        .display(glory::style::Display::Flex)
                        .flex_direction(if axis == Orientation::Horizontal {
                            glory::style::FlexDirection::Row
                        } else {
                            glory::style::FlexDirection::Column
                        })
                        .gap(self.member_gap),
                )
                .fill(children),
        );
        // Root listeners are installed after the finite native children exist.
        // A scope-owned relay avoids keeping an owning Scope in its handlers.
        let navigation = Rc::new(std::cell::RefCell::new(None::<Rc<Selection>>));
        let handler = navigation.clone();
        let weak = scope.downgrade();
        let mut navigation_observers = self
            .common
            .handlers
            .remove(&EventKind::Navigate)
            .unwrap_or_default();
        root = root.on(EventKind::Navigate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let live = handler.borrow().clone();
            if let (Some(live), Event::Navigate { from, command }) = (live, &event)
                && let Err(error) = live.navigate(&scope, *from, *command)
            {
                scope.report_error(error);
                return;
            }
            observe(&mut navigation_observers, owner, event);
        });
        if has_typeahead {
            let handler = navigation.clone();
            let weak = scope.downgrade();
            let mut observers = self
                .common
                .handlers
                .remove(&EventKind::Typeahead)
                .unwrap_or_default();
            root = root.on(EventKind::Typeahead, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let live = handler.borrow().clone();
                if let (
                    Some(live),
                    Event::Typeahead {
                        from,
                        text,
                        at_millis,
                    },
                ) = (live, &event)
                    && let Err(error) = live.search(&scope, *from, text, *at_millis)
                {
                    scope.report_error(error);
                    return;
                }
                observe(&mut observers, owner, event);
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        roving.repair(Some(&initial))?;
        let selection = Rc::new(Selection {
            control,
            keys: keys.clone(),
            collection,
            typeahead: RefCell::new(search),
            enabled: enabled_flags.clone(),
            root_enabled: enabled.clone(),
            readonly: readonly.into(),
            roving: roving.clone(),
            dirty: dirty.clone(),
            callback: Callback::new(self.on_change.take()),
            owner,
        });
        *navigation.borrow_mut() = Some(selection.clone());
        for (position, handlers) in members.into_iter().enumerate() {
            let node = references[position]
                .try_get()?
                .ok_or(glory::ui::Error::Gone)?;
            let mut handlers = handlers;
            let mut checked_observers = handlers
                .remove(&EventKind::CheckedChanged)
                .unwrap_or_default();
            let controller = selection.clone();
            let weak = scope.downgrade();
            scope.on(node, EventKind::CheckedChanged, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if event == Event::CheckedChanged(true)
                    && let Err(error) = controller.select(position)
                {
                    scope.report_error(error);
                    return;
                }
                // False is a peer-uncheck observation during hydration, not a
                // request to clear the group's single selection.
                observe(&mut checked_observers, owner, event);
            })?;
            let mut focus_observers = handlers
                .remove(&EventKind::FocusChanged)
                .unwrap_or_default();
            let weak = scope.downgrade();
            let focus_controller = roving.clone();
            let token = tokens[position];
            scope.on(node, EventKind::FocusChanged, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                if event == Event::FocusChanged(true)
                    && let Some(member) = token.try_get().ok().flatten()
                    && let Err(error) = focus_controller.observe(&member)
                {
                    scope.report_error(error);
                    return;
                }
                observe(&mut focus_observers, owner, event);
            })?;
            for (kind, mut handlers) in handlers {
                scope.on(node, kind, move |event| {
                    observe(&mut handlers, owner, event)
                })?;
            }
        }
        let weak = scope.downgrade();
        let mut previous = initial;
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let value = selected.try_get()?;
                let edited = dirty.take();
                roving.repair((value != previous).then_some(value.as_str()))?;
                if edited || value != previous {
                    // A native radio toggle implicitly unchecks another member.
                    // Reassert the complete live group even when a controlled
                    // callback rejected the proposed value.
                    for (key, reference) in keys.iter().zip(references.iter()) {
                        if let Some(node) = reference.try_get()? {
                            scope.patch(node, Patch::Checked(Change::Set(key == &value)))?;
                        }
                    }
                    previous = value;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        Ok(())
    }
}
