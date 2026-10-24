use glory::render::{memory::Memory, *};
use glory::ui::App;
use sapid::{FloatingIndicatorContext, floating_indicator, floating_indicator_target};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut found = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            found.push(id)
        }
        todo.extend(node.children.iter().rev());
    }
    found
}
fn drawing(app: &App) -> GraphicProps {
    let tree = app.tree();
    let node = tree
        .node(nodes(app, "sapid-floating-indicator-shape")[0])
        .unwrap();
    let Props::Graphic(value) = &node.descriptor.view().props else {
        panic!("graphic")
    };
    value.clone()
}
fn settle(app: &App, executor: &glory::core::executor::Executor) {
    for _ in 0..12 {
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
    panic!("floating observation did not quiesce")
}
#[derive(Default)]
struct Wire {
    defer: Cell<bool>,
    inputs: RefCell<Vec<Input>>,
    requests: RefCell<Vec<(QueryToken, u64)>>,
    focus: Cell<usize>,
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
        let node = self.memory.node(query.node())?;
        let applied = self.memory.receipt().applied;
        self.wire.requests.borrow_mut().push((token, applied));
        if self.wire.defer.get() {
            return Ok(());
        }
        let area = node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|class| class == "sapid-floating-area");
        let second = node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|class| class == "target-b");
        let rect = if area {
            Rect {
                x: 100.,
                y: 50.,
                width: 300.,
                height: 100.,
            }
        } else {
            Rect {
                x: if second { 220. } else { 120. },
                y: 60.,
                width: 80.,
                height: 40.,
            }
        };
        self.wire.inputs.borrow_mut().push(Input::Response {
            token,
            applied,
            result: Ok(QueryValue::Bounds(rect)),
        });
        Ok(())
    }
    fn action(&mut self, _: Action) -> Result<(), Error> {
        self.wire.focus.set(self.wire.focus.get() + 1);
        panic!("decoration must never issue an action")
    }
    fn drain(&mut self, out: &mut Vec<Input>) {
        out.extend(self.wire.inputs.take())
    }
}
type Fixture = (
    App,
    Rc<glory::core::executor::Executor>,
    Rc<Wire>,
    FloatingIndicatorContext,
    glory::Cage<Option<String>>,
    glory::Cage<u16>,
);
fn fixture() -> Fixture {
    let executor = Rc::new(glory::core::executor::Executor::new(8));
    let wire = Rc::new(Wire::default());
    let output = wire.clone();
    let app = App::with_host(executor.clone(), move |runtime| Host {
        memory: Memory::new(runtime),
        wire: output,
    });
    let active = app.scope().cage(Some("a".to_owned()));
    let duration = app.scope().cage(200u16);
    let context = app.scope().cage(None);
    app.mount(
        floating_indicator("selection")
            .active(active)
            .duration(duration)
            .context_ref(context)
            .content(|context| {
                glory::ui::row().fill((
                    floating_indicator_target(context.clone(), "a")
                        .class("target-a")
                        .fill(glory::ui::button().label("First")),
                    floating_indicator_target(context, "b")
                        .class("target-b")
                        .fill(glory::ui::input().value("Draft")),
                ))
            }),
    )
    .unwrap();
    (
        app,
        executor,
        wire,
        context.get().unwrap(),
        active,
        duration,
    )
}
#[test]
fn two_real_queries_follow_only_the_current_registered_target_without_focus_or_selection_semantics()
 {
    let (app, executor, wire, context, active, _) = fixture();
    settle(&app, &executor);
    assert_eq!(context.registered(), 2);
    let first = drawing(&app);
    assert_eq!(first.width, 300.);
    assert_eq!(first.height, 100.);
    assert_eq!(first.alternative, None);
    assert_eq!(first.transition, None);
    assert_eq!(
        first.commands[0],
        PathCommand::Move(GraphicPoint::new(20., 10.))
    );
    let input = nodes(&app, "target-b")[0];
    active.revise(|value| *value = Some("b".to_owned()));
    app.flush().unwrap();
    settle(&app, &executor);
    let second = drawing(&app);
    assert_eq!(
        second.commands[0],
        PathCommand::Move(GraphicPoint::new(120., 10.))
    );
    assert_eq!(
        second.transition,
        Some(GraphicTransition {
            duration_millis: 200
        })
    );
    assert_eq!(nodes(&app, "target-b"), [input]);
    assert_eq!(wire.focus.get(), 0);
    for id in nodes(&app, "sapid-floating-target") {
        let tree = app.tree();
        let node = tree.node(id).unwrap().descriptor.view();
        assert_eq!(node.role, None);
        assert_eq!(node.focus, FocusMode::Auto);
        assert!(
            node.semantics
                .as_ref()
                .is_none_or(|value| value.selected.is_none())
        );
    }
}
#[test]
fn old_queries_are_revoked_on_new_active_and_owner_retirement_without_retaining_visit_history()
{
    let (app, executor, wire, context, active, _) = fixture();
    settle(&app, &executor);
    wire.defer.set(true);
    wire.requests.borrow_mut().clear();
    context.refresh().unwrap();
    for _ in 0..8 {
        if !executor.poll_one() {
            break;
        }
    }
    assert_eq!(app.stats().queries, 2);
    active.revise(|value| *value = Some("b".to_owned()));
    app.flush().unwrap();
    assert_eq!(app.stats().queries, 0);
    for (token, applied) in wire.requests.take() {
        wire.inputs.borrow_mut().push(Input::Response {
            token,
            applied,
            result: Ok(QueryValue::Bounds(Rect {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
            })),
        })
    }
    wire.defer.set(false);
    settle(&app, &executor);
    assert_eq!(
        drawing(&app).commands[0],
        PathCommand::Move(GraphicPoint::new(120., 10.))
    );
    app.clear().unwrap();
    assert_eq!(context.registered(), 0);
    assert!(context.refresh().is_err());
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.runtime_stats().tasks, 0);
}
#[test]
fn external_place_uses_current_staged_ancestry_and_hides_a_target_that_left_its_area() {
    let (app, executor, _, context, _, _) = fixture();
    settle(&app, &executor);
    let other = app.scope().create(Descriptor::new(Props::View)).unwrap();
    let target = nodes(&app, "target-a")[0];
    app.scope().place(target, Some(other), None).unwrap();
    app.flush().unwrap();
    context.refresh().unwrap();
    for _ in 0..8 {
        if !executor.poll_one() {
            break;
        }
    }
    assert!(app.pump().is_err());
    assert_eq!(app.stats().queries, 0);
    let graphic = nodes(&app, "sapid-floating-indicator-marker")[0];
    assert!(
        app.tree()
            .node(graphic)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
}
#[test]
fn finite_duration_program_updates_retain_the_last_accepted_drawing_and_absent_active_hides() {
    let (app, executor, _, _, active, duration) = fixture();
    settle(&app, &executor);
    active.revise(|value| *value = Some("b".to_owned()));
    app.flush().unwrap();
    settle(&app, &executor);
    duration.revise(|value| *value = 501);
    assert!(app.flush().is_err());
    assert_eq!(duration.get(), 501);
    assert_eq!(drawing(&app).transition.unwrap().duration_millis, 200);
    duration.revise(|value| *value = 0);
    app.flush().unwrap();
    assert_eq!(drawing(&app).transition, None);
    active.revise(|value| *value = Some("absent".to_owned()));
    app.flush().unwrap();
    settle(&app, &executor);
    assert!(
        app.tree()
            .node(nodes(&app, "sapid-floating-indicator-marker")[0])
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .hidden
    );
}
#[test]
fn unknown_hosts_keep_deterministic_hidden_ssr_and_target_registration_is_bounded() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let first = glory::ssr::snapshot(
            floating_indicator("ssr")
                .theme(theme.clone())
                .active(Some("a".to_owned()))
                .content(|context| {
                    floating_indicator_target(context, "a")
                        .fill(glory::ui::text().content("One"))
                }),
        )
        .unwrap();
        let second = glory::ssr::snapshot(
            floating_indicator("ssr")
                .theme(theme)
                .active(Some("a".to_owned()))
                .content(|context| {
                    floating_indicator_target(context, "a")
                        .fill(glory::ui::text().content("One"))
                }),
        )
        .unwrap();
        assert_eq!(first.html, second.html);
        assert!(first.html.contains("sapid-floating-indicator-shape"));
        assert!(!first.html.contains("aria-selected"));
    }
    let app = App::memory();
    assert!(
        app.mount(floating_indicator("bounded").content(|context| {
            glory::ui::col().fill(
                (0..129)
                    .map(move |index| {
                        floating_indicator_target(context.clone(), format!("key-{index}"))
                    })
                    .collect::<Vec<_>>(),
            )
        }))
        .is_err()
    );
    assert!(
        App::memory()
            .mount(floating_indicator("duration").duration(501u16))
            .is_err()
    );
}
