#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{Direction, DragPhase, EventKind, GridCommand, NodeId, Props, TableMetadata};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn date(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
#[derive(Default)]
struct Wire {
    requests: Vec<(glory::render::QueryToken, u64)>,
    inputs: Vec<glory::render::Input>,
}
struct Host {
    memory: glory::render::memory::Memory,
    wire: Rc<RefCell<Wire>>,
    applied: u64,
}
impl glory::render::Render for Host {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> glory::render::Capabilities {
        glory::render::Capabilities {
            measure: true,
            focus: true,
            ..Default::default()
        }
    }
    fn apply(
        &mut self,
        m: &[glory::render::Mutation],
    ) -> std::result::Result<glory::render::Receipt, glory::render::Error> {
        let receipt = self.memory.apply(m)?;
        self.applied = receipt.applied;
        Ok(receipt)
    }
    fn request(
        &mut self,
        q: glory::render::Query,
        token: glory::render::QueryToken,
    ) -> std::result::Result<(), glory::render::Error> {
        q.validate_target(&self.memory.node(q.node())?.descriptor)?;
        self.wire.borrow_mut().requests.push((token, self.applied));
        Ok(())
    }
    fn action(
        &mut self,
        _: glory::render::Action,
    ) -> std::result::Result<(), glory::render::Error> {
        Ok(())
    }
    fn drain(&mut self, inputs: &mut Vec<glory::render::Input>) {
        inputs.append(&mut self.wire.borrow_mut().inputs)
    }
}
fn fixture() -> (App, Rc<RefCell<Wire>>) {
    let wire = Rc::new(RefCell::new(Wire::default()));
    let out = wire.clone();
    (
        App::new(move |runtime| Host {
            memory: glory::render::memory::Memory::new(runtime),
            wire: out,
            applied: 0,
        }),
        wire,
    )
}
fn reply(wire: &RefCell<Wire>, at: usize, width: f64, height: f64) {
    let (token, applied) = wire.borrow().requests[at];
    wire.borrow_mut()
        .inputs
        .push(glory::render::Input::Response {
            token,
            applied,
            result: Ok(glory::render::QueryValue::Bounds(glory::render::Rect {
                x: 0.,
                y: 0.,
                width,
                height,
            })),
        });
}
fn drag(app: &App, id: NodeId, phase: DragPhase, x: f64, y: f64) {
    app.dispatch(
        id,
        Event::Drag {
            from: id,
            phase,
            x,
            y,
        },
    )
    .unwrap();
}
#[test]
fn one_canvas_request_coalesces_drop_and_preserves_typed_before_generic_proposal() {
    for axis in [ResourceAxis::Columns, ResourceAxis::Rows] {
        let (app, wire) = fixture();
        let edits = Rc::new(RefCell::new(vec![]));
        let out = edits.clone();
        let order = Rc::new(RefCell::new(vec![]));
        let a = order.clone();
        let b = order.clone();
        app.mount(
            resources_day_view("pointer", model(1, 0, 2))
                .axis(axis)
                .on_edit(move |e| {
                    a.borrow_mut().push("typed");
                    out.borrow_mut().push(e)
                })
                .on(EventKind::Drag, move |e| {
                    if matches!(
                        e,
                        Event::Drag {
                            phase: DragPhase::Drop,
                            ..
                        }
                    ) {
                        b.borrow_mut().push("drop")
                    }
                }),
        )
        .unwrap();
        let id=classes(&app,"sapid-resources-drag-move").into_iter().find(|id|{
            let parent=app.tree().node(*id).unwrap().parent.unwrap();let card=app.tree().node(parent).unwrap().parent.unwrap();
            all(&app).iter().any(|button|{let t=app.tree();let n=t.node(*button).unwrap();n.parent==Some(card)&&matches!(n.descriptor.view().props,Props::Button(p)if p.label=="Planning")})
        }).unwrap();
        let (start, end, size) = if axis == ResourceAxis::Columns {
            ((100., 100.), (300., 132.), (384., 672.))
        } else {
            ((300., 100.), (364., 300.), (1440., 352.))
        };
        drag(&app, id, DragPhase::Start, start.0, start.1);
        drag(&app, id, DragPhase::Move, end.0, end.1);
        drag(&app, id, DragPhase::Drop, end.0, end.1);
        assert_eq!(wire.borrow().requests.len(), 1);
        assert!(edits.borrow().is_empty());
        assert_eq!(app.stats().queries, 1);
        reply(&wire, 0, size.0, size.1);
        app.pump().unwrap();
        let edits = edits.borrow();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].proposed.start(), dt("2024-02-29T09:30"));
        assert_eq!(edits[0].proposed_resource, Some(key("r1")));
        assert_eq!(*order.borrow(), ["typed", "drop"]);
        assert_eq!(app.stats().queries, 0);
    }
}
#[test]
fn late_geometry_cannot_cross_accepted_source_readonly_or_owner_generations() {
    let (app, wire) = fixture();
    let source = app.scope().cage(model(1, 0, 2));
    let readonly = app.scope().cage(false);
    let visible = app.scope().cage(true);
    let edits = app.scope().cage(0usize);
    app.mount(glory::ui::when(visible, move || {
        resources_day_view("cancel", source)
            .readonly(readonly)
            .on_edit(move |_| edits.revise(|n| *n += 1))
    }))
    .unwrap();
    for request in 0..3 {
        let id = classes(&app, "sapid-resources-drag-move")[0];
        drag(&app, id, DragPhase::Start, 100., 100.);
        drag(&app, id, DragPhase::Drop, 100., 132.);
        assert_eq!(app.stats().queries, 1);
        match request {
            0 => source.revise(|m| *m = model(1, 2, 2)),
            1 => readonly.revise(|v| *v = true),
            _ => visible.revise(|v| *v = false),
        }
        app.flush().unwrap();
        assert_eq!(app.stats().queries, 0);
        reply(&wire, request, 384., 672.);
        app.pump().unwrap();
        assert_eq!(edits.get(), 0);
        source.revise(|m| *m = model(1, 0, 2));
        readonly.revise(|v| *v = false);
        app.flush().unwrap();
    }
    assert!(classes(&app, "sapid-resources-event-card").is_empty());
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn key(s: &str) -> ScheduleKey {
    ScheduleKey::new(s).unwrap()
}
fn resources(n: usize) -> ScheduleResources {
    ScheduleResources::new(
        (0..n).map(|i| ScheduleResource::new(key(&format!("r{i}")), format!("Room {i}")).unwrap()),
    )
    .unwrap()
}
fn model(days: u16, first: usize, count: usize) -> ResourceScheduleData {
    let window = ScheduleWindow::days(date("2024-02-29"), days).unwrap();
    let permissions = SchedulePermissions {
        select: true,
        move_event: true,
        resize_event: true,
    };
    ResourceScheduleData::new(
        ScheduleData::new(
            ScheduleZone::Civil,
            window,
            [
                ScheduleEvent::new(
                    key("a"),
                    "Planning",
                    ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
                )
                .unwrap()
                .resource(key("r0"))
                .permissions(permissions),
                ScheduleEvent::new(
                    key("b"),
                    "Review",
                    ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
                )
                .unwrap()
                .resource(key("r1"))
                .permissions(SchedulePermissions {
                    select: false,
                    move_event: true,
                    resize_event: false,
                }),
                ScheduleEvent::new(
                    key("holiday"),
                    "Holiday",
                    ScheduleInterval::all_day(date("2024-02-29"), date("2024-03-01")).unwrap(),
                )
                .unwrap()
                .all_day()
                .unwrap()
                .resource(key("r1"))
                .permissions(permissions),
            ],
        )
        .unwrap(),
        resources(5),
        ScheduleResourceWindow::new(first, count).unwrap(),
    )
    .unwrap()
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut out = vec![];
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        pending.extend(tree.node(id).unwrap().children.iter().copied());
        out.push(id)
    }
    out
}
fn classes(app: &App, name: &str) -> Vec<NodeId> {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|s| s == name)
        })
        .collect()
}
fn button(app: &App, label: &str) -> NodeId {
    all(app).into_iter().find(|id|matches!(app.tree().node(*id).unwrap().descriptor.view().props,Props::Button(p) if p.label==label||p.label.starts_with(label))).unwrap_or_else(||panic!("missing {label}"))
}
fn slot(app: &App, prefix: &str) -> NodeId {
    classes(app, "sapid-resources-slot")
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_ref()
                .is_some_and(|s| s.starts_with(prefix))
        })
        .unwrap()
}
#[test]
fn column_and_row_axes_have_real_grid_transpose_and_distinct_resource_lanes() {
    for axis in [ResourceAxis::Columns, ResourceAxis::Rows] {
        let app = App::memory();
        let grid = app.scope().cage(None);
        app.mount(
            resources_day_view("axes", model(1, 0, 2))
                .axis(axis)
                .grid_ref(grid)
                .on_edit(|_| {}),
        )
        .unwrap();
        let tree = app.tree();
        let v = tree.node(grid.get().unwrap()).unwrap().descriptor.view();
        assert_eq!(v.role, Some(Role::Grid));
        assert_eq!(
            v.semantics.as_ref().unwrap().table,
            Some(if axis == ResourceAxis::Columns {
                TableMetadata::Table {
                    rows: Some(21),
                    columns: Some(3),
                }
            } else {
                TableMetadata::Table {
                    rows: Some(3),
                    columns: Some(21),
                }
            })
        );
        drop(tree);
        assert_eq!(classes(&app, "sapid-resources-slot").len(), 40);
        assert_eq!(classes(&app, "sapid-resources-event-card").len(), 2);
        assert_eq!(classes(&app, "sapid-resources-all-day-event").len(), 1);
        let positions = classes(&app, "sapid-resources-event-card")
            .into_iter()
            .map(|id| {
                let t = app.tree();
                let v = t.node(id).unwrap().descriptor.view();
                (v.style.left, v.style.top, v.style.width, v.style.height)
            })
            .collect::<Vec<_>>();
        if axis == ResourceAxis::Columns {
            assert_ne!(positions[0].0, positions[1].0);
            assert_eq!(positions[0].1, positions[1].1);
            assert_eq!(positions[0].2, Some(glory::style::Length::Px(158.)))
        } else {
            assert_ne!(positions[0].1, positions[1].1);
            assert_eq!(positions[0].0, positions[1].0);
            assert_eq!(positions[0].2, Some(glory::style::Length::Px(128.)))
        }
        assert_eq!(classes(&app, "sapid-resources-drag-move").len(), 2);
        let review = button(&app, "Review");
        assert!(
            matches!(app.tree().node(review).unwrap().descriptor.view().props,Props::Button(p) if !p.enabled)
        );
        assert!(
            matches!(app.tree().node(button(&app,"Edit Review")).unwrap().descriptor.view().props,Props::Button(p) if p.enabled),
            "move permission independent from select permission"
        );
    }
}
#[test]
fn axes_keyboard_same_tick_navigation_selection_and_readonly_are_physical() {
    for axis in [ResourceAxis::Columns, ResourceAxis::Rows] {
        let app = App::memory();
        let grid = app.scope().cage(None);
        let selected = app.scope().cage(None);
        let readonly = app.scope().cage(false);
        app.mount(
            resources_day_view("nav", model(1, 0, 2))
                .axis(axis)
                .grid_ref(grid)
                .readonly(readonly)
                .selection(selected)
                .on_selection_change(move |v| selected.revise(|s| *s = Some(v))),
        )
        .unwrap();
        let from = slot(&app, "Room 0 · Thursday, February 29, 2024 08:00");
        app.scope()
            .batch(|| {
                for command in [GridCommand::Right, GridCommand::Down] {
                    app.dispatch(grid.get().unwrap(), Event::GridNavigate { from, command })
                        .unwrap();
                }
                app.dispatch(from, Event::Activate).unwrap();
            })
            .unwrap();
        assert!(
            matches!(selected.get(),Some(ScheduleSelection::Slot{interval,resource:Some(r)})if interval.start()==dt("2024-02-29T08:30")&&r==key("r1"))
        );
        readonly.revise(|r| *r = true);
        app.flush().unwrap();
        let before = selected.get();
        app.dispatch(from, Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(selected.get(), before);
    }
}
#[test]
fn last_partial_resource_page_returns_with_original_stride_and_request_refusal_is_ordered() {
    let app = App::memory();
    let out = Rc::new(RefCell::new(vec![]));
    let calls = out.clone();
    let source = app.scope().cage(model(1, 0, 2));
    app.mount(
        resources_day_view("pages", source).on_window_request(move |r| calls.borrow_mut().push(r)),
    )
    .unwrap();
    let next = button(&app, "Next resources");
    // Controlled refusal keeps rendered page0, but same-turn requests retain a
    // bounded request cursor, including the one-element final page.
    app.scope()
        .batch(|| {
            app.dispatch(next, Event::Activate).unwrap();
            app.dispatch(next, Event::Activate).unwrap();
        })
        .unwrap();
    source.revise(|v| *v = model(1, 4, 1));
    app.flush().unwrap();
    app.dispatch(button(&app, "Previous resources"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    let calls = out.borrow();
    assert_eq!(
        calls
            .iter()
            .map(|r| (r.resource_window.first(), r.resource_window.count()))
            .collect::<Vec<_>>(),
        [(2, 2), (4, 1), (2, 2)]
    );
    assert_eq!(
        calls.iter().map(|r| r.revision).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(app.tree().node(slot(&app, "Room 4 · Thursday")).is_ok());
}
#[test]
fn same_tick_edit_fifo_accumulates_controlled_refusal_and_respects_resize_boundaries() {
    let app = App::memory();
    let out = Rc::new(RefCell::new(vec![]));
    let calls = out.clone();
    let trace = Rc::new(RefCell::new(vec![]));
    let typed = trace.clone();
    let generic = trace.clone();
    app.mount(
        resources_day_view("edit", model(1, 0, 2))
            .on_edit(move |e| {
                typed.borrow_mut().push("typed");
                calls.borrow_mut().push(e)
            })
            .on(EventKind::Activate, move |_| {
                generic.borrow_mut().push("generic")
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "Edit Planning"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    trace.borrow_mut().clear();
    let later = button(&app, "Move later");
    app.scope()
        .batch(|| {
            app.dispatch(later, Event::Activate).unwrap();
            app.dispatch(later, Event::Activate).unwrap();
        })
        .unwrap();
    let calls = out.borrow();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].proposed.start(), dt("2024-02-29T09:30"));
    assert_eq!(calls[1].proposed.start(), dt("2024-02-29T10:00"));
    assert_eq!(calls[0].original, calls[1].original);
    drop(calls);
    assert_eq!(*trace.borrow(), ["typed", "generic", "typed", "generic"]);
    // A fresh turn starts at the actually accepted9am, not the refused10am.
    app.dispatch(later, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(
        out.borrow().last().unwrap().proposed.start(),
        dt("2024-02-29T09:30")
    );
    out.borrow_mut().clear();
    app.scope()
        .batch(|| {
            for label in ["Start later", "Start later", "Start earlier"] {
                app.dispatch(button(&app, label), Event::Activate).unwrap();
            }
        })
        .unwrap();
    let calls = out.borrow();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].proposed.start(), dt("2024-02-29T09:30"));
    assert_eq!(calls[1].proposed.start(), dt("2024-02-29T09:00"));
}
#[test]
fn same_tick_accepted_edit_uses_latest_original_and_generic_readonly_cancels_following_intents() {
    let app = App::memory();
    let source = app.scope().cage(model(1, 0, 2));
    let readonly = app.scope().cage(false);
    let changes = app.scope().cage(0usize);
    let stop = app.scope().cage(false);
    let edits = Rc::new(RefCell::new(vec![]));
    let out = edits.clone();
    app.mount(
        resources_day_view("accepted", source)
            .readonly(readonly)
            .on_edit(move |edit| {
                changes.revise(|n| *n += 1);
                source.revise(|old| {
                    *old = ResourceScheduleData::new(
                        ScheduleData::new(
                            old.data().zone().clone(),
                            old.data().window(),
                            old.data().events().iter().map(|event| {
                                if event.key() == &edit.event {
                                    ScheduleEvent::new(
                                        event.key().clone(),
                                        event.title(),
                                        edit.proposed,
                                    )
                                    .unwrap()
                                    .resource(edit.proposed_resource.clone().unwrap())
                                    .permissions(event.edit_permissions())
                                } else {
                                    event.clone()
                                }
                            }),
                        )
                        .unwrap(),
                        old.resources().clone(),
                        old.resource_window(),
                    )
                    .unwrap()
                });
                out.borrow_mut().push(edit);
            })
            .on(EventKind::Activate, move |_| {
                if stop.get_untracked() && changes.get_untracked() > 0 {
                    readonly.revise(|v| *v = true)
                }
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "Edit Planning"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    let later = button(&app, "Move later");
    app.scope()
        .batch(|| {
            app.dispatch(later, Event::Activate).unwrap();
            app.dispatch(later, Event::Activate).unwrap();
        })
        .unwrap();
    let recorded = edits.borrow();
    assert_eq!(recorded.len(), 2);
    assert_eq!(recorded[1].original, recorded[0].proposed);
    assert_eq!(recorded[1].proposed.start(), dt("2024-02-29T10:00"));
    drop(recorded);
    stop.revise(|v| *v = true);
    app.scope()
        .batch(|| {
            app.dispatch(later, Event::Activate).unwrap();
            app.dispatch(later, Event::Activate).unwrap();
        })
        .unwrap();
    assert_eq!(edits.borrow().len(), 3);
    assert!(readonly.get());
}
#[test]
fn row_rtl_and_month_resources_have_distinct_actual_geometry_and_selection_keys() {
    let app = App::memory();
    let selected = app.scope().cage(None);
    let day = model(1, 0, 2);
    let month = ResourceScheduleData::new(
        ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap(),
            day.data().events().iter().cloned(),
        )
        .unwrap(),
        day.resources().clone(),
        day.resource_window(),
    )
    .unwrap();
    app.mount(
        resources_month_view("month", month)
            .selection(selected)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
    )
    .unwrap();
    assert_eq!(classes(&app, "sapid-resources-slot").len(), 58);
    let grids = all(&app)
        .into_iter()
        .filter(|id| app.tree().node(*id).unwrap().descriptor.view().role == Some(Role::Grid))
        .count();
    assert_eq!(grids, 1);
    let target = slot(&app, "Room 0 · Thursday, February 29, 2024");
    app.dispatch(target, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(matches!(
        selected.get(),
        Some(ScheduleSelection::Slot {
            resource: Some(_),
            ..
        })
    ));
    let other = App::memory();
    other
        .mount(
            resources_day_view("rtl", model(1, 0, 2))
                .axis(ResourceAxis::Rows)
                .direction(Direction::Rtl),
        )
        .unwrap();
    let a = slot(&other, "Room 0 · Thursday, February 29, 2024 08:00");
    let b = slot(&other, "Room 0 · Thursday, February 29, 2024 08:30");
    let tree = other.tree();
    assert!(matches!(
        (
            tree.node(a).unwrap().descriptor.view().style.width,
            tree.node(b).unwrap().descriptor.view().style.width
        ),
        (
            Some(glory::style::Length::Percent(100.)),
            Some(glory::style::Length::Percent(100.))
        )
    ));
}
#[test]
fn owner_retirement_count_change_and_source_recovery_have_no_visited_resource_nodes() {
    let app = App::memory();
    let source = app.scope().cage(model(1, 0, 2));
    let visible = app.scope().cage(true);
    let reference = app.scope().cage(None);
    app.mount(glory::ui::when(visible, move || {
        resources_day_view("live", source).node_ref(reference)
    }))
    .unwrap();
    let first = slot(&app, "Room 0 · Thursday");
    source.revise(|v| *v = model(1, 2, 2));
    app.flush().unwrap();
    assert_eq!(slot(&app, "Room 2 · Thursday"), first);
    source.revise(|v| *v = model(1, 4, 1));
    app.flush().unwrap();
    assert!(app.tree().node(first).is_err());
    assert_eq!(classes(&app, "sapid-resources-slot").len(), 20);
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(reference.try_get().is_err() || reference.get().is_none());
    assert!(classes(&app, "sapid-resource-schedule").is_empty());
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn edit_turn_budget_is_sixty_four_and_new_turn_does_not_retain_refused_history() {
    let app = App::memory();
    let edits = app.scope().cage(0usize);
    let last = app.scope().cage(None);
    app.mount(
        resources_day_view("bounded-edit", model(1, 0, 2)).on_edit(move |v| {
            edits.revise(|n| *n += 1);
            last.revise(|old| *old = Some(v.proposed))
        }),
    )
    .unwrap();
    app.dispatch(button(&app, "Edit Planning"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    let later = button(&app, "Move later");
    app.scope()
        .batch(|| {
            for _ in 0..1000 {
                app.dispatch(later, Event::Activate).unwrap();
            }
        })
        .unwrap();
    assert_eq!(edits.get(), 64);
    assert_eq!(last.get().unwrap().start(), dt("2024-03-01T17:00"));
    app.dispatch(later, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(edits.get(), 65);
    assert_eq!(last.get().unwrap().start(), dt("2024-02-29T09:30"));
}
#[test]
fn non_editable_events_fill_the_card_and_styled_buttons_receive_the_actual_theme() {
    for unstyled in [false, true] {
        let app = App::memory();
        let theme = Theme::green_dark();
        let mut view = resources_day_view("whole-hit", model(1, 0, 2)).theme(theme.clone());
        if unstyled {
            view = view.unstyled()
        }
        app.mount(view).unwrap();
        let planning = button(&app, "Planning");
        let tree = app.tree();
        let v = tree.node(planning).unwrap().descriptor.view();
        assert_eq!(v.style.height, Some(glory::style::Length::Percent(100.)));
        if !unstyled {
            assert_eq!(v.style.background, Some(theme.accent_soft));
            assert_eq!(v.style.color, Some(theme.accent_text));
        }
        assert!(classes(&app, "sapid-resources-drag-handles").is_empty());
    }
}
