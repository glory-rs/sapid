use glory::prelude::*;
use glory::render::{EventKind, Orientation};
use sapid::{FieldOptions, FormState, StepStatus, Theme, step, stepper};
use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, Waker},
};

#[derive(Default)]
pub struct Gate {
    result: Option<Vec<String>>,
    waker: Option<Waker>,
}
pub struct State {
    pub active: Cage<usize>,
    pub enabled: Cage<bool>,
    pub accept: Cage<bool>,
    pub skip: Cage<bool>,
    pub async_enabled: Cage<bool>,
    pub requests: Cage<usize>,
    pub email: Cage<String>,
    pub ordered: Cage<bool>,
    pub model: FormState,
    pub gate: Rc<RefCell<Option<Rc<RefCell<Gate>>>>>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let active = scope.cage(0_usize);
    let enabled = scope.cage(true);
    let accept = scope.cage(true);
    let skip = scope.cage(false);
    let async_enabled = scope.cage(false);
    let requests = scope.cage(0_usize);
    let email = scope.cage(String::new());
    let ordered = scope.cage(true);
    let stage = scope.cage(false);
    let model = FormState::new(scope);
    let gate = Rc::new(RefCell::new(None));
    let factory = gate.clone();
    let field = model
        .register(
            scope,
            "email",
            email,
            FieldOptions::default()
                .validate(|value: &String| {
                    if value.contains('@') {
                        vec![]
                    } else {
                        vec!["Enter a valid email before continuing".into()]
                    }
                })
                .validate_async(move |_| {
                    if !async_enabled.get_untracked() {
                        return Box::pin(async { vec![] });
                    }
                    let current = Rc::new(RefCell::new(Gate::default()));
                    *factory.borrow_mut() = Some(current.clone());
                    Box::pin(std::future::poll_fn(move |context| {
                        let mut state = current.borrow_mut();
                        if let Some(result) = state.result.take() {
                            Poll::Ready(result)
                        } else {
                            state.waker = Some(context.waker().clone());
                            Poll::Pending
                        }
                    }))
                }),
        )
        .expect("bounded email field");
    let status = model.state();
    let field_status = field.state();
    let error = scope.bond(move || field_status.get().errors.join(" · "));
    let step_status =
        scope.bond(move || (!field_status.get().errors.is_empty()).then_some(StepStatus::Error));
    let submit = model.clone();
    let request: Rc<dyn Fn(usize)> = Rc::new(move |next| {
        requests.revise(|value| *value += 1);
        stage.revise(|value| *value = true);
        let version = active.try_version().expect("live active source");
        submit
            .submit(move |token| {
                if active.try_version().ok() == Some(version)
                    && accept.get_untracked()
                    && token.is_current()
                {
                    active.revise(|value| *value = next);
                    token.complete(Ok(()));
                } else {
                    token.complete(Err("Step proposal was rejected".into()));
                }
            })
            .expect("live workflow");
    });
    let header_request = request.clone();
    let next_request = request.clone();
    let field_events = field.clone();
    let body_theme = theme.clone();
    let first = step("account")
        .label("Account")
        .description(error)
        .status(step_status)
        .content(move || {
            let next_request = next_request.clone();
            let field_events = field_events.clone();
            sapid::form("stepper-account")
                .theme(body_theme.clone())
                .name("Account step")
                .on_submit(move |_| next_request(1))
                .fill((
                    sapid::field("stepper-email")
                        .theme(body_theme.clone())
                        .label("Account email")
                        .required(true)
                        .error(error)
                        .control({
                            let field_events = field_events.clone();
                            move |context| {
                                let field_events = field_events.clone();
                                sapid::text_field()
                                    .field(context)
                                    .class("stepper-email")
                                    .value(email)
                                    .on_value_change(move |next| {
                                        email.revise(|value| *value = next);
                                        field_events.changed().expect("live field");
                                    })
                            }
                        }),
                    sapid::submit_button()
                        .theme(body_theme.clone())
                        .label("Continue with validation"),
                ))
        });
    let next_request = request.clone();
    let body_theme = theme.clone();
    let second = step("details")
        .label("Details")
        .description("Optional review note")
        .optional(true)
        .content(move || {
            let next_request = next_request.clone();
            col()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(12.0),
                )
                .fill((
                    glory::ui::input()
                        .class("stepper-note")
                        .name("Review note")
                        .value("Local note survives hidden steps"),
                    sapid::button()
                        .theme(body_theme.clone())
                        .label("Continue to review")
                        .on_press(move || next_request(2)),
                ))
        });
    let next_request = request;
    let body_theme = theme.clone();
    let third = step("review")
        .label("Review")
        .description("Confirm the workflow")
        .content(move || {
            let next_request = next_request.clone();
            sapid::button()
                .theme(body_theme.clone())
                .label("Finish workflow")
                .on_press(move || next_request(3))
        });
    let status_text = scope.bond(move || {
        format!(
            "active={}; requests={}; ordered={}; submit={:?}; pending={}",
            active.get(),
            requests.get(),
            ordered.get(),
            status.get().submit,
            status.get().validating
        )
    });
    let widget = col()
        .class("stepper-demo")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(20.0),
        )
        .fill((
            sapid::title(1)
                .theme(theme.clone())
                .content("A workflow with explicit validation"),
            stepper("web-workflow")
                .theme(theme.clone())
                .class("main-stepper")
                .active(active)
                .enabled(enabled)
                .selectable(true)
                .allow_skip(skip)
                .steps([first, second, third])
                .on_step_change(move |next| header_request(next))
                .on(EventKind::Activate, move |_| {
                    ordered.revise(|value| *value &= stage.get_untracked())
                })
                .completed(|| glory::ui::text().content("Your workflow is complete.")),
            stepper("web-progress")
                .theme(theme.clone())
                .class("readonly-stepper")
                .name("Read-only progress")
                .orientation(Orientation::Vertical)
                .default_active(1)
                .steps([
                    step("received").label("Received"),
                    step("processed")
                        .label("Processing")
                        .description("Waiting for application confirmation"),
                    step("delivered").label("Delivered"),
                ]),
            sapid::text()
                .theme(theme)
                .class("stepper-status")
                .content(status_text),
        ));
    (
        widget,
        State {
            active,
            enabled,
            accept,
            skip,
            async_enabled,
            requests,
            email,
            ordered,
            model,
            gate,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use wasm_bindgen::prelude::*;
    thread_local! {static STEPPER:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(value: impl std::fmt::Display) -> JsValue {
        value.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_stepper(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing stepper root"))?;
        let theme = crate::themes::select(
            if hue == "green" { "green" } else { "blue" },
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
        let old = STEPPER.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn stepper_state(accept: bool, enabled: bool, skip: bool, asynchronous: bool) -> Result {
        STEPPER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted stepper"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|value| *value = accept);
            state.enabled.revise(|value| *value = enabled);
            state.skip.revise(|value| *value = skip);
            state.async_enabled.revise(|value| *value = asynchronous);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn stepper_active(index: u32) -> Result {
        STEPPER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted stepper"))?;
            app.app().pump().map_err(js)?;
            state.model.cancel().map_err(js)?;
            state.active.revise(|value| *value = index as usize);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn stepper_finish_async(valid: bool) -> Result {
        let wake = STEPPER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted stepper"))?;
            app.app().pump().map_err(js)?;
            let gate = state
                .gate
                .borrow()
                .clone()
                .ok_or_else(|| js("no pending step validation"))?;
            let mut gate = gate.borrow_mut();
            gate.result = Some(if valid {
                vec![]
            } else {
                vec!["Server rejected this email".into()]
            });
            Ok::<_, JsValue>(gate.waker.take())
        })?;
        if let Some(wake) = wake {
            wake.wake();
        }
        Ok(())
    }
    #[wasm_bindgen]
    pub fn unmount_stepper() {
        let old = STEPPER.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
