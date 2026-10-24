use glory::render::{Event, EventKind, NodeId, SheetCommand, SheetConfig};
use glory::ui::App;
use sapid::{DrawerSide, DrawerSnap, DrawerSnaps, Theme, drawer};
use std::{cell::RefCell, rc::Rc};
fn points() -> DrawerSnaps {
    DrawerSnaps::new([
        DrawerSnap::new(0.25, "Compact"),
        DrawerSnap::new(0.5, "Half"),
        DrawerSnap::new(1.0, "Full"),
    ])
}
fn config(app: &App, id: NodeId) -> SheetConfig {
    app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .layer
        .as_ref()
        .unwrap()
        .sheet
        .clone()
        .unwrap()
}
#[test]
fn controlled_snap_refusal_is_silent_and_internal_acceptance_precedes_generic_observers() {
    let app = App::memory();
    let scope = app.scope();
    let open = scope.cage(true);
    let snap = scope.cage(1usize);
    let panel = scope.cage(None);
    let options = scope.cage(points());
    let order = Rc::new(RefCell::new(vec![]));
    let typed = order.clone();
    let generic = order.clone();
    app.mount(
        drawer("sheet-control")
            .title("Preview")
            .snap_config(options)
            .snap_index(snap)
            .open(open)
            .panel_ref(panel)
            .on_snap_change(move |index| typed.borrow_mut().push(format!("typed-{index}")))
            .on(EventKind::Sheet, move |_| {
                generic.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    let id = panel.get().unwrap();
    let initial = config(&app, id);
    assert_eq!(initial.current, 1);
    assert_eq!(initial.revision, 0);
    let writes = app.runtime_stats().source_writes;
    app.dispatch(
        id,
        Event::Sheet {
            from: id,
            revision: 0,
            command: SheetCommand::Snap(2),
        },
    )
    .unwrap();
    assert_eq!(snap.get(), 1);
    assert_eq!(config(&app, id), initial);
    assert_eq!(app.runtime_stats().source_writes, writes);
    assert_eq!(*order.borrow(), ["typed-2", "generic"]);
    snap.revise(|v| *v = 2);
    app.flush().unwrap();
    assert_eq!(config(&app, id).revision, 1);
    assert_eq!(config(&app, id).current, 2);
    assert!(
        !app.dispatch(
            id,
            Event::Sheet {
                from: id,
                revision: 0,
                command: SheetCommand::Snap(0)
            }
        )
        .unwrap()
    );
    assert!(
        app.dispatch(
            id,
            Event::Sheet {
                from: id,
                revision: 2,
                command: SheetCommand::Snap(0)
            }
        )
        .is_err()
    );
    options.revise(|o| o.max_extent = 800.0);
    app.flush().unwrap();
    assert_eq!(config(&app, id).revision, 2);
    options.revise(|o| o.max_extent = 800.0);
    app.flush().unwrap();
    assert_eq!(config(&app, id).revision, 2);
    let before = config(&app, id);
    options.revise(|o| o.points[0].fraction = f64::NAN);
    assert!(app.flush().is_err());
    assert_eq!(config(&app, id), before);
    assert!(options.get().points[0].fraction.is_nan());
    options.revise(|o| o.points[0].fraction = 0.25);
    app.flush().unwrap();
    assert_eq!(config(&app, id), before);
    app.clear().unwrap();
    assert!(panel.get().is_none());
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn local_snaps_keyboard_buttons_and_gesture_dismiss_have_real_semantics_and_lifecycle() {
    let app = App::memory();
    let panel = app.scope().cage(None);
    let log = app.scope().cage(vec![]);
    app.mount(
        drawer("local-sheet")
            .title("Local sheet")
            .default_open(true)
            .default_snap(0)
            .snap_config(points())
            .panel_ref(panel)
            .on_dismiss(move |r| log.revise(|v| v.push(r))),
    )
    .unwrap();
    let id = panel.get().unwrap();
    let full=descendants(&app,id).into_iter().find(|child|matches!(app.tree().node(*child).unwrap().descriptor.view().props,glory::render::Props::Button(button) if button.label=="Full")).unwrap();
    app.dispatch(full, Event::Activate).unwrap();
    assert_eq!(config(&app, id).current, 2);
    assert_eq!(config(&app, id).revision, 1);
    app.dispatch(
        id,
        Event::Sheet {
            from: id,
            revision: 1,
            command: SheetCommand::Dismiss,
        },
    )
    .unwrap();
    assert!(app.tree().node(id).unwrap().descriptor.view().hidden());
    assert_eq!(log.get(), [glory::render::DismissReason::Gesture]);
    app.clear().unwrap();
    assert!(panel.get().is_none());
}
#[test]
fn finite_iteration_labels_initial_rejection_and_four_theme_html_are_complete() {
    let consumed = Rc::new(std::cell::Cell::new(0));
    let count = consumed.clone();
    let options = DrawerSnaps::new((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        DrawerSnap::new(0.5, "Snap")
    }));
    assert_eq!(consumed.get(), 9);
    assert_eq!(options.points.len(), 9);
    for options in [
        options,
        DrawerSnaps::new([]),
        DrawerSnaps::new([DrawerSnap::new(0.5, "")]),
        points().max_extent(4097.0),
    ] {
        let app = App::memory();
        assert!(
            app.mount(drawer("invalid").title("Invalid").snap_config(options))
                .is_err()
        );
        assert!(app.tree().roots().is_empty());
    }
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for side in [
            DrawerSide::Left,
            DrawerSide::Right,
            DrawerSide::Top,
            DrawerSide::Bottom,
        ] {
            let page = glory::ssr::render(
                drawer("ssr-sheet")
                    .title("Snapshot")
                    .theme(theme.clone())
                    .side(side)
                    .snap_config(points())
                    .default_snap(1)
                    .default_open(true),
            )
            .unwrap();
            assert!(page.contains("data-gly-sheet=\"0.25,0.5,1\""));
            assert!(page.contains("data-gly-sheet-revision=\"0\""));
            assert!(page.contains("data-gly-sheet-grip"));
            assert!(page.contains("aria-pressed=\"true\""));
            assert!(page.contains("Full"));
        }
    }
}

fn descendants(app: &App, id: NodeId) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.node(id).unwrap().children.to_vec();
    let mut result = vec![];
    while let Some(next) = pending.pop() {
        result.push(next);
        pending.extend(tree.node(next).unwrap().children.iter().copied());
    }
    result
}
