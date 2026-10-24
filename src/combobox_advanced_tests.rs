#[test]
fn autocomplete_inserts_business_text_accepts_free_values_and_has_no_proxy() {
    let app = App::memory();
    let commits = Rc::new(std::cell::RefCell::new(Vec::new()));
    let out = commits.clone();
    app.mount(
        combobox("auto")
            .autocomplete()
            .label("Complete")
            .option(crate::select_option("a", "Alpha label").value("inserted text"))
            .form_name("free")
            .required(true)
            .default_open(true)
            .on_search_commit(move |value| out.borrow_mut().push(value)),
    )
    .unwrap();
    app.dispatch(option(&app, "Alpha label"), Event::Activate)
        .unwrap();
    assert_eq!(input(&app).value, "inserted text");
    assert!(selected(&app, "Alpha label"));
    edit(&app, "Unlisted free text", false);
    assert!(!selected(&app, "Alpha label"));
    let editor = editor(&app);
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::Commit,
        },
    )
    .unwrap();
    assert_eq!(*commits.borrow(), ["Unlisted free text"]);
    assert_eq!(input(&app).value, "Unlisted free text");
    assert!(!nodes(&app).iter().any(|id| matches!(
        app.tree().node(*id).unwrap().descriptor.view().props,
        Props::Select(_)
    )));
    let tree = app.tree();
    let meta = tree
        .node(editor)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    assert_eq!(meta.form_control.as_ref().unwrap().name, "free");
    assert!(meta.required);
    assert_eq!(meta.combobox, Some(ComboboxKind::Autocomplete));
}
#[test]
fn autocomplete_rejected_proposal_retains_source_and_composition_freezes_filter() {
    let app = App::memory();
    let value = app.scope().cage("Alpha".to_owned());
    app.mount(choices().autocomplete().search(value).default_open(true))
        .unwrap();
    app.dispatch(option(&app, "Bravo"), Event::Activate)
        .unwrap();
    assert_eq!(input(&app).value, "Alpha");
    app.dispatch(editor(&app), Event::Composition { active: true })
        .unwrap();
    let before = input(&app);
    edit(&app, "z", true);
    assert_eq!(input(&app), before);
    app.dispatch(editor(&app), Event::Composition { active: false })
        .unwrap();
    assert_eq!(input(&app).value, "Alpha");
}
#[test]
fn autocomplete_required_is_on_real_successful_input_and_insertion_text_is_single_line() {
    let html = glory::ssr::snapshot(
        crate::field("auto-field")
            .label("Free")
            .form_name("free")
            .required(true)
            .control(|field| {
                crate::autocomplete("auto-field")
                    .field(field)
                    .default_value("custom")
                    .option(crate::autocomplete_option("a", "Alpha"))
            }),
    )
    .unwrap()
    .html;
    assert!(
        html.contains("<input")
            && html.contains("name=\"free\"")
            && html.contains("value=\"custom\"")
    );
    assert_eq!(html.matches(" required=\"\"").count(), 1);
    assert!(!html.contains("<select"));
    let app = App::memory();
    assert!(
        app.mount(
            choices()
                .autocomplete()
                .option(crate::select_option("newline", "Line").value("a\nb"))
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    assert!(
        app.mount(
            crate::autocomplete("huge")
                .option(crate::autocomplete_option("a", "x".repeat(1_000_000)))
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn multiple_keeps_search_separate_toggles_options_and_removes_last_selected_key() {
    let app = App::memory();
    app.mount(
        choices()
            .multiple()
            .default_values(vec!["a".into()])
            .default_open(true),
    )
    .unwrap();
    assert_eq!(input(&app).value, "");
    assert!(selected(&app, "Alpha"));
    app.dispatch(option(&app, "Bravo"), Event::Activate)
        .unwrap();
    assert!(selected(&app, "Alpha") && selected(&app, "Bravo"));
    let editor = editor(&app);
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::RemoveLast,
        },
    )
    .unwrap();
    assert!(selected(&app, "Alpha") && !selected(&app, "Bravo"));
    app.dispatch(option(&app, "Alpha"), Event::Activate)
        .unwrap();
    assert!(!selected(&app, "Alpha"));
}
#[test]
fn multiple_controlled_rejections_and_same_tick_search_recheck_do_not_delete_tags() {
    use glory::render::Input;
    let (app, queue) = queued();
    let values = app.scope().cage(vec!["a".to_owned(), "b".to_owned()]);
    let search = app.scope().cage("Br".to_owned());
    let proposals = Rc::new(std::cell::RefCell::new(Vec::new()));
    let out = proposals.clone();
    app.mount(
        choices()
            .multiple()
            .values(values)
            .search(search)
            .default_open(true)
            .on_values_change(move |value| out.borrow_mut().push(value)),
    )
    .unwrap();
    let editor = editor(&app);
    queue.borrow_mut().extend([
        Input::Event {
            node: editor,
            event: Event::ValueChanged {
                value: "".into(),
                composing: false,
                revision: input(&app).revision,
            },
        },
        Input::Event {
            node: editor,
            event: Event::Choice {
                from: editor,
                command: ChoiceCommand::RemoveLast,
            },
        },
    ]);
    app.pump().unwrap();
    assert!(proposals.borrow().is_empty());
    assert_eq!(values.get(), ["a", "b"]);
    assert_eq!(input(&app).value, "Br");
    search.revise(String::clear);
    app.flush().unwrap();
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::RemoveLast,
        },
    )
    .unwrap();
    assert_eq!(*proposals.borrow(), [vec!["a".to_owned()]]);
    assert!(selected(&app, "Bravo"));
}
#[test]
fn multiple_composition_readonly_disabled_and_retired_callbacks_do_not_mutate() {
    let app = App::memory();
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    app.mount(
        choices()
            .multiple()
            .default_values(vec!["a".into()])
            .readonly(readonly)
            .enabled(enabled),
    )
    .unwrap();
    let editor = editor(&app);
    app.dispatch(editor, Event::Composition { active: true })
        .unwrap();
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::RemoveLast,
        },
    )
    .unwrap();
    assert!(selected(&app, "Alpha"));
    app.dispatch(editor, Event::Composition { active: false })
        .unwrap();
    readonly.revise(|value| *value = true);
    app.flush().unwrap();
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::RemoveLast,
        },
    )
    .unwrap();
    assert!(selected(&app, "Alpha"));
    readonly.revise(|value| *value = false);
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::RemoveLast,
        },
    )
    .unwrap();
    assert!(selected(&app, "Alpha"));
    app.clear().unwrap();
    app.dispatch(
        editor,
        Event::Choice {
            from: editor,
            command: ChoiceCommand::RemoveLast,
        },
    )
    .unwrap();
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn multiple_successful_proxy_is_real_multiple_and_search_is_not_natively_required() {
    let html = glory::ssr::snapshot(
        crate::field("many-field")
            .label("Many")
            .form_name("items")
            .required(true)
            .control(|field| {
                crate::multi_select("many-field")
                    .field(field)
                    .default_value(vec!["a".into(), "b".into()])
                    .options([
                        crate::select_option("a", "Alpha").value("business-a"),
                        crate::select_option("b", "Bravo").value("business-b"),
                    ])
            }),
    )
    .unwrap()
    .html;
    assert!(html.contains("aria-multiselectable=\"true\""));
    assert!(html.contains("multiple=\"\"") && html.contains("name=\"items\""));
    assert!(
        html.contains("value=\"business-a\" selected")
            && html.contains("value=\"business-b\" selected")
    );
    assert!(!html.contains(" required=\"\""));
    assert!(html.contains("aria-required=\"true\""));
    assert_eq!(html.matches("sapid-multi-select-tag").count(), 2);
}
#[test]
fn multiple_selected_budget_validates_before_cloning_and_live_tags_remain_below_4096() {
    let app = App::memory();
    assert!(
        app.mount(
            choices()
                .multiple()
                .default_values(vec!["a".into(); 1_000_000])
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    let values = app
        .scope()
        .cage((0..256).map(|i| format!("k{i}")).collect::<Vec<_>>());
    app.mount(combobox("max-many").multiple().values(values).options(
        (0..1024).map(|i| crate::select_option(format!("k{i}"), format!("Option {i}"))),
    ))
    .unwrap();
    assert!(
        app.stats().nodes < 2200,
        "bounded declarations + tags: {}",
        app.stats().nodes
    );
    values.revise(|values| *values = vec!["k0".into(); 1_000_000]);
    assert!(app.flush().is_err());
    assert!(app.stats().nodes < 2200);
    values.revise(|values| *values = vec!["k0".into()]);
    app.flush().unwrap();
    assert!(app.stats().nodes < 1100);
}
#[test]
fn form_names_reject_oversized_initial_and_live_sources_without_changing_metadata() {
    let app = App::memory();
    assert!(
        app.mount(choices().form_name("n".repeat(1_000_000)))
            .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    let name = app.scope().cage("valid".to_owned());
    app.mount(choices().autocomplete().form_name(name)).unwrap();
    name.revise(|name| *name = "n".repeat(1_000_000));
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(editor(&app))
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .name,
        "valid"
    );
    name.revise(|name| *name = "recovered".into());
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(editor(&app))
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .name,
        "recovered"
    );
}

#[test]
fn listbox_name_uses_field_label_or_explicit_name_instead_of_editor_value() {
    for explicit in [false, true] {
        let app = App::memory();
        app.mount(
            crate::field("labelled-list")
                .label("Countries")
                .control(move |field| {
                    let widget = choices().multiple().field(field).default_open(true);
                    if explicit {
                        widget.name("Explicit choices")
                    } else {
                        widget
                    }
                }),
        )
        .unwrap();
        edit(&app, "Br", false);
        let tree = app.tree();
        let input = tree.node(editor(&app)).unwrap().descriptor.view();
        let panel = nodes(&app)
            .into_iter()
            .find(|id| tree.node(*id).unwrap().descriptor.view().role == Some(Role::Listbox))
            .unwrap();
        let panel = tree.node(panel).unwrap().descriptor.view();
        if explicit {
            assert_eq!(panel.name.as_deref(), Some("Explicit choices"));
            assert!(panel.semantics.as_ref().unwrap().labelled_by.is_none());
        } else {
            assert_eq!(
                panel.semantics.as_ref().unwrap().labelled_by,
                input.semantics.as_ref().unwrap().labelled_by
            );
            assert_ne!(
                panel.semantics.as_ref().unwrap().labelled_by,
                input.semantic_id().cloned()
            );
        }
    }
}

#[test]
fn multiple_callback_capture_is_destroyed_outside_graph_borrow_on_owner_retirement() {
    struct Capture(Cage<usize>, Rc<std::cell::Cell<bool>>);
    impl Drop for Capture {
        fn drop(&mut self) {
            self.1.set(self.0.try_get().is_ok());
        }
    }
    let app = App::memory();
    let released = Rc::new(std::cell::Cell::new(false));
    let capture = Capture(app.scope().cage(7), released.clone());
    let mounted = choices()
        .multiple()
        .default_values(vec!["a".into()])
        .on_values_change(move |_| {
            let _ = &capture;
        })
        .mount(app.scope())
        .unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert!(released.get());
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn multiple_select_all_uses_current_filter_skips_disabled_and_clear_preserves_search() {
    let app = App::memory();
    let slot = Rc::new(std::cell::RefCell::new(None));
    let ready = slot.clone();
    let search = app.scope().cage("Br".to_owned());
    app.mount(
        choices()
            .multiple()
            .search(search)
            .default_values(vec!["a".into()])
            .on_multiple_ready(move |handle| *ready.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = slot.borrow().clone().unwrap();
    handle.select_all().unwrap();
    app.flush().unwrap();
    assert!(selected(&app, "Alpha") && selected(&app, "Bravo") && selected(&app, "Branch"));
    assert!(!selected(&app, "Blocked"));
    handle.clear_all().unwrap();
    app.flush().unwrap();
    assert!(!selected(&app, "Alpha") && !selected(&app, "Bravo") && !selected(&app, "Branch"));
    assert_eq!(input(&app).value, "Br");
    search.revise(String::clear);
    app.flush().unwrap();
    handle.select_all().unwrap();
    app.flush().unwrap();
    assert!(selected(&app, "Alpha") && selected(&app, "Bravo") && selected(&app, "Branch"));
    assert!(!selected(&app, "Blocked"));
}

#[test]
fn multiple_select_all_is_a_controlled_proposal_and_commands_reject_inactive_policies() {
    let app = App::memory();
    let slot = Rc::new(std::cell::RefCell::new(None));
    let ready = slot.clone();
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let values = app.scope().cage(vec!["a".to_owned()]);
    let proposed = Rc::new(std::cell::RefCell::new(Vec::new()));
    let out = proposed.clone();
    let mounted = choices()
        .multiple()
        .values(values)
        .readonly(readonly)
        .enabled(enabled)
        .on_multiple_ready(move |handle| *ready.borrow_mut() = Some(handle))
        .on_values_change(move |value| out.borrow_mut().push(value))
        .mount(app.scope())
        .unwrap();
    app.flush().unwrap();
    let handle = slot.borrow().clone().unwrap();
    handle.select_all().unwrap();
    handle.clear_all().unwrap();
    app.flush().unwrap();
    assert_eq!(
        *proposed.borrow(),
        [vec!["a".to_owned(), "b".into(), "c".into()], vec![]]
    );
    assert_eq!(values.get(), ["a"]);
    assert!(selected(&app, "Alpha"));
    readonly.revise(|value| *value = true);
    assert!(handle.select_all().is_err());
    assert!(handle.clear_all().is_err());
    readonly.revise(|value| *value = false);
    enabled.revise(|value| *value = false);
    assert!(handle.select_all().is_err());
    enabled.revise(|value| *value = true);
    app.flush().unwrap();
    app.dispatch(editor(&app), Event::Composition { active: true })
        .unwrap();
    assert!(handle.select_all().is_err());
    app.dispatch(editor(&app), Event::Composition { active: false })
        .unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert!(handle.select_all().is_err());
    assert!(handle.clear_all().is_err());
}

#[test]
fn multiple_select_all_over_budget_rejects_the_whole_proposal_before_any_callback() {
    let app = App::memory();
    let slot = Rc::new(std::cell::RefCell::new(None));
    let ready = slot.clone();
    let calls = app.scope().cage(0);
    app.mount(
        combobox("all-budget")
            .multiple()
            .default_values(vec!["k0".into()])
            .options(
                (0..257).map(|i| crate::select_option(format!("k{i}"), format!("Option {i}"))),
            )
            .on_multiple_ready(move |handle| *ready.borrow_mut() = Some(handle))
            .on_values_change(move |_| calls.revise(|count| *count += 1)),
    )
    .unwrap();
    assert!(slot.borrow().as_ref().unwrap().select_all().is_err());
    app.flush().unwrap();
    assert_eq!(calls.get(), 0);
    assert!(selected(&app, "Option 0"));
    assert!(!selected(&app, "Option 1"));
}
