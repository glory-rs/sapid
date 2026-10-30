use glory::prelude::*;
use glory::render::{Event, EventKind, NodeId, Props, Rect};
use glory::style::Limit;
use sapid::spoiler;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn measure(app: &App, height: f64) {
    let size = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height,
    };
    app.dispatch(
        find(app, "sapid-spoiler-measure"),
        Event::Resize {
            size,
            viewport: size,
        },
    )
    .unwrap();
}
fn expanded(app: &App) -> bool {
    app.tree()
        .node(find(app, "sapid-spoiler-trigger"))
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .expanded
        .unwrap()
}
fn hidden_actions(app: &App) -> bool {
    app.tree()
        .node(find(app, "sapid-spoiler-actions"))
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap()
        .hidden
}
#[test]
fn measured_local_expansion_preserves_text_and_runs_typed_before_observer() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let typed = calls.clone();
    let observer = calls.clone();
    app.mount(
        spoiler("local")
            .max_height(60.0)
            .content("Long text")
            .on_expanded_change(move |value| {
                typed
                    .borrow_mut()
                    .push(if value { "open" } else { "close" })
            })
            .on(EventKind::Activate, move |_| {
                observer.borrow_mut().push("observe")
            }),
    )
    .unwrap();
    let text = find(&app, "sapid-spoiler-text");
    let trigger = find(&app, "sapid-spoiler-trigger");
    measure(&app, 300.0);
    assert!(!hidden_actions(&app));
    for (open, cap) in [(true, Limit::None), (false, Limit::Px(60.0))] {
        app.dispatch(trigger, Event::Activate).unwrap();
        assert_eq!(expanded(&app), open);
        assert_eq!(
            app.tree()
                .node(find(&app, "sapid-spoiler-viewport"))
                .unwrap()
                .descriptor
                .view()
                .style
                .max_height,
            Some(cap)
        );
        assert_eq!(find(&app, "sapid-spoiler-text"), text);
    }
    assert_eq!(&*calls.borrow(), &["open", "observe", "close", "observe"]);
}
#[test]
fn controlled_refusal_and_external_updates_do_not_replace_text_or_emit_extra_proposals() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        spoiler("controlled")
            .content("Long text")
            .max_height(20.0)
            .expanded(open)
            .on_expanded_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    let text = find(&app, "sapid-spoiler-text");
    measure(&app, 100.0);
    app.dispatch(find(&app, "sapid-spoiler-trigger"), Event::Activate)
        .unwrap();
    assert!(!expanded(&app));
    assert_eq!(calls.get(), 1);
    open.revise(|open| *open = true);
    app.flush().unwrap();
    assert!(expanded(&app));
    assert_eq!(calls.get(), 1);
    assert_eq!(find(&app, "sapid-spoiler-text"), text);
}
#[test]
fn resize_hides_unnecessary_actions_and_stale_activation_cannot_change_state() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        spoiler("short")
            .content("Short")
            .on_expanded_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    measure(&app, 20.0);
    assert!(hidden_actions(&app));
    app.dispatch(find(&app, "sapid-spoiler-trigger"), Event::Activate)
        .unwrap();
    assert!(!expanded(&app));
    assert_eq!(calls.get(), 0);
    measure(&app, 300.0);
    assert!(!hidden_actions(&app));
}
#[test]
fn invalid_configuration_has_no_semantic_allocations_and_large_updates_can_recover() {
    for widget in [
        spoiler("bad").max_height(f64::NAN),
        spoiler("bad").max_height(0.0),
        spoiler("bad").labels("", "Less"),
        spoiler("bad").content("x".repeat(sapid::MAX_SPOILER_BYTES + 1)),
        spoiler("bad key"),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
        assert_eq!(app.stats().node_slots, 0);
    }
    let app = App::memory();
    let content = app.scope().cage("accepted".to_string());
    app.mount(spoiler("recovery").content(content)).unwrap();
    let id = find(&app, "sapid-spoiler-text");
    content.revise(|text| *text = "x".repeat(sapid::MAX_SPOILER_BYTES + 1));
    assert!(app.flush().is_err());
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(),Props::Text(value) if value.content=="accepted")
    );
    content.revise(|text| *text = "recovered".into());
    app.flush().unwrap();
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(),Props::Text(value) if value.content=="recovered")
    );
}
#[test]
fn empty_text_starts_without_action_and_unstyled_retains_structural_clipping() {
    let app = App::memory();
    app.mount(spoiler("empty").unstyled().content("").max_height(40.0))
        .unwrap();
    assert!(hidden_actions(&app));
    let style = app
        .tree()
        .node(find(&app, "sapid-spoiler-viewport"))
        .unwrap()
        .descriptor
        .view()
        .style
        .clone();
    assert_eq!(style.max_height, Some(Limit::Px(40.0)));
    assert_eq!(style.overflow_y, Some(glory::style::Overflow::Hidden));
    assert_eq!(
        app.tree()
            .node(find(&app, "sapid-spoiler-text"))
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new()
    );
}
#[test]
fn owner_retirement_clears_refs_and_drops_callbacks() {
    struct DropCount(Rc<Cell<u32>>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1)
        }
    }
    let app = App::memory();
    let viewport = app.scope().cage(None);
    let trigger = app.scope().cage(None);
    let drops = Rc::new(Cell::new(0));
    let capture = DropCount(drops.clone());
    app.mount(
        spoiler("lifetime")
            .content("Text")
            .viewport_ref(viewport)
            .trigger_ref(trigger)
            .on_expanded_change(move |_| {
                let _ = &capture;
            }),
    )
    .unwrap();
    let id = trigger.get().unwrap();
    app.clear().unwrap();
    assert_eq!(viewport.get(), None);
    assert_eq!(trigger.get(), None);
    assert_eq!(drops.get(), 1);
    assert!(!app.dispatch(id, Event::Activate).unwrap());
}
#[test]
fn ssr_retains_full_escaped_text_and_explicit_disclosure_relationship() {
    let html =
        glory::ssr::render(spoiler("server").content("Text <&> full").max_height(70.0)).unwrap();
    assert!(html.contains("Text &lt;&amp;&gt; full"));
    assert!(html.contains("max-height:70px"));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("aria-controls=\"sapid-spoiler-"));
    assert!(!html.contains("role=\"button\""));
}

#[test]
fn shrinking_content_requests_focus_before_its_disclosure_disappears() {
    use glory::render::{
        Action, Capabilities, Input, Mutation, Query, QueryToken, Receipt, Render,
    };
    struct Host {
        memory: glory::render::memory::Memory,
        actions: Rc<RefCell<Vec<Action>>>,
    }
    impl Render for Host {
        fn style_profile(&self) -> glory::style::Profile {
            glory::style::Profile::RECORD
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                focus: true,
                ..Default::default()
            }
        }
        fn apply(
            &mut self,
            changes: &[Mutation],
        ) -> std::result::Result<Receipt, glory::render::Error> {
            self.memory.apply(changes)
        }
        fn request(
            &mut self,
            query: Query,
            token: QueryToken,
        ) -> std::result::Result<(), glory::render::Error> {
            self.memory.request(query, token)
        }
        fn drain(&mut self, inputs: &mut Vec<Input>) {
            self.memory.drain(inputs)
        }
        fn action(&mut self, action: Action) -> std::result::Result<(), glory::render::Error> {
            assert!(self.memory.active(action.node()));
            self.actions.borrow_mut().push(action);
            Ok(())
        }
    }
    let actions = Rc::new(RefCell::new(Vec::new()));
    let seen = actions.clone();
    let app = App::new(move |runtime| Host {
        memory: glory::render::memory::Memory::new(runtime),
        actions: seen,
    });
    app.mount(spoiler("focus").content("Long text").max_height(30.0))
        .unwrap();
    measure(&app, 100.0);
    let trigger = find(&app, "sapid-spoiler-trigger");
    let body = find(&app, "sapid-spoiler-measure");
    app.dispatch(trigger, Event::FocusChanged(true)).unwrap();
    measure(&app, 20.0);
    assert!(hidden_actions(&app));
    assert_eq!(&*actions.borrow(), &[Action::Focus(body)]);
}
