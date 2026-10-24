//! Owner-scoped form bookkeeping. Values remain in the application's Cages.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};

use glory::core::{AccessError, TaskToken};
use glory::{Bond, Cage, Lotus, Scope};

pub type ValidationErrors = Vec<String>;
pub type ValidationFuture = Pin<Box<dyn Future<Output = ValidationErrors>>>;
type Validator<T> = Rc<dyn Fn(&T) -> ValidationErrors>;
type AsyncValidator<T> = Rc<dyn Fn(&T) -> ValidationFuture>;
type Encoder<T> = Rc<dyn Fn(&T) -> ControlValue>;
type SubmitCallback = Box<dyn FnOnce(SubmitToken)>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FormFieldState {
    pub touched: bool,
    /// Explicit user changes since reset. External Cage writes remain silent.
    pub dirty: bool,
    pub validating: bool,
    pub errors: ValidationErrors,
    /// Validator output exceeded the configured storage budget; submission fails.
    pub validation_overflow: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SubmitStatus {
    #[default]
    Idle,
    Validating,
    Submitting,
    Succeeded,
    Invalid,
    Failed(String),
    Cancelled,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FormStatus {
    pub fields: usize,
    pub touched: usize,
    pub dirty: usize,
    pub validating: usize,
    pub invalid: usize,
    pub submit: SubmitStatus,
}
#[derive(Clone, Copy, Debug)]
pub struct FormLimits {
    pub fields: usize,
    pub errors_per_field: usize,
    pub error_bytes: usize,
}
impl Default for FormLimits {
    fn default() -> Self {
        Self {
            fields: 1024,
            errors_per_field: 16,
            error_bytes: 4096,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormError {
    Retired,
    DuplicateKey,
    InvalidKey,
    Capacity,
    Exhausted,
    Access(AccessError),
    UnsupportedFiles,
    Batch(String),
    Superseded,
}
impl From<AccessError> for FormError {
    fn from(value: AccessError) -> Self {
        Self::Access(value)
    }
}
impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FormError {}

/// Explicit encoding of one successful control; no HTTP, filesystem or serde.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlValue {
    Missing,
    Text(String),
    Checked {
        checked: bool,
        value: String,
    },
    Values(Vec<String>),
    /// File handles require a future host-owned protocol; never infer paths.
    UnsupportedFile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormEntry {
    pub name: String,
    pub value: String,
}
#[derive(Clone, Copy, Debug)]
pub struct SubmissionLimits {
    pub entries: usize,
    pub bytes: usize,
}
impl Default for SubmissionLimits {
    fn default() -> Self {
        Self {
            entries: 1024,
            bytes: 1024 * 1024,
        }
    }
}

pub struct FieldOptions<T: 'static> {
    validator: Option<Validator<T>>,
    async_validator: Option<AsyncValidator<T>>,
    reset: Option<Rc<dyn Fn()>>,
    disabled: Lotus<bool>,
    name: Option<String>,
    encoder: Option<Encoder<T>>,
}
impl<T> Default for FieldOptions<T> {
    fn default() -> Self {
        Self {
            validator: None,
            async_validator: None,
            reset: None,
            disabled: false.into(),
            name: None,
            encoder: None,
        }
    }
}
impl<T> FieldOptions<T> {
    /// Add a composite's validation gate without discarding application disabling.
    #[cfg(feature = "chat")]
    pub(crate) fn gate_disabled(mut self, scope: &Scope, gate: Lotus<bool>) -> Self {
        let disabled = self.disabled;
        self.disabled = scope.bond(move || disabled.get() || gate.get()).into();
        self
    }
    pub fn validate(mut self, callback: impl Fn(&T) -> ValidationErrors + 'static) -> Self {
        self.validator = Some(Rc::new(callback));
        self
    }
    /// The factory borrows one field. Clone only data actually needed by its future.
    pub fn validate_async(
        mut self,
        callback: impl Fn(&T) -> ValidationFuture + 'static,
    ) -> Self {
        self.async_validator = Some(Rc::new(callback));
        self
    }
    /// Explicit reset policy; omitting it resets bookkeeping but preserves the value.
    pub fn on_reset(mut self, callback: impl Fn() + 'static) -> Self {
        self.reset = Some(Rc::new(callback));
        self
    }
    pub fn disabled(mut self, value: impl Into<Lotus<bool>>) -> Self {
        self.disabled = value.into();
        self
    }
    pub fn serialize(
        mut self,
        name: impl Into<String>,
        callback: impl Fn(&T) -> ControlValue + 'static,
    ) -> Self {
        self.name = Some(name.into());
        self.encoder = Some(Rc::new(callback));
        self
    }
}
trait Registered {
    fn validate(self: Rc<Self>) -> Result<(), FormError>;
    fn reset(&self) -> Result<(), FormError>;
    fn cancel(&self);
    fn state(&self) -> FormFieldState;
    fn current(&self) -> bool;
    fn encode(&self) -> Result<Option<(String, ControlValue)>, FormError>;
    fn retire(&self);
}
struct Inner {
    scope: glory::ui::WeakScope,
    live: Cell<bool>,
    limits: FormLimits,
    fields: RefCell<BTreeMap<String, Rc<dyn Registered>>>,
    status: Cage<FormStatus>,
    read: Bond<FormStatus>,
    ticket: Cell<u64>,
    preparing: Cell<usize>,
    callback: RefCell<Option<SubmitCallback>>,
}
#[derive(Clone)]
pub struct FormState(Rc<Inner>);
/// Completion permission for one submit attempt. Late completions are ignored.
#[derive(Clone)]
pub struct SubmitToken {
    form: Weak<Inner>,
    ticket: u64,
}
impl SubmitToken {
    pub fn complete(&self, result: Result<(), String>) -> bool {
        let Some(form) = self.form.upgrade() else {
            return false;
        };
        if !form.is_live() || form.ticket.get() != self.ticket {
            return false;
        }
        if form.status.get_untracked().submit != SubmitStatus::Submitting {
            return false;
        }
        if !form.snapshot().iter().all(|field| field.current()) {
            form.cancel_submit();
            return false;
        }
        let status = match result {
            Ok(()) => SubmitStatus::Succeeded,
            Err(mut error) => {
                let mut end = error.len().min(form.limits.error_bytes);
                while !error.is_char_boundary(end) {
                    end -= 1;
                }
                error.truncate(end);
                SubmitStatus::Failed(error)
            }
        };
        form.set_submit(status);
        true
    }
    pub fn is_current(&self) -> bool {
        self.form.upgrade().is_some_and(|f| {
            f.is_live()
                && f.ticket.get() == self.ticket
                && f.status.get_untracked().submit == SubmitStatus::Submitting
                && f.snapshot().iter().all(|field| field.current())
        })
    }
}
impl Inner {
    fn is_live(&self) -> bool {
        self.live.get() && self.status.is_alive()
    }
    fn snapshot(&self) -> Vec<Rc<dyn Registered>> {
        self.fields.borrow().values().cloned().collect()
    }
    fn advance(&self) -> Result<u64, FormError> {
        let ticket = self
            .ticket
            .get()
            .checked_add(1)
            .ok_or(FormError::Exhausted)?;
        self.ticket.set(ticket);
        Ok(ticket)
    }
    fn set_submit(&self, submit: SubmitStatus) {
        if self.is_live() {
            let _ = self.status.try_revise(|s| s.submit = submit);
        }
    }
    fn publish(self: &Rc<Self>) {
        if !self.is_live() || self.preparing.get() != 0 {
            return;
        }
        let fields = self.snapshot();
        let mut status = FormStatus {
            fields: fields.len(),
            submit: self.status.get_untracked().submit,
            ..Default::default()
        };
        for field in &fields {
            let state = field.state();
            status.touched += usize::from(state.touched);
            status.dirty += usize::from(state.dirty);
            status.validating += usize::from(state.validating);
            status.invalid +=
                usize::from(!state.errors.is_empty() || state.validation_overflow);
        }
        let ready = self.preparing.get() == 0
            && status.submit == SubmitStatus::Validating
            && status.validating == 0;
        if ready {
            status.submit = if status.invalid == 0 && fields.iter().all(|f| f.current()) {
                SubmitStatus::Submitting
            } else {
                SubmitStatus::Invalid
            };
        }
        let run = ready && status.submit == SubmitStatus::Submitting;
        let _ = self.status.try_revise(|s| *s = status);
        if ready {
            let callback = self.callback.borrow_mut().take();
            if run
                && let Some(callback) = callback
                && let Some(scope) = self.scope.upgrade()
            {
                let token = SubmitToken {
                    form: Rc::downgrade(self),
                    ticket: self.ticket.get(),
                };
                scope
                    .owner()
                    .enter(|| glory::core::untracked(|| callback(token)));
            }
        }
    }
    fn cancel_submit(&self) {
        let old = self.callback.borrow_mut().take();
        if matches!(
            self.status.try_with_untracked(|s| s.submit.clone()),
            Ok(SubmitStatus::Validating | SubmitStatus::Submitting)
        ) {
            self.set_submit(SubmitStatus::Cancelled);
        }
        drop(old);
    }
}
impl FormState {
    pub fn new(scope: &Scope) -> Self {
        Self::with_limits(scope, FormLimits::default())
    }
    pub fn with_limits(scope: &Scope, limits: FormLimits) -> Self {
        let status = scope.cage(FormStatus::default());
        let read = scope.bond(move || status.get());
        let form = Rc::new(Inner {
            scope: scope.downgrade(),
            live: Cell::new(true),
            limits,
            fields: RefCell::new(BTreeMap::new()),
            status,
            read,
            ticket: Cell::new(0),
            preparing: Cell::new(0),
            callback: RefCell::new(None),
        });
        let weak = Rc::downgrade(&form);
        scope
            .on_cleanup(move || {
                if let Some(form) = weak.upgrade() {
                    form.live.set(false);
                    let callback = form.callback.borrow_mut().take();
                    let fields = std::mem::take(&mut *form.fields.borrow_mut());
                    for field in fields.values() {
                        field.retire();
                    }
                    drop((fields, callback));
                }
            })
            .expect("live form Owner");
        Self(form)
    }
    pub fn state(&self) -> Bond<FormStatus> {
        self.0.read
    }
    pub fn register<T: 'static>(
        &self,
        scope: &Scope,
        key: impl Into<String>,
        value: Cage<T>,
        options: FieldOptions<T>,
    ) -> Result<FormField<T>, FormError> {
        if !self.0.is_live() || !scope.is_alive() {
            return Err(FormError::Retired);
        }
        if value.runtime() != scope.id().runtime() || value.runtime() != self.0.status.runtime()
        {
            return Err(FormError::InvalidKey);
        }
        let key = key.into();
        if key.is_empty()
            || key.len() > 128
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
            || options
                .name
                .as_ref()
                .is_some_and(|n| n.len() > 256 || n.contains('\0'))
        {
            return Err(FormError::InvalidKey);
        }
        {
            let fields = self.0.fields.borrow();
            if fields.contains_key(&key) {
                return Err(FormError::DuplicateKey);
            }
            if fields.len() >= self.0.limits.fields {
                return Err(FormError::Capacity);
            }
        }
        value.try_with_untracked(|_| ())?;
        let state = scope.cage(FormFieldState::default());
        let read = scope.bond(move || state.get());
        let field = Rc::new(Field {
            form: Rc::downgrade(&self.0),
            scope: scope.downgrade(),
            value,
            state,
            read,
            live: Cell::new(true),
            options: RefCell::new(Some(options)),
            ticket: Cell::new(0),
            task: Cell::new(None),
            active: Cell::new(None),
            validated: Cell::new(None),
            validated_disabled: Cell::new(None),
            pending_version: Cell::new(None),
        });
        self.0.cancel_submit();
        self.0
            .fields
            .borrow_mut()
            .insert(key.clone(), field.clone());
        let weak = Rc::downgrade(&field);
        let form = Rc::downgrade(&self.0);
        scope.on_cleanup(move || {
            if let Some(field) = weak.upgrade() {
                field.retire();
            }
            if let Some(form) = form.upgrade() {
                let removed = form.fields.borrow_mut().remove(&key);
                drop(removed);
                if form.is_live() {
                    form.cancel_submit();
                    form.publish();
                }
            }
        })?;
        let weak = Rc::downgrade(&field);
        scope.effect(move || {
            let Some(field) = weak.upgrade() else { return };
            if !field.live.get() {
                return;
            }
            let _ = field.value.try_with(|_| ());
            let disabled = field.options.borrow().as_ref().map(|o| o.disabled.clone());
            let disabled = disabled.is_none_or(|d| d.get());
            let version = field.value.try_version().ok();
            if (disabled && field.pending_version.get().is_some())
                || field
                    .validated_disabled
                    .get()
                    .is_some_and(|old| old != disabled)
                || field
                    .pending_version
                    .get()
                    .is_some_and(|v| Some(v) != version)
                || field.validated.get().is_some_and(|v| Some(v) != version)
            {
                field.cancel();
                let _ = field.state.try_revise(|s| {
                    s.errors.clear();
                    s.validation_overflow = false;
                    s.validating = false;
                });
                if let Some(form) = field.form.upgrade() {
                    form.publish();
                }
            }
        });
        self.0.publish();
        Ok(FormField(field))
    }
    /// Validates the live registry before handing a completion token to application code.
    pub fn submit(
        &self,
        callback: impl FnOnce(SubmitToken) + 'static,
    ) -> Result<(), FormError> {
        if !self.0.is_live() {
            return Err(FormError::Retired);
        }
        let ticket = self.0.advance()?;
        self.0.cancel_submit();
        let fields = self.0.snapshot();
        struct Preparing<'a>(&'a Cell<usize>);
        impl Drop for Preparing<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() - 1);
            }
        }
        self.0.preparing.set(self.0.preparing.get() + 1);
        let preparing = Preparing(&self.0.preparing);
        *self.0.callback.borrow_mut() = Some(Box::new(callback));
        self.0.set_submit(SubmitStatus::Validating);
        let mut error = None;
        for field in fields {
            if let Err(e) = field.validate() {
                error.get_or_insert(e);
            }
            if self.0.ticket.get() != ticket
                || self.0.status.get_untracked().submit != SubmitStatus::Validating
            {
                drop(preparing);
                self.0.publish();
                return Err(FormError::Superseded);
            }
        }
        drop(preparing);
        if let Some(error) = error {
            for field in self.0.snapshot() {
                field.cancel();
            }
            self.0.cancel_submit();
            self.0.publish();
            return Err(error);
        }
        self.0.publish();
        Ok(())
    }
    pub fn cancel(&self) -> Result<(), FormError> {
        if !self.0.is_live() {
            return Err(FormError::Retired);
        }
        self.0.advance()?;
        self.0.cancel_submit();
        self.0.set_submit(SubmitStatus::Cancelled);
        for field in self.0.snapshot() {
            field.cancel();
        }
        self.0.publish();
        Ok(())
    }
    pub fn reset(&self) -> Result<(), FormError> {
        let scope = self.0.scope.upgrade().ok_or(FormError::Retired)?;
        scope
            .batch(|| {
                self.cancel()?;
                for field in self.0.snapshot() {
                    field.reset()?;
                }
                self.0.set_submit(SubmitStatus::Idle);
                self.0.publish();
                Ok(())
            })
            .map_err(|e| FormError::Batch(e.to_string()))?
    }
    /// Deterministic registration-key order, retaining duplicate submitted names.
    pub fn serialize(&self, limits: SubmissionLimits) -> Result<Vec<FormEntry>, FormError> {
        if !self.0.is_live() {
            return Err(FormError::Retired);
        }
        let mut entries = Vec::new();
        let mut bytes = 0usize;
        for field in self.0.snapshot() {
            let Some((name, value)) = field.encode()? else {
                continue;
            };
            let values = match value {
                ControlValue::Missing | ControlValue::Checked { checked: false, .. } => {
                    Vec::new()
                }
                ControlValue::Text(value)
                | ControlValue::Checked {
                    checked: true,
                    value,
                } => vec![value],
                ControlValue::Values(values) => values,
                ControlValue::UnsupportedFile => return Err(FormError::UnsupportedFiles),
            };
            for value in values {
                bytes = bytes
                    .checked_add(name.len())
                    .and_then(|b| b.checked_add(value.len()))
                    .ok_or(FormError::Capacity)?;
                if entries.len() >= limits.entries || bytes > limits.bytes {
                    return Err(FormError::Capacity);
                }
                entries.push(FormEntry {
                    name: name.clone(),
                    value,
                });
            }
        }
        Ok(entries)
    }
}
struct Field<T: 'static> {
    form: Weak<Inner>,
    scope: glory::ui::WeakScope,
    value: Cage<T>,
    state: Cage<FormFieldState>,
    read: Bond<FormFieldState>,
    live: Cell<bool>,
    options: RefCell<Option<FieldOptions<T>>>,
    ticket: Cell<u64>,
    task: Cell<Option<TaskToken>>,
    active: Cell<Option<u64>>,
    validated: Cell<Option<u64>>,
    validated_disabled: Cell<Option<bool>>,
    pending_version: Cell<Option<u64>>,
}
pub struct FormField<T: 'static>(Rc<Field<T>>);
impl<T> Clone for FormField<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> FormField<T> {
    pub fn value(&self) -> Cage<T> {
        self.0.value
    }
    pub fn state(&self) -> Bond<FormFieldState> {
        self.0.read
    }
    /// Call after accepting a user proposal. Does not write the business Cage.
    pub fn changed(&self) -> Result<(), FormError> {
        self.0.ensure_live()?;
        self.0.cancel();
        self.0.state.try_revise(|s| {
            s.dirty = true;
            s.errors.clear();
            s.validation_overflow = false;
        })?;
        if let Some(form) = self.0.form.upgrade() {
            form.cancel_submit();
            form.publish();
        }
        Ok(())
    }
    pub fn touch(&self) -> Result<(), FormError> {
        self.0.ensure_live()?;
        self.0.state.try_revise(|s| s.touched = true)?;
        if let Some(form) = self.0.form.upgrade() {
            form.publish();
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), FormError> {
        if let Some(form) = self.0.form.upgrade() {
            form.cancel_submit();
        }
        self.0.clone().validate()
    }
    pub fn reset(&self) -> Result<(), FormError> {
        let scope = self.0.scope.upgrade().ok_or(FormError::Retired)?;
        scope
            .batch(|| {
                self.0.reset()?;
                if let Some(form) = self.0.form.upgrade() {
                    form.cancel_submit();
                    form.publish();
                }
                Ok(())
            })
            .map_err(|e| FormError::Batch(e.to_string()))?
    }
    pub fn cancel(&self) {
        self.0.cancel();
        if let Some(form) = self.0.form.upgrade() {
            form.cancel_submit();
            form.publish();
        }
    }
}
impl<T> Field<T> {
    fn is_live(&self) -> bool {
        self.live.get() && self.state.is_alive()
    }
    fn ensure_live(&self) -> Result<(), FormError> {
        if !self.is_live() || !self.form.upgrade().is_some_and(|f| f.is_live()) {
            Err(FormError::Retired)
        } else {
            Ok(())
        }
    }
    fn finish(self: &Rc<Self>, ticket: u64, version: u64, errors: ValidationErrors) {
        if self.ensure_live().is_err() || self.active.get() != Some(ticket) {
            return;
        }
        self.active.set(None);
        self.task.set(None);
        self.pending_version.set(None);
        if self.value.try_version().ok() != Some(version) {
            self.cancel();
            if let Some(form) = self.form.upgrade() {
                form.publish();
            }
            return;
        }
        let Some(form) = self.form.upgrade() else {
            return;
        };
        let mut retained = Vec::new();
        let mut bytes = 0usize;
        let mut overflow = false;
        for error in errors {
            if retained.len() >= form.limits.errors_per_field
                || bytes.saturating_add(error.len()) > form.limits.error_bytes
            {
                retained.clear();
                overflow = true;
                break;
            }
            bytes += error.len();
            retained.push(error);
        }
        self.validated.set(Some(version));
        self.validated_disabled.set(Some(false));
        let _ = self.state.try_revise(|s| {
            s.validating = false;
            s.errors = retained;
            s.validation_overflow = overflow;
        });
        form.publish();
    }
}
impl<T> Registered for Field<T> {
    fn validate(self: Rc<Self>) -> Result<(), FormError> {
        self.ensure_live()?;
        let previous = self.ticket.get();
        self.cancel();
        if self.ticket.get() != previous {
            return Err(FormError::Superseded);
        }
        let ticket = self
            .ticket
            .get()
            .checked_add(1)
            .ok_or(FormError::Exhausted)?;
        self.ticket.set(ticket);
        self.active.set(Some(ticket));
        let (disabled, sync, asynchronous) = {
            let options = self.options.borrow();
            let options = options.as_ref().ok_or(FormError::Retired)?;
            (
                options.disabled.clone(),
                options.validator.clone(),
                options.async_validator.clone(),
            )
        };
        let version = self.value.try_version()?;
        if glory::core::untracked(|| disabled.get()) {
            self.validated.set(Some(version));
            self.validated_disabled.set(Some(true));
            self.state.try_revise(|s| {
                s.errors.clear();
                s.validation_overflow = false;
                s.validating = false;
            })?;
            return Ok(());
        }
        self.state.try_revise(|s| {
            s.touched = true;
            s.errors.clear();
            s.validation_overflow = false;
        })?;
        let scope = self.scope.upgrade().ok_or(FormError::Retired)?;
        let errors = if let Some(sync) = sync {
            scope.owner().enter(|| {
                self.value
                    .try_with_untracked(|value| glory::core::untracked(|| sync(value)))
            })?
        } else {
            Vec::new()
        };
        if self.active.get() != Some(ticket) || self.ensure_live().is_err() {
            return Err(FormError::Retired);
        }
        if !errors.is_empty() || asynchronous.is_none() {
            self.finish(ticket, version, errors);
            return Ok(());
        }
        let asynchronous = asynchronous.expect("checked async validator");
        let future = scope.owner().enter(|| {
            self.value
                .try_with_untracked(|value| glory::core::untracked(|| asynchronous(value)))
        })?;
        if self.active.get() != Some(ticket) || self.ensure_live().is_err() {
            return Err(FormError::Retired);
        }
        if self.value.try_version()? != version {
            self.cancel();
            return Ok(());
        }
        self.pending_version.set(Some(version));
        self.state.try_revise(|s| s.validating = true)?;
        let weak = Rc::downgrade(&self);
        let scope = self.scope.upgrade().ok_or(FormError::Retired)?;
        let spawned = scope.spawn(async move {
            let errors = future.await;
            if let Some(field) = weak.upgrade() {
                field.finish(ticket, version, errors);
            }
        });
        let task = match spawned {
            Ok(task) => task,
            Err(error) => {
                self.cancel();
                if let Some(form) = self.form.upgrade() {
                    form.publish();
                }
                return Err(error.into());
            }
        };
        self.task.set(Some(task));
        if let Some(form) = self.form.upgrade() {
            form.publish();
        }
        Ok(())
    }
    fn reset(&self) -> Result<(), FormError> {
        self.ensure_live()?;
        self.cancel();
        let reset = self.options.borrow().as_ref().and_then(|o| o.reset.clone());
        if let Some(reset) = reset {
            let scope = self.scope.upgrade().ok_or(FormError::Retired)?;
            scope.owner().enter(|| glory::core::untracked(|| reset()));
        }
        if !self.is_live() {
            return Err(FormError::Retired);
        }
        self.state.try_revise(|s| *s = FormFieldState::default())?;
        Ok(())
    }
    fn cancel(&self) {
        self.active.set(None);
        self.validated.set(None);
        self.validated_disabled.set(None);
        self.pending_version.set(None);
        // No wrapping. Clearing the permission invalidates even a saturated ticket.
        let task = self.task.take();
        let _ = self.state.try_revise(|s| s.validating = false);
        if let Some(task) = task {
            task.cancel();
        }
    }
    fn state(&self) -> FormFieldState {
        self.state
            .try_with_untracked(Clone::clone)
            .unwrap_or_default()
    }
    fn current(&self) -> bool {
        let disabled = self.options.borrow().as_ref().map(|o| o.disabled.clone());
        self.is_live()
            && self
                .validated
                .get()
                .is_some_and(|v| self.value.try_version().ok() == Some(v))
            && disabled.is_some_and(|d| {
                self.validated_disabled.get() == Some(glory::core::untracked(|| d.get()))
            })
    }
    fn encode(&self) -> Result<Option<(String, ControlValue)>, FormError> {
        self.ensure_live()?;
        let (disabled, name, encoder) = {
            let options = self.options.borrow();
            let options = options.as_ref().ok_or(FormError::Retired)?;
            (
                options.disabled.clone(),
                options.name.clone(),
                options.encoder.clone(),
            )
        };
        if glory::core::untracked(|| disabled.get()) {
            return Ok(None);
        }
        let (Some(name), Some(encoder)) = (name, encoder) else {
            return Ok(None);
        };
        if name.is_empty() {
            return Ok(None);
        }
        let scope = self.scope.upgrade().ok_or(FormError::Retired)?;
        let value = scope.owner().enter(|| {
            self.value
                .try_with_untracked(|v| glory::core::untracked(|| encoder(v)))
        })?;
        self.ensure_live()?;
        Ok(Some((name, value)))
    }
    fn retire(&self) {
        self.live.set(false);
        self.cancel();
        let options = self.options.borrow_mut().take();
        drop(options);
    }
}
