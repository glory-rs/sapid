use glory::prelude::*;
use glory::render::{EventKind, NodeId};
use sapid::{
    ControlValue, FieldOptions, FormState, SubmissionLimits, Theme, checkbox, field,
    form as form_widget, reset_button, submit_button, text_field, textarea,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Poll, Waker};

#[derive(Default)]
pub struct Gate {
    result: Option<Vec<String>>,
    waker: Option<Waker>,
}
pub struct State {
    pub model: FormState,
    pub email: Cage<String>,
    pub consent: Cage<bool>,
    pub reject: Cage<bool>,
    pub disabled: Cage<bool>,
    pub readonly: Cage<bool>,
    pub async_enabled: Cage<bool>,
    pub gate: Rc<RefCell<Option<Rc<RefCell<Gate>>>>>,
    pub submissions: Cage<usize>,
    pub requests: Cage<usize>,
    pub resets: Cage<usize>,
    pub snapshot: Cage<String>,
    pub email_field: sapid::FormField<String>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let model = FormState::new(scope);
    let email = scope.cage("alex@studio.design".to_owned());
    let consent = scope.cage(true);
    let notes = scope.cage("A short note".to_owned());
    let reject = scope.cage(false);
    let disabled = scope.cage(false);
    let readonly = scope.cage(false);
    let async_enabled = scope.cage(false);
    let submissions = scope.cage(0usize);
    let requests = scope.cage(0usize);
    let resets = scope.cage(0usize);
    let snapshot = scope.cage(String::new());
    let stage = scope.cage(String::new());
    let order_ok = scope.cage(true);
    let submit_ref = scope.cage(None::<NodeId>);
    let gate = Rc::new(RefCell::new(None));
    let gate_factory = gate.clone();
    let email_field = model
        .register(
            scope,
            "a-email",
            email,
            FieldOptions::default()
                .disabled(disabled)
                .validate(|value: &String| {
                    if value.contains('@') {
                        vec![]
                    } else {
                        vec!["Enter a valid email".into()]
                    }
                })
                .validate_async(move |_| {
                    if !async_enabled.get_untracked() {
                        return Box::pin(async { vec![] });
                    }
                    let current = Rc::new(RefCell::new(Gate::default()));
                    *gate_factory.borrow_mut() = Some(current.clone());
                    Box::pin(std::future::poll_fn(move |context| {
                        let mut gate = current.borrow_mut();
                        if let Some(errors) = gate.result.take() {
                            Poll::Ready(errors)
                        } else {
                            gate.waker = Some(context.waker().clone());
                            Poll::Pending
                        }
                    }))
                })
                .on_reset(move || email.revise(|value| *value = "alex@studio.design".into()))
                .serialize("email", |value| ControlValue::Text(value.clone())),
        )
        .expect("email registration");
    let consent_field = model
        .register(
            scope,
            "b-consent",
            consent,
            FieldOptions::default()
                .disabled(disabled)
                .on_reset(move || consent.revise(|value| *value = true))
                .serialize("consent", |value| ControlValue::Checked {
                    checked: *value,
                    value: "yes".into(),
                }),
        )
        .expect("consent registration");
    let notes_field = model
        .register(
            scope,
            "c-notes",
            notes,
            FieldOptions::default()
                .on_reset(move || notes.revise(|value| *value = "A short note".into()))
                .serialize("note", |value: &String| ControlValue::Text(value.clone())),
        )
        .expect("notes registration");
    let error_state = email_field.state();
    let error = scope.bond(move || {
        let state = error_state.get();
        if state.validation_overflow {
            "Validation budget exceeded".to_owned()
        } else {
            state.errors.join(". ")
        }
    });
    let enabled = scope.bond(move || !disabled.get());
    let email_events = email_field.clone();
    let email_blur = email_field.clone();
    let submit_model = model.clone();
    let reset_model = model.clone();
    let status = model.state();
    let widgets = form_widget("profile-form")
        .theme(theme.clone())
        .name("Profile settings")
        .on_submit(move |submitter| {
            requests.revise(|count| *count += 1);
            stage.revise(|value| *value = "typed-submit".into());
            let current = submit_model.clone();
            submit_model
                .submit(move |token| {
                    let mut entries = current
                        .serialize(SubmissionLimits::default())
                        .expect("bounded fixture snapshot");
                    if submitter.is_some() && submitter == submit_ref.get_untracked() {
                        entries.push(sapid::FormEntry {
                            name: "intent".into(),
                            value: "save".into(),
                        });
                    }
                    snapshot.revise(|value| {
                        *value = entries
                            .iter()
                            .map(|e| format!("{}={}", e.name, e.value))
                            .collect::<Vec<_>>()
                            .join("&")
                    });
                    submissions.revise(|count| *count += 1);
                    token.complete(Ok(()));
                })
                .expect("fixture submit");
        })
        .on(EventKind::Submit, move |_| {
            order_ok.revise(|value| *value &= stage.get_untracked() == "typed-submit");
            stage.revise(|value| *value = "observed-submit".into());
        })
        .on_reset(move || {
            reset_model.reset().expect("fixture reset");
            resets.revise(|count| *count += 1);
            stage.revise(|value| *value = "typed-reset".into());
        })
        .on(EventKind::Reset, move |_| {
            order_ok.revise(|value| *value &= stage.get_untracked() == "typed-reset");
            stage.revise(|value| *value = "observed-reset".into());
        })
        .fill((
            field("form-email")
                .theme(theme.clone())
                .label("Account email")
                .description("Your work email.")
                .form_name("email")
                .required(true)
                .readonly(readonly)
                .enabled(enabled)
                .error(error)
                .control(move |context| {
                    text_field()
                        .field(context)
                        .class("form-email")
                        .value(email)
                        .on_value_change({
                            let field = email_events.clone();
                            move |value| {
                                if !reject.get() {
                                    email.revise(|old| *old = value);
                                    field.changed().expect("live email");
                                }
                            }
                        })
                        .on(EventKind::FocusChanged, {
                            let field = email_blur.clone();
                            move |event| {
                                if let Event::FocusChanged(false) = event {
                                    field.touch().expect("live blur");
                                }
                            }
                        })
                }),
            field("form-consent")
                .theme(theme.clone())
                .label("Product updates")
                .form_name("consent")
                .form_value("yes")
                .readonly(readonly)
                .enabled(enabled)
                .control(move |context| {
                    checkbox()
                        .field(context)
                        .class("form-consent")
                        .checked(consent)
                        .on_checked_change({
                            let field = consent_field.clone();
                            move |value| {
                                if !reject.get() {
                                    consent.revise(|old| *old = value);
                                    field.changed().expect("live consent");
                                }
                            }
                        })
                }),
            field("form-notes")
                .theme(theme.clone())
                .label("Notes")
                .form_name("note")
                .control(move |context| {
                    textarea()
                        .field(context)
                        .class("form-notes")
                        .rows(2)
                        .value(notes)
                        .on_value_change({
                            let field = notes_field.clone();
                            move |value| {
                                notes.revise(|old| *old = value);
                                field.changed().expect("live notes");
                            }
                        })
                }),
            submit_button()
                .theme(theme.clone())
                .label("Save profile")
                .class("form-save")
                .entry("intent", "save")
                .enabled(enabled)
                .node_ref(submit_ref),
            reset_button()
                .theme(theme)
                .label("Restore defaults")
                .class("form-reset"),
            text().class("form-status").content(scope.bond(move || {
                format!(
                    "{:?}; requests={}; submissions={}; resets={}; dirty={}; order={}",
                    status.get().submit,
                    requests.get(),
                    submissions.get(),
                    resets.get(),
                    status.get().dirty,
                    order_ok.get()
                )
            })),
            text().class("form-snapshot").content(snapshot),
        ));
    (
        widgets,
        State {
            model,
            email,
            consent,
            reject,
            disabled,
            readonly,
            async_enabled,
            gate,
            submissions,
            requests,
            resets,
            snapshot,
            email_field,
        },
    )
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use wasm_bindgen::prelude::*;
    thread_local! { static FORM: RefCell<Option<(WebApp, State)>> = const { RefCell::new(None) }; }
    type Result<T = ()> = std::result::Result<T, JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_form(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing form root"))?;
        let theme = Theme::new(
            if hue == "green" {
                sapid::Hue::Green
            } else {
                sapid::Hue::Blue
            },
            if scheme == "dark" {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        );
        let mut state = None;
        let build = |scope: &Scope| {
            let (widget, source) = form(scope, theme);
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = FORM.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn form_state(kind: String, value: bool) -> Result {
        FORM.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted form"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "reject" => state.reject.revise(|v| *v = value),
                "disabled" => state.disabled.revise(|v| *v = value),
                "readonly" => state.readonly.revise(|v| *v = value),
                "async" => state.async_enabled.revise(|v| *v = value),
                "consent" => state.consent.revise(|v| *v = value),
                _ => return Err(js("unknown form state")),
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn form_value(value: String) -> Result {
        FORM.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted form"))?;
            app.app().pump().map_err(js)?;
            state.email.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn form_snapshot() -> Result<String> {
        FORM.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted form"))?;
            app.app().pump().map_err(js)?;
            Ok(format!(
                "{:?}|{}|{}|{}|{}",
                state.model.state().get().submit,
                state.requests.get(),
                state.submissions.get(),
                state.resets.get(),
                state.snapshot.get()
            ))
        })
    }
    #[wasm_bindgen]
    pub fn form_validate() -> Result {
        FORM.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted form"))?;
            app.app().pump().map_err(js)?;
            state.email_field.validate().map_err(js)?;
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn form_finish_async(valid: bool) -> Result {
        FORM.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted form"))?;
            let gate = state.gate.borrow_mut().take();
            if let Some(gate) = gate {
                let waker = {
                    let mut gate = gate.borrow_mut();
                    gate.result = Some(if valid {
                        vec![]
                    } else {
                        vec!["Email is already registered".into()]
                    });
                    gate.waker.take()
                };
                if let Some(waker) = waker {
                    waker.wake();
                }
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_form() {
        let old = FORM.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
