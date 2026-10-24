use crate::Theme;
use crate::appearance::{Common, merge};
use glory::render::{NodeId, SemanticId};
use glory::style::{Display, FlexDirection, Length, Style};
use glory::ui::list::{List, ListStats};
use glory::ui::{Children, IntoChildren, Result, WeakScope, Widget};
use glory::{Cage, Lotus, Scope};
use std::{cell::Cell, rc::Rc};

pub const MAX_MESSAGE_ROWS: usize = 256;
pub const MAX_MESSAGE_MEASUREMENTS: usize = 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageScrollStatus {
    pub following: bool,
    pub unread: u64,
}
struct Controller {
    scope: WeakScope,
    maximum: Cell<f64>,
    position: Cage<f64>,
    status: Cage<MessageScrollStatus>,
}
impl Controller {
    fn scope(&self) -> Result<Scope> {
        self.scope.upgrade().ok_or(glory::ui::Error::Gone)
    }
    fn accept(&self, stats: &ListStats) -> Result {
        self.scope()?;
        let maximum = (stats.total - stats.viewport.height).max(0.);
        self.maximum.set(maximum);
        let mut status = self.status.try_get()?;
        if stats.user_moved {
            status.following = maximum - stats.logical <= 2.;
            if status.following {
                status.unread = 0;
            }
        }
        if self.status.try_get()? != status {
            self.status.try_revise(|current| *current = status)?;
        }
        if status.following && self.position.try_get()? != maximum {
            self.position.try_revise(|position| *position = maximum)?;
        }
        Ok(())
    }
}
/// A current Owner-bound controller, without message data or visited-key history.
#[derive(Clone)]
pub struct MessageScrollerHandle {
    inner: Rc<Controller>,
}
impl MessageScrollerHandle {
    pub fn status(&self) -> Result<MessageScrollStatus> {
        self.inner.scope()?;
        Ok(self.inner.status.try_get()?)
    }
    pub fn status_source(&self) -> Result<Lotus<MessageScrollStatus>> {
        self.inner.scope()?;
        Ok(self.inner.status.into())
    }
    /// Report accepted newly appended messages, after the application's data
    /// update. Prepending history, editing content and rejected proposals must
    /// not call this method. Repeated calls in one tick accumulate exactly.
    pub fn appended(&self, count: u64) -> Result {
        self.inner.scope()?;
        let mut status = self.inner.status.try_get()?;
        if !status.following && count != 0 {
            status.unread =
                status
                    .unread
                    .checked_add(count)
                    .ok_or(glory::render::Error::InvalidProps(
                        "message unread count exhausted",
                    ))?;
            self.inner.status.try_revise(|current| *current = status)?;
        }
        Ok(())
    }
    pub fn pause(&self) -> Result {
        self.inner.scope()?;
        if self.inner.status.try_get()?.following {
            self.inner
                .status
                .try_revise(|status| status.following = false)?;
        }
        Ok(())
    }
    /// Explicit programmatic positioning. This pauses following without
    /// inventing a physical user event or discarding unread notifications.
    pub fn seek(&self, logical: f64) -> Result {
        self.inner.scope()?;
        if !logical.is_finite() || !(0. ..=glory::ui::list::MAX_EXTENT).contains(&logical) {
            return Err(
                glory::render::Error::InvalidProps("invalid logical message seek").into(),
            );
        }
        let position = logical.min(self.inner.maximum.get());
        self.pause()?;
        if self.inner.position.try_get()? != position {
            self.inner.position.try_revise(|value| *value = position)?;
        }
        Ok(())
    }
    pub fn follow_tail(&self) -> Result {
        self.inner.scope()?;
        self.inner.status.try_revise(|status| {
            status.following = true;
            status.unread = 0;
        })?;
        let maximum = self.inner.maximum.get();
        if self.inner.position.try_get()? != maximum {
            self.inner
                .position
                .try_revise(|position| *position = maximum)?;
        }
        Ok(())
    }
}

pub struct MessageScroller {
    list: Option<List<SemanticId>>,
    common: Common,
    height: f64,
    estimate: f64,
    row_limit: usize,
    measure_limit: usize,
    overscan: usize,
    following: bool,
    ready: Option<Box<dyn FnMut(MessageScrollerHandle)>>,
    inspect: Option<Box<dyn FnMut(ListStats)>>,
}
/// Only mounted-window indices and one focused key are read. `locate` must be
/// an indexed lookup, rather than a scan of the application message source.
pub fn message_scroller<W: Widget>(
    count: impl Into<Lotus<usize>>,
    key: impl Fn(usize) -> SemanticId + 'static,
    locate: impl Fn(&SemanticId) -> Option<usize> + 'static,
    render: impl FnMut(Cage<usize>, SemanticId) -> W + 'static,
) -> MessageScroller {
    MessageScroller {
        list: Some(glory::ui::list(count, key, render).locate(locate)),
        common: Common::default(),
        height: 480.,
        estimate: 96.,
        row_limit: 64,
        measure_limit: 256,
        overscan: 2,
        following: true,
        ready: None,
        inspect: None,
    }
}
impl MessageScroller {
    /// Accessible name of the actual windowed list, independently of any
    /// surrounding Chat/Terminal group.
    pub fn name(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.name = Some(value.into());
        self
    }
    pub fn height(mut self, value: f64) -> Self {
        self.height = value;
        self
    }
    pub fn estimate(mut self, value: f64) -> Self {
        self.estimate = value;
        self
    }
    pub fn row_limit(mut self, value: usize) -> Self {
        self.row_limit = value;
        self
    }
    pub fn measure_limit(mut self, value: usize) -> Self {
        self.measure_limit = value;
        self
    }
    pub fn overscan(mut self, value: usize) -> Self {
        self.overscan = value;
        self
    }
    pub fn initially_following(mut self, value: bool) -> Self {
        self.following = value;
        self
    }
    pub fn revision(mut self, value: impl Into<Lotus<u64>>) -> Self {
        self.list = self.list.take().map(|list| list.revision(value));
        self
    }
    pub fn on_ready(mut self, value: impl FnMut(MessageScrollerHandle) + 'static) -> Self {
        self.ready = Some(Box::new(value));
        self
    }
    pub fn inspect(mut self, value: impl FnMut(ListStats) + 'static) -> Self {
        self.inspect = Some(Box::new(value));
        self
    }
    pub fn theme(mut self, value: Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
}
impl Widget for MessageScroller {
    fn build(&mut self, scope: &Scope) -> Result {
        if !self.height.is_finite()
            || !(1. ..=1_000_000.).contains(&self.height)
            || !self.estimate.is_finite()
            || !(1. ..=65536.).contains(&self.estimate)
            || !(1..=MAX_MESSAGE_ROWS).contains(&self.row_limit)
            || !(1..=MAX_MESSAGE_MEASUREMENTS).contains(&self.measure_limit)
            || self.overscan > 8
        {
            return Err(glory::render::Error::InvalidProps(
                "invalid message window budget or geometry",
            )
            .into());
        }
        let controller = Rc::new(Controller {
            scope: scope.downgrade(),
            maximum: Cell::new(0.),
            position: scope.cage(0.),
            status: scope.cage(MessageScrollStatus {
                following: self.following,
                unread: 0,
            }),
        });
        let name = self.common.name.take().unwrap_or_else(|| "Messages".into());
        let name = crate::composite::validated(scope, &name, |value| {
            if value.len() > 128
                || value.trim().is_empty()
                || value.contains(['\0', '\r', '\n'])
            {
                return Err(glory::render::Error::InvalidProps(
                    "invalid bounded message window name",
                )
                .into());
            }
            Ok(())
        })?;
        let weak = Rc::downgrade(&controller);
        // Acknowledging the list's current logical position is essential: a
        // later explicit follow command must differ from its last accepted
        // position, even if the content's maximum has not changed.
        let position = controller.position;
        let mut inspect = self.inspect.take();
        let list = self
            .list
            .take()
            .ok_or(glory::ui::Error::Gone)?
            .name(name)
            .height(self.height)
            .measure(self.estimate)
            .row_limit(self.row_limit)
            .measure_limit(self.measure_limit)
            .overscan(self.overscan)
            .position(controller.position)
            .on_scroll(move |actual| {
                if position.get_untracked() != actual {
                    position.revise(|current| *current = actual);
                }
            })
            .inspect(move |stats| {
                let Some(controller) = weak.upgrade() else {
                    return;
                };
                if let Err(error) = controller.accept(&stats) {
                    if let Some(scope) = controller.scope.upgrade() {
                        scope.report_error(error);
                    }
                    return;
                }
                if let Some(callback) = &mut inspect {
                    callback(stats);
                }
            });
        let (class, style) = self.common.appearance(
            scope,
            "sapid-message-scroller",
            Style::new().min_width(Length::Px(0.)),
        );
        let mut root = glory::ui::view().class(class).style(style);
        if let Some(reference) = self.common.node_ref {
            root = root.node_ref(reference);
        }
        root.fill(list).show_in(scope)?;
        if let Some(callback) = &mut self.ready {
            callback(MessageScrollerHandle {
                inner: controller.clone(),
            });
        }
        scope.retain(controller)?;
        Ok(())
    }
}

pub struct Chat {
    common: Common,
    scroller: Option<MessageScroller>,
    title: String,
    prompt: Option<Children>,
}
pub fn chat(scroller: MessageScroller) -> Chat {
    Chat {
        common: Common::default(),
        scroller: Some(scroller),
        title: "Conversation".into(),
        prompt: None,
    }
}
impl Chat {
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }
    pub fn theme(mut self, value: Theme) -> Self {
        self.common.theme = value;
        self
    }
    pub fn class(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.common.class = value.into();
        self
    }
    pub fn style(mut self, value: impl Into<Lotus<Style>>) -> Self {
        self.common.style = value.into();
        self
    }
    pub fn prompt(mut self, value: impl IntoChildren) -> Self {
        self.prompt = Some(value.into_children());
        self
    }
}
impl Widget for Chat {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.title.len() > 128
            || self.title.trim().is_empty()
            || self.title.contains(['\0', '\r', '\n'])
        {
            return Err(glory::render::Error::InvalidProps(
                "invalid bounded conversation name",
            )
            .into());
        }
        let (class, custom) = self.common.appearance(scope, "sapid-chat", Style::new());
        let style = scope.bond(move || {
            merge(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .min_width(Length::Px(0.)),
                custom.get(),
            )
        });
        let root = glory::ui::view()
            .class(class)
            .style(style)
            .name(self.title.clone());
        let scroller = self.scroller.take().ok_or(glory::ui::Error::Gone)?;
        if let Some(prompt) = self.prompt.take() {
            root.fill((
                scroller,
                glory::ui::view().class("sapid-chat-prompt").fill(prompt),
            ))
            .show_in(scope)
        } else {
            root.fill(scroller).show_in(scope)
        }
    }
}
