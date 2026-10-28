//! Literal stack frames and explicit application location requests.
use super::trace_model::*;
use super::traces::payload_text;
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback, Observers};
use glory::render::{DocumentPart, Event, EventKind, Role, TextKind};
use glory::style::{Display, Edges, Family, Length, Limit, Overflow, Style, WhiteSpace};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackLocationRequest {
    pub key: TraceKey,
    pub location: StackLocation,
}
struct State {
    owner: glory::core::InstanceId,
    live: Cell<bool>,
    accepted: Cage<StackTraceData>,
    enabled: Vary<bool>,
    requested: Callback<StackLocationRequest>,
    observers: Observers,
    interactive: bool,
    theme: crate::Theme,
    size: crate::Size,
    unstyled: bool,
}
impl State {
    fn request(&self, key: &TraceKey) -> Result {
        if !self.live.get() || !self.owner.is_alive() || !self.enabled.try_get()? {
            return Ok(());
        }
        let location = self
            .accepted
            .try_with(|d| d.frame(key).and_then(|f| f.source().cloned()))?;
        if let Some(location) = location {
            self.requested.emit(
                self.owner,
                StackLocationRequest {
                    key: key.clone(),
                    location,
                },
            );
        }
        Ok(())
    }
}
pub struct StackTrace {
    common: Common,
    data: Vary<StackTraceData>,
    requested: Option<Box<dyn FnMut(StackLocationRequest)>>,
    payload_height: f64,
}
pub fn stack_trace(data: impl Into<Vary<StackTraceData>>) -> StackTrace {
    StackTrace {
        common: Common::default(),
        data: data.into(),
        requested: None,
        payload_height: 240.,
    }
}
impl StackTrace {
    common_methods!();
    /// Requests application navigation; it never opens a path or URL itself.
    pub fn on_location(mut self, callback: impl FnMut(StackLocationRequest) + 'static) -> Self {
        self.requested = Some(Box::new(callback));
        self
    }
    pub fn payload_height(mut self, value: f64) -> Self {
        self.payload_height = value;
        self
    }
}
impl Widget for StackTrace {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.payload_height.is_finite() || !(80. ..=640.).contains(&self.payload_height) {
            return Err(
                glory::render::Error::InvalidProps("invalid stack payload height").into(),
            );
        }
        let accepted = composite::validated(scope, &self.data, |_: &StackTraceData| Ok(()))?;
        let state = Rc::new(State {
            owner: scope.id(),
            live: Cell::new(true),
            accepted,
            enabled: self.common.enabled.clone(),
            interactive: self.requested.is_some(),
            requested: Callback::new(self.requested.take()),
            observers: Observers::new(
                self.common
                    .handlers
                    .remove(&EventKind::Activate)
                    .unwrap_or_default(),
            ),
            theme: self.common.theme.clone(),
            size: self.common.size,
            unstyled: self.common.unstyled,
        });
        let weak = Rc::downgrade(&state);
        scope.on_cleanup(move || {
            if let Some(state) = weak.upgrade() {
                state.live.set(false);
                state.requested.clear();
                state.observers.clear();
            }
        })?;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let (class, appearance) =
            self.common
                .appearance(scope, "sapid-stack-trace", composite::column(8.));
        let style = scope.bond(move || {
            let mut s = appearance.get();
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
                    .unwrap_or_else(|| "Stack trace".to_owned().into()),
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
        let message = scope.bond(move || accepted.with(|d| payload_text(d.message())));
        let mut payload_style = Style::new()
            .display(Display::Block)
            .white_space(WhiteSpace::PreWrap)
            .min_width(Length::Px(0.))
            .max_width(Limit::Percent(100.))
            .max_height(Limit::Px(self.payload_height))
            .overflow_x(Overflow::Auto)
            .overflow_y(Overflow::Auto);
        if !self.common.unstyled {
            payload_style.background = Some(self.common.theme.surface);
            payload_style.color = Some(self.common.theme.foreground);
            payload_style.padding = Some(Edges::all(8.));
            payload_style.font_family = Some(vec![Family::Mono]);
        }
        glory::ui::text()
            .kind(TextKind::Code)
            .class("sapid-stack-message")
            .content(message)
            .style(payload_style)
            .show_in(&area)?;
        let list_ref = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::OrderedList)
            .class("sapid-stack-frames")
            .style(
                Style::new()
                    .min_width(Length::Px(0.))
                    .margin(Edges::all(0.))
                    .padding(Edges {
                        left: 20.,
                        ..Edges::all(0.)
                    }),
            )
            .node_ref(list_ref)
            .show_in(&area)?;
        let list = scope.child_in(list_ref.get().ok_or(glory::ui::Error::Gone)?)?;
        let frames = scope.bond(move || {
            accepted.with(|d| {
                d.frames()
                    .iter()
                    .map(|f| f.key().clone())
                    .collect::<Vec<_>>()
            })
        });
        let rows = state.clone();
        glory::ui::each(
            frames,
            |key| key.clone(),
            move |key, _| FrameRow {
                key: key.get_untracked(),
                state: rows.clone(),
            },
        )
        .show_in(&list)?;
        glory::ui::when(accepted.map(|d| d.frames().is_empty()), || {
            glory::ui::text().content("No stack frames")
        })
        .show_in(&area)?;
        scope.retain(list)?;
        scope.retain(area)?;
        scope.retain(state)
    }
}
struct FrameRow {
    key: TraceKey,
    state: Rc<State>,
}
impl Widget for FrameRow {
    fn build(&mut self, scope: &Scope) -> Result {
        let reference = scope.cage(None);
        glory::ui::view()
            .document_part(DocumentPart::ListItem)
            .class("sapid-stack-frame")
            .style(Style::new().min_width(Length::Px(0.)).margin(Edges {
                bottom: 8.,
                ..Edges::all(0.)
            }))
            .node_ref(reference)
            .show_in(scope)?;
        let area = scope.child_in(reference.get().ok_or(glory::ui::Error::Gone)?)?;
        let accepted = self.state.accepted;
        let key = self.key.clone();
        let function = scope.bond(move || {
            accepted.with(|d| {
                d.frame(&key)
                    .map(|f| f.function().to_owned())
                    .unwrap_or_default()
            })
        });
        glory::ui::text()
            .kind(TextKind::Code)
            .content(function)
            .class("sapid-stack-function")
            .style(
                Style::new()
                    .display(Display::Block)
                    .min_width(Length::Px(0.)),
            )
            .show_in(&area)?;
        let key = self.key.clone();
        let has_location = scope.bond(move || {
            accepted.with(|d| d.frame(&key).is_some_and(|f| f.source().is_some()))
        });
        let key = self.key.clone();
        let state = self.state.clone();
        glory::ui::when(has_location, move || LocationWidget {
            key: key.clone(),
            state: state.clone(),
        })
        .show_in(&area)?;
        scope.retain(area)
    }
}
struct LocationWidget {
    key: TraceKey,
    state: Rc<State>,
}
impl Widget for LocationWidget {
    fn build(&mut self, scope: &Scope) -> Result {
        let accepted = self.state.accepted;
        let key = self.key.clone();
        let location = scope.bond(move || {
            accepted.with(|d| {
                d.frame(&key)
                    .and_then(|f| f.source())
                    .map_or_else(String::new, |l| {
                        format!(
                            "{}:{}{}",
                            l.file(),
                            l.line(),
                            l.column().map_or_else(String::new, |c| format!(":{c}"))
                        )
                    })
            })
        });
        if self.state.interactive {
            let state = self.state.clone();
            let key = self.key.clone();
            let weak = scope.downgrade();
            let origin = scope.id();
            let mut button = crate::button()
                .theme(self.state.theme.clone())
                .size(self.state.size)
                .class("sapid-stack-location")
                .label(location)
                .enabled(self.state.enabled.clone())
                .on_press(move || {
                    if let Err(error) = state.request(&key)
                        && let Some(scope) = weak.upgrade()
                    {
                        scope.report_error(error);
                    }
                    if origin.is_alive() {
                        state.observers.emit(state.owner, Event::Activate);
                    }
                });
            if self.state.unstyled {
                button = button.unstyled();
            }
            button.show_in(scope)
        } else {
            glory::ui::text()
                .kind(TextKind::Code)
                .class("sapid-stack-location")
                .content(location)
                .style(
                    Style::new()
                        .display(Display::Block)
                        .min_width(Length::Px(0.)),
                )
                .show_in(scope)
        }
    }
}
