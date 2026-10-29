use glory::Scope;
use glory::core::Vary;
use glory::render::EventKind;
use glory::ui::{Children, IntoChildren, Result, Widget};

use crate::appearance::{Common, common_methods, configure, observe};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Light,
    Outline,
    Subtle,
}

pub struct Button {
    inner: Option<glory::ui::Button>,
    common: Common,
    variant: ButtonVariant,
    loading: Vary<bool>,
    toolbar: Option<Box<dyn ToolbarHooks>>,
    label: Vary<String>,
    body: Option<Children>,
    left: Option<Children>,
    right: Option<Children>,
    presses: Vec<Box<dyn FnMut()>>,
}
struct ToolbarItem {
    collection: crate::Collection,
    roving: crate::RovingFocus,
    key: String,
    reference: glory::Cage<Option<glory::render::NodeId>>,
    token: Option<glory::Cage<Option<crate::CollectionMember>>>,
}

// Only in_toolbar constructs this implementation/vtable. An ordinary Button
// calls the optional hooks without retaining the collection/roving code itself.
trait ToolbarHooks {
    fn configure(
        &mut self,
        scope: &Scope,
        inner: glory::ui::Button,
        handlers: &mut crate::appearance::Handlers,
    ) -> glory::ui::Button;
    fn register(self: Box<Self>, scope: &Scope, enabled: Vary<bool>) -> Result;
}

pub fn button() -> Button {
    Button {
        inner: Some(glory::ui::button()),
        common: Common::default(),
        variant: ButtonVariant::default(),
        loading: false.into(),
        toolbar: None,
        label: "".into(),
        body: None,
        left: None,
        right: None,
        presses: Vec::new(),
    }
}

impl Button {
    common_methods!();

    pub(crate) fn in_toolbar(
        mut self,
        scope: &Scope,
        collection: crate::Collection,
        roving: crate::RovingFocus,
        key: String,
    ) -> Result<Self> {
        if self.common.focus != glory::render::FocusMode::Auto {
            return Err(glory::render::Error::InvalidProps(
                "toolbar owns its button tab stops",
            )
            .into());
        }
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        self.common.node_ref = Some(reference);
        self.toolbar = Some(Box::new(ToolbarItem {
            collection,
            roving,
            key,
            reference,
            token: None,
        }));
        Ok(self)
    }

    /// Apply group-owned layout before explicit child declarations. Reactive
    /// child styles and enabled state retain their original source of truth.
    pub(crate) fn in_group(
        mut self,
        scope: &Scope,
        theme: crate::Theme,
        size: crate::Size,
        enabled: Vary<bool>,
        unstyled: bool,
        layout: glory::style::Style,
    ) -> Self {
        self.common.theme = theme;
        self.common.size = size;
        self.common.unstyled |= unstyled;
        let own_enabled = self.common.enabled.clone();
        self.common.enabled = scope
            .bond(move || own_enabled.get() && enabled.get())
            .into();
        if !self.common.unstyled {
            let own_style = self.common.style.clone();
            let merged = move || crate::appearance::merge(layout.clone(), own_style.get());
            self.common.style = if self.common.style.is_reactive() {
                scope.bond(merged).into()
            } else {
                merged().into()
            };
        }
        self
    }

    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }

    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.body = Some(children.into_children());
        self
    }
    pub fn left_section(mut self, children: impl IntoChildren) -> Self {
        self.left = Some(children.into_children());
        self
    }
    pub fn right_section(mut self, children: impl IntoChildren) -> Self {
        self.right = Some(children.into_children());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Loading disables native activation. The application supplies any loading
    /// label/status content; no timer, spinner, or live announcement is implied.
    pub fn loading(mut self, value: impl Into<Vary<bool>>) -> Self {
        self.loading = value.into();
        self
    }

    pub fn on_press(mut self, handler: impl FnMut() + 'static) -> Self {
        self.presses.push(Box::new(handler));
        self
    }
}

impl Widget for Button {
    fn build(&mut self, scope: &Scope) -> Result {
        let mut inner = self.inner.take().ok_or(glory::ui::Error::Gone)?;
        let has_sections = self.left.is_some() || self.right.is_some();
        if has_sections {
            if let Some(name) = self.common.name.take() {
                self.common.name = Some(
                    crate::composite::validated(scope, &name, |value| section_name(value))?
                        .into(),
                );
            } else {
                let accepted = crate::composite::validated(scope, &self.label, |value| {
                    section_name(value)
                })?;
                self.label = accepted.into();
                self.common.name = Some(accepted.into());
            }
            inner = inner.label("").fill((
                glory::ui::group().fill(self.left.take().unwrap_or_else(|| ().into_children())),
                glory::ui::text()
                    .class("sapid-button-label")
                    .content(self.label.clone())
                    .selectable(false),
                glory::ui::group().fill(self.body.take().unwrap_or_else(|| ().into_children())),
                glory::ui::group()
                    .fill(self.right.take().unwrap_or_else(|| ().into_children())),
            ));
            let mut structural = glory::style::Style::new()
                .display(glory::style::Display::InlineFlex)
                .gap(8.0);
            structural.align = Some(glory::style::Align::Center);
            structural.justify = Some(glory::style::Justify::Center);
            let own = self.common.style.clone();
            self.common.style = if own.is_reactive() {
                scope
                    .bond(move || crate::appearance::merge(structural.clone(), own.get()))
                    .into()
            } else {
                crate::appearance::merge(structural, own.get()).into()
            };
        } else {
            inner = inner
                .label(self.label.clone())
                .fill(self.body.take().unwrap_or_else(|| ().into_children()));
        }
        let mut base = self.common.theme.control(self.common.size);
        let class = match self.variant {
            ButtonVariant::Primary => {
                base.background = Some(self.common.theme.accent);
                base.color = Some(self.common.theme.on_accent);
                base.border_color = Some(self.common.theme.accent);
                "sapid-button sapid-button-primary"
            }
            ButtonVariant::Secondary => "sapid-button sapid-button-secondary",
            ButtonVariant::Light | ButtonVariant::Outline | ButtonVariant::Subtle => {
                base.background = Some(if self.variant == ButtonVariant::Light {
                    self.common.theme.accent_soft
                } else {
                    glory::style::Color(0, 0, 0, 0)
                });
                base.color = Some(self.common.theme.accent_text);
                base.border_color = Some(if self.variant == ButtonVariant::Outline {
                    self.common.theme.accent
                } else {
                    glory::style::Color(0, 0, 0, 0)
                });
                match self.variant {
                    ButtonVariant::Light => "sapid-button sapid-button-light",
                    ButtonVariant::Outline => "sapid-button sapid-button-outline",
                    _ => "sapid-button sapid-button-subtle",
                }
            }
            ButtonVariant::Danger => {
                base.background = Some(self.common.theme.danger_fill);
                base.color = Some(self.common.theme.on_danger);
                base.border_color = Some(self.common.theme.danger_fill);
                "sapid-button sapid-button-danger"
            }
        };
        let enabled = self.common.enabled.clone();
        let loading = self.loading.clone();
        let reactive = enabled.is_reactive() || loading.is_reactive();
        let effective = move || enabled.get() && !loading.get();
        let enabled: Vary<bool> = if reactive {
            scope.bond(effective).into()
        } else {
            effective().into()
        };
        // State colors are typed declarations so native and CSS-free renders
        // show the same disabled control. Explicit user declarations still win.
        let own_style = self.common.style.clone();
        let style_enabled = enabled.clone();
        let theme = self.common.theme.clone();
        let unstyled = self.common.unstyled;
        let state_style = move || {
            let mut base = if unstyled {
                glory::style::Style::new()
            } else {
                base.clone()
            };
            if !unstyled && !style_enabled.get() {
                base.background = Some(theme.disabled);
                base.color = Some(theme.disabled_text);
                base.border_color = Some(theme.border);
            }
            crate::appearance::merge(base, own_style.get())
        };
        self.common.style = if enabled.is_reactive() || self.common.style.is_reactive() {
            scope.bond(state_style).into()
        } else {
            state_style().into()
        };
        let mut inner =
            configure!(self.common, inner, scope, class, glory::style::Style::new());
        inner = inner.enabled(enabled.clone());
        let loading = self.loading.clone();
        let navigation_item = self.toolbar.is_some();
        let metadata = move || {
            Some(Box::new(glory::render::Semantics {
                busy: Some(loading.get()),
                navigation_item,
                ..Default::default()
            }))
        };
        inner = if self.loading.is_reactive() {
            inner.semantics(scope.bond(metadata))
        } else {
            inner.semantics(metadata())
        };
        let mut registration = self.toolbar.take();
        if let Some(toolbar) = &mut registration {
            inner = toolbar.configure(scope, inner, &mut self.common.handlers);
        }
        let mut presses = std::mem::take(&mut self.presses);
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let owner = scope.id();
        if !presses.is_empty() || !observers.is_empty() {
            inner = inner.on(EventKind::Activate, move |event| {
                for press in &mut presses {
                    if !owner.is_alive() {
                        return;
                    }
                    press();
                }
                observe(&mut observers, owner, event);
            });
        }
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            let owner = scope.id();
            inner = inner.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        inner.show_in(scope)?;
        if let Some(toolbar) = registration {
            toolbar.register(scope, enabled)?;
        }
        Ok(())
    }
}
impl ToolbarHooks for ToolbarItem {
    fn configure(
        &mut self,
        scope: &Scope,
        inner: glory::ui::Button,
        handlers: &mut crate::appearance::Handlers,
    ) -> glory::ui::Button {
        let token = scope.cage(None::<crate::CollectionMember>);
        self.token = Some(token);
        let roving = self.roving.clone();
        let inner = inner.focus(scope.bond(move || {
            if token
                .get()
                .as_ref()
                .is_some_and(|member| roving.is_tab_stop(member).unwrap_or(false))
            {
                glory::render::FocusMode::TabStop
            } else {
                glory::render::FocusMode::Programmatic
            }
        }));
        let roving = self.roving.clone();
        let mut observers = handlers
            .remove(&EventKind::FocusChanged)
            .unwrap_or_default();
        let owner = scope.id();
        let weak = scope.downgrade();
        inner.on(EventKind::FocusChanged, move |event| {
            if event == glory::render::Event::FocusChanged(true) {
                if let Some(scope) = weak.upgrade() {
                    let result = (|| -> Result {
                        if let Some(member) = token.try_get()? {
                            roving.observe(&member)?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                        return;
                    }
                } else {
                    return;
                }
            }
            observe(&mut observers, owner, event);
        })
    }
    fn register(self: Box<Self>, scope: &Scope, enabled: Vary<bool>) -> Result {
        let member = self.collection.register(
            scope,
            self.key,
            self.reference,
            enabled,
            String::new(),
        )?;
        self.token
            .expect("configured toolbar button")
            .try_revise(|value| *value = Some(member))?;
        Ok(())
    }
}
fn section_name(name: &str) -> Result {
    if name.trim().is_empty() {
        return Err(glory::render::Error::InvalidProps(
            "section buttons require a nonempty label or name",
        )
        .into());
    }
    Ok(())
}

impl Default for Button {
    fn default() -> Self {
        button()
    }
}
