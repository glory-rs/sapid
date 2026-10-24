//! A focus-preserving listbox over intrinsic combobox/option refinements.
use std::{cell::RefCell, rc::Rc};

use glory::render::{
    ChoiceCommand, ComboboxKind, Event, EventKind, FocusMode, FormControl, Layer, LayerFocus,
    LayerPlacement, Navigation, OptionSemantics, Orientation, PopupKind, Props, Role,
    SelectEntry, SelectGroup, SelectProps, SemanticId, Semantics,
};
use glory::style::{
    BorderStyle, Color, Display, Edges, FlexDirection, Length, Limit, Overflow, Style,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

use crate::appearance::{Common, common_methods, configure};
use crate::collection::{
    Collection, CollectionLimits, MountedItem, RovingFocus, Typeahead, TypeaheadConfig,
};
use crate::composite::{self, Callback, Observers};
use crate::native_select::{NativeOption, NativeOptionGroup};
use crate::state::{Control, Value};

enum Entry {
    Option(NativeOption),
    Group(NativeOptionGroup),
}
pub struct Select {
    common: Common,
    key: String,
    entries: Vec<Entry>,
    count: usize,
    value: Value<Option<String>>,
    open: Value<bool>,
    changed: Option<Box<dyn FnMut(Option<String>)>>,
    opened: Option<Box<dyn FnMut(bool)>>,
    placeholder: String,
    clearable: bool,
    field: Option<crate::FieldControl>,
    form_name: Option<Lotus<String>>,
    required: Lotus<bool>,
    readonly: Lotus<bool>,
    invalid: Lotus<bool>,
}
pub fn select(key: impl Into<String>) -> Select {
    Select {
        common: Common::default(),
        key: key.into(),
        entries: Vec::new(),
        count: 0,
        value: Value::default(),
        open: Value::default(),
        changed: None,
        opened: None,
        placeholder: "Choose an option".into(),
        clearable: false,
        field: None,
        form_name: None,
        required: false.into(),
        readonly: false.into(),
        invalid: false.into(),
    }
}
impl Select {
    common_methods!();
    pub fn label(mut self, label: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(label.into());
        self
    }
    pub fn field(mut self, field: crate::FieldControl) -> Self {
        self.common.theme = field.theme.clone();
        self.common.size = field.size;
        self.common.unstyled = field.unstyled;
        self.field = Some(field);
        self
    }
    pub fn form_name(mut self, name: impl Into<Lotus<String>>) -> Self {
        self.form_name = Some(name.into());
        self
    }
    pub fn value(mut self, value: impl Into<Lotus<Option<String>>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.value.default = value;
        self
    }
    pub fn open(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.open.source = Some(value.into());
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.open.default = value;
        self
    }
    pub fn on_value_change(mut self, callback: impl FnMut(Option<String>) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_open_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.opened = Some(Box::new(callback));
        self
    }
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }
    pub fn clearable(mut self, value: bool) -> Self {
        self.clearable = value;
        self
    }
    pub fn required(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.required = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn invalid(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.invalid = value.into();
        self
    }
    pub fn option(mut self, value: NativeOption) -> Self {
        if self.count <= 1024 {
            self.entries.push(Entry::Option(value));
            self.count += 1;
        }
        self
    }
    pub fn options(mut self, values: impl IntoIterator<Item = NativeOption>) -> Self {
        for value in values
            .into_iter()
            .take(1025usize.saturating_sub(self.count))
        {
            self.entries.push(Entry::Option(value));
            self.count += 1;
        }
        self
    }
    pub fn group(mut self, value: NativeOptionGroup) -> Self {
        if self.count <= 1024 && self.entries.len() <= 1056 {
            self.count += value.options.len();
            self.entries.push(Entry::Group(value));
        }
        self
    }
}

struct State {
    owner: glory::core::InstanceId,
    value: Control<Option<String>>,
    selected: Cage<Option<String>>,
    open: Control<bool>,
    visible: Lotus<bool>,
    editable: Lotus<bool>,
    collection: Collection,
    cursor: RovingFocus,
    typeahead: RefCell<Typeahead>,
    config: Rc<SelectProps>,
    changed: Callback<Option<String>>,
    opened: Callback<bool>,
    activated: Observers,
}
impl State {
    fn opened(&self, open: bool) -> Result {
        if !self.owner.is_alive() {
            return Ok(());
        }
        if self.open.source.try_get()? != open {
            self.open.accept(&open)?;
            self.opened.emit(self.owner, open);
        }
        Ok(())
    }
    fn choose(&self, value: Option<String>) -> Result {
        if !self.editable.try_get()? {
            return Ok(());
        }
        if value.is_some() && !self.visible.try_get()? {
            return Ok(());
        }
        self.value
            .source
            .try_with(|current| validate(&self.config, current))??;
        validate(&self.config, &value)?;
        self.value.accept(&value)?;
        self.changed.emit(self.owner, value);
        if self.owner.is_alive() {
            self.opened(false)?;
        }
        Ok(())
    }
    fn current(&self) -> Result<Option<MountedItem>> {
        let current = self.cursor.current()?;
        Ok(self
            .collection
            .mounted()?
            .into_iter()
            .find(|item| Some(&item.member) == current.as_ref() && item.enabled))
    }
    fn set_current(&self, item: &MountedItem) -> Result {
        self.cursor.accept(item)
    }
    fn initial_cursor(&self) -> Result {
        if let Some(key) = self.selected.try_get()? {
            self.cursor.repair(Some(&key))
        } else {
            self.navigate(Navigation::First)
        }
    }
    fn navigate(&self, command: Navigation) -> Result {
        let items = self.collection.mounted()?;
        let destination = match command {
            Navigation::First => items.iter().find(|item| item.enabled).cloned(),
            Navigation::Last => items.iter().rev().find(|item| item.enabled).cloned(),
            _ => match self.current()? {
                Some(item) => self.cursor.destination(item.node, command)?,
                None => items.into_iter().find(|item| item.enabled),
            },
        };
        if let Some(item) = destination {
            self.set_current(&item)?;
        }
        Ok(())
    }
    fn command(&self, command: ChoiceCommand) -> Result {
        if command == ChoiceCommand::Close {
            return self.opened(false);
        }
        if !self.editable.try_get()? {
            return Ok(());
        }
        match command {
            ChoiceCommand::OpenFirst => {
                self.opened(true)?;
                self.initial_cursor()?;
            }
            ChoiceCommand::OpenLast => {
                self.opened(true)?;
                self.navigate(Navigation::Last)?;
            }
            ChoiceCommand::First => {
                self.opened(true)?;
                self.navigate(Navigation::First)?;
            }
            ChoiceCommand::Last => {
                self.opened(true)?;
                self.navigate(Navigation::Last)?;
            }
            ChoiceCommand::Previous => {
                if self.visible.try_get()? {
                    self.navigate(Navigation::Previous)?;
                } else {
                    self.opened(true)?;
                    self.navigate(Navigation::Last)?;
                }
            }
            ChoiceCommand::Next => {
                if self.visible.try_get()? {
                    self.navigate(Navigation::Next)?;
                } else {
                    self.opened(true)?;
                    self.initial_cursor()?;
                }
            }
            ChoiceCommand::Commit => {
                if self.visible.try_get()?
                    && let Some(item) = self.current()?
                {
                    self.choose(Some(item.key))?;
                } else if !self.visible.try_get()? {
                    self.opened(true)?;
                    self.initial_cursor()?;
                }
            }
            ChoiceCommand::Close => self.opened(false)?,
            ChoiceCommand::RemoveLast => {
                return Err(glory::render::Error::InvalidEvent(
                    "remove-last requires multiple selection",
                )
                .into());
            }
        }
        Ok(())
    }
    fn search(&self, text: &str, at_millis: u64) -> Result {
        if !self.editable.try_get()? {
            return Ok(());
        }
        let was_visible = self.visible.try_get()?;
        self.opened(true)?;
        if was_visible {
            self.cursor.repair(None)?;
        } else {
            self.initial_cursor()?;
        }
        if let Some(current) = self.current()? {
            let result = self.typeahead.borrow_mut().search(
                &self.collection,
                current.node,
                text,
                at_millis,
            )?;
            if let Some(item) = result {
                self.set_current(&item)?;
            }
        }
        Ok(())
    }
}
fn validate(config: &SelectProps, value: &Option<String>) -> Result {
    if let Some(value) = value
        && (value.len() > 64
            || !config
                .options()
                .any(|(option, _)| option.key.as_str() == value))
    {
        return Err(glory::render::Error::InvalidProps(
            "select value must be a known bounded option key",
        )
        .into());
    }
    Ok(())
}

impl Widget for Select {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.count > 1024
            || self.entries.len() > 1056
            || self.placeholder.len() > 256
            || self.placeholder.contains(['\0', '\r'])
        {
            return Err(glory::render::Error::InvalidProps(
                "select configuration exceeds budget",
            )
            .into());
        }
        let trigger_id = self
            .field
            .as_ref()
            .map(|field| Ok(field.id.clone()))
            .unwrap_or_else(|| composite::id("select", &self.key, "trigger", "root"))?;
        let panel_id = composite::id("select", &self.key, "listbox", "root")?;
        let proxy_id = composite::id("select", &self.key, "proxy", "root")?;
        let entries: Result<Vec<_>> = std::mem::take(&mut self.entries)
            .into_iter()
            .map(|entry| match entry {
                Entry::Option(value) => Ok(SelectEntry::Option(value.lower()?)),
                Entry::Group(value) => Ok(SelectEntry::Group(SelectGroup {
                    label: value.label,
                    disabled: value.disabled,
                    options: value
                        .options
                        .into_iter()
                        .map(NativeOption::lower)
                        .collect::<Result<Vec<_>>>()?,
                })),
            })
            .collect();
        let config = Rc::new(SelectProps {
            entries: Rc::new(entries?),
            ..Default::default()
        });
        config.validate()?;
        let form_name = composite::form_name(scope, &self.form_name)?;
        let value = std::mem::take(&mut self.value).mount(scope);
        let check = config.clone();
        let selected =
            composite::validated(scope, &value.source, move |value| validate(&check, value))?;
        let open = std::mem::take(&mut self.open).mount(scope);
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
        let input_meta = metadata.clone();
        let enabled_for_edit = enabled.clone();
        let editable: Lotus<bool> = scope
            .bond(move || {
                enabled_for_edit.get() && !input_meta.get().is_some_and(|s| s.readonly)
            })
            .into();
        let source = open.source.clone();
        let enabled_for_panel = enabled.clone();
        let visible: Lotus<bool> = scope
            .bond(move || source.get() && enabled_for_panel.get())
            .into();
        let collection = Collection::new(scope, CollectionLimits::default())?;
        let cursor = RovingFocus::new(scope, collection.clone(), true)?;
        let activated = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Activate)
                .unwrap_or_default(),
        );
        let state = Rc::new(State {
            owner: scope.id(),
            value,
            selected,
            open,
            visible: visible.clone(),
            editable,
            collection,
            cursor,
            typeahead: RefCell::new(Typeahead::new(TypeaheadConfig::default())?),
            config: config.clone(),
            changed: Callback::new(self.changed.take()),
            opened: Callback::new(self.opened.take()),
            activated,
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.changed.clear();
                state.opened.clear();
                state.activated.clear();
            }
        })?;
        let trigger_ref = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let display = config.clone();
        let placeholder = std::mem::take(&mut self.placeholder);
        let current = selected;
        let label = scope.bond(move || {
            let selected = current.get();
            display
                .options()
                .find(|(option, _)| Some(option.key.as_str()) == selected.as_deref())
                .map_or_else(|| placeholder.clone(), |(option, _)| option.label.clone())
        });
        let trigger_meta = metadata.clone();
        let button_state = state.clone();
        let trigger_name = trigger_id.clone();
        let listbox_name = panel_id.clone();
        let key = self.key.clone();
        let trigger_metadata = scope.bond(move || {
            let mut value = trigger_meta.get().unwrap_or_default();
            value.id = Some(trigger_name.clone());
            value.form_control = None;
            value.combobox = Some(ComboboxKind::Select);
            value.popup = Some(PopupKind::Listbox);
            value.controls = Some(listbox_name.clone());
            value.expanded = Some(button_state.visible.get());
            value.typeahead = true;
            value.active_descendant =
                if button_state.visible.get() {
                    button_state.current().ok().flatten().and_then(|item| {
                        composite::id("select", &key, "option", &item.key).ok()
                    })
                } else {
                    None
                };
            Some(value)
        });
        let base = self.common.theme.control(self.common.size);
        let mut trigger = configure!(
            self.common,
            glory::ui::button(),
            scope,
            "sapid-select",
            base
        )
        .label(label)
        .enabled(enabled.clone())
        .semantics(trigger_metadata)
        .node_ref(trigger_ref);
        let controller = state.clone();
        let weak = scope.downgrade();
        trigger = trigger.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Err(error) = (|| -> Result {
                if controller.editable.try_get()? {
                    controller.opened(!controller.visible.try_get()?)?;
                    controller.initial_cursor()?;
                }
                Ok(())
            })() {
                scope.report_error(error);
                return;
            }
            controller.activated.emit(scope.id(), event);
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let choices = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Choice)
                .unwrap_or_default(),
        );
        trigger = trigger.on(EventKind::Choice, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::Choice { command, .. } = &event
                && let Err(error) = controller.command(*command)
            {
                scope.report_error(error);
                return;
            }
            choices.emit(scope.id(), event);
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let searches = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::Typeahead)
                .unwrap_or_default(),
        );
        trigger = trigger.on(EventKind::Typeahead, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Event::Typeahead {
                text, at_millis, ..
            } = &event
                && let Err(error) = controller.search(text, *at_millis)
            {
                scope.report_error(error);
                return;
            }
            searches.emit(scope.id(), event);
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let focus_observers = Observers::new(
            self.common
                .handlers
                .remove(&EventKind::FocusChanged)
                .unwrap_or_default(),
        );
        trigger = trigger.on(EventKind::FocusChanged, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if event == Event::FocusChanged(false)
                && let Err(error) = controller.opened(false)
            {
                scope.report_error(error);
                return;
            }
            focus_observers.emit(scope.id(), event);
        });
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            let handlers = Observers::new(handlers);
            let owner = scope.id();
            trigger = trigger.on(kind, move |event| handlers.emit(owner, event));
        }
        let root = scope.create(glory::render::Descriptor::new(Props::Group))?;
        let content = scope.child_in(root)?;
        trigger.show_in(&content)?;
        let panel_label = trigger_id.clone();
        let layer = Layer {
            modal: false,
            focus: LayerFocus::Preserve,
            placement: LayerPlacement::Anchor(glory::render::AnchorPlacement::new(
                glory::render::AnchorSide::Bottom,
                glory::render::AnchorAlign::Start,
            )),
            anchor: Some(trigger_id),
            initial_focus: None,
            return_focus: None,
            ..Default::default()
        };
        let panel_visible = visible.clone();
        let listbox_name = panel_id.clone();
        let panel_meta = scope.bond(move || {
            Some(Box::new(Semantics {
                id: Some(listbox_name.clone()),
                labelled_by: Some(panel_label.clone()),
                navigation: Some(Orientation::Vertical),
                hidden: !panel_visible.get(),
                layer: Some(layer.clone()),
                ..Default::default()
            }))
        });
        let panel_ref = scope.cage(None);
        let controller = state.clone();
        let weak = scope.downgrade();
        let mut panel_style = Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .min_width(Length::Px(180.0))
            .max_height(Limit::Px(288.0))
            .overflow_y(Overflow::Auto);
        if !self.common.unstyled {
            panel_style.background = Some(self.common.theme.surface);
            panel_style = panel_style
                .padding(Edges::all(4.0))
                .border_width(Edges::all(1.0))
                .border_style(BorderStyle::Solid)
                .border_color(self.common.theme.border)
                .radius(6.0);
        }
        glory::ui::view()
            .role(Role::Listbox)
            .semantics(panel_meta)
            .node_ref(panel_ref)
            .class(format!(
                "sapid-select-listbox {}",
                self.common.theme.class()
            ))
            .style(panel_style)
            .on(EventKind::Dismiss, move |_| {
                if let Some(scope) = weak.upgrade()
                    && let Err(error) = controller.opened(false)
                {
                    scope.report_error(error);
                }
            })
            .show_in(&content)?;
        let panel_node = panel_ref.get().ok_or(glory::ui::Error::Gone)?;
        let items_scope = scope.child_in(panel_node)?;
        for entry in config.entries.iter() {
            match entry {
                SelectEntry::Option(option) => mount_option(
                    &items_scope,
                    &state,
                    option,
                    false,
                    &self.key,
                    &panel_id,
                    &self.common,
                )?,
                SelectEntry::Group(group) => {
                    let reference = scope.cage(None);
                    glory::ui::view()
                        .role(Role::Group)
                        .name(group.label.clone())
                        .style(
                            Style::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Column),
                        )
                        .node_ref(reference)
                        .fill(glory::ui::text().content(group.label.clone()))
                        .show_in(&items_scope)?;
                    let group_scope =
                        scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
                    for option in &group.options {
                        mount_option(
                            &group_scope,
                            &state,
                            option,
                            group.disabled,
                            &self.key,
                            &panel_id,
                            &self.common,
                        )?;
                    }
                    items_scope.retain(group_scope)?;
                }
            }
        }
        content.retain(items_scope)?;
        let form_meta = metadata;
        let proxy_meta = scope.bond(move || {
            let value = form_meta.get();
            Some(Box::new(Semantics {
                id: Some(proxy_id.clone()),
                form_control: form_name
                    .as_ref()
                    .map(|name| FormControl {
                        name: name.get(),
                        value: None,
                    })
                    .or_else(|| value.and_then(|s| s.form_control)),
                ..Default::default()
            }))
        });
        let hidden = scope.cage(None);
        glory::ui::view()
            .semantics(Some(Box::new(Semantics {
                hidden: true,
                ..Default::default()
            })))
            .node_ref(hidden)
            .show_in(&content)?;
        let proxy_scope = scope.child_in(hidden.get().ok_or(glory::ui::Error::Gone)?)?;
        let selection = scope.bond(move || {
            selected
                .get()
                .into_iter()
                .map(|key| SemanticId::new(key).expect("validated option key"))
                .collect::<Vec<_>>()
        });
        glory::ui::select()
            .configuration((*config).clone())
            .selected(selection)
            .enabled(enabled)
            .semantics(proxy_meta)
            .show_in(&proxy_scope)?;
        content.retain(proxy_scope)?;
        if self.clearable {
            let controller = state.clone();
            let weak = scope.downgrade();
            let editable = state.editable.clone();
            let can_clear = scope.bond(move || editable.get() && selected.get().is_some());
            glory::ui::button()
                .label("Clear selection")
                .style(if self.common.unstyled {
                    Style::new()
                } else {
                    self.common.theme.control(self.common.size)
                })
                .enabled(can_clear)
                .on(EventKind::Activate, move |event| {
                    let Some(scope) = weak.upgrade() else { return };
                    if let Err(error) = controller.choose(None) {
                        scope.report_error(error);
                        return;
                    }
                    controller.activated.emit(scope.id(), event);
                })
                .show_in(&content)?;
        }
        let controller = state.clone();
        let weak = scope.downgrade();
        let mut previous_selected = selected.get();
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let current = selected.try_get()?;
                let changed = current != previous_selected;
                previous_selected = current;
                if !controller.visible.try_get()? {
                    controller.typeahead.borrow_mut().clear();
                    controller.initial_cursor()?;
                } else if changed || controller.current()?.is_none() {
                    controller.initial_cursor()?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let controller = state.clone();
        let weak = scope.downgrade();
        let mut revealed = None;
        scope.effect(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let current = if controller.visible.try_get()? {
                    controller.current()?.map(|item| item.node)
                } else {
                    None
                };
                if current != revealed {
                    revealed = current;
                    if let Some(node) = current
                        && scope.capabilities()?.scroll
                    {
                        scope.action(glory::render::Action::RevealOption { node })?;
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        scope.retain(content)?;
        scope.retain(state)?;
        Ok(())
    }
}

fn mount_option(
    scope: &Scope,
    state: &Rc<State>,
    option: &glory::render::SelectOption,
    group_disabled: bool,
    key: &str,
    owner: &SemanticId,
    common: &Common,
) -> Result {
    let parent = scope;
    let item_scope = parent.child()?;
    let scope = &item_scope;
    let id = composite::id("select", key, "option", option.key.as_str())?;
    let reference = scope.cage(None);
    let enabled = !option.disabled && !group_disabled;
    let member = state.collection.register(
        scope,
        option.key.as_str(),
        reference,
        enabled,
        option.label.clone(),
    )?;
    let selected = state.selected;
    let option_key = option.key.as_str().to_owned();
    let semantic_owner = owner.clone();
    let metadata = scope.bond(move || {
        Some(Box::new(Semantics {
            id: Some(id.clone()),
            option: Some(OptionSemantics {
                owner: semantic_owner.clone(),
            }),
            selected: Some(selected.get().as_deref() == Some(&option_key)),
            ..Default::default()
        }))
    });
    let controller = state.clone();
    let weak = scope.downgrade();
    let choose_key = option.key.as_str().to_owned();
    let pointer = state.clone();
    let weak_pointer = scope.downgrade();
    let pointer_member = member.clone();
    let mut base = if common.unstyled {
        Style::new()
    } else {
        common.theme.control(common.size)
    };
    if !common.unstyled {
        base.border_width = Some(Edges::all(0.0));
        base.background = Some(Color(0, 0, 0, 0));
    }
    let styled = state.clone();
    let styled_member = member.clone();
    let accent = common.theme.accent_soft;
    let foreground = common.theme.accent_text;
    let plain = common.unstyled;
    let style = scope.bond(move || {
        let mut value = base.clone();
        if !plain && styled.cursor.current().ok().flatten().as_ref() == Some(&styled_member) {
            value.background = Some(accent);
            value.color = Some(foreground);
        }
        value
    });
    glory::ui::button()
        .label(option.label.clone())
        .focus(FocusMode::Programmatic)
        .class(format!("sapid-select-option {}", common.theme.class()))
        .style(style)
        .semantics(metadata)
        .node_ref(reference)
        .enabled(enabled)
        .on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            if let Err(error) = controller.choose(Some(choose_key.clone())) {
                scope.report_error(error);
                return;
            }
            controller.activated.emit(scope.id(), event);
        })
        .on(EventKind::Pointer, move |event| {
            let Some(scope) = weak_pointer.upgrade() else {
                return;
            };
            if matches!(
                event,
                Event::Pointer {
                    phase: glory::render::PointerPhase::Move,
                    ..
                }
            ) && let Err(error) = pointer.cursor.observe(&pointer_member)
            {
                scope.report_error(error);
            }
        })
        .show_in(scope)?;
    parent.retain(item_scope)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glory::render::NodeId;
    use glory::ui::App;
    fn nodes(app: &App) -> Vec<NodeId> {
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        let mut result = Vec::new();
        while let Some(id) = pending.pop() {
            result.push(id);
            pending.extend(tree.node(id).unwrap().children.iter().copied());
        }
        result
    }
    fn trigger(app: &App) -> NodeId {
        nodes(app)
            .into_iter()
            .find(|id| {
                app.tree()
                    .node(*id)
                    .unwrap()
                    .descriptor
                    .view()
                    .semantics
                    .as_ref()
                    .is_some_and(|s| s.combobox.is_some())
            })
            .unwrap()
    }
    fn option_node(app: &App, key: &str) -> NodeId {
        nodes(app)
            .into_iter()
            .find(|id| {
                let tree = app.tree();
                let d = tree.node(*id).unwrap().descriptor.view();
                d.semantics.as_ref().is_some_and(|s| s.option.is_some())
                    && matches!(d.props,Props::Button(button) if button.label==key)
            })
            .unwrap()
    }
    #[test]
    fn listbox_keeps_trigger_focus_contract_and_selection_distinct_from_activity() {
        let app = App::memory();
        app.mount(select("local").options([
            crate::select_option("a", "A"),
            crate::select_option("b", "B"),
            crate::select_option("c", "C").disabled(true),
        ]))
        .unwrap();
        let trigger = trigger(&app);
        app.dispatch(
            trigger,
            Event::Choice {
                from: trigger,
                command: ChoiceCommand::OpenLast,
            },
        )
        .unwrap();
        let tree = app.tree();
        let metadata = tree
            .node(trigger)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        assert_eq!(metadata.expanded, Some(true));
        assert!(
            metadata
                .active_descendant
                .as_ref()
                .unwrap()
                .as_str()
                .ends_with("1_b")
        );
        drop(tree);
        let b = option_node(&app, "B");
        assert_eq!(
            app.tree()
                .node(b)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected,
            Some(false)
        );
        app.dispatch(
            trigger,
            Event::Choice {
                from: trigger,
                command: ChoiceCommand::Commit,
            },
        )
        .unwrap();
        assert_eq!(
            app.tree()
                .node(b)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected,
            Some(true)
        );
        let tree = app.tree();
        let metadata = tree
            .node(trigger)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        assert_eq!(metadata.expanded, Some(false));
        assert_eq!(metadata.active_descendant, None);
    }
    #[test]
    fn controlled_rejection_does_not_write_sources_or_leave_stale_active_relations() {
        let app = App::memory();
        let value = app.scope().cage(Some("a".to_owned()));
        let open = app.scope().cage(true);
        let calls = Rc::new(RefCell::new(Vec::new()));
        let typed = calls.clone();
        app.mount(
            select("controlled")
                .value(value)
                .open(open)
                .options([
                    crate::select_option("a", "A"),
                    crate::select_option("b", "B"),
                ])
                .on_value_change(move |value| typed.borrow_mut().push(value)),
        )
        .unwrap();
        let trigger = trigger(&app);
        let b = option_node(&app, "B");
        app.dispatch(b, Event::Activate).unwrap();
        assert_eq!(value.get(), Some("a".into()));
        assert!(open.get());
        assert_eq!(*calls.borrow(), [Some("b".into())]);
        assert_eq!(
            app.tree()
                .node(b)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected,
            Some(false)
        );
        open.revise(|value| *value = false);
        app.flush().unwrap();
        assert_eq!(
            app.tree()
                .node(trigger)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .active_descendant,
            None
        );
    }
    #[test]
    fn field_required_is_on_the_visible_trigger_and_proxy_only_carries_successful_values() {
        let snapshot = glory::ssr::snapshot(
            crate::field("field-choice")
                .label("Plan")
                .required(true)
                .form_name("plan")
                .control(|field| {
                    select("plan")
                        .field(field)
                        .default_value(Some("a".into()))
                        .option(crate::select_option("a", "A").value("business"))
                }),
        )
        .unwrap();
        assert!(
            snapshot.html.contains("role=\"combobox\"")
                && snapshot.html.contains("aria-required=\"true\"")
                && snapshot.html.contains("role=\"listbox\"")
                && snapshot.html.contains("role=\"option\"")
        );
        assert!(
            !snapshot.html.contains(" required=\"\"")
                && snapshot.html.contains("name=\"plan\"")
                && snapshot.html.contains("value=\"business\" selected")
        );
    }
    #[test]
    fn bounded_configuration_and_initial_controlled_sources_fail_before_nodes() {
        let app = App::memory();
        let seen = Rc::new(std::cell::Cell::new(0));
        let count = seen.clone();
        let widget = select("bounded").options((0..1_000_000).map(move |index| {
            count.set(count.get() + 1);
            crate::select_option(format!("k{index}"), "Label")
        }));
        assert_eq!(seen.get(), 1025);
        assert!(app.mount(widget).is_err());
        assert_eq!(app.stats().nodes, 0);
        assert!(
            app.mount(select("large").value(Some("x".repeat(1_000_000))))
                .is_err()
        );
        assert_eq!(app.stats().nodes, 0);
    }

    #[test]
    fn readonly_after_open_still_allows_escape_without_value_interaction() {
        let app = App::memory();
        let readonly = app.scope().cage(false);
        app.mount(
            select("readonly-open")
                .readonly(readonly)
                .default_open(true)
                .default_value(Some("a".into()))
                .options([
                    crate::select_option("a", "A"),
                    crate::select_option("b", "B"),
                ]),
        )
        .unwrap();
        readonly.revise(|value| *value = true);
        app.flush().unwrap();
        let trigger = trigger(&app);
        app.dispatch(option_node(&app, "B"), Event::Activate)
            .unwrap();
        assert_eq!(
            app.tree()
                .node(option_node(&app, "A"))
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected,
            Some(true)
        );
        app.dispatch(
            trigger,
            Event::Choice {
                from: trigger,
                command: ChoiceCommand::Close,
            },
        )
        .unwrap();
        let tree = app.tree();
        let semantics = tree
            .node(trigger)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap();
        assert_eq!(semantics.expanded, Some(false));
        assert_eq!(semantics.active_descendant, None);
    }

    #[test]
    fn unstyled_retains_layout_without_panel_or_clear_button_appearance() {
        let app = App::memory();
        app.mount(
            select("unstyled")
                .unstyled()
                .clearable(true)
                .default_value(Some("a".into()))
                .option(crate::select_option("a", "A")),
        )
        .unwrap();
        let tree = app.tree();
        for id in nodes(&app) {
            let descriptor = tree.node(id).unwrap().descriptor.view();
            if descriptor.role == Some(Role::Listbox)
                || matches!(descriptor.props, Props::Button(button) if button.label == "Clear selection" || button.label == "A")
            {
                assert_eq!(descriptor.style.background, None);
                assert_eq!(descriptor.style.border_width, None);
                assert_eq!(descriptor.style.border_color, None);
            }
        }
    }

    #[test]
    fn invalid_live_source_preserves_last_valid_selection_and_recovers() {
        let app = App::memory();
        let value = app.scope().cage(Some("a".to_owned()));
        app.mount(select("recover").value(value).options([
            crate::select_option("a", "A"),
            crate::select_option("b", "B"),
        ]))
        .unwrap();
        value.revise(|value| *value = Some("x".repeat(1_000_000)));
        assert!(app.flush().is_err());
        let a = option_node(&app, "A");
        assert_eq!(
            app.tree()
                .node(a)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected,
            Some(true)
        );
        value.revise(|value| *value = Some("b".into()));
        app.flush().unwrap();
        let b = option_node(&app, "B");
        assert_eq!(
            app.tree()
                .node(b)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .unwrap()
                .selected,
            Some(true)
        );
    }

    #[test]
    fn repeated_typeahead_cycles_activity_without_reselecting_the_controlled_value() {
        let app = App::memory();
        let selected = app.scope().cage(Some("a".to_owned()));
        let open = app.scope().cage(true);
        let readonly = app.scope().cage(false);
        let enabled = app.scope().cage(true);
        app.mount(
            select("typeahead-cycle")
                .value(selected)
                .open(open)
                .readonly(readonly)
                .enabled(enabled)
                .options([
                    crate::select_option("a", "Alpha"),
                    crate::select_option("b", "Beta"),
                    crate::select_option("c", "Bravo"),
                ]),
        )
        .unwrap();
        let trigger = trigger(&app);
        let search = |at_millis| Event::Typeahead {
            from: trigger,
            text: "b".into(),
            at_millis,
        };
        app.dispatch(trigger, search(100)).unwrap();
        assert_eq!(
            app.tree().resolve_active_descendant(trigger).unwrap(),
            Some(option_node(&app, "Beta"))
        );
        app.dispatch(trigger, search(101)).unwrap();
        assert_eq!(
            app.tree().resolve_active_descendant(trigger).unwrap(),
            Some(option_node(&app, "Bravo"))
        );
        assert_eq!(selected.get(), Some("a".into()));
        readonly.revise(|value| *value = true);
        app.flush().unwrap();
        app.dispatch(trigger, search(102)).unwrap();
        assert_eq!(
            app.tree().resolve_active_descendant(trigger).unwrap(),
            Some(option_node(&app, "Bravo"))
        );
        enabled.revise(|value| *value = false);
        app.flush().unwrap();
        app.dispatch(trigger, search(103)).unwrap();
        assert_eq!(app.tree().resolve_active_descendant(trigger).unwrap(), None);
        assert_eq!(selected.get(), Some("a".into()));
        enabled.revise(|value| *value = true);
        readonly.revise(|value| *value = false);
        open.revise(|value| *value = false);
        app.flush().unwrap();
        app.dispatch(trigger, search(104)).unwrap();
        assert_eq!(app.tree().resolve_active_descendant(trigger).unwrap(), None);
        assert!(!open.get());
        assert_eq!(selected.get(), Some("a".into()));
    }

    #[test]
    fn native_keys_in_one_drained_tick_use_live_open_state_instead_of_old_dom_metadata() {
        use glory::render::{
            Capabilities, Error, Input, KeyboardInput, Mutation, Query, QueryToken, Receipt,
            Render, RuntimeId, memory::Memory,
        };
        struct Queued {
            memory: Memory,
            input: Rc<RefCell<Vec<Input>>>,
        }
        impl Render for Queued {
            fn style_profile(&self) -> glory::style::Profile {
                glory::style::Profile::RECORD
            }
            fn capabilities(&self) -> Capabilities {
                Capabilities::default()
            }
            fn apply(&mut self, mutations: &[Mutation]) -> std::result::Result<Receipt, Error> {
                self.memory.apply(mutations)
            }
            fn request(
                &mut self,
                query: Query,
                token: QueryToken,
            ) -> std::result::Result<(), Error> {
                self.memory.request(query, token)
            }
            fn drain(&mut self, inputs: &mut Vec<Input>) {
                inputs.append(&mut self.input.borrow_mut());
                self.memory.drain(inputs);
            }
        }
        let input = Rc::new(RefCell::new(Vec::new()));
        let queue = input.clone();
        let app = App::new(move |runtime: RuntimeId| Queued {
            memory: Memory::new(runtime),
            input: queue,
        });
        let selected = app.scope().cage(Some("a".to_owned()));
        let proposals = Rc::new(RefCell::new(Vec::new()));
        let out = proposals.clone();
        app.mount(
            select("rapid")
                .value(selected)
                .on_value_change(move |value| {
                    out.borrow_mut().push(value.clone());
                    selected.revise(|current| *current = value);
                })
                .options([
                    crate::select_option("a", "Alpha"),
                    crate::select_option("b", "Beta"),
                ]),
        )
        .unwrap();
        let trigger = trigger(&app);
        for key in ["ArrowDown", "ArrowDown", "Enter"] {
            let command = KeyboardInput {
                key,
                ..Default::default()
            }
            .choice(ComboboxKind::Select, false)
            .unwrap();
            input.borrow_mut().push(Input::Event {
                node: trigger,
                event: Event::Choice {
                    from: trigger,
                    command,
                },
            });
        }
        app.pump().unwrap();
        assert_eq!(selected.get(), Some("b".into()));
        assert_eq!(*proposals.borrow(), [Some("b".into())]);
        assert_eq!(app.tree().resolve_active_descendant(trigger).unwrap(), None);
        app.dispatch(
            trigger,
            Event::Choice {
                from: trigger,
                command: ChoiceCommand::Next,
            },
        )
        .unwrap();
        input.borrow_mut().extend([
            Input::Event {
                node: trigger,
                event: Event::Choice {
                    from: trigger,
                    command: ChoiceCommand::Close,
                },
            },
            Input::Event {
                node: option_node(&app, "Alpha"),
                event: Event::Activate,
            },
        ]);
        app.pump().unwrap();
        assert_eq!(selected.get(), Some("b".into()));
        assert_eq!(*proposals.borrow(), [Some("b".into())]);
    }
}
