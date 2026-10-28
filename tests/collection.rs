use glory::render::{
    Direction, Event, FocusMode, KeyboardInput, Navigation, NodeId, Orientation,
};
use glory::ui::{App, Widget};
use sapid::{
    Collection, CollectionLimits, RovingFocus, Typeahead, TypeaheadConfig, radio, radio_group,
};

fn item(
    app: &App,
    collection: &Collection,
    key: &str,
    label: &str,
    enabled: glory::Vary<bool>,
) -> (glory::Scope, sapid::CollectionMember, NodeId) {
    let owner = app.scope().child().unwrap();
    let reference = owner.cage(None);
    glory::ui::button()
        .label(label)
        .node_ref(reference)
        .show_in(&owner)
        .unwrap();
    let member = collection
        .register(&owner, key, reference, enabled, label)
        .unwrap();
    (owner, member, reference.get().unwrap())
}

#[test]
fn collection_retains_only_mounted_owners_and_key_reuse_never_revives_old_tokens() {
    let app = App::memory();
    let collection = Collection::new(
        app.scope(),
        CollectionLimits {
            items: 2,
            label_bytes: 32,
        },
    )
    .unwrap();
    let (first, old, _) = item(&app, &collection, "first", "First", true.into());
    for _ in 0..100 {
        let (owner, token, _) = item(&app, &collection, "transient", "Temporary", true.into());
        assert_eq!(collection.registered_len().unwrap(), 2);
        let reference = owner.cage(None);
        assert!(
            collection
                .register(&owner, "third", reference, true, "Third")
                .is_err()
        );
        assert!(
            collection
                .register(&owner, "first", reference, true, "Duplicate")
                .is_err()
        );
        owner.dispose();
        assert_eq!(collection.registered_len().unwrap(), 1);
        assert!(
            collection
                .mounted()
                .unwrap()
                .iter()
                .all(|item| item.member != token)
        );
    }
    first.dispose();
    let (_, fresh, _) = item(&app, &collection, "first", "Reinserted", true.into());
    assert_ne!(old, fresh);
    let other = App::memory();
    assert!(
        collection
            .register(
                other.scope(),
                "foreign",
                other.scope().cage(None),
                true,
                "Foreign"
            )
            .is_err()
    );
    app.scope().dispose();
    assert!(collection.registered_len().is_err());
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn roving_repairs_retirement_and_disabling_without_claiming_actual_focus() {
    let app = App::memory();
    let collection = Collection::new(app.scope(), CollectionLimits::default()).unwrap();
    let enabled = app.scope().cage(true);
    let (first, one, from) = item(&app, &collection, "one", "One", true.into());
    let (_two_owner, _, _) = item(&app, &collection, "two", "Two", false.into());
    let (_three_owner, three, end) = item(&app, &collection, "three", "Three", enabled.into());
    let roving = RovingFocus::new(app.scope(), collection.clone(), false).unwrap();
    roving.repair(Some("one")).unwrap();
    assert!(roving.is_tab_stop(&one).unwrap());
    assert_eq!(
        roving
            .destination(from, Navigation::Next)
            .unwrap()
            .unwrap()
            .node,
        end
    );
    assert!(roving.destination(end, Navigation::Next).unwrap().is_none());
    let wrap = RovingFocus::new(app.scope(), collection.clone(), true).unwrap();
    assert_eq!(
        wrap.destination(end, Navigation::Next)
            .unwrap()
            .unwrap()
            .node,
        from
    );
    assert_eq!(
        roving
            .destination(end, Navigation::First)
            .unwrap()
            .unwrap()
            .node,
        from
    );
    roving.observe(&three).unwrap();
    enabled.revise(|value| *value = false);
    roving.repair(None).unwrap();
    assert_eq!(roving.current().unwrap(), Some(one));
    first.dispose();
    roving.repair(None).unwrap();
    assert_eq!(roving.current().unwrap(), None);
    let looping = RovingFocus::new(app.scope(), collection, true).unwrap();
    assert!(
        looping
            .destination(end, Navigation::Next)
            .unwrap()
            .is_none()
    );
    let input = KeyboardInput {
        key: "ArrowLeft",
        ..Default::default()
    };
    assert_eq!(
        Orientation::Horizontal.key_with_direction(Direction::Rtl, input),
        Some(Navigation::Next)
    );
    assert_eq!(
        Orientation::Vertical.key_with_direction(Direction::Rtl, input),
        None
    );
}

#[test]
fn typeahead_has_explicit_time_repeat_cycle_unicode_and_transactional_budgets() {
    let app = App::memory();
    let collection = Collection::new(
        app.scope(),
        CollectionLimits {
            items: 8,
            label_bytes: 16,
        },
    )
    .unwrap();
    let (_alpha_owner, _, alpha) = item(&app, &collection, "alpha", "Alpha", true.into());
    let (_apple_owner, _, apple) = item(&app, &collection, "apple", "Apple", true.into());
    let (_disabled_owner, _, _) = item(&app, &collection, "absent", "Albatross", false.into());
    let (_omega_owner, _, omega) = item(&app, &collection, "omega", "Ωmega", true.into());
    let mut search = Typeahead::new(TypeaheadConfig {
        timeout_millis: 20,
        query_bytes: 4,
    })
    .unwrap();
    assert_eq!(
        search
            .search(&collection, alpha, "a", 10)
            .unwrap()
            .unwrap()
            .node,
        apple
    );
    assert_eq!(
        search
            .search(&collection, apple, "A", 11)
            .unwrap()
            .unwrap()
            .node,
        alpha
    );
    assert_eq!(
        search
            .search(&collection, alpha, "p", 12)
            .unwrap()
            .unwrap()
            .node,
        apple
    );
    assert_eq!(search.query(), "ap");
    assert!(search.search(&collection, apple, "x", 9).is_err());
    assert_eq!(search.query(), "ap");
    for text in ["ab", " ", "\n", ""] {
        assert!(search.search(&collection, apple, text, 13).is_err());
    }
    assert!(search.search(&collection, apple, "x", u64::MAX).is_err());
    assert_eq!(
        search
            .search(&collection, apple, "Ω", 40)
            .unwrap()
            .unwrap()
            .node,
        omega
    );
    search.search(&collection, omega, "a", 41).unwrap();
    assert!(search.search(&collection, omega, "é", 42).is_err());
    assert_eq!(search.query(), "ωa");
    let label = app.scope().cage("Bounded".to_string());
    let owner = app.scope().child().unwrap();
    let reference = owner.cage(Some(alpha));
    collection
        .register(&owner, "growing", reference, true, label)
        .unwrap();
    label.revise(|label| *label = "x".repeat(17));
    assert!(search.search(&collection, omega, "z", 80).is_err());
    assert_eq!(search.query(), "ωa");
    search.clear();
    assert_eq!(search.query(), "");
    assert!(
        Typeahead::new(TypeaheadConfig {
            timeout_millis: 0,
            query_bytes: 1
        })
        .is_err()
    );
}

#[test]
fn radio_uses_shared_roving_and_typeahead_and_preserves_rejected_selection() {
    let app = App::memory();
    let value = app.scope().cage("alpha".to_string());
    let refs = (0..4).map(|_| app.scope().cage(None)).collect::<Vec<_>>();
    app.mount(
        radio_group("search")
            .value(value)
            .orientation(Orientation::Horizontal)
            .direction(Direction::Rtl)
            .typeahead(TypeaheadConfig::default())
            .items([
                radio("alpha").label("Alpha").node_ref(refs[0]),
                radio("apple").label("Apple").node_ref(refs[1]),
                radio("absent")
                    .label("Apricot")
                    .enabled(false)
                    .node_ref(refs[2]),
                radio("beta").label("Beta").node_ref(refs[3]),
            ]),
    )
    .unwrap();
    let alpha = refs[0].get().unwrap();
    let apple = refs[1].get().unwrap();
    let disabled = refs[2].get().unwrap();
    app.dispatch(
        disabled,
        Event::Typeahead {
            from: disabled,
            text: "b".into(),
            at_millis: 0,
        },
    )
    .unwrap();
    assert_eq!(value.get(), "alpha");
    assert_eq!(
        app.tree().node(alpha).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    app.dispatch(
        alpha,
        Event::Typeahead {
            from: alpha,
            text: "a".into(),
            at_millis: 1,
        },
    )
    .unwrap();
    assert_eq!(value.get(), "alpha");
    assert_eq!(
        app.tree().node(apple).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(
        matches!(&app.tree().node(alpha).unwrap().descriptor.view().props.clone(), glory::render::Props::Check(check) if check.checked)
    );
    app.dispatch(
        apple,
        Event::Typeahead {
            from: apple,
            text: "b".into(),
            at_millis: 1000,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(refs[3].get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .focus,
        FocusMode::TabStop
    );
    let group = app.tree().roots()[0];
    assert!(
        app.tree()
            .node(group)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .typeahead
    );
}
