//! Measured plain-text truncation. Interactive descendants are not accepted.
use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::{self, Callback};
use crate::state::Value;
use glory::render::{Change, Event, EventKind, NodeId, Patch, Semantics};
use glory::style::{Length, Limit, Overflow, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

pub const MAX_SPOILER_BYTES: usize = 65536;
pub struct Spoiler {
    common: Common,
    key: String,
    content: Lotus<String>,
    expanded: Value<bool>,
    max_height: f64,
    more: String,
    less: String,
    on_change: Option<Box<dyn FnMut(bool)>>,
    viewport: Option<Cage<Option<NodeId>>>,
    trigger: Option<Cage<Option<NodeId>>>,
}
pub fn spoiler(key: impl Into<String>) -> Spoiler {
    Spoiler {
        common: Common::default(),
        key: key.into(),
        content: "".into(),
        expanded: Value::default(),
        max_height: 120.0,
        more: "Show more".into(),
        less: "Show less".into(),
        on_change: None,
        viewport: None,
        trigger: None,
    }
}
impl Spoiler {
    common_methods!();
    pub fn content(mut self, content: impl Into<Lotus<String>>) -> Self {
        self.content = content.into();
        self
    }
    pub fn max_height(mut self, height: f64) -> Self {
        self.max_height = height;
        self
    }
    pub fn expanded(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.expanded.source = Some(value.into());
        self
    }
    pub fn default_expanded(mut self, value: bool) -> Self {
        self.expanded.default = value;
        self
    }
    pub fn labels(mut self, more: impl Into<String>, less: impl Into<String>) -> Self {
        self.more = more.into();
        self.less = less.into();
        self
    }
    pub fn on_expanded_change(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    pub fn viewport_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.viewport = Some(reference);
        self
    }
    pub fn trigger_ref(mut self, reference: Cage<Option<NodeId>>) -> Self {
        self.trigger = Some(reference);
        self
    }
}
impl Widget for Spoiler {
    fn build(&mut self, scope: &Scope) -> Result {
        let root_id = composite::id("spoiler", &self.key, "root", "root")?;
        let content_id = composite::id("spoiler", &self.key, "content", "root")?;
        if !self.max_height.is_finite()
            || self.max_height <= 0.0
            || [&self.more, &self.less]
                .into_iter()
                .any(|label| label.trim().is_empty() || label.contains(['\0', '\r', '\n']))
        {
            return Err(glory::render::Error::InvalidProps(
                "spoiler needs positive finite height and nonempty action labels",
            )
            .into());
        }
        let initial = glory::core::untracked(|| self.content.try_get())?;
        if initial.len() > MAX_SPOILER_BYTES {
            return Err(
                glory::render::Error::InvalidProps("spoiler text exceeds 64KiB").into(),
            );
        }
        let measured = scope.cage(if initial.is_empty() { Some(0.0) } else { None });
        let control = std::mem::take(&mut self.expanded).mount(scope);
        let open = control.source.clone();
        let style_open = open.clone();
        let max_height = self.max_height;
        let clip_style = scope.bond(move || {
            Style::new()
                .min_width(Length::Px(0.0))
                .max_height(if style_open.get() {
                    Limit::None
                } else {
                    Limit::Px(max_height)
                })
                .overflow_y(Overflow::Hidden)
        });
        let mut text_style = Style::new();
        if !self.common.unstyled {
            text_style = self.common.theme.label(self.common.size);
            text_style.font_weight = Some(400.0);
        }
        let trigger_focused = scope.cage(false);
        let text_ref = scope.cage(None);
        let body_ref = scope.cage(None);
        let on_measure = scope.downgrade();
        let body = glory::ui::view()
            .focus(glory::render::FocusMode::Programmatic)
            .class("sapid-spoiler-measure")
            .style(Style::new().min_width(Length::Px(0.0)))
            .node_ref(body_ref)
            .on(EventKind::Resize, move |event| {
                if let Event::Resize { size, .. } = event
                    && let Some(scope) = on_measure.upgrade()
                {
                    let result = (|| -> Result {
                        if size.height <= max_height + 0.5
                            && trigger_focused.try_get()?
                            && let Some(body) = body_ref.try_get()?
                        {
                            scope.action(glory::render::Action::Focus(body))?;
                        }
                        if measured.try_with(|old| *old != Some(size.height))? {
                            measured.try_revise(|height| *height = Some(size.height))?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        scope.report_error(error);
                    }
                }
            })
            .fill(
                glory::ui::text()
                    .kind(glory::render::TextKind::Paragraph)
                    .class("sapid-spoiler-text")
                    .style(text_style)
                    .content(initial.clone())
                    .node_ref(text_ref),
            );
        let viewport_ref = self.viewport.unwrap_or_else(|| scope.cage(None));
        let viewport = glory::ui::view()
            .class("sapid-spoiler-viewport")
            .style(clip_style)
            .node_ref(viewport_ref)
            .semantics(composite::metadata(Semantics {
                id: Some(content_id.clone()),
                ..Default::default()
            }))
            .fill(body);
        let more = self.more.clone();
        let less = self.less.clone();
        let label_open = open.clone();
        let label = scope.bond(move || {
            if label_open.get() {
                less.clone()
            } else {
                more.clone()
            }
        });
        let trigger_ref = self.trigger.unwrap_or_else(|| scope.cage(None));
        let metadata_open = open;
        let callback = Callback::new(self.on_change.take());
        let mut observers = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let weak = scope.downgrade();
        let focus_weak = scope.downgrade();
        let owner = scope.id();
        let mut trigger = glory::ui::button()
            .class("sapid-button sapid-spoiler-trigger")
            .label(label)
            .enabled(self.common.enabled.clone())
            .node_ref(trigger_ref)
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    controls: Some(content_id.clone()),
                    expanded: Some(metadata_open.get()),
                    ..Default::default()
                })
            }))
            .on(EventKind::FocusChanged, move |event| {
                if let Event::FocusChanged(focused) = event
                    && let Some(scope) = focus_weak.upgrade()
                    && let Err(error) = trigger_focused.try_revise(|value| *value = focused)
                {
                    scope.report_error(error);
                }
            })
            .on(EventKind::Activate, move |event| {
                let Some(scope) = weak.upgrade() else { return };
                let result = (|| -> Result {
                    if measured
                        .try_get()?
                        .is_some_and(|height| height <= max_height + 0.5)
                    {
                        return Ok(());
                    }
                    let next = !control.source.try_get()?;
                    control.accept(&next)?;
                    callback.emit(owner, next);
                    observe(&mut observers, owner, event);
                    Ok(())
                })();
                if let Err(error) = result {
                    scope.report_error(error);
                }
            });
        if !self.common.unstyled {
            trigger = trigger.style(self.common.theme.control(self.common.size));
        }
        let actions = glory::ui::view()
            .class("sapid-spoiler-actions")
            .semantics(scope.bond(move || {
                composite::metadata(Semantics {
                    hidden: measured
                        .get()
                        .is_some_and(|height| height <= max_height + 0.5),
                    ..Default::default()
                })
            }))
            .fill(trigger);
        let mut root = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-spoiler",
            composite::column(8.0)
        )
        .semantics(composite::metadata(Semantics {
            id: Some(root_id),
            ..Default::default()
        }))
        .fill((viewport, actions));
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            root = root.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        root.show_in(scope)?;
        let source = self.content.clone();
        let weak = scope.downgrade();
        let mut previous = initial;
        scope.binding(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let value = source.try_get()?;
                if value == previous {
                    return Ok(());
                }
                if value.len() > MAX_SPOILER_BYTES {
                    return Err(glory::render::Error::InvalidProps(
                        "spoiler text exceeds 64KiB",
                    )
                    .into());
                }
                if let Some(id) = text_ref.try_get()? {
                    scope.patch(id, Patch::Text(Change::Set(value.clone())))?;
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
