use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::task::Poll;

use glory::core::executor::Executor;
use glory::prelude::*;
use sapid::{
    ControlValue, FieldOptions, FormError, FormLimits, FormState, SubmissionLimits,
    SubmitStatus,
};

#[test]
fn values_need_no_clone_and_external_writes_are_silent() {
    struct NonClone(String);
    let app = App::memory();
    let source = app.scope().cage(NonClone("a".into()));
    let form = FormState::new(app.scope());
    let field = form
        .register(app.scope(), "name", source, FieldOptions::default())
        .unwrap();
    app.flush().unwrap();
    source.revise(|v| v.0 = "external".into());
    app.flush().unwrap();
    assert!(!field.state().get().dirty);
    assert!(!field.state().get().touched);
    field.changed().unwrap();
    field.touch().unwrap();
    app.flush().unwrap();
    assert!(field.state().get().dirty && field.state().get().touched);
    assert_eq!(
        (form.state().get().dirty, form.state().get().touched),
        (1, 1)
    );
    assert_eq!(source.with(|v| v.0.clone()), "external");
}

#[test]
fn synchronous_errors_and_submit_barrier() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let value = app.scope().cage(String::new());
    let field = form
        .register(
            app.scope(),
            "required",
            value,
            FieldOptions::default().validate(|v: &String| {
                if v.is_empty() {
                    vec!["required".into()]
                } else {
                    vec![]
                }
            }),
        )
        .unwrap();
    let submitted = Rc::new(Cell::new(0));
    let called = submitted.clone();
    form.submit(move |_| called.set(called.get() + 1)).unwrap();
    assert_eq!(form.state().get().submit, SubmitStatus::Invalid);
    assert_eq!(field.state().get().errors, vec!["required"]);
    assert_eq!(submitted.get(), 0);
    value.revise(|v| *v = "yes".into());
    field.changed().unwrap();
    let called = submitted.clone();
    form.submit(move |token| {
        called.set(called.get() + 1);
        assert!(token.complete(Ok(())));
    })
    .unwrap();
    assert_eq!(submitted.get(), 1);
    assert_eq!(form.state().get().submit, SubmitStatus::Succeeded);
}

#[test]
fn async_validation_waits_for_every_field_before_submit() {
    let executor = Rc::new(Executor::new(8));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    for key in ["one", "two"] {
        form.register(
            app.scope(),
            key,
            app.scope().cage(1),
            FieldOptions::default().validate_async(|_| Box::pin(async { vec![] })),
        )
        .unwrap();
    }
    app.flush().unwrap();
    let count = Rc::new(Cell::new(0));
    let called = count.clone();
    form.submit(move |t| {
        called.set(called.get() + 1);
        t.complete(Ok(()));
    })
    .unwrap();
    assert_eq!(form.state().get().validating, 2);
    assert!(executor.poll_one());
    assert_eq!(count.get(), 0);
    assert!(executor.poll_one());
    assert_eq!(count.get(), 1);
    assert_eq!(form.state().get().submit, SubmitStatus::Succeeded);
    assert!(executor.is_empty());
}

#[test]
fn source_aba_before_flush_cannot_accept_async_result() {
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let value = app.scope().cage(1);
    let field = form
        .register(
            app.scope(),
            "one",
            value,
            FieldOptions::default()
                .validate_async(|_| Box::pin(async { vec!["obsolete".into()] })),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    value.revise(|v| *v = 2);
    value.revise(|v| *v = 1);
    assert!(executor.poll_one());
    assert!(field.state().get().errors.is_empty());
    assert!(!field.state().get().validating);
}

#[test]
fn external_source_write_cancels_pending_without_dirty_or_touch_notifications() {
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let value = app.scope().cage(1);
    let field = form
        .register(
            app.scope(),
            "one",
            value,
            FieldOptions::default().validate_async(|_| Box::pin(std::future::pending())),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    assert_eq!(executor.len(), 1);
    value.revise(|v| *v += 1);
    app.flush().unwrap();
    assert!(executor.is_empty());
    assert!(!field.state().get().dirty);
    assert!(!field.state().get().validating);
}

#[test]
fn new_validation_cancels_old_future_and_reset_rejects_completion_token() {
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let value = app.scope().cage(1);
    let field = form
        .register(
            app.scope(),
            "one",
            value,
            FieldOptions::default().validate_async(|_| Box::pin(std::future::pending())),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    field.validate().unwrap();
    assert_eq!(executor.len(), 1);
    field.cancel();
    assert!(executor.is_empty());
    // Disabled fields do not start validation, including during submit.
    let disabled = app.scope().cage(true);
    let form2 = FormState::new(app.scope());
    form2
        .register(
            app.scope(),
            "disabled",
            value,
            FieldOptions::default()
                .disabled(disabled)
                .validate_async(|_| Box::pin(std::future::pending())),
        )
        .unwrap();
    let token = Rc::new(RefCell::new(None));
    let captured = token.clone();
    form2
        .submit(move |t| *captured.borrow_mut() = Some(t))
        .unwrap();
    let token = token.borrow_mut().take().unwrap();
    assert!(token.is_current());
    form2.reset().unwrap();
    assert!(!token.complete(Ok(())));
}

#[test]
fn owner_retirement_unregisters_and_releases_callbacks_even_with_handles() {
    struct DropCount(Rc<Cell<usize>>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let app = App::memory();
    let form = FormState::new(app.scope());
    let child = app.scope().child().unwrap();
    let count = Rc::new(Cell::new(0));
    let captured = DropCount(count.clone());
    let field = form
        .register(
            &child,
            "row",
            child.cage(3),
            FieldOptions::default().validate(move |_| {
                let _ = &captured;
                vec![]
            }),
        )
        .unwrap();
    child.dispose();
    app.flush().unwrap();
    assert_eq!(count.get(), 1);
    assert_eq!(form.state().get().fields, 0);
    assert_eq!(field.changed(), Err(FormError::Retired));
    let replacement = form
        .register(
            app.scope(),
            "row",
            app.scope().cage(4),
            FieldOptions::default(),
        )
        .unwrap();
    replacement.touch().unwrap();
    assert_eq!(form.state().get().fields, 1);
}

#[test]
fn reset_is_atomic_even_when_callback_attempts_flush() {
    let app = Rc::new(App::memory());
    let form = FormState::new(app.scope());
    let a = app.scope().cage(1);
    let b = app.scope().cage(2);
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = observed.clone();
    app.scope()
        .effect(move || output.borrow_mut().push((a.get(), b.get())));
    let app_callback = app.clone();
    form.register(
        app.scope(),
        "a",
        a,
        FieldOptions::default().on_reset(move || {
            a.revise(|v| *v = 0);
            app_callback.flush().unwrap();
        }),
    )
    .unwrap();
    form.register(
        app.scope(),
        "b",
        b,
        FieldOptions::default().on_reset(move || b.revise(|v| *v = 0)),
    )
    .unwrap();
    app.flush().unwrap();
    form.reset().unwrap();
    assert_eq!(*observed.borrow(), vec![(1, 2), (0, 0)]);
    assert_eq!(form.state().get().submit, SubmitStatus::Idle);
}

#[test]
fn registration_budgets_duplicate_keys_and_wrong_runtime_are_rejected() {
    let app = App::memory();
    let other = App::memory();
    let form = FormState::with_limits(
        app.scope(),
        FormLimits {
            fields: 1,
            ..Default::default()
        },
    );
    form.register(
        app.scope(),
        "valid",
        app.scope().cage(1),
        FieldOptions::default(),
    )
    .unwrap();
    assert!(matches!(
        form.register(
            app.scope(),
            "valid",
            app.scope().cage(2),
            FieldOptions::default()
        ),
        Err(FormError::DuplicateKey)
    ));
    assert!(matches!(
        form.register(
            app.scope(),
            "other",
            app.scope().cage(2),
            FieldOptions::default()
        ),
        Err(FormError::Capacity)
    ));
    assert!(matches!(
        form.register(
            other.scope(),
            "wrong",
            other.scope().cage(2),
            FieldOptions::default()
        ),
        Err(FormError::InvalidKey)
    ));
    assert!(matches!(
        form.register(
            app.scope(),
            "bad space",
            app.scope().cage(2),
            FieldOptions::default()
        ),
        Err(FormError::InvalidKey)
    ));
}

#[test]
fn serialization_preserves_duplicates_and_skips_missing_unchecked_disabled() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let value = app.scope().cage("keep".to_owned());
    for (key, encoded, disabled) in [
        ("a", ControlValue::Text("first".into()), false),
        ("b", ControlValue::Missing, false),
        (
            "c",
            ControlValue::Checked {
                checked: false,
                value: "no".into(),
            },
            false,
        ),
        (
            "d",
            ControlValue::Checked {
                checked: true,
                value: "yes".into(),
            },
            false,
        ),
        ("e", ControlValue::Text("disabled".into()), true),
        (
            "f",
            ControlValue::Values(vec!["two".into(), "three".into()]),
            false,
        ),
    ] {
        form.register(
            app.scope(),
            key,
            value,
            FieldOptions::default()
                .disabled(disabled)
                .serialize("same", move |_| encoded.clone()),
        )
        .unwrap();
    }
    let entries = form.serialize(SubmissionLimits::default()).unwrap();
    assert_eq!(
        entries.iter().map(|e| e.value.as_str()).collect::<Vec<_>>(),
        vec!["first", "yes", "two", "three"]
    );
    assert!(entries.iter().all(|e| e.name == "same"));
    assert_eq!(
        form.serialize(SubmissionLimits {
            entries: 3,
            bytes: 999
        }),
        Err(FormError::Capacity)
    );
    assert_eq!(
        form.serialize(SubmissionLimits {
            entries: 99,
            bytes: 5
        }),
        Err(FormError::Capacity)
    );
}

#[test]
fn unsupported_files_are_explicit_and_readonly_values_are_included() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    form.register(
        app.scope(),
        "readonly",
        app.scope().cage(7),
        FieldOptions::default().serialize("value", |v: &i32| ControlValue::Text(v.to_string())),
    )
    .unwrap();
    assert_eq!(
        form.serialize(SubmissionLimits::default()).unwrap()[0].value,
        "7"
    );
    form.register(
        app.scope(),
        "file",
        app.scope().cage(()),
        FieldOptions::default().serialize("upload", |_| ControlValue::UnsupportedFile),
    )
    .unwrap();
    assert_eq!(
        form.serialize(SubmissionLimits::default()),
        Err(FormError::UnsupportedFiles)
    );
}

#[test]
fn validation_error_storage_is_bounded() {
    let app = App::memory();
    let form = FormState::with_limits(
        app.scope(),
        FormLimits {
            errors_per_field: 1,
            error_bytes: 10,
            ..Default::default()
        },
    );
    let field = form
        .register(
            app.scope(),
            "limited",
            app.scope().cage(0),
            FieldOptions::default().validate(|_| vec!["one".into(), "two".into()]),
        )
        .unwrap();
    field.validate().unwrap();
    assert!(field.state().get().errors.is_empty());
    assert!(field.state().get().validation_overflow);
    assert_eq!(form.state().get().invalid, 1);
}

#[test]
fn membership_changes_invalidate_submit_tokens_and_old_owner_results() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let token = Rc::new(RefCell::new(None));
    let copy = token.clone();
    form.submit(move |t| *copy.borrow_mut() = Some(t)).unwrap();
    let token = token.borrow_mut().take().unwrap();
    form.register(
        app.scope(),
        "new",
        app.scope().cage(0),
        FieldOptions::default(),
    )
    .unwrap();
    assert!(!token.complete(Ok(())));
    let child = app.scope().child().unwrap();
    let child_form = FormState::new(&child);
    child.dispose();
    assert_eq!(child_form.reset(), Err(FormError::Retired));
}

#[test]
fn cancelled_validation_drops_future_outside_registry_borrows() {
    struct OnDrop(Rc<dyn Fn()>);
    impl Drop for OnDrop {
        fn drop(&mut self) {
            (self.0)();
        }
    }
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let clone = form.clone();
    let field = form
        .register(
            app.scope(),
            "pending",
            app.scope().cage(0),
            FieldOptions::default().validate_async(move |_| {
                let clone = clone.clone();
                let guard = OnDrop(Rc::new(move || {
                    let _ = clone.serialize(SubmissionLimits::default());
                }));
                Box::pin(async move {
                    let _guard = guard;
                    std::future::poll_fn(|_| Poll::<Vec<String>>::Pending).await
                })
            }),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    field.cancel();
    assert!(executor.is_empty());
}

#[test]
fn host_spawn_failure_clears_pending_validation() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let field = form
        .register(
            app.scope(),
            "no-host",
            app.scope().cage(()),
            FieldOptions::default().validate_async(|_| Box::pin(async { vec![] })),
        )
        .unwrap();
    assert!(matches!(
        field.validate(),
        Err(FormError::Access(glory::core::AccessError::NoHost))
    ));
    assert!(!field.state().get().validating);
    assert_eq!(form.state().get().validating, 0);
}

#[test]
fn reentrant_cancel_during_validator_cannot_submit_or_leave_preparation_stuck() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let nested = form.clone();
    let once = Rc::new(Cell::new(false));
    let seen = once.clone();
    form.register(
        app.scope(),
        "reentrant",
        app.scope().cage(()),
        FieldOptions::default().validate(move |_| {
            if !seen.replace(true) {
                nested.cancel().unwrap();
            }
            vec![]
        }),
    )
    .unwrap();
    assert_eq!(
        form.submit(|_| panic!("cancelled submit must not run")),
        Err(FormError::Superseded)
    );
    let count = Rc::new(Cell::new(0));
    let called = count.clone();
    form.submit(move |t| {
        called.set(1);
        t.complete(Ok(()));
    })
    .unwrap();
    assert_eq!(count.get(), 1);
}

#[test]
fn disabled_and_external_changes_clear_obsolete_errors() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let disabled = app.scope().cage(false);
    let value = app.scope().cage(1);
    let field = form
        .register(
            app.scope(),
            "error",
            value,
            FieldOptions::default()
                .disabled(disabled)
                .validate(|_| vec!["error".into()]),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    value.revise(|v| *v += 1);
    app.flush().unwrap();
    assert!(field.state().get().errors.is_empty());
    field.validate().unwrap();
    disabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(field.state().get().errors.is_empty());
}

#[test]
fn submission_completion_rechecks_source_versions_before_flush() {
    let app = App::memory();
    let form = FormState::new(app.scope());
    let value = app.scope().cage(0);
    form.register(app.scope(), "value", value, FieldOptions::default())
        .unwrap();
    let token = Rc::new(RefCell::new(None));
    let copy = token.clone();
    form.submit(move |t| *copy.borrow_mut() = Some(t)).unwrap();
    let token = token.borrow_mut().take().unwrap();
    value.revise(|v| *v = 1);
    assert!(!token.is_current());
    assert!(!token.complete(Ok(())));
    assert_eq!(form.state().get().submit, SubmitStatus::Cancelled);
}

#[test]
fn retiring_pending_field_cancels_task_before_key_reuse() {
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let child = app.scope().child().unwrap();
    let field = form
        .register(
            &child,
            "row",
            child.cage(0),
            FieldOptions::default().validate_async(|_| Box::pin(std::future::pending())),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    assert_eq!(executor.len(), 1);
    child.dispose();
    assert!(executor.is_empty());
    let replacement = form
        .register(
            app.scope(),
            "row",
            app.scope().cage(1),
            FieldOptions::default(),
        )
        .unwrap();
    replacement.validate().unwrap();
    app.flush().unwrap();
    assert!(replacement.state().get().errors.is_empty());
    assert_eq!(form.state().get().fields, 1);
}

#[test]
fn disabled_snapshot_survives_first_effect_but_enabling_revokes_submit() {
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let disabled = app.scope().cage(true);
    form.register(
        app.scope(),
        "disabled",
        app.scope().cage(()),
        FieldOptions::default()
            .disabled(disabled)
            .validate(|_| vec!["must never validate while disabled".into()]),
    )
    .unwrap();
    form.register(
        app.scope(),
        "async",
        app.scope().cage(()),
        FieldOptions::default().validate_async(|_| Box::pin(async { vec![] })),
    )
    .unwrap();
    let token = Rc::new(RefCell::new(None));
    let copy = token.clone();
    form.submit(move |t| *copy.borrow_mut() = Some(t)).unwrap();
    app.flush().unwrap();
    assert!(executor.poll_one());
    let token = token
        .borrow_mut()
        .take()
        .expect("first disabled observer must preserve validation");
    disabled.revise(|v| *v = false);
    assert!(!token.is_current());
    assert!(!token.complete(Ok(())));
}

#[test]
fn future_destructor_reentry_cannot_orphan_restarted_validation() {
    struct OnDrop(Rc<dyn Fn()>);
    impl Drop for OnDrop {
        fn drop(&mut self) {
            (self.0)();
        }
    }
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let form = FormState::new(app.scope());
    let slot = Rc::new(RefCell::new(None::<sapid::FormField<()>>));
    let holder = slot.clone();
    let reentered = Rc::new(Cell::new(false));
    let field = form
        .register(
            app.scope(),
            "reentry",
            app.scope().cage(()),
            FieldOptions::default().validate_async(move |_| {
                let holder = holder.clone();
                let reentered = reentered.clone();
                let guard = OnDrop(Rc::new(move || {
                    if !reentered.replace(true) {
                        holder.borrow().as_ref().unwrap().validate().unwrap();
                    }
                }));
                Box::pin(async move {
                    let _guard = guard;
                    std::future::pending().await
                })
            }),
        )
        .unwrap();
    *slot.borrow_mut() = Some(field.clone());
    app.flush().unwrap();
    field.validate().unwrap();
    assert_eq!(field.validate(), Err(FormError::Superseded));
    assert_eq!(executor.len(), 1);
    assert!(field.state().get().validating);
    field.cancel();
    assert!(executor.is_empty());
    slot.borrow_mut().take();
}

#[test]
fn disposing_form_and_fields_in_the_same_owner_never_reads_retired_status() {
    let executor = Rc::new(Executor::new(4));
    let app = App::with_host(executor.clone(), glory::render::memory::Memory::new);
    let child = app.scope().child().unwrap();
    let form = FormState::new(&child);
    let field = form
        .register(
            &child,
            "field",
            child.cage(0),
            FieldOptions::default().validate_async(|_| Box::pin(std::future::pending())),
        )
        .unwrap();
    app.flush().unwrap();
    field.validate().unwrap();
    child.dispose();
    assert!(executor.is_empty());
    assert_eq!(field.changed(), Err(FormError::Retired));
    assert_eq!(form.cancel(), Err(FormError::Retired));
}
