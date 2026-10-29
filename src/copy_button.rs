use crate::appearance::{Common, common_methods, configure, observe};
use crate::composite::Callback;
use crate::state::{DelayFactory, OwnerDelay};
use glory::render::{
    ClipboardOutcome, ClipboardText, Event, EventKind, MAX_CLIPBOARD_BYTES, Semantics,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::rc::Rc;

/// Current feedback, never an optimistic claim that the clipboard was written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CopyState {
    #[default]
    Idle,
    Pending,
    Written,
    Denied,
    Unavailable,
    Failed,
}
impl From<ClipboardOutcome> for CopyState {
    fn from(value: ClipboardOutcome) -> Self {
        match value {
            ClipboardOutcome::Written => Self::Written,
            ClipboardOutcome::Denied => Self::Denied,
            ClipboardOutcome::Unavailable => Self::Unavailable,
            ClipboardOutcome::Failed => Self::Failed,
        }
    }
}
pub struct CopyButton {
    common: Common,
    source: Vary<String>,
    label: Vary<String>,
    copied_label: String,
    failed_label: String,
    state_ref: Option<Cage<CopyState>>,
    delay: Option<DelayFactory>,
    result: Option<Box<dyn FnMut(ClipboardOutcome)>>,
}
pub fn copy_button() -> CopyButton {
    CopyButton {
        common: Common::default(),
        source: "".into(),
        label: "Copy".into(),
        copied_label: "Copied".into(),
        failed_label: "Copy failed".into(),
        state_ref: None,
        delay: None,
        result: None,
    }
}
impl CopyButton {
    common_methods!();
    pub fn source(mut self, value: impl Into<Vary<String>>) -> Self {
        self.source = value.into();
        self
    }
    pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
        self.label = value.into();
        self
    }
    pub fn copied_label(mut self, value: impl Into<String>) -> Self {
        self.copied_label = value.into();
        self
    }
    pub fn failed_label(mut self, value: impl Into<String>) -> Self {
        self.failed_label = value.into();
        self
    }
    /// Output only. A write by the application does not request an OS operation.
    pub fn state_ref(mut self, value: Cage<CopyState>) -> Self {
        self.state_ref = Some(value);
        self
    }
    /// Host-supplied timer; one Owner-owned task resets terminal feedback.
    /// Without a timer feedback lasts until the next request or source change.
    pub fn feedback_delay(
        mut self,
        factory: impl Fn() -> glory::core::LocalFuture + 'static,
    ) -> Self {
        self.delay = Some(Rc::new(factory));
        self
    }
    pub fn on_result(mut self, callback: impl FnMut(ClipboardOutcome) + 'static) -> Self {
        self.result = Some(Box::new(callback));
        self
    }
}
fn canonical(source: &str) -> Result<String> {
    if source.len() > MAX_CLIPBOARD_BYTES {
        return Err(glory::render::Error::InvalidProps(
            "clipboard text exceeds 65536 UTF-8 bytes",
        )
        .into());
    }
    let text = glory::render::normalize_text(source, true);
    if text.len() > MAX_CLIPBOARD_BYTES {
        return Err(glory::render::Error::InvalidProps(
            "canonical clipboard text exceeds 65536 UTF-8 bytes",
        )
        .into());
    }
    Ok(text.into_owned())
}
impl Widget for CopyButton {
    fn build(&mut self, scope: &Scope) -> Result {
        let initial = self.source.try_with(|value| canonical(value))??;
        let accepted = scope.cage(ClipboardText {
            text: initial,
            revision: 0,
        });
        let state = self
            .state_ref
            .unwrap_or_else(|| scope.cage(CopyState::Idle));
        state.try_revise(|v| *v = CopyState::Idle)?;
        let pending = scope.cage(None::<(u64, u64)>);
        let delay = OwnerDelay::new(scope)?;
        let source = self.source.clone();
        let reset = delay.clone();
        let weak = scope.downgrade();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                let next = source.try_with(|value| canonical(value))??;
                let current = glory::core::untracked(|| accepted.try_get())?;
                if current.text != next {
                    let revision = current.revision.checked_add(1).ok_or(
                        glory::render::Error::InvalidProps("clipboard revision exhausted"),
                    )?;
                    accepted.try_revise(|v| {
                        *v = ClipboardText {
                            text: next,
                            revision,
                        }
                    })?;
                    pending.try_revise(|v| *v = None)?;
                    reset.cancel();
                    state.try_revise(|v| *v = CopyState::Idle)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error)
            }
        });
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let label = self.label.clone();
        let copied = std::mem::take(&mut self.copied_label);
        let failed = std::mem::take(&mut self.failed_label);
        let label = scope.bond(move || match state.get() {
            CopyState::Written => copied.clone(),
            CopyState::Denied | CopyState::Unavailable | CopyState::Failed => failed.clone(),
            _ => label.get(),
        });
        let metadata = scope.bond(move || {
            Some(Box::new(Semantics {
                clipboard: Some(accepted.get()),
                busy: Some(state.get() == CopyState::Pending),
                ..Default::default()
            }))
        });
        let base = self.common.theme.control(self.common.size);
        let mut inner = configure!(
            self.common,
            glory::ui::button(),
            scope,
            "sapid-copy-button",
            base
        )
        .label(label)
        .enabled(self.common.enabled.clone())
        .node_ref(reference)
        .semantics(metadata);
        let callback = Callback::new(self.result.take());
        let cleanup = callback.clone();
        scope.on_cleanup(move || cleanup.clear())?;
        let timer = self.delay.clone();
        let owner = scope.id();
        let weak = scope.downgrade();
        let completed = callback.clone();
        let feedback = delay.clone();
        let finish = Rc::new(move |outcome: ClipboardOutcome| -> Result {
            let Some(scope) = weak.upgrade() else {
                return Ok(());
            };
            pending.try_revise(|v| *v = None)?;
            state.try_revise(|v| *v = outcome.into())?;
            completed.emit(owner, outcome);
            if owner.is_alive()
                && pending.try_get()?.is_none()
                && state.try_get()? == CopyState::from(outcome)
                && timer.is_some()
            {
                feedback.run(&scope, timer.as_ref(), move || {
                    state.try_revise(|v| *v = CopyState::Idle)?;
                    Ok(())
                })?;
            }
            Ok(())
        });
        let mut activate = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let weak = scope.downgrade();
        let immediate = finish.clone();
        inner = inner.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                if pending.try_get()?.is_none() {
                    delay.cancel();
                    let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
                    if !scope.capabilities()?.clipboard_write {
                        immediate(ClipboardOutcome::Unavailable)?;
                    } else {
                        let request = scope.copy_text(node)?;
                        let revision = accepted.try_with(|v| v.revision)?;
                        pending.try_revise(|v| *v = Some((request, revision)))?;
                        state.try_revise(|v| *v = CopyState::Pending)?;
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
                return;
            }
            observe(&mut activate, owner, event);
        });
        let mut results = self
            .common
            .handlers
            .remove(&EventKind::Clipboard)
            .unwrap_or_default();
        let weak = scope.downgrade();
        inner = inner.on(EventKind::Clipboard, move |event| {
            let Some(scope) = weak.upgrade() else { return };
            let result = (|| -> Result {
                if let Event::Clipboard {
                    revision,
                    request,
                    outcome,
                } = event
                    && pending.try_get()? == Some((request, revision))
                {
                    finish(outcome)?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
                return;
            }
            observe(&mut results, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            inner = inner.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        inner.show_in(scope)
    }
}

impl Default for CopyButton {
    fn default() -> Self {
        copy_button()
    }
}
