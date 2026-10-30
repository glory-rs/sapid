//! Finite typed question factories. Values and validation remain application-owned.
use crate::appearance::{Common, common_methods};
use crate::composite::{self, Callback};
use crate::state::Value;
use crate::{
    FieldControl, FieldOptions, FormField, FormFieldState, FormLimits, FormState, SubmitStatus,
    SubmitToken, Theme,
};
use glory::ui::{Result, Widget};
use glory::{Cage, Scope, Vary};
use std::{cell::RefCell, rc::Rc};

pub const MAX_QUESTIONNAIRE_STEPS: usize = 16;
pub const MAX_QUESTIONS_PER_STEP: usize = 32;
pub const MAX_QUESTIONNAIRE_QUESTIONS: usize = 256;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
}
fn label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.contains(['\0', '\r', '\n'])
}
trait QuestionFactory {
    fn key(&self) -> &str;
    fn validate(&self, scope: &Scope) -> Result;
    fn mount(
        &mut self,
        scope: &Scope,
        form: &FormState,
        gate: Vary<bool>,
        prefix: &str,
        appearance: (Theme, crate::Size, bool),
        states: &mut Vec<glory::Bond<FormFieldState>>,
    ) -> Result;
}
/// A typed field declaration, erased only after its typed render factory is captured.
pub struct Question<T: 'static> {
    key: String,
    label: String,
    description: String,
    name: String,
    required: bool,
    value: Cage<T>,
    options: Option<FieldOptions<T>>,
    factory: Box<dyn FnMut(FormField<T>, FieldControl) -> Box<dyn Widget>>,
}
pub fn question<T: 'static, W: Widget>(
    key: impl Into<String>,
    value: Cage<T>,
    options: FieldOptions<T>,
    mut factory: impl FnMut(FormField<T>, FieldControl) -> W + 'static,
) -> Question<T> {
    let key = key.into();
    Question {
        label: key.clone(),
        name: key.clone(),
        key,
        description: String::new(),
        required: false,
        value,
        options: Some(options),
        factory: Box::new(move |field, control| Box::new(factory(field, control))),
    }
}
impl<T: 'static> Question<T> {
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }
    pub fn form_name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }
    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }
}
impl<T: 'static> QuestionFactory for Question<T> {
    fn key(&self) -> &str {
        &self.key
    }
    fn validate(&self, scope: &Scope) -> Result {
        if self.value.runtime() != scope.id().runtime() {
            return Err(glory::render::Error::ForeignRuntime.into());
        }
        self.value.try_with_untracked(|_| ())?;
        if !key(&self.key)
            || !label(&self.label)
            || self.description.len() > 4096
            || self.description.contains('\0')
            || self.name.is_empty()
            || self.name.len() > 256
            || self.name.contains(['\0', '\r', '\n'])
        {
            return Err(invalid("invalid typed question declaration"));
        }
        Ok(())
    }
    fn mount(
        &mut self,
        scope: &Scope,
        form: &FormState,
        gate: Vary<bool>,
        prefix: &str,
        appearance: (Theme, crate::Size, bool),
        states: &mut Vec<glory::Bond<FormFieldState>>,
    ) -> Result {
        let field = form
            .register(
                scope,
                format!(
                    "question-{}",
                    form.state().with_untracked(|state| state.fields)
                ),
                self.value,
                self.options
                    .take()
                    .ok_or_else(|| invalid("question factory mounted twice"))?
                    .gate_disabled(scope, gate),
            )
            .map_err(|error| invalid_form(&error))?;
        let state = field.state();
        states.push(state);
        let error = scope.bond(move || state.with(|state| state.errors.join("\n")));
        let invalid = scope.bond(move || {
            state.with(|state| state.validation_overflow || !state.errors.is_empty())
        });
        let mut factory = std::mem::replace(
            &mut self.factory,
            Box::new(|_, _| Box::new(glory::ui::text())),
        );
        let mut builder = crate::field(format!("{prefix}-{}", self.key))
            .theme(appearance.0)
            .size(appearance.1)
            .label(self.label.clone())
            .description(self.description.clone())
            .form_name(self.name.clone())
            .required(self.required)
            .invalid(invalid)
            .error(error)
            .control(move |control| factory(field.clone(), control));
        if appearance.2 {
            builder = builder.unstyled()
        }
        builder.show_in(scope)
    }
}
fn invalid_form(error: &crate::FormError) -> glory::ui::Error {
    glory::render::Error::Backend(format!("questionnaire form: {error}")).into()
}
pub struct QuestionStep {
    key: String,
    label: String,
    questions: Vec<Box<dyn QuestionFactory>>,
    overflow: bool,
}
pub fn question_step(key: impl Into<String>, label: impl Into<String>) -> QuestionStep {
    QuestionStep {
        key: key.into(),
        label: label.into(),
        questions: Vec::new(),
        overflow: false,
    }
}
impl QuestionStep {
    pub fn question<T: 'static>(mut self, question: Question<T>) -> Self {
        if self.questions.len() == MAX_QUESTIONS_PER_STEP {
            self.overflow = true
        } else {
            self.questions.push(Box::new(question));
        }
        self
    }
}
pub struct Questionnaire {
    common: Common,
    key: String,
    title: String,
    steps: Vec<QuestionStep>,
    overflow: bool,
    active: Value<usize>,
    change: Option<Box<dyn FnMut(usize)>>,
    complete: Option<Box<dyn FnMut(SubmitToken)>>,
}
pub fn questionnaire(key: impl Into<String>) -> Questionnaire {
    Questionnaire {
        common: Common::default(),
        key: key.into(),
        title: "Questionnaire".into(),
        steps: Vec::new(),
        overflow: false,
        active: Value::default(),
        change: None,
        complete: None,
    }
}
impl Questionnaire {
    common_methods!();
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }
    pub fn step(mut self, value: QuestionStep) -> Self {
        if self.steps.len() == MAX_QUESTIONNAIRE_STEPS {
            self.overflow = true
        } else {
            self.steps.push(value)
        }
        self
    }
    pub fn steps(mut self, values: impl IntoIterator<Item = QuestionStep>) -> Self {
        for value in values
            .into_iter()
            .take(MAX_QUESTIONNAIRE_STEPS - self.steps.len() + 1)
        {
            self = self.step(value);
            if self.overflow {
                break;
            }
        }
        self
    }
    pub fn active(mut self, value: impl Into<Vary<usize>>) -> Self {
        self.active.source = Some(value.into());
        self
    }
    pub fn default_active(mut self, value: usize) -> Self {
        self.active.default = value;
        self
    }
    pub fn on_step_change(mut self, value: impl FnMut(usize) + 'static) -> Self {
        self.change = Some(Box::new(value));
        self
    }
    /// Called only after all live typed fields have passed fresh validation.
    /// The application owns submission, authorization and token completion.
    pub fn on_complete(mut self, value: impl FnMut(SubmitToken) + 'static) -> Self {
        self.complete = Some(Box::new(value));
        self
    }
}
struct Questions {
    questions: Vec<Box<dyn QuestionFactory>>,
    form: FormState,
    gate: Vary<bool>,
    prefix: String,
    appearance: (Theme, crate::Size, bool),
    states: Rc<RefCell<Vec<Vec<glory::Bond<FormFieldState>>>>>,
    index: usize,
}
impl Widget for Questions {
    fn build(&mut self, scope: &Scope) -> Result {
        if self.questions.is_empty() {
            return Err(invalid("question fields may mount only once"));
        }
        let mut states = Vec::with_capacity(self.questions.len());
        for question in &mut self.questions {
            question.mount(
                scope,
                &self.form,
                self.gate.clone(),
                &self.prefix,
                self.appearance.clone(),
                &mut states,
            )?;
        }
        self.states.borrow_mut()[self.index] = states;
        Ok(())
    }
}

impl Widget for Questionnaire {
    fn build(&mut self, scope: &Scope) -> Result {
        composite::id("questionnaire", &self.key, "root", "root")?;
        let count = self.steps.len();
        let mut total = 0usize;
        if self.overflow || count == 0 || !label(&self.title) {
            return Err(invalid("invalid questionnaire step budget or title"));
        }
        let mut step_keys = std::collections::BTreeSet::new();
        for step in &self.steps {
            if !key(&step.key)
                || !label(&step.label)
                || !step_keys.insert(step.key.as_str())
                || step.overflow
                || step.questions.is_empty()
            {
                return Err(invalid("invalid questionnaire step"));
            }
            total += step.questions.len();
            let mut keys = std::collections::BTreeSet::new();
            for question in &step.questions {
                question.validate(scope)?;
                if !keys.insert(question.key()) {
                    return Err(invalid("duplicate question key"));
                }
            }
        }
        if total > MAX_QUESTIONNAIRE_QUESTIONS {
            return Err(invalid("questionnaire exceeds 256 typed fields"));
        }
        let control = std::mem::take(&mut self.active).mount(scope);
        let active = composite::validated(scope, &control.source, move |index| {
            if *index < count {
                Ok(())
            } else {
                Err(invalid("questionnaire active step out of range"))
            }
        })?;
        let all = scope.cage(false);
        let form = FormState::with_limits(
            scope,
            FormLimits {
                fields: MAX_QUESTIONNAIRE_QUESTIONS,
                errors_per_field: 8,
                error_bytes: 4096,
            },
        );
        let status = form.state();
        let states = Rc::new(RefCell::new(vec![
            Vec::<glory::Bond<FormFieldState>>::new();
            count
        ]));
        let change = Callback::new(self.change.take());
        let complete = Callback::new(self.complete.take());
        let owner = scope.id();
        let root_reference = self.common.node_ref.unwrap_or_else(|| scope.cage(None));
        let actual_root = Rc::new(std::cell::Cell::new(None));
        let enabled = self.common.enabled.clone();
        let busy = scope.bond(move || {
            matches!(
                status.with(|status| status.submit.clone()),
                SubmitStatus::Validating | SubmitStatus::Submitting
            )
        });
        // An accepted external step change revokes an in-flight validation/submission.
        let previous = std::cell::Cell::new(active.get_untracked());
        let observing = form.clone();
        scope.effect(move || {
            let next = active.get();
            if previous.replace(next) != next {
                let _ = observing.cancel();
                if all.get_untracked() {
                    all.revise(|value| *value = false)
                }
            }
        });
        // All-field validation can fail on an earlier, currently hidden step. Propose
        // that step without overriding a controlled active source.
        let error_states = states.clone();
        let error_control = control.clone();
        let error_change = change.clone();
        let error_scope = scope.downgrade();
        let error_root = actual_root.clone();
        let attempt = Rc::new(std::cell::Cell::new(0u64));
        let error_attempt = attempt.clone();
        let recovering = std::cell::Cell::new(None);
        scope.effect(move || {
            if !status.with(|status| status.submit == SubmitStatus::Invalid) {
                recovering.set(None);
                return;
            }
            let current_attempt = error_attempt.get();
            if all.get_untracked()
                && recovering.replace(Some(current_attempt)) != Some(current_attempt)
            {
                let first = {
                    let states = error_states.borrow();
                    states.iter().position(|fields| {
                        fields.iter().any(|state| {
                            state
                                .with(|state| state.validation_overflow || !state.errors.is_empty())
                        })
                    })
                };
                // Keep all-field validation results live until the accepted step
                // changes. A refused controlled proposal must not clear its errors.
                if let Some(first) = first
                    && first != active.get_untracked()
                    && let Some(scope) = error_scope.upgrade()
                {
                    let weak = error_scope.clone();
                    let root = error_root.clone();
                    let control = error_control.clone();
                    let changed = error_change.clone();
                    let expected = active.get_untracked();
                    let attempt = error_attempt.clone();
                    let failed = error_states.clone();
                    if let Err(error) = scope.spawn(async move {
                        let Some(scope) = weak.upgrade() else { return };
                        let Some(root) = root.get() else { return };
                        if active.get_untracked() != expected
                            || attempt.get() != current_attempt
                            || !status
                                .with_untracked(|status| status.submit == SubmitStatus::Invalid)
                            || !failed.borrow()[first].iter().any(|field| {
                                field.with_untracked(|state| {
                                    state.validation_overflow || !state.errors.is_empty()
                                })
                            })
                            || !scope.is_active(root).unwrap_or(false)
                            || !scope.is_enabled(root).unwrap_or(false)
                        {
                            return;
                        }
                        if let Err(error) = control.accept(&first) {
                            scope.report_error(error)
                        } else {
                            changed.emit(owner, first)
                        }
                    }) {
                        scope.report_error(error)
                    }
                }
            }
        });
        let request = crate::state::Dirty::new(scope);
        let pending = request.clone();
        let action = Rc::new(std::cell::Cell::new(false));
        let requested_back = action.clone();
        let queued = Rc::new(std::cell::Cell::new(false));
        let weak = scope.downgrade();
        let request_form = form.clone();
        let request_control = control.clone();
        let request_change = change.clone();
        let request_complete = complete.clone();
        let request_root = actual_root.clone();
        let request_enabled = enabled.clone();
        scope.effect(move || {
            if !pending.take() || queued.replace(true) {
                return;
            }
            let Some(scope) = weak.upgrade() else { return };
            let weak = weak.clone();
            let queued_task = queued.clone();
            let root = request_root.clone();
            let form = request_form.clone();
            let control = request_control.clone();
            let change = request_change.clone();
            let complete = request_complete.clone();
            let enabled = request_enabled.clone();
            let back = requested_back.get();
            let attempt = attempt.clone();
            if let Err(error) = scope.spawn(async move {
                queued_task.set(false);
                let Some(scope) = weak.upgrade() else { return };
                let Some(root) = root.get() else { return };
                if !scope.is_active(root).unwrap_or(false)
                    || !scope.is_enabled(root).unwrap_or(false)
                    || !enabled.get()
                {
                    return;
                }
                let current = active.get_untracked();
                if back {
                    if let Err(error) = form.cancel() {
                        scope.report_error(invalid_form(&error));
                        return;
                    }
                    all.revise(|value| *value = false);
                    if current > 0 {
                        if let Err(error) = control.accept(&(current - 1)) {
                            scope.report_error(error)
                        } else {
                            change.emit(owner, current - 1)
                        }
                    }
                    return;
                }
                if matches!(
                    status.with_untracked(|status| status.submit.clone()),
                    SubmitStatus::Validating | SubmitStatus::Submitting
                ) {
                    return;
                }
                let final_step = current + 1 == count;
                let Some(next_attempt) = attempt.get().checked_add(1) else {
                    scope.report_error(invalid("questionnaire validation attempts exhausted"));
                    return;
                };
                attempt.set(next_attempt);
                all.revise(|value| *value = final_step);
                let weak = scope.downgrade();
                let completion = move |token: SubmitToken| {
                    let Some(scope) = weak.upgrade() else { return };
                    let weak = weak.clone();
                    if let Err(error) = scope.spawn(async move {
                        let Some(scope) = weak.upgrade() else { return };
                        if !token.is_current()
                            || active.get_untracked() != current
                            || !scope.is_active(root).unwrap_or(false)
                            || !scope.is_enabled(root).unwrap_or(false)
                        {
                            return;
                        }
                        if final_step {
                            complete.emit(owner, token)
                        } else if token.complete(Ok(())) {
                            if let Err(error) = control.accept(&(current + 1)) {
                                scope.report_error(error)
                            } else {
                                change.emit(owner, current + 1)
                            }
                        }
                    }) {
                        scope.report_error(error)
                    }
                };
                if let Err(error) = form.submit(completion) {
                    scope.report_error(invalid_form(&error))
                }
            }) {
                queued.set(false);
                scope.report_error(error)
            }
        });
        let mut stepper = crate::stepper(self.key.clone())
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .active(active)
            .selectable(false);
        if self.common.unstyled {
            stepper = stepper.unstyled()
        }
        for (index, step) in self.steps.drain(..).enumerate() {
            let gate_enabled = enabled.clone();
            let gate =
                scope.bond(move || !gate_enabled.get() || !all.get() && active.get() != index);
            let ui_disabled = scope.bond(move || active.get() != index);
            let form = form.clone();
            let states = states.clone();
            let appearance = (
                self.common.theme.clone(),
                self.common.size,
                self.common.unstyled,
            );
            let prefix = format!("{}-s{index}", self.key);
            let mut questions = Some(step.questions);
            let legend = step.label.clone();
            let step_appearance = appearance.clone();
            stepper = stepper.step(crate::step(step.key).label(step.label).content(move || {
                let mut fields = crate::fieldset(prefix.clone())
                    .theme(step_appearance.0.clone())
                    .size(step_appearance.1)
                    .legend(legend.clone())
                    .disabled(ui_disabled)
                    .fill(Questions {
                        questions: questions.take().unwrap_or_default(),
                        form: form.clone(),
                        gate: gate.into(),
                        prefix: prefix.clone(),
                        appearance: appearance.clone(),
                        states: states.clone(),
                        index,
                    });
                if step_appearance.2 {
                    fields = fields.unstyled()
                }
                fields
            }));
        }
        let back_request = request.clone();
        let back_action = action.clone();
        let next_request = request.clone();
        let next_action = action.clone();
        let back_enabled = enabled.clone();
        let next_enabled = enabled.clone();
        let actions = glory::ui::view()
            .class("sapid-questionnaire-actions")
            .style(
                glory::style::Style::new()
                    .display(glory::style::Display::Flex)
                    .gap(8.),
            )
            .fill((
                action_button(&self.common)
                    .label("Previous question step")
                    .enabled(scope.bond(move || back_enabled.get() && active.get() > 0))
                    .on_press(move || {
                        back_action.set(true);
                        let _ = back_request.touch();
                    }),
                action_button(&self.common)
                    .label(scope.bond(move || {
                        if active.get() + 1 == count {
                            "Submit answers".to_owned()
                        } else {
                            "Next question step".to_owned()
                        }
                    }))
                    .enabled(scope.bond(move || next_enabled.get() && !busy.get()))
                    .on_press(move || {
                        next_action.set(false);
                        let _ = next_request.touch();
                    }),
            ));
        let mut title = crate::title(2)
            .theme(self.common.theme.clone())
            .size(self.common.size)
            .content(self.title.clone());
        if self.common.unstyled {
            title = title.unstyled()
        }
        let class = self.common.class.clone();
        let mut root = crate::form(self.key.clone())
            .theme(self.common.theme.clone())
            .class(scope.bond(move || format!("sapid-questionnaire {}", class.get())))
            .style(self.common.style.clone())
            .name(
                self.common
                    .name
                    .take()
                    .unwrap_or_else(|| self.title.clone().into()),
            )
            .enabled(enabled)
            .focus(self.common.focus)
            .node_ref(root_reference)
            .fill((title, stepper, actions))
            .on_submit(move |_| {
                action.set(false);
                let _ = request.touch();
            });
        for (kind, handlers) in std::mem::take(&mut self.common.handlers) {
            for handler in handlers {
                root = root.on(kind, handler);
            }
        }
        if self.common.unstyled {
            root = root.unstyled()
        }
        root.show_in(scope)?;
        actual_root.set(root_reference.try_get()?);
        scope.on_cleanup(move || {
            change.clear();
            complete.clear();
            states.borrow_mut().clear();
        })?;
        Ok(())
    }
}

fn action_button(common: &Common) -> crate::Button {
    let mut button = crate::button()
        .theme(common.theme.clone())
        .size(common.size);
    if common.unstyled {
        button = button.unstyled()
    }
    button
}
