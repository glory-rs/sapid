use glory::prelude::*;
use glory::render::{Direction, Event, EventKind, FocusMode, Navigation, NodeId, Props, Role};
use sapid::{button, button_group};
use std::{cell::Cell, rc::Rc};
fn children(app: &App) -> Vec<NodeId> {
    app.tree()
        .node(app.tree().roots()[0])
        .unwrap()
        .children
        .to_vec()
}
#[test]
fn toolbar_has_one_roving_entry_and_never_activates_on_navigation() {
    let app = App::memory();
    let loading = app.scope().cage(false);
    let disabled = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let out = calls.clone();
    app.mount(
        button_group()
            .toolbar(true)
            .name("Editing")
            .direction(Direction::Rtl)
            .buttons([
                button()
                    .label("First")
                    .on_press(move || out.set(out.get() + 1)),
                button().label("Busy").loading(loading),
                button().label("Disabled").enabled(false),
                button()
                    .label("Last")
                    .enabled(app.scope().bond(move || !disabled.get())),
            ]),
    )
    .unwrap();
    let ids = children(&app);
    let root = app.tree().roots()[0];
    assert_eq!(
        app.tree().node(root).unwrap().descriptor.view().role,
        Some(Role::Toolbar)
    );
    assert_eq!(
        ids.iter()
            .filter(|id| app.tree().node(**id).unwrap().descriptor.view().focus
                == FocusMode::TabStop)
            .count(),
        1
    );
    for (from, command, to) in [
        (ids[0], Navigation::Next, ids[1]),
        (ids[1], Navigation::Next, ids[3]),
        (ids[3], Navigation::Next, ids[0]),
        (ids[0], Navigation::Last, ids[3]),
    ] {
        app.dispatch(from, Event::Navigate { from, command })
            .unwrap();
        assert_eq!(
            app.tree().node(to).unwrap().descriptor.view().focus,
            FocusMode::TabStop
        );
        assert_eq!(calls.get(), 0);
    }
    disabled.revise(|value| *value = true);
    loading.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(
        app.tree().node(ids[0]).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(
        matches!(&app.tree().node(ids[1]).unwrap().descriptor.view().props.clone(),Props::Button(p) if !p.enabled)
    );
    app.dispatch(ids[0], Event::Activate).unwrap();
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    loading.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(app.tree().is_empty());
}
#[test]
fn actual_focus_runs_before_generic_observers_and_plain_groups_keep_natural_tab_order() {
    let app = App::memory();
    let node = app.scope().cage(None);
    let seen = Rc::new(Cell::new(false));
    let out = seen.clone();
    app.mount(
        button_group().toolbar(true).buttons([
            button().label("First"),
            button()
                .label("Second")
                .node_ref(node)
                .on(EventKind::FocusChanged, move |_| {
                    out.set(node.get().is_some())
                }),
        ]),
    )
    .unwrap();
    let ids = children(&app);
    app.dispatch(ids[1], Event::FocusChanged(true)).unwrap();
    assert!(seen.get());
    assert_eq!(
        app.tree().node(ids[1]).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    app.clear().unwrap();
    app.mount(button_group().buttons([button().label("One"), button().label("Two")]))
        .unwrap();
    assert!(
        children(&app)
            .iter()
            .all(|id| app.tree().node(*id).unwrap().descriptor.view().focus == FocusMode::Auto)
    );
}
#[test]
fn unstyled_keeps_layout_non_looping_edges_and_explicit_focus_is_rejected() {
    let app = App::memory();
    app.mount(
        button_group()
            .toolbar(true)
            .unstyled()
            .loop_focus(false)
            .orientation(glory::render::Orientation::Vertical)
            .buttons([button().label("First"), button().label("Last")]),
    )
    .unwrap();
    let root = app.tree().roots()[0];
    let ids = children(&app);
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .style
            .display,
        Some(glory::style::Display::Flex)
    );
    app.dispatch(
        ids[0],
        Event::Navigate {
            from: ids[0],
            command: Navigation::Previous,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree().node(ids[0]).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(
        App::memory()
            .mount(
                button_group()
                    .toolbar(true)
                    .button(button().focus(FocusMode::TabStop))
            )
            .is_err()
    );
}
#[test]
fn ssr_has_true_toolbar_orientation_and_one_native_button_tab_entry() {
    let html = glory::ssr::render(
        button_group()
            .toolbar(true)
            .orientation(glory::render::Orientation::Vertical)
            .name("Edit <tools>")
            .buttons([button().label("One"), button().label("Two")]),
    )
    .unwrap();
    assert!(
        html.contains("role=\"toolbar\"") && html.contains("aria-orientation=\"vertical\"")
    );
    assert!(html.contains("Edit &lt;tools&gt;"));
    assert_eq!(html.matches("tabindex=\"0\"").count(), 1);
    assert_eq!(html.matches("tabindex=\"-1\"").count(), 1);
}
