use glory::render::{memory::Memory, *};
use glory::ui::App;
use sapid::{marquee, rolling_number};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            result.push(id);
        }
        todo.extend(node.children.iter().rev());
    }
    result
}
fn motion(app: &App, class: &str) -> Option<TranslationMotion> {
    app.tree()
        .node(nodes(app, class)[0])
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_deref()
        .and_then(|s| s.translation)
}
fn accessible(app: &App) -> String {
    let tree = app.tree();
    let id = nodes(app, "sapid-visually-hidden")[0];
    let Props::Text(text) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    text.content.clone()
}
#[derive(Default)]
struct Wire {
    requests: Cell<usize>,
    defer: Cell<bool>,
    width: Cell<f64>,
    inputs: RefCell<Vec<Input>>,
}
struct Host {
    memory: Memory,
    wire: Rc<Wire>,
}
impl Render for Host {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            measure: true,
            ..Default::default()
        }
    }
    fn apply(&mut self, mutations: &[Mutation]) -> Result<Receipt, Error> {
        self.memory.apply(mutations)
    }
    fn request(&mut self, query: Query, token: QueryToken) -> Result<(), Error> {
        self.wire.requests.set(self.wire.requests.get() + 1);
        if !self.wire.defer.get() {
            let first = self
                .memory
                .node(query.node())?
                .descriptor
                .view()
                .classes
                .iter()
                .any(|value| value == "sapid-marquee-first");
            self.wire.inputs.borrow_mut().push(Input::Response {
                token,
                applied: self.memory.receipt().applied,
                result: Ok(QueryValue::Bounds(Rect {
                    x: 100.,
                    y: 200.,
                    width: if first { self.wire.width.get() } else { 180. },
                    height: 24.,
                })),
            });
        }
        Ok(())
    }
    fn drain(&mut self, inputs: &mut Vec<Input>) {
        inputs.extend(self.wire.inputs.take());
    }
    fn action(&mut self, _: Action) -> Result<(), Error> {
        panic!("passive text motion must not change focus or scroll")
    }
}
fn settle(app: &App, executor: &glory::core::executor::Executor) {
    for _ in 0..16 {
        for _ in 0..16 {
            if !executor.poll_one() {
                break;
            }
        }
        app.pump().unwrap();
        if executor.is_empty() && app.stats().queries == 0 {
            return;
        }
    }
    panic!("text motion did not quiesce");
}
fn fixture() -> (App, Rc<glory::core::executor::Executor>, Rc<Wire>) {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let wire = Rc::new(Wire::default());
    wire.width.set(420.);
    let output = wire.clone();
    let app = App::with_host(executor.clone(), move |runtime| Host {
        memory: Memory::new(runtime),
        wire: output,
    });
    (app, executor, wire)
}
#[test]
fn marquee_measures_only_two_current_boxes_pause_and_stop_use_host_motion() {
    let (app, executor, wire) = fixture();
    let paused = app.scope().cage(false);
    let animated = app.scope().cage(true);
    let revision = app.scope().cage(0u64);
    app.mount(
        marquee()
            .text("A finite long status message")
            .paused(paused)
            .animated(animated)
            .measurement_revision(revision)
            .gap(20.),
    )
    .unwrap();
    settle(&app, &executor);
    assert_eq!(wire.requests.get(), 2);
    let first = motion(&app, "sapid-marquee-track").unwrap();
    assert_eq!(
        first.kind,
        TranslationKind::Loop {
            from: TranslationOffset::ZERO,
            to: TranslationOffset::new(-440., 0.),
            period_millis: 8000
        }
    );
    paused.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(motion(&app, "sapid-marquee-track").unwrap().paused);
    assert_eq!(wire.requests.get(), 2);
    animated.revise(|value| *value = false);
    app.flush().unwrap();
    settle(&app, &executor);
    assert!(motion(&app, "sapid-marquee-track").is_none());
    assert_eq!(app.stats().queries, 0);
    animated.revise(|value| *value = true);
    wire.width.set(90.);
    revision.revise(|value| *value += 1);
    app.flush().unwrap();
    settle(&app, &executor);
    assert!(motion(&app, "sapid-marquee-track").is_none());
    assert_eq!(wire.requests.get(), 4);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn marquee_replaces_pending_pair_and_retirement_cancels_without_visited_history() {
    let (app, executor, wire) = fixture();
    wire.defer.set(true);
    let text = app.scope().cage("Old status".to_string());
    let animated = app.scope().cage(true);
    app.mount(marquee().text(text).animated(animated)).unwrap();
    for _ in 0..4 {
        executor.poll_one();
    }
    assert_eq!(app.stats().queries, 2);
    text.revise(|value| *value = "New status".into());
    app.flush().unwrap();
    for _ in 0..4 {
        executor.poll_one();
    }
    assert_eq!(app.stats().queries, 2);
    assert_eq!(wire.requests.get(), 4);
    animated.revise(|value| *value = false);
    app.flush().unwrap();
    assert_eq!(app.stats().queries, 0);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
}
#[test]
fn text_limits_and_reactive_rejection_preserve_accepted_visual_and_static_value() {
    let app = App::memory();
    assert!(app.mount(marquee().text("x".repeat(4097))).is_err());
    assert_eq!(app.tree().len(), 0);
    let text = app.scope().cage("Accepted".to_string());
    app.mount(marquee().text(text)).unwrap();
    text.revise(|value| *value = "x".repeat(4097));
    assert!(app.flush().is_err());
    assert_eq!(accessible(&app), "Accepted");
    assert_eq!(text.with(String::len), 4097);
    for widget in [
        rolling_number().value(f64::NAN),
        rolling_number().value(1e25),
        rolling_number().duration_millis(2001u16),
        rolling_number()
            .min_fraction_digits(3)
            .max_fraction_digits(1),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert_eq!(app.tree().len(), 0);
    }
    for gap in [f64::NAN, -1., 4097.] {
        let app = App::memory();
        assert!(app.mount(marquee().gap(gap)).is_err());
    }
}
#[test]
fn rolling_reuses_formatter_and_keyed_reels_without_proposing_any_source_change() {
    let app = App::memory();
    let source = app.scope().cage(123.5);
    let duration = app.scope().cage(500u16);
    app.mount(
        rolling_number()
            .value(source)
            .locale(sapid::NumberLocale::DeDe)
            .min_fraction_digits(2)
            .max_fraction_digits(2)
            .duration_millis(duration),
    )
    .unwrap();
    assert_eq!(accessible(&app), "123,50");
    let before = nodes(&app, "sapid-rolling-number-reel");
    assert_eq!(before.len(), 5);
    source.revise(|value| *value = 129.75);
    app.flush().unwrap();
    assert_eq!(accessible(&app), "129,75");
    assert_eq!(before, nodes(&app, "sapid-rolling-number-reel"));
    assert_eq!(source.get(), 129.75);
    source.revise(|value| *value = f64::INFINITY);
    assert!(app.flush().is_err());
    assert_eq!(accessible(&app), "129,75");
    duration.revise(|value| *value = 2001);
    assert!(app.flush().is_err());
    assert_eq!(
        motion(&app, "sapid-rolling-number-reel").unwrap().kind,
        TranslationKind::To {
            target: TranslationOffset::new(0., -24.),
            duration_millis: 500
        }
    );
}
#[test]
fn rolling_sign_length_and_literal_changes_keep_only_current_bounded_owners() {
    let app = App::memory();
    let source = app.scope().cage(-9.);
    app.mount(
        rolling_number()
            .value(source)
            .max_fraction_digits(0)
            .grouping(false),
    )
    .unwrap();
    let count = app.runtime_stats().nodes;
    for value in [123., -9., 123., -9.] {
        source.revise(|old| *old = value);
        app.flush().unwrap();
        assert!(nodes(&app, "sapid-rolling-number-reel").len() <= 3);
    }
    assert_eq!(accessible(&app), "-9");
    assert_eq!(app.runtime_stats().nodes, count);
    app.clear().unwrap();
    assert_eq!(app.runtime_stats().tasks, 0);
}
#[test]
fn four_theme_ssr_and_unstyled_keep_deterministic_single_static_text_and_no_initial_motion() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        for unstyled in [false, true] {
            let marquee = marquee().theme(theme.clone()).text("Static first shape");
            let marquee = if unstyled {
                marquee.unstyled()
            } else {
                marquee
            };
            let first = glory::ssr::snapshot(marquee).unwrap();
            assert!(
                first.html.contains("sapid-marquee-copy")
                    && !first.html.contains("data-gly-translation=")
            );
            let rolling = rolling_number().theme(theme.clone()).value(12.);
            let rolling = if unstyled {
                rolling.unstyled()
            } else {
                rolling
            };
            let first = glory::ssr::snapshot(rolling).unwrap();
            assert!(
                first.html.contains("transform:translate(0px,-24px);")
                    && first.html.contains("data-gly-inert")
            );
        }
    }
}
