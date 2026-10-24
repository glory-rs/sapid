use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
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
        let step_status = scope
            .bond(move || (!field_status.get().errors.is_empty()).then_some(StepStatus::Error));
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
}
