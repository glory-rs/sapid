//! Optional file controls. References are host-owned identities, never ambient paths.
use crate::appearance::{Common, common_methods, configure, merge, observe};
use crate::composite::Callback;
use glory::render::{
    self, Change, Event, EventKind, FileConfig, FileInfo, FileOutcome, FilePreview, FileRead,
    FileRejection, FileToken, NodeId, Patch, Query, QueryToken, QueryValue, Semantics,
};
use glory::style::{
    BorderStyle, Display, Edges, FlexDirection, Length, Limit, ObjectFit, Overflow, Style,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileStatus {
    #[default]
    Idle,
    Choosing,
    Selected,
    Cancelled,
    Rejected(FileRejection),
}
#[derive(Clone)]
pub struct FileController {
    scope: glory::ui::WeakScope,
    node: Cage<Option<NodeId>>,
    files: Cage<Vec<FileInfo>>,
    status: Cage<FileStatus>,
    pending: Cage<Option<u64>>,
    output_files: Option<Cage<Vec<FileInfo>>>,
    output_status: Option<Cage<FileStatus>>,
}
impl FileController {
    pub fn files(&self) -> Vary<Vec<FileInfo>> {
        self.files.into()
    }
    pub fn status(&self) -> Vary<FileStatus> {
        self.status.into()
    }
    fn current(&self) -> Result<(Scope, NodeId)> {
        let scope = self.scope.upgrade().ok_or(glory::ui::Error::Gone)?;
        let node = self.node.try_get()?.ok_or(glory::ui::Error::Gone)?;
        Ok((scope, node))
    }
    fn set_status(&self, status: FileStatus) -> Result {
        self.status.try_revise(|v| *v = status)?;
        if let Some(output) = self.output_status {
            output.try_revise(|v| *v = status)?;
        }
        Ok(())
    }
    fn set_files(&self, files: Vec<FileInfo>) -> Result {
        let output_value = self.output_files.map(|_| files.clone());
        self.files.try_revise(|v| *v = files)?;
        if let Some(output) = self.output_files {
            output.try_revise(|v| *v = output_value.unwrap_or_default())?;
        }
        Ok(())
    }
    fn reset(&self) -> Result {
        self.pending.try_revise(|v| *v = None)?;
        self.set_files(Vec::new())?;
        self.set_status(FileStatus::Idle)
    }
    /// Programmatic clear is silent; it does not imitate a user selection callback.
    pub fn clear(&self) -> Result {
        let (scope, node) = self.current()?;
        scope.clear_files(node)?;
        self.reset()
    }
    pub fn remove(&self, file: FileToken) -> Result {
        self.ensure(file)?;
        let (scope, _) = self.current()?;
        scope.remove_file(file)?;
        let files = self.files.try_with(|files| {
            files
                .iter()
                .filter(|info| info.token != file)
                .cloned()
                .collect::<Vec<_>>()
        })?;
        let empty = files.is_empty();
        self.set_files(files)?;
        self.set_status(if empty {
            FileStatus::Idle
        } else {
            FileStatus::Selected
        })
    }
    fn ensure(&self, file: FileToken) -> Result {
        let (_, node) = self.current()?;
        if file.node != node
            || !self
                .files
                .try_with(|files| files.iter().any(|info| info.token == file))?
        {
            return Err(render::Error::Unsupported(
                "file reference is not selected by this controller",
            )
            .into());
        }
        Ok(())
    }
    pub fn read(
        &self,
        request: FileRead,
        reply: impl FnOnce(std::result::Result<Vec<u8>, render::Error>) + 'static,
    ) -> Result<QueryToken> {
        let (scope, _) = self.current()?;
        self.read_in(&scope, request, reply)
    }
    pub fn read_in(
        &self,
        scope: &Scope,
        request: FileRead,
        reply: impl FnOnce(std::result::Result<Vec<u8>, render::Error>) + 'static,
    ) -> Result<QueryToken> {
        self.ensure(request.file)?;
        request.validate()?;
        scope.request(Query::FileRead(request), move |result| {
            reply(result.and_then(|value| match value {
                QueryValue::FileBytes(chunk) => Ok(chunk.bytes),
                _ => Err(render::Error::InvalidResponse("file read result kind")),
            }))
        })
    }
    pub fn preview(
        &self,
        file: FileToken,
        reply: impl FnOnce(std::result::Result<Option<FilePreview>, render::Error>) + 'static,
    ) -> Result<QueryToken> {
        let (scope, _) = self.current()?;
        self.preview_in(&scope, file, reply)
    }
    pub fn preview_in(
        &self,
        scope: &Scope,
        file: FileToken,
        reply: impl FnOnce(std::result::Result<Option<FilePreview>, render::Error>) + 'static,
    ) -> Result<QueryToken> {
        self.ensure(file)?;
        scope.request(Query::FilePreview(file), move |result| {
            reply(result.and_then(|value| match value {
                QueryValue::FilePreview(preview) => Ok(preview.map(|value| *value)),
                _ => Err(render::Error::InvalidResponse("file preview result kind")),
            }))
        })
    }
}
pub struct FileButton {
    common: Common,
    label: Vary<String>,
    configuration: Vary<FileConfig>,
    files_ref: Option<Cage<Vec<FileInfo>>>,
    status_ref: Option<Cage<FileStatus>>,
    controller_ref: Option<Cage<Option<FileController>>>,
    changed: Option<Box<dyn FnMut(Vec<FileInfo>)>>,
    result: Option<Box<dyn FnMut(FileOutcome)>>,
    force_drop: bool,
}
pub fn file_button() -> FileButton {
    FileButton {
        common: Common::default(),
        label: "Choose file".into(),
        configuration: FileConfig::default().into(),
        files_ref: None,
        status_ref: None,
        controller_ref: None,
        changed: None,
        result: None,
        force_drop: false,
    }
}
impl FileButton {
    common_methods!();
    pub fn label(mut self, label: impl Into<Vary<String>>) -> Self {
        self.label = label.into();
        self
    }
    pub fn configuration(mut self, configuration: impl Into<Vary<FileConfig>>) -> Self {
        self.configuration = configuration.into();
        self
    }
    /// Output only; external writes never prepopulate host file references.
    pub fn files_ref(mut self, value: Cage<Vec<FileInfo>>) -> Self {
        self.files_ref = Some(value);
        self
    }
    pub fn status_ref(mut self, value: Cage<FileStatus>) -> Self {
        self.status_ref = Some(value);
        self
    }
    pub fn controller_ref(mut self, value: Cage<Option<FileController>>) -> Self {
        self.controller_ref = Some(value);
        self
    }
    pub fn on_files(mut self, callback: impl FnMut(Vec<FileInfo>) + 'static) -> Self {
        self.changed = Some(Box::new(callback));
        self
    }
    pub fn on_result(mut self, callback: impl FnMut(FileOutcome) + 'static) -> Self {
        self.result = Some(Box::new(callback));
        self
    }
}
fn label(value: &str) -> Result<String> {
    if value.len() > 4096 {
        return Err(render::Error::InvalidProps("file label exceeds 4096 UTF-8 bytes").into());
    }
    let value = render::normalize_text(value, true);
    if value.is_empty() || value.len() > 4096 {
        return Err(render::Error::InvalidProps(
            "file label is empty or exceeds canonical byte budget",
        )
        .into());
    }
    Ok(value.into_owned())
}
fn configuration(value: &FileConfig, force_drop: bool) -> Result<FileConfig> {
    value.validate()?;
    let mut value = value.clone();
    value.drop |= force_drop;
    Ok(value)
}
impl FileButton {
    fn build_control(&mut self, scope: &Scope) -> Result<FileController> {
        let initial_config = self
            .configuration
            .try_with(|value| configuration(value, self.force_drop))??;
        let initial_label = self.label.try_with(|value| label(value))??;
        let reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let files = scope.cage(Vec::new());
        let status = scope.cage(FileStatus::Idle);
        let pending = scope.cage(None);
        let control = FileController {
            scope: scope.downgrade(),
            node: reference,
            files,
            status,
            pending,
            output_files: self.files_ref,
            output_status: self.status_ref,
        };
        control.reset()?;
        let changed = Callback::new(self.changed.take());
        let completed = Callback::new(self.result.take());
        let changed_cleanup = changed.clone();
        let completed_cleanup = completed.clone();
        scope.on_cleanup(move || {
            changed_cleanup.clear();
            completed_cleanup.clear();
        })?;
        if let Some(output) = self.controller_ref {
            output.try_revise(|value| *value = Some(control.clone()))?;
            scope.on_cleanup(move || {
                let _ = output.try_revise(|value| *value = None);
            })?;
        }
        let output_files = self.files_ref;
        let output_status = self.status_ref;
        scope.on_cleanup(move || {
            if let Some(output) = output_files {
                let _ = output.try_revise(Vec::clear);
            }
            if let Some(output) = output_status {
                let _ = output.try_revise(|v| *v = FileStatus::Idle);
            }
        })?;
        let config_source = self.configuration.clone();
        let label_source = self.label.clone();
        let mut accepted_config = initial_config.clone();
        let mut accepted_label = initial_label.clone();
        let force_drop = self.force_drop;
        let weak = scope.downgrade();
        let driver = control.clone();
        scope.structural(move || {
            let Some(scope) = weak.upgrade() else {
                return;
            };
            let result = (|| -> Result {
                let next = config_source.try_with(|value| configuration(value, force_drop))??;
                let text = label_source.try_with(|value| label(value))??;
                let Some(node) = reference.try_get()? else {
                    return Ok(());
                };
                if next != accepted_config {
                    scope.set_file_configuration(node, next.clone())?;
                    accepted_config = next;
                    driver.reset()?;
                }
                if text != accepted_label {
                    scope.patch(node, Patch::Label(Change::Set(text.clone())))?;
                    accepted_label = text;
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
            }
        });
        let busy = scope.bond(move || {
            Some(Box::new(Semantics {
                busy: Some(status.get() == FileStatus::Choosing),
                ..Default::default()
            }))
        });
        let overrides = self.common.style.clone();
        let enabled = self.common.enabled.clone();
        let theme = self.common.theme.clone();
        let plain = self.common.unstyled;
        self.common.style = scope
            .bond(move || {
                let mut state = Style::new();
                if !plain && !enabled.get() {
                    state.color = Some(theme.disabled_text);
                    state.background = Some(theme.disabled);
                }
                merge(state, overrides.get())
            })
            .into();
        let base = self.common.theme.control(self.common.size);
        let base = if self.force_drop {
            base.min_height(Length::Px(96.))
                .padding(Edges::all(20.))
                .border_style(BorderStyle::Solid)
        } else {
            base
        };
        let mut inner = configure!(
            self.common,
            glory::ui::file()
                .configuration(initial_config)
                .label(initial_label),
            scope,
            "sapid-file-button",
            base
        )
        .enabled(self.common.enabled.clone())
        .node_ref(reference)
        .semantics(busy);
        let owner = scope.id();
        let mut activate = self
            .common
            .handlers
            .remove(&EventKind::Activate)
            .unwrap_or_default();
        let driver = control.clone();
        let unavailable = completed.clone();
        let weak = scope.downgrade();
        inner = inner.on(EventKind::Activate, move |event| {
            let Some(scope) = weak.upgrade() else {
                return;
            };
            let result = (|| -> Result {
                if pending.try_get()?.is_none() {
                    let node = reference.try_get()?.ok_or(glory::ui::Error::Gone)?;
                    match scope.pick_files(node) {
                        Ok(request) => {
                            pending.try_revise(|v| *v = Some(request))?;
                            driver.set_status(FileStatus::Choosing)?;
                        }
                        Err(glory::ui::Error::Render(render::Error::Unsupported(_))) => {
                            driver.set_status(FileStatus::Rejected(FileRejection::Unavailable))?;
                            unavailable
                                .emit(owner, FileOutcome::Rejected(FileRejection::Unavailable));
                        }
                        Err(error) => return Err(error),
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
        let mut observed = self
            .common
            .handlers
            .remove(&EventKind::Files)
            .unwrap_or_default();
        let driver = control.clone();
        let weak = scope.downgrade();
        inner = inner.on(EventKind::Files, move |event| {
            let Some(scope) = weak.upgrade() else {
                return;
            };
            let result = (|| -> Result {
                if let Event::Files(result) = &event {
                    if result.request.is_some()
                        || matches!(result.outcome, FileOutcome::Selected(_))
                    {
                        pending.try_revise(|value| *value = None)?;
                    }
                    match &result.outcome {
                        FileOutcome::Selected(selected) => {
                            driver.set_files(selected.clone())?;
                            driver.set_status(FileStatus::Selected)?;
                            changed.emit(owner, selected.clone());
                        }
                        FileOutcome::Cancelled => driver.set_status(FileStatus::Cancelled)?,
                        FileOutcome::Rejected(reason) => {
                            if pending.try_get()?.is_none() {
                                driver.set_status(FileStatus::Rejected(*reason))?;
                            }
                        }
                    }
                    completed.emit(owner, result.outcome.clone());
                }
                Ok(())
            })();
            if let Err(error) = result {
                scope.report_error(error);
                return;
            }
            observe(&mut observed, owner, event);
        });
        for (kind, mut handlers) in std::mem::take(&mut self.common.handlers) {
            inner = inner.on(kind, move |event| observe(&mut handlers, owner, event));
        }
        inner.show_in(scope)?;
        Ok(control)
    }
}
impl Widget for FileButton {
    fn build(&mut self, scope: &Scope) -> Result {
        self.build_control(scope).map(|_| ())
    }
}

/// One selected host reference with metadata, independently named removal, and
/// an optional on-demand bounded image preview. No path is opened implicitly.
pub struct FileAttachment {
    common: Common,
    info: FileInfo,
    controller: FileController,
    preview: bool,
}
pub fn file_attachment(info: FileInfo, controller: FileController) -> FileAttachment {
    FileAttachment {
        common: Common::default(),
        info,
        controller,
        preview: false,
    }
}
impl FileAttachment {
    common_methods!();
    pub fn preview(mut self, value: bool) -> Self {
        self.preview = value;
        self
    }
}
impl Widget for FileAttachment {
    fn build(&mut self, scope: &Scope) -> Result {
        self.info.validate()?;
        self.controller.ensure(self.info.token)?;
        let info = self.info.clone();
        let controller = self.controller.clone();
        let enabled = self.common.enabled.clone();
        let theme = self.common.theme.clone();
        let plain = self.common.unstyled;
        let base = Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .min_width(Length::Px(0.))
            .gap(6.)
            .overflow_x(Overflow::Auto);
        let base = if plain {
            base
        } else {
            let mut value = base.padding(Edges::all(8.)).radius(theme.radius);
            value.background = Some(theme.surface);
            value.color = Some(theme.foreground);
            value
        };
        let mut row = configure!(
            self.common,
            glory::ui::view(),
            scope,
            "sapid-file-attachment",
            base
        )
        .fill(
            glory::ui::text()
                .class("sapid-file-name")
                .content(info.name.clone()),
        )
        .fill(
            glory::ui::text()
                .class("sapid-file-metadata")
                .content(format!(
                    "{} · {} bytes",
                    if info.media_type.is_empty() {
                        "Unknown type"
                    } else {
                        &info.media_type
                    },
                    info.size
                )),
        );
        let remove = controller.clone();
        let weak = scope.downgrade();
        let token = info.token;
        let mut button = crate::button()
            .theme(theme.clone())
            .enabled(enabled.clone())
            .label("Remove")
            .name(format!("Remove {}", info.name))
            .class("sapid-file-remove")
            .on_press(move || {
                let Some(scope) = weak.upgrade() else {
                    return;
                };
                if let Err(error) = remove.remove(token) {
                    scope.report_error(error);
                }
            });
        if plain {
            button = button.unstyled();
        }
        row = row.fill(button);
        if self.preview {
            let state = scope.cage(None::<FilePreview>);
            let feedback = scope.cage(String::new());
            let pending = scope.cage(false);
            let weak = scope.downgrade();
            let request = controller.clone();
            let usable = enabled.clone();
            let mut button = crate::button()
                .theme(theme.clone())
                .enabled(scope.bond(move || usable.get() && !pending.get()))
                .label("Preview")
                .name(format!("Preview {}", info.name))
                .class("sapid-file-preview-request")
                .on_press(move || {
                    let Some(scope) = weak.upgrade() else {
                        return;
                    };
                    if pending.get() {
                        return;
                    }
                    pending.revise(|value| *value = true);
                    feedback.revise(String::clear);
                    let reply = request.preview_in(&scope, token, move |result| {
                        let _ = pending.try_revise(|value| *value = false);
                        match result {
                            Ok(Some(value)) => {
                                let _ = state.try_revise(|current| *current = Some(value));
                            }
                            Ok(None) => {
                                let _ = feedback.try_revise(|value| {
                                    *value = "Preview unavailable for this format".into()
                                });
                            }
                            Err(_) => {
                                let _ = feedback
                                    .try_revise(|value| *value = "Preview unavailable".into());
                            }
                        }
                    });
                    if let Err(error) = reply {
                        pending.revise(|value| *value = false);
                        feedback.revise(|value| *value = format!("Preview unavailable: {error}"));
                    }
                });
            if plain {
                button = button.unstyled();
            }
            row = row.fill(button).fill(
                glory::ui::text()
                    .class("sapid-file-preview-feedback")
                    .content(feedback),
            );
            let alternative = format!("Preview of {}", info.name);
            row = row.fill(glory::ui::when(
                scope.bond(move || state.get().is_some()),
                move || {
                    let state = state.get().expect("preview branch");
                    crate::image(state.source)
                        .alt(alternative.clone())
                        .theme(theme.clone())
                        .class("sapid-file-preview")
                        .width(Length::Px(120.))
                        .height(Length::Px(90.))
                        .fit(ObjectFit::Contain)
                },
            ));
        }
        row.show_in(scope)
    }
}

/// FileInput and Dropzone share the same real control and reference controller.
/// Attachments are siblings of the button, never interactive button children.
type AttachmentFactory = Box<dyn FnMut(FileInfo, FileController) -> Box<dyn Widget>>;
pub struct FileInput {
    button: FileButton,
    drop: bool,
    preview: bool,
    description: Option<crate::composite::Factory>,
    attachment: Option<AttachmentFactory>,
}
pub type Dropzone = FileInput;
pub fn file_input() -> FileInput {
    FileInput {
        button: file_button(),
        drop: false,
        preview: false,
        description: None,
        attachment: None,
    }
}
pub fn dropzone() -> Dropzone {
    let mut value = file_input();
    value.drop = true;
    value.button.force_drop = true;
    value.button.label = "Drop files or choose files".into();
    value
}
macro_rules! file_composite_methods {
    () => {
        pub fn theme(mut self, value: crate::Theme) -> Self {
            self.button = self.button.theme(value);
            self
        }
        pub fn size(mut self, value: crate::Size) -> Self {
            self.button = self.button.size(value);
            self
        }
        pub fn unstyled(mut self) -> Self {
            self.button = self.button.unstyled();
            self
        }
        pub fn class(mut self, value: impl Into<Vary<String>>) -> Self {
            self.button = self.button.class(value);
            self
        }
        pub fn style(mut self, value: impl Into<Vary<Style>>) -> Self {
            self.button = self.button.style(value);
            self
        }
        pub fn name(mut self, value: impl Into<Vary<String>>) -> Self {
            self.button = self.button.name(value);
            self
        }
        pub fn enabled(mut self, value: impl Into<Vary<bool>>) -> Self {
            self.button = self.button.enabled(value);
            self
        }
        pub fn focus(mut self, value: glory::render::FocusMode) -> Self {
            self.button = self.button.focus(value);
            self
        }
        pub fn node_ref(mut self, value: Cage<Option<NodeId>>) -> Self {
            self.button = self.button.node_ref(value);
            self
        }
        pub fn label(mut self, value: impl Into<Vary<String>>) -> Self {
            self.button = self.button.label(value);
            self
        }
        pub fn configuration(mut self, value: impl Into<Vary<FileConfig>>) -> Self {
            self.button = self.button.configuration(value);
            self
        }
        pub fn files_ref(mut self, value: Cage<Vec<FileInfo>>) -> Self {
            self.button = self.button.files_ref(value);
            self
        }
        pub fn status_ref(mut self, value: Cage<FileStatus>) -> Self {
            self.button = self.button.status_ref(value);
            self
        }
        pub fn controller_ref(mut self, value: Cage<Option<FileController>>) -> Self {
            self.button = self.button.controller_ref(value);
            self
        }
        pub fn on_files(mut self, value: impl FnMut(Vec<FileInfo>) + 'static) -> Self {
            self.button = self.button.on_files(value);
            self
        }
        pub fn on_result(mut self, value: impl FnMut(FileOutcome) + 'static) -> Self {
            self.button = self.button.on_result(value);
            self
        }
        pub fn on(mut self, kind: EventKind, value: impl FnMut(Event) + 'static) -> Self {
            self.button = self.button.on(kind, value);
            self
        }
    };
}
impl FileInput {
    file_composite_methods!();
    /// Enable explicit Preview buttons. The host validates image headers before publishing a preview resource.
    pub fn previews(mut self, value: bool) -> Self {
        self.preview = value;
        self
    }
    pub fn description<W: Widget>(mut self, mut factory: impl FnMut() -> W + 'static) -> Self {
        self.description = Some(Box::new(move || Box::new(factory())));
        self
    }
    /// Called once per newly accepted token, outside graph borrows. Retained tokens
    /// preserve the attachment Owner; removal/clear retires it and its queries.
    pub fn attachment<W: Widget>(
        mut self,
        mut factory: impl FnMut(FileInfo, FileController) -> W + 'static,
    ) -> Self {
        self.attachment = Some(Box::new(move |info, controller| {
            Box::new(factory(info, controller))
        }));
        self
    }
}
impl Widget for FileInput {
    fn build(&mut self, scope: &Scope) -> Result {
        // Validate both sources before creating the wrapper or invoking slots.
        self.button
            .configuration
            .try_with(|value| configuration(value, self.button.force_drop))??;
        self.button.label.try_with(|value| label(value))??;
        let reference = scope.cage(None);
        glory::ui::view()
            .node_ref(reference)
            .class(if self.drop {
                "sapid-dropzone"
            } else {
                "sapid-file-input"
            })
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .min_width(Length::Px(0.))
                    .max_width(Limit::Percent(100.))
                    .gap(8.),
            )
            .show_in(scope)?;
        let root = reference.get().ok_or(glory::ui::Error::Gone)?;
        let child = scope.child_in(root)?;
        if let Some(factory) = self.description.as_mut() {
            factory().show_in(&child)?;
        }
        let theme = self.button.common.theme.clone();
        let plain = self.button.common.unstyled;
        let enabled = self.button.common.enabled.clone();
        let controller = self.button.build_control(&child)?;
        let current = controller.files();
        let status = controller.status();
        glory::ui::text()
            .class("sapid-file-status")
            .content(child.bond(move || match status.get() {
                FileStatus::Idle => "No files selected".into(),
                FileStatus::Choosing => "Choosing files".into(),
                FileStatus::Selected => "Files selected".into(),
                FileStatus::Cancelled => "Selection cancelled".into(),
                FileStatus::Rejected(reason) => format!("Files rejected: {reason:?}"),
            }))
            .show_in(&child)?;
        let preview = self.preview;
        let mut factory = self.attachment.take();
        glory::ui::each(current)
            .key(|info| info.token)
            .children(move |entry| {
                let info = entry.value;
                let controller = controller.clone();
                let info = info.get();
                if let Some(factory) = factory.as_mut() {
                    factory(info, controller)
                } else {
                    let mut attachment = file_attachment(info, controller)
                        .theme(theme.clone())
                        .enabled(enabled.clone())
                        .preview(preview);
                    if plain {
                        attachment = attachment.unstyled();
                    }
                    Box::new(attachment) as Box<dyn Widget>
                }
            })
            .show_in(&child)?;
        scope.retain(child)?;
        Ok(())
    }
}

impl Default for FileButton {
    fn default() -> Self {
        file_button()
    }
}

impl Default for FileInput {
    fn default() -> Self {
        file_input()
    }
}
