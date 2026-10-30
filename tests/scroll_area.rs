use glory::prelude::*;
use glory::render::memory::Memory;
use glory::render::{
    Action, Capabilities, Error, Input, Mutation, Query, QueryToken, QueryValue, Receipt, Render,
    ScrollAxes, ScrollMetrics,
};
use sapid::{ScrollHandle, scroll_area};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
struct Wire {
    requests: Vec<(QueryToken, u64)>,
    inputs: Vec<Input>,
    actions: Vec<Action>,
}
struct Host {
    memory: Memory,
    wire: Rc<RefCell<Wire>>,
    applied: u64,
}
impl Render for Host {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            scroll: true,
            measure: true,
            ..Default::default()
        }
    }
    fn apply(&mut self, mutations: &[Mutation]) -> std::result::Result<Receipt, Error> {
        let receipt = self.memory.apply(mutations)?;
        self.applied = receipt.applied;
        Ok(receipt)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> std::result::Result<(), Error> {
        query.validate_target(&self.memory.node(query.node())?.descriptor)?;
        assert!(matches!(query, Query::ScrollMetrics(_)));
        self.wire.borrow_mut().requests.push((token, self.applied));
        Ok(())
    }
    fn action(&mut self, action: Action) -> std::result::Result<(), Error> {
        self.wire.borrow_mut().actions.push(action);
        Ok(())
    }
    fn drain(&mut self, inputs: &mut Vec<Input>) {
        inputs.append(&mut self.wire.borrow_mut().inputs);
    }
}
fn fixture() -> (App, Rc<RefCell<Wire>>, ScrollHandle) {
    let wire = Rc::new(RefCell::new(Wire::default()));
    let saved = wire.clone();
    let app = App::new(move |runtime| Host {
        memory: Memory::new(runtime),
        wire: saved,
        applied: 0,
    });
    let output = Rc::new(RefCell::new(None));
    let saved = output.clone();
    app.mount(
        scroll_area("test")
            .axes(ScrollAxes::Both)
            .on_ready(move |handle| *saved.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = output.borrow_mut().take().unwrap();
    (app, wire, handle)
}
fn reply(wire: &RefCell<Wire>, metrics: ScrollMetrics) {
    let (token, applied) = wire.borrow().requests[0];
    wire.borrow_mut().inputs.push(Input::Response {
        token,
        applied,
        result: Ok(QueryValue::ScrollMetrics(metrics)),
    });
}
#[test]
fn one_metrics_barrier_resolves_a_bounded_fifo_without_losing_edge_clamping_order() {
    let (app, wire, handle) = fixture();
    handle.scroll_by(0.0, -100.0).unwrap();
    handle.scroll_by(0.0, 50.0).unwrap();
    handle.refresh().unwrap();
    assert_eq!(wire.borrow().requests.len(), 1);
    reply(
        &wire,
        ScrollMetrics {
            viewport_width: 200.0,
            viewport_height: 100.0,
            content_width: 200.0,
            content_height: 500.0,
            ..Default::default()
        },
    );
    app.pump().unwrap();
    assert_eq!(wire.borrow().actions.len(), 1);
    assert!(matches!(
        wire.borrow().actions[0],
        Action::ScrollTo {
            x: 0.0,
            y: 50.0,
            ..
        }
    ));
    assert_eq!(
        handle.metrics().get().unwrap().y,
        0.0,
        "only actual query observations are published"
    );
}
#[test]
fn pending_commands_are_bounded_and_invalid_commands_do_not_create_queries() {
    let (app, wire, handle) = fixture();
    assert!(handle.scroll_to(f64::NAN, 0.0).is_err());
    assert!(handle.scroll_by(0.0, f64::INFINITY).is_err());
    assert!(wire.borrow().requests.is_empty());
    for _ in 0..sapid::MAX_PENDING_SCROLL_INTENTS {
        handle.scroll_by(0.0, 1.0).unwrap();
    }
    for _ in 0..1000 {
        assert!(handle.scroll_by(0.0, 1.0).is_err());
    }
    assert_eq!(wire.borrow().requests.len(), 1);
    reply(
        &wire,
        ScrollMetrics {
            viewport_height: 100.0,
            content_height: 1000.0,
            ..Default::default()
        },
    );
    app.pump().unwrap();
    assert!(matches!(
        wire.borrow().actions[0],
        Action::ScrollTo { y: 128.0, .. }
    ));
}
#[test]
fn retained_handle_retires_query_callback_captures_and_refs_with_owner() {
    struct Probe(Rc<Cell<usize>>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let app = App::memory();
    let dropped = Rc::new(Cell::new(0));
    let probe = Probe(dropped.clone());
    let reference = app.scope().cage(None);
    let output = Rc::new(RefCell::new(None));
    let saved = output.clone();
    app.mount(
        scroll_area("retire")
            .node_ref(reference)
            .on_metrics_change(move |_| {
                let _ = &probe;
            })
            .on_ready(move |handle| *saved.borrow_mut() = Some(handle)),
    )
    .unwrap();
    let handle = output.borrow_mut().take().unwrap();
    assert!(reference.get().is_some());
    app.clear().unwrap();
    assert_eq!(dropped.get(), 1);
    assert!(reference.get().is_none());
    assert!(handle.refresh().is_err());
    assert!(handle.scroll_by(0.0, 2.0).is_err());
}
#[test]
fn ssr_scroll_area_has_real_native_overflow_and_independent_keyboard_without_custom_dom() {
    let html = glory::ssr::render(
        scroll_area("ssr")
            .name("Documents")
            .scrollbars(sapid::Scrollbars::Always)
            .content(|_| glory::ui::text().content("Long content")),
    )
    .unwrap();
    assert!(html.contains("data-gly-scroll-keyboard=\"vertical\""));
    assert!(html.contains("overflow-y:scroll"));
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("Long content"));
}
