use std::cell::{Cell, RefCell};
use std::rc::Rc;

use glory::render::memory::Memory;
use glory::render::{
    Capabilities, Event, EventKind, Input, InputProps, Mutation, NodeId, Patch, Props, Query,
    QueryToken, Receipt, Render,
};
use glory::style::{Edges, Length, Style};
use glory::ui::{App, Widget};
use sapid::{ButtonVariant, Size, Theme, button, checkbox, text_field};

fn node(app: &App) -> NodeId {
    app.tree().roots()[0]
}

fn input(app: &App, id: NodeId) -> InputProps {
    let Props::Input(props) = app.tree().node(id).unwrap().descriptor.view().props.clone()
    else {
        panic!("expected input")
    };
    props
}

fn edit(app: &App, id: NodeId, value: &str) -> bool {
    app.dispatch(
        id,
        Event::ValueChanged {
            value: value.into(),
            composing: false,
            revision: input(app, id).revision,
        },
    )
    .unwrap()
}

#[test]
fn button_combines_observers_and_preserves_label_children() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let first = calls.clone();
    let second = calls.clone();
    let third = calls.clone();
    app.mount(
        button()
            .label("Save")
            .fill(glory::ui::text().content("icon"))
            .on_press(move || first.borrow_mut().push(1))
            .on(EventKind::Activate, move |_| second.borrow_mut().push(2))
            .on_press(move || third.borrow_mut().push(3)),
    )
    .unwrap();
    let id = node(&app);
    assert_eq!(app.tree().len(), 2);
    assert!(app.dispatch(id, Event::Activate).unwrap());
    // Typed press callbacks precede generic observers, independent of setter order.
    assert_eq!(*calls.borrow(), [1, 3, 2]);
}

#[test]
fn loading_and_enabled_both_gate_native_activation() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let loading = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        button()
            .enabled(enabled)
            .loading(loading)
            .on_press(move || seen.set(seen.get() + 1)),
    )
    .unwrap();
    let id = node(&app);
    assert!(app.dispatch(id, Event::Activate).unwrap());
    loading.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(!app.dispatch(id, Event::Activate).unwrap());
    enabled.revise(|v| *v = false);
    loading.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.dispatch(id, Event::Activate).unwrap());
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(app.dispatch(id, Event::Activate).unwrap());
    assert_eq!(calls.get(), 2);
    assert_eq!(node(&app), id);
}

#[test]
fn user_classes_and_dynamic_style_overrides_preserve_component_defaults() {
    let app = App::memory();
    let class = app.scope().cage("custom".to_string());
    let style = app
        .scope()
        .cage(Style::new().padding(Edges::all(2.0)).radius(0.0));
    app.mount(
        button()
            .variant(ButtonVariant::Danger)
            .size(Size::Small)
            .class(class)
            .style(style),
    )
    .unwrap();
    let id = node(&app);
    let descriptor = app.tree().node(id).unwrap().descriptor.view().to_owned();
    assert!(descriptor.classes.iter().any(|v| v == "sapid-button"));
    assert!(descriptor.classes.iter().any(|v| v == "custom"));
    assert_eq!(descriptor.style.background, Some(Theme::light().danger));
    assert_eq!(descriptor.style.radius, Some(0.0));
    assert_eq!(descriptor.style.padding, Some(Edges::all(2.0)));
    class.revise(|v| *v = "updated".into());
    style.revise(|v| *v = Style::new().width(Length::Px(180.0)));
    app.flush().unwrap();
    let descriptor = app.tree().node(id).unwrap().descriptor.view().to_owned();
    assert!(!descriptor.classes.iter().any(|v| v == "custom"));
    assert!(descriptor.classes.iter().any(|v| v == "updated"));
    assert_eq!(descriptor.style.width, Some(Length::Px(180.0)));
    assert_eq!(descriptor.style.radius, Some(Theme::light().radius));
    assert_eq!(descriptor.style.background, Some(Theme::light().danger));
}

#[test]
fn unstyled_keeps_semantics_and_explicit_styles() {
    let app = App::memory();
    app.mount(
        button()
            .unstyled()
            .label("Save")
            .style(Style::new().opacity(0.7)),
    )
    .unwrap();
    let descriptor = app
        .tree()
        .node(node(&app))
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert_eq!(descriptor.style, Style::new().opacity(0.7));
    assert!(matches!(descriptor.props, Props::Button(_)));
    assert!(descriptor.classes.iter().any(|v| v == "sapid-button"));
}

#[test]
fn uncontrolled_text_accepts_edits_and_commit_without_duplicate_change() {
    let app = App::memory();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let commits = Rc::new(RefCell::new(Vec::new()));
    let changed = changes.clone();
    let committed = commits.clone();
    app.mount(
        text_field()
            .name("Name")
            .default_value("initial")
            .on_value_change(move |v| changed.borrow_mut().push(v))
            .on_commit(move |v| committed.borrow_mut().push(v)),
    )
    .unwrap();
    let id = node(&app);
    assert_eq!(input(&app, id).value, "initial");
    assert!(edit(&app, id, "edited"));
    app.dispatch(
        id,
        Event::Commit {
            value: "final".into(),
            revision: input(&app, id).revision,
        },
    )
    .unwrap();
    assert_eq!(input(&app, id).value, "final");
    assert_eq!(*changes.borrow(), ["edited"]);
    assert_eq!(*commits.borrow(), ["final"]);
}

#[test]
fn controlled_text_can_accept_transform_or_reject_and_external_updates_are_silent() {
    let app = App::memory();
    let value = app.scope().cage("initial".to_string());
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let seen = proposals.clone();
    app.mount(
        text_field()
            .value(value)
            .default_value("ignored")
            .on_value_change(move |v| {
                seen.borrow_mut().push(v.clone());
                if v != "reject" {
                    value.revise(|current| *current = v.to_uppercase());
                }
            }),
    )
    .unwrap();
    let id = node(&app);
    edit(&app, id, "accepted");
    assert_eq!(value.get(), "ACCEPTED");
    assert_eq!(input(&app, id).value, "ACCEPTED");
    edit(&app, id, "reject");
    assert_eq!(input(&app, id).value, "ACCEPTED");
    value.revise(|v| *v = "program".into());
    app.flush().unwrap();
    assert_eq!(input(&app, id).value, "program");
    assert_eq!(*proposals.borrow(), ["accepted", "reject"]);
    assert_eq!(node(&app), id);
}

#[test]
fn static_controlled_values_remain_read_only() {
    let app = App::memory();
    app.mount(text_field().value("fixed")).unwrap();
    let id = node(&app);
    edit(&app, id, "ignored");
    assert_eq!(input(&app, id).value, "fixed");
}

#[test]
fn old_and_future_text_versions_never_reach_callbacks() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(text_field().on_value_change(move |_| seen.set(seen.get() + 1)))
        .unwrap();
    let id = node(&app);
    let old = input(&app, id).revision;
    edit(&app, id, "accepted");
    assert!(
        !app.dispatch(
            id,
            Event::ValueChanged {
                value: "old".into(),
                composing: false,
                revision: old,
            }
        )
        .unwrap()
    );
    assert!(
        app.dispatch(
            id,
            Event::ValueChanged {
                value: "future".into(),
                composing: false,
                revision: u64::MAX,
            }
        )
        .is_err()
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(input(&app, id).value, "accepted");
}

#[test]
fn composition_and_generic_observers_do_not_replace_change_behavior() {
    let app = App::memory();
    let log = Rc::new(RefCell::new(Vec::new()));
    let changed = log.clone();
    let observed = log.clone();
    let composition = log.clone();
    app.mount(
        text_field()
            .on_value_change(move |_| changed.borrow_mut().push("change"))
            .on(EventKind::ValueChanged, move |_| {
                observed.borrow_mut().push("observe")
            })
            .on(EventKind::Composition, move |_| {
                composition.borrow_mut().push("composition")
            }),
    )
    .unwrap();
    let id = node(&app);
    app.dispatch(id, Event::Composition { active: true })
        .unwrap();
    app.dispatch(
        id,
        Event::ValueChanged {
            value: "中".into(),
            composing: true,
            revision: input(&app, id).revision,
        },
    )
    .unwrap();
    app.dispatch(id, Event::Composition { active: false })
        .unwrap();
    assert_eq!(input(&app, id).value, "中");
    assert_eq!(
        *log.borrow(),
        ["composition", "change", "observe", "composition"]
    );
}

#[test]
fn checkbox_supports_local_and_rejected_controlled_toggles() {
    let app = App::memory();
    app.mount(checkbox().label("Local").default_checked(true))
        .unwrap();
    let id = node(&app);
    app.dispatch(id, Event::CheckedChanged(false)).unwrap();
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props.clone(), Props::Check(ref p) if !p.checked)
    );
    app.clear().unwrap();
    let source = app.scope().cage(false);
    let log = Rc::new(RefCell::new(Vec::new()));
    let typed = log.clone();
    let raw = log.clone();
    app.mount(
        checkbox()
            .checked(source)
            .default_checked(true)
            .on_checked_change(move |v| typed.borrow_mut().push(("typed", v)))
            .on(EventKind::CheckedChanged, move |event| {
                if let Event::CheckedChanged(v) = event {
                    raw.borrow_mut().push(("raw", v));
                }
            }),
    )
    .unwrap();
    let id = node(&app);
    let before = app.tree().stats().patched;
    app.dispatch(id, Event::CheckedChanged(true)).unwrap();
    assert!(!source.get());
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props.clone(), Props::Check(ref p) if !p.checked)
    );
    assert_eq!(
        app.tree().stats().patched,
        before + 1,
        "rejected host toggle must be restored"
    );
    source.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props.clone(), Props::Check(ref p) if p.checked)
    );
    assert_eq!(*log.borrow(), [("typed", true), ("raw", true)]);
}

#[test]
fn accepted_controlled_checkbox_updates_the_source_once() {
    let app = App::memory();
    let source = app.scope().cage(false);
    app.mount(
        checkbox()
            .checked(source)
            .on_checked_change(move |v| source.revise(|current| *current = v)),
    )
    .unwrap();
    let id = node(&app);
    app.dispatch(id, Event::CheckedChanged(true)).unwrap();
    assert!(source.get());
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props.clone(), Props::Check(ref p) if p.checked)
    );
}

#[test]
fn disabled_text_and_checkbox_reject_user_changes() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        text_field()
            .enabled(false)
            .default_value("fixed")
            .on_value_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    assert!(!edit(&app, node(&app), "ignored"));
    assert_eq!(calls.get(), 0);
    assert_eq!(input(&app, node(&app)).value, "fixed");
    app.clear().unwrap();
    let seen = calls.clone();
    app.mount(
        checkbox()
            .enabled(false)
            .on_checked_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    assert!(
        !app.dispatch(node(&app), Event::CheckedChanged(true))
            .unwrap()
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn multiline_canonicalization_and_purpose_validation_are_inherited() {
    let app = App::memory();
    app.mount(text_field().multiline(true).default_value("a\r\nb\rc\0"))
        .unwrap();
    assert_eq!(input(&app, node(&app)).value, "a\nb\nc\u{fffd}");
    app.clear().unwrap();
    assert!(app.mount(text_field().default_value("a\nb")).is_err());
    assert_eq!(app.stats().nodes, 0);
    assert!(
        app.mount(
            text_field()
                .multiline(true)
                .purpose(glory::ui::Purpose::Password)
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn callbacks_stop_after_retirement_and_component_resources_are_released() {
    let app = App::memory();
    let baseline = app.runtime_stats();
    let reference = app.scope().cage(None);
    let seen = Rc::new(Cell::new(0));
    for _ in 0..100 {
        let capture = seen.clone();
        app.mount(
            text_field()
                .node_ref(reference)
                .on_value_change(move |_| capture.set(capture.get() + 1)),
        )
        .unwrap();
        let retired = reference.get().unwrap();
        edit(&app, retired, "edit");
        app.clear().unwrap();
        assert_eq!(reference.get(), None);
        assert!(
            !app.dispatch(retired, Event::Composition { active: false })
                .unwrap()
        );
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().handlers, 0);
        assert_eq!(app.runtime_stats().nodes, baseline.nodes + 1);
        assert_eq!(app.runtime_stats().owners, baseline.owners);
        assert_eq!(app.runtime_stats().edges, baseline.edges);
        assert_eq!(Rc::strong_count(&seen), 1);
    }
    assert_eq!(seen.get(), 100);
    assert!(app.runtime_stats().node_slots < 20);
}

#[test]
fn observers_skip_callbacks_after_an_earlier_callback_disposes_the_owner() {
    let app = App::memory();
    let component = app.scope().child().unwrap();
    let cancel = component.clone();
    button()
        .on_press(move || cancel.dispose())
        .on_press(|| panic!("observer after retirement"))
        .show_in(&component)
        .unwrap();
    app.flush().unwrap();
    app.dispatch(node(&app), Event::Activate).unwrap();
    assert_eq!(app.stats().nodes, 0);
}

struct Queued {
    memory: Memory,
    queue: Rc<RefCell<Vec<Input>>>,
}

impl Render for Queued {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn apply(&mut self, mutations: &[Mutation]) -> Result<Receipt, glory::render::Error> {
        self.memory.apply(mutations)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), glory::render::Error> {
        self.memory.request(query, token)
    }
    fn drain(&mut self, inputs: &mut Vec<Input>) {
        inputs.extend(self.queue.borrow_mut().drain(..));
    }
}

#[test]
fn one_host_tick_delivers_all_edits_before_allocating_a_new_revision() {
    let queue = Rc::new(RefCell::new(Vec::new()));
    let queued = queue.clone();
    let app = App::new(move |runtime| Queued {
        memory: Memory::new(runtime),
        queue: queued,
    });
    let values = Rc::new(RefCell::new(Vec::new()));
    let changes = values.clone();
    app.mount(text_field().on_value_change(move |v| changes.borrow_mut().push(v)))
        .unwrap();
    let id = node(&app);
    let revision = input(&app, id).revision;
    for value in ["a", "ab", "abc"] {
        queue.borrow_mut().push(Input::Event {
            node: id,
            event: Event::ValueChanged {
                value: value.into(),
                composing: false,
                revision,
            },
        });
    }
    assert_eq!(app.pump().unwrap(), 3);
    assert_eq!(*values.borrow(), ["a", "ab", "abc"]);
    assert_eq!(input(&app, id).value, "abc");
    assert_eq!(input(&app, id).revision, revision + 1);
}

#[test]
fn explicit_glory_writes_and_widget_reconciliation_share_the_live_revision() {
    let app = App::memory();
    app.mount(text_field().default_value("initial")).unwrap();
    let id = node(&app);
    app.scope()
        .patch(
            id,
            Patch::Value {
                value: glory::render::Change::Set("explicit".into()),
                revision: 200,
            },
        )
        .unwrap();
    app.flush().unwrap();
    edit(&app, id, "next");
    assert_eq!(input(&app, id).revision, 201);
    assert_eq!(input(&app, id).value, "next");
}

#[test]
fn invalid_dynamic_styles_and_values_surface_as_typed_flush_errors() {
    let app = App::memory();
    let source = app.scope().cage("valid".to_string());
    app.mount(text_field().value(source)).unwrap();
    source.revise(|v| *v = "invalid\nline".into());
    assert!(app.flush().is_err());
    assert_eq!(input(&app, node(&app)).value, "valid");
    source.revise(|v| *v = "recovered".into());
    app.flush().unwrap();
    assert_eq!(input(&app, node(&app)).value, "recovered");
    app.clear().unwrap();
    let style = app.scope().cage(Style::new());
    app.mount(button().style(style)).unwrap();
    style.revise(|v| v.opacity = Some(f64::NAN));
    assert!(app.flush().is_err());
    style.revise(|v| v.opacity = Some(0.5));
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(node(&app))
            .unwrap()
            .descriptor
            .view()
            .style
            .opacity,
        Some(0.5)
    );
}

#[test]
fn dark_theme_does_not_change_control_semantics() {
    let app = App::memory();
    app.mount(
        text_field()
            .theme(Theme::dark())
            .name("Email")
            .purpose(glory::ui::Purpose::Email),
    )
    .unwrap();
    let descriptor = app
        .tree()
        .node(node(&app))
        .unwrap()
        .descriptor
        .view()
        .to_owned();
    assert_eq!(descriptor.style.background, Some(Theme::dark().input));
    assert_eq!(descriptor.name.as_deref(), Some("Email"));
    assert_eq!(input(&app, node(&app)).purpose, glory::ui::Purpose::Email);
}
