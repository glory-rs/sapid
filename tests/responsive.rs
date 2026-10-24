use glory::render::{Event, Props, Rect};
use glory::style::{Length, Style};
use glory::ui::App;
use sapid::{Breakpoint, Breakpoints, ResponsiveValues, responsive_root};
use std::{cell::Cell, rc::Rc};
#[path = "../examples/web/src/responsive_form.rs"]
mod fixture;

#[test]
fn thresholds_are_finite_ordered_and_resolve_boundary_equality() {
    let points = Breakpoints::default();
    for (width, tier) in [
        (0., Breakpoint::Base),
        (575.5, Breakpoint::Base),
        (576., Breakpoint::Small),
        (768., Breakpoint::Medium),
        (992., Breakpoint::Large),
        (1200., Breakpoint::ExtraLarge),
    ] {
        assert_eq!(points.resolve(width).unwrap(), tier);
    }
    for values in [
        [0., 1., 2., 3.],
        [1., 1., 2., 3.],
        [1., 2., 3., f64::INFINITY],
        [1., 2., 3., 1_000_001.],
    ] {
        assert!(Breakpoints::new(values).is_err());
    }
    assert!(points.resolve(f64::NAN).is_err());
}

#[test]
fn resize_updates_existing_layout_styles_and_retired_context_fails() {
    let app = App::memory();
    let root = app.scope().cage(None);
    let context = app.scope().cage(None);
    let factories = Rc::new(Cell::new(0));
    let count = factories.clone();
    app.mount(
        responsive_root(500.)
            .node_ref(root)
            .context_ref(context)
            .content(move |context| {
                count.set(count.get() + 1);
                let values = ResponsiveValues::new(10., 20., 30., 40., 50.);
                let style = ResponsiveValues::new(
                    Style::new().height(Length::Px(10.)),
                    Style::new().height(Length::Px(20.)),
                    Style::new().height(Length::Px(30.)),
                    Style::new().height(Length::Px(40.)),
                    Style::new().height(Length::Px(50.)),
                );
                let _ = context.select(values).unwrap();
                sapid::space()
                    .class("responsive-space")
                    .style(context.styles(style).unwrap())
            }),
    )
    .unwrap();
    let root = root.get().unwrap();
    let context = context.get().unwrap();
    let before = app.tree().node(root).unwrap().children.to_vec();
    let resize = |width| Event::Resize {
        size: Rect {
            x: 0.,
            y: 0.,
            width,
            height: 100.,
        },
        viewport: Rect {
            x: 0.,
            y: 0.,
            width,
            height: 100.,
        },
    };
    app.dispatch(root, resize(900.)).unwrap();
    app.flush().unwrap();
    assert_eq!(context.breakpoint().unwrap(), Breakpoint::Medium);
    assert_eq!(context.width().unwrap(), 900.);
    assert_eq!(context.pending_queries().unwrap(), 0);
    assert_eq!(app.tree().node(root).unwrap().children.to_vec(), before);
    assert_eq!(factories.get(), 1);
    let tree = app.tree();
    let child = tree.node(before[0]).unwrap();
    assert!(matches!(child.descriptor.view().props, Props::View));
    assert_eq!(child.descriptor.view().style.height, Some(Length::Px(30.)));
    drop(tree);
    app.clear().unwrap();
    assert!(context.width().is_err());
    assert!(
        context
            .select(ResponsiveValues::new(1, 2, 3, 4, 5))
            .is_err()
    );
}

#[test]
fn invalid_ssr_width_and_invalid_style_are_rejected_before_publication() {
    let app = App::memory();
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    assert!(
        app.mount(responsive_root(-1.).content(move |_| {
            count.set(1);
        }))
        .is_err()
    );
    assert_eq!(calls.get(), 0);
    let app = App::memory();
    app.mount(responsive_root(300.).content(|context| {
        assert!(
            context
                .styles(ResponsiveValues::new(
                    Style::new(),
                    Style::new(),
                    Style::new().width(Length::Px(f64::NAN)),
                    Style::new(),
                    Style::new()
                ))
                .is_err()
        );
    }))
    .unwrap();
}

#[test]
fn layout_fixture_constructs_with_deterministic_initial_width() {
    let app = App::memory();
    app.mount(fixture::responsive_form(sapid::Theme::blue_light()))
        .unwrap();
}

#[test]
fn initial_measurement_is_one_queued_task_and_query_and_cannot_overwrite_a_new_resize() {
    use glory::core::executor::Executor;
    use glory::render::{
        Capabilities, Input, Mutation, Query, QueryToken, QueryValue, Receipt, Render,
        memory::Memory,
    };
    use std::{cell::RefCell, task::Waker};
    #[derive(Default)]
    struct Wire {
        applied: u64,
        requests: Vec<(QueryToken, u64)>,
        inputs: Vec<Input>,
    }
    struct Delayed {
        memory: Memory,
        wire: Rc<RefCell<Wire>>,
    }
    impl Render for Delayed {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                measure: true,
                ..Default::default()
            }
        }
        fn apply(&mut self, mutations: &[Mutation]) -> Result<Receipt, glory::render::Error> {
            let receipt = self.memory.apply(mutations)?;
            self.wire.borrow_mut().applied = receipt.applied;
            Ok(receipt)
        }
        fn request(
            &mut self,
            query: Query,
            token: QueryToken,
        ) -> Result<(), glory::render::Error> {
            self.memory.node(query.node())?;
            let mut wire = self.wire.borrow_mut();
            let applied = wire.applied;
            wire.requests.push((token, applied));
            Ok(())
        }
        fn drain(&mut self, inputs: &mut Vec<Input>) {
            inputs.append(&mut self.wire.borrow_mut().inputs);
        }
    }
    let wire = Rc::new(RefCell::new(Wire::default()));
    let copy = wire.clone();
    let executor = Rc::new(Executor::new(8));
    let app = App::with_host(executor.clone(), move |runtime| Delayed {
        memory: Memory::new(runtime),
        wire: copy,
    });
    let root = app.scope().cage(None);
    let context = app.scope().cage(None);
    app.mount(responsive_root(500.).node_ref(root).context_ref(context))
        .unwrap();
    assert_eq!(
        app.runtime_stats().tasks,
        1,
        "spawn does not poll during the effect"
    );
    assert!(wire.borrow().requests.is_empty());
    executor.observe(Waker::noop());
    while executor.poll_one() {}
    app.flush().unwrap();
    assert_eq!(wire.borrow().requests.len(), 1);
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(app.stats().queries, 1);
    let context = context.get().unwrap();
    assert_eq!(context.pending_queries().unwrap(), 1);
    app.dispatch(
        root.get().unwrap(),
        Event::Resize {
            size: Rect {
                x: 0.,
                y: 0.,
                width: 1000.,
                height: 20.,
            },
            viewport: Rect {
                x: 0.,
                y: 0.,
                width: 1000.,
                height: 20.,
            },
        },
    )
    .unwrap();
    app.flush().unwrap();
    let (token, applied) = wire.borrow().requests[0];
    wire.borrow_mut().inputs.push(Input::Response {
        token,
        applied,
        result: Ok(QueryValue::Bounds(Rect {
            x: 0.,
            y: 0.,
            width: 300.,
            height: 20.,
        })),
    });
    app.pump().unwrap();
    assert_eq!(context.width().unwrap(), 1000.);
    assert_eq!(context.breakpoint().unwrap(), Breakpoint::Large);
    assert_eq!(context.pending_queries().unwrap(), 0);
    assert_eq!(wire.borrow().requests.len(), 1);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(app.stats().queries, 0);
    assert!(context.width().is_err());
    let next_context = app.scope().cage(None);
    app.mount(responsive_root(500.).context_ref(next_context))
        .unwrap();
    executor.observe(Waker::noop());
    while executor.poll_one() {}
    app.flush().unwrap();
    assert_eq!(wire.borrow().requests.len(), 2);
    assert_eq!(app.stats().queries, 1);
    let retired = next_context.get().unwrap();
    let (token, applied) = wire.borrow().requests[1];
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert!(retired.pending_queries().is_err());
    wire.borrow_mut().inputs.push(Input::Response {
        token,
        applied,
        result: Ok(QueryValue::Bounds(Rect {
            x: 0.,
            y: 0.,
            width: 900.,
            height: 20.,
        })),
    });
    app.pump().unwrap();
    assert_eq!(app.stats().queries, 0);
    app.mount(responsive_root(500.)).unwrap();
    assert_eq!(app.runtime_stats().tasks, 1);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
    executor.observe(Waker::noop());
    while executor.poll_one() {}
    assert_eq!(wire.borrow().requests.len(), 2);
}
