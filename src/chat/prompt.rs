use crate::appearance::Common;
use crate::composite::{self, Callback};
use crate::state::{Dirty, Value};
use crate::{FileController, FileStatus, Theme};
use glory::render::{Event, EventKind, FileConfig, FileInfo, KeyboardCommand, NodeId};
use glory::style::{Display, FlexWrap, Style};
use glory::ui::{Result, Widget};
use glory::{Cage, Lotus, Scope};

pub const MAX_PROMPT_BYTES: usize = 65536;
pub const MAX_PROMPT_SUGGESTIONS: usize = 16;
pub const MAX_PROMPT_SUGGESTION_BYTES: usize = 8192;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PromptSubmitMode {
    #[default]
    ButtonOnly,
    EnterToSend,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptRejection {
    TooLong,
    Disabled,
    Readonly,
    Busy,
    Composing,
    ChoosingFiles,
    Empty,
}
#[derive(Clone)]
pub struct PromptSubmission {
    pub text: String,
    pub files: Vec<FileInfo>,
    /// Current document-owned read access. Tokens expire with removal or the
    /// picker Owner. Applications choose their own read/transport policy.
    pub file_controller: Option<FileController>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptSuggestion {
    pub label: String,
    pub value: String,
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn validate_text(value: &str) -> Result {
    if value.len() > MAX_PROMPT_BYTES
        || glory::render::normalize_text(value, true).len() > MAX_PROMPT_BYTES
    {
        return Err(invalid("prompt exceeds 65536 UTF8 bytes"));
    }
    Ok(())
}
fn label(value: &str) -> Result {
    if value.len() > 128 || value.trim().is_empty() || value.contains(['\0', '\r', '\n']) {
        return Err(invalid("invalid bounded prompt label"));
    }
    Ok(())
}
pub struct PromptInput {
    key: String,
    common: Common,
    value: Value<String>,
    readonly: Lotus<bool>,
    busy: Lotus<bool>,
    mode: PromptSubmitMode,
    rows: u16,
    label: String,
    form_name: String,
    send_label: String,
    cancel_label: String,
    attachment_label: String,
    attachments: Option<FileConfig>,
    suggestions: Vec<PromptSuggestion>,
    overflow: bool,
    input_ref: Option<Cage<Option<NodeId>>>,
    change: Option<Box<dyn FnMut(String)>>,
    send: Option<Box<dyn FnMut(PromptSubmission)>>,
    cancel: Option<Box<dyn FnMut()>>,
    reject: Option<Box<dyn FnMut(PromptRejection)>>,
}
pub fn prompt_input(key: impl Into<String>) -> PromptInput {
    PromptInput {
        key: key.into(),
        common: Common::default(),
        value: Value::default(),
        readonly: false.into(),
        busy: false.into(),
        mode: PromptSubmitMode::ButtonOnly,
        rows: 3,
        label: "Message".into(),
        form_name: "prompt".into(),
        send_label: "Send message".into(),
        cancel_label: "Cancel request".into(),
        attachment_label: "Attach files".into(),
        attachments: None,
        suggestions: Vec::new(),
        overflow: false,
        input_ref: None,
        change: None,
        send: None,
        cancel: None,
        reject: None,
    }
}
impl PromptInput {
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
    pub fn enabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.common.enabled = value.into();
        self
    }
    pub fn readonly(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.readonly = value.into();
        self
    }
    pub fn busy(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.busy = value.into();
        self
    }
    pub fn value(mut self, value: impl Into<Lotus<String>>) -> Self {
        self.value.source = Some(value.into());
        self
    }
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.value.default = value.into();
        self
    }
    pub fn rows(mut self, value: u16) -> Self {
        self.rows = value;
        self
    }
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }
    pub fn form_name(mut self, value: impl Into<String>) -> Self {
        self.form_name = value.into();
        self
    }
    pub fn send_label(mut self, value: impl Into<String>) -> Self {
        self.send_label = value.into();
        self
    }
    pub fn cancel_label(mut self, value: impl Into<String>) -> Self {
        self.cancel_label = value.into();
        self
    }
    pub fn attachment_label(mut self, value: impl Into<String>) -> Self {
        self.attachment_label = value.into();
        self
    }
    pub fn submit_mode(mut self, value: PromptSubmitMode) -> Self {
        self.mode = value;
        self
    }
    pub fn attachments(mut self, value: FileConfig) -> Self {
        self.attachments = Some(value);
        self
    }
    pub fn input_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.input_ref = Some(value);
        self
    }
    pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
        self.common.node_ref = Some(value);
        self
    }
    pub fn on_value_change(mut self, value: impl FnMut(String) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self
    }
    pub fn on_send(mut self, value: impl FnMut(PromptSubmission) + 'static) -> Self {
        self.send = Some(Box::new(value));
        self
    }
    pub fn on_cancel(mut self, value: impl FnMut() + 'static) -> Self {
        self.cancel = Some(Box::new(value));
        self
    }
    pub fn on_reject(mut self, value: impl FnMut(PromptRejection) + 'static) -> Self {
        self.reject = Some(Box::new(value));
        self
    }
    pub fn suggestion(mut self, value: PromptSuggestion) -> Self {
        if self.suggestions.len() == MAX_PROMPT_SUGGESTIONS {
            self.overflow = true;
        } else {
            self.suggestions.push(value);
        }
        self
    }
    pub fn suggestions(mut self, values: impl IntoIterator<Item = PromptSuggestion>) -> Self {
        for value in values
            .into_iter()
            .take(MAX_PROMPT_SUGGESTIONS - self.suggestions.len() + 1)
        {
            if self.suggestions.len() == MAX_PROMPT_SUGGESTIONS {
                self.overflow = true;
                break;
            }
            self.suggestions.push(value);
        }
        self
    }
}
impl Widget for PromptInput {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("prompt", &self.key, "root", "root")?;
        if self.overflow || self.rows == 0 || self.rows > 32 {
            return Err(invalid("invalid prompt suggestion or row budget"));
        }
        for value in [
            &self.label,
            &self.send_label,
            &self.cancel_label,
            &self.attachment_label,
        ] {
            label(value)?;
        }
        if self.form_name.is_empty()
            || self.form_name.len() > 256
            || self.form_name.contains(['\0', '\r', '\n'])
        {
            return Err(invalid("invalid prompt successful-control name"));
        }
        let mut bytes = 0usize;
        for suggestion in &self.suggestions {
            label(&suggestion.label)?;
            bytes += suggestion.value.len();
            if suggestion.value.len() > MAX_PROMPT_SUGGESTION_BYTES || bytes > 32768 {
                return Err(invalid("prompt suggestion text budget exceeded"));
            }
        }
        if let Some(configuration) = &self.attachments {
            configuration.validate()?;
        }
        let control = std::mem::take(&mut self.value).mount(scope);
        let accepted =
            composite::validated(scope, &control.source, |value| validate_text(value))?;
        let composing = scope.cage(false);
        let files = scope.cage(Vec::<FileInfo>::new());
        let picker_status = scope.cage(FileStatus::Idle);
        let file_controller = scope.cage(None::<FileController>);
        let enabled = self.common.enabled.clone();
        let readonly = self.readonly.clone();
        let busy = self.busy.clone();
        let editable = scope.bond({
            let enabled = enabled.clone();
            let readonly = readonly.clone();
            let busy = busy.clone();
            move || enabled.get() && !readonly.get() && !busy.get()
        });
        let can_send = scope.bond(move || {
            editable.get() && !composing.get() && picker_status.get() != FileStatus::Choosing
        });
        let change = Callback::new(self.change.take());
        let send = Callback::new(self.send.take());
        let cancel = Callback::new(
            self.cancel
                .take()
                .map(|mut callback| Box::new(move |()| callback()) as Box<dyn FnMut(())>),
        );
        let reject = Callback::new(self.reject.take());
        let owner = scope.id();
        let form_reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let actual_form = std::rc::Rc::new(std::cell::Cell::new(None));
        let task_form = actual_form.clone();
        let request = Dirty::new(scope);
        let pending = request.clone();
        let send_callback = send.clone();
        let rejection = reject.clone();
        let weak = scope.downgrade();
        let queued = std::rc::Rc::new(std::cell::Cell::new(false));
        scope.effect(move || {
            if !pending.take() || queued.replace(true) {
                return;
            }
            let Some(scope) = weak.upgrade() else { return };
            let task_weak = weak.clone();
            let task_form = task_form.clone();
            let task_queued = queued.clone();
            let enabled = enabled.clone();
            let readonly = readonly.clone();
            let busy = busy.clone();
            let rejection = rejection.clone();
            let callback = send_callback.clone();
            let result = scope.spawn(async move {
                task_queued.set(false);
                let Some(scope) = task_weak.upgrade() else {
                    return;
                };
                let Some(form) = task_form.get() else { return };
                if !scope.is_active(form).unwrap_or(false)
                    || !scope.is_enabled(form).unwrap_or(false)
                    || scope.is_inert(form).unwrap_or(true)
                {
                    return;
                }
                let reason = if !enabled.get() {
                    Some(PromptRejection::Disabled)
                } else if readonly.get() {
                    Some(PromptRejection::Readonly)
                } else if busy.get() {
                    Some(PromptRejection::Busy)
                } else if composing.get() {
                    Some(PromptRejection::Composing)
                } else if picker_status.get() == FileStatus::Choosing {
                    Some(PromptRejection::ChoosingFiles)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    rejection.emit(owner, reason);
                    return;
                }
                let result = (|| -> Result<PromptSubmission> {
                    let text = accepted.try_with(|value| {
                        glory::render::normalize_text(value, true).into_owned()
                    })?;
                    let files = files.try_with(|files| -> Result<Vec<FileInfo>> {
                        if files.len() > glory::render::MAX_FILE_COUNT {
                            return Err(invalid("prompt file count exceeded"));
                        }
                        for info in files {
                            info.validate()?;
                        }
                        Ok(files.clone())
                    })??;
                    Ok(PromptSubmission {
                        text,
                        files,
                        file_controller: file_controller.try_get()?,
                    })
                })();
                match result {
                    Ok(value) if value.text.trim().is_empty() && value.files.is_empty() => {
                        rejection.emit(owner, PromptRejection::Empty)
                    }
                    Ok(value) => callback.emit(owner, value),
                    Err(error) => scope.report_error(error),
                }
            });
            if let Err(error) = result {
                queued.set(false);
                scope.report_error(error);
            }
        });
        let input_ref = self.input_ref.unwrap_or_else(|| scope.cage(None));
        let input_change = change.clone();
        let input_control = control.clone();
        let rejection = reject.clone();
        let update_scope = scope.downgrade();
        let update = move |value: String| {
            if !editable.get() {
                return;
            }
            if validate_text(&value).is_err() {
                rejection.emit(owner, PromptRejection::TooLong);
                return;
            }
            if let Err(error) = input_control.accept(&value) {
                if let Some(scope) = update_scope.upgrade() {
                    scope.report_error(error);
                }
                return;
            }
            input_change.emit(owner, value);
        };
        let update: Callback<String> = Callback::new(Some(Box::new(update)));
        let input_theme = self.common.theme.clone();
        let rows = self.rows;
        let mode = self.mode;
        let input_readonly = self.readonly.clone();
        let request_key = request.clone();
        let editor_update = update.clone();
        let editor = crate::field(self.key.clone())
            .theme(self.common.theme.clone())
            .label(self.label.clone())
            .form_name(self.form_name.clone())
            .control(move |field| {
                let changed = editor_update.clone();
                let committed = editor_update.clone();
                let mut editor = crate::textarea()
                    .theme(input_theme.clone())
                    .field(field)
                    .rows(rows)
                    .value(accepted)
                    .enabled(editable)
                    .readonly(input_readonly.clone())
                    .node_ref(input_ref)
                    .on_value_change(move |value| changed.emit(owner, value))
                    .on_commit(move |value| committed.emit(owner, value))
                    .on(EventKind::Composition, move |event| {
                        if let Event::Composition { active } = event
                            && composing.get_untracked() != active
                        {
                            composing.revise(|value| *value = active);
                        }
                    });
                if mode == PromptSubmitMode::EnterToSend {
                    let request = request_key.clone();
                    editor = editor.on(EventKind::Keyboard, move |event| {
                        if matches!(
                            event,
                            Event::Keyboard {
                                command: KeyboardCommand::Submit,
                                ..
                            }
                        ) {
                            let _ = request.touch();
                        }
                    });
                }
                editor
            });
        let mut actions = glory::ui::view()
            .class("sapid-prompt-actions")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_wrap(FlexWrap::Wrap)
                    .gap(8.),
            )
            .fill(
                crate::submit_button()
                    .theme(self.common.theme.clone())
                    .label(self.send_label.clone())
                    .enabled(can_send),
            );
        let cancel_callback = cancel.clone();
        let cancel_busy = self.busy.clone();
        let cancel_enabled = self.common.enabled.clone();
        actions = actions.fill(
            crate::button()
                .theme(self.common.theme.clone())
                .label(self.cancel_label.clone())
                .enabled(scope.bond({
                    let busy = cancel_busy.clone();
                    let enabled = cancel_enabled.clone();
                    move || enabled.get() && busy.get()
                }))
                .on_press(move || {
                    if cancel_busy.get() && cancel_enabled.get() {
                        cancel_callback.emit(owner, ());
                    }
                }),
        );
        if let Some(configuration) = self.attachments.take() {
            actions = actions.fill(
                crate::file_input()
                    .theme(self.common.theme.clone())
                    .label(self.attachment_label.clone())
                    .configuration(configuration)
                    .enabled(editable)
                    .files_ref(files)
                    .status_ref(picker_status)
                    .controller_ref(file_controller),
            );
        }
        let mut suggestions = glory::ui::view().class("sapid-prompt-suggestions").style(
            Style::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(8.),
        );
        for suggestion in self.suggestions.drain(..) {
            let proposal = suggestion.value;
            let update = update.clone();
            suggestions = suggestions.fill(
                crate::button()
                    .theme(self.common.theme.clone())
                    .label(suggestion.label)
                    .enabled(can_send)
                    .on_press(move || {
                        if can_send.get() {
                            update.emit(owner, proposal.clone());
                        }
                    }),
            );
        }
        let source_class = self.common.class.clone();
        let class = scope.bond(move || format!("sapid-prompt-input {}", source_class.get()));
        let mut form = crate::form(self.key.clone())
            .theme(self.common.theme.clone())
            .class(class)
            .style(self.common.style.clone())
            .enabled(self.common.enabled.clone())
            .fill((editor, actions, suggestions))
            .on_submit(move |_| {
                let _ = request.touch();
            });
        form = form.node_ref(form_reference);
        form.show_in(scope)?;
        actual_form.set(form_reference.try_get()?);
        scope.on_cleanup(move || {
            update.clear();
            change.clear();
            send.clear();
            cancel.clear();
            reject.clear();
        })?;
        Ok(())
    }
}
