use glory::render::{Event, Props};
use glory::{Widget, core::Scope, view};
use sapid::Button;
use std::{cell::Cell, rc::Rc};
#[path = "../examples/fixtures/dsl_demo.rs"]
mod fixture;

#[test]
fn shared_example_renders_the_same_widget_api() {
    let html = glory::ssr::render(fixture::demo(sapid::Theme::light())).unwrap();
    for value in [
        "dsl-raw",
        "dsl-type",
        "dsl-factory",
        "dsl-concrete",
        "dsl-editor",
        "Shared Widget branch",
    ] {
        assert!(html.contains(value), "missing {value}");
    }
}

#[glory::component]
fn Custom(scope: &Scope, value: glory::Cage<String>, calls: Rc<Cell<usize>>) -> impl Widget {
    calls.set(calls.get() + 1);
    let label = scope.enter(|| value.map(|v| format!("custom {v}")));
    sapid::button().label(label)
}

#[glory::component]
fn Panel(children: glory::ui::Children) -> impl Widget {
    sapid::card().fill(children)
}

#[test]
fn sapid_paths_aliases_and_custom_functions_are_widgets_on_semantic_hosts() {
    use sapid::Button as Save;
    let html = glory::ssr::render(view! {<sapid::Button label="full path"/>}).unwrap();
    assert!(html.contains("full path"));
    let html = glory::ssr::render(view! {<Save label="alias"/>}).unwrap();
    assert!(html.contains("alias"));
    let html = glory::ssr::render(view! {<Panel><Button label="child"/></Panel>}).unwrap();
    assert!(html.contains("sapid-card"));
    assert!(html.contains("child"));
    let html = glory::ssr::render(
        view! {<Panel><Button label="first"/><Button label="second"/></Panel>},
    )
    .unwrap();
    assert!(html.contains("first") && html.contains("second"));
    let html = glory::ssr::render(view! {<><Button label="first"/><Button label="second"/></>})
        .unwrap();
    assert!(html.contains("first") && html.contains("second"));
    let html = glory::ssr::render(view! {
        <sapid::ActionIcon init={sapid::action_icon("Refresh")} icon={glory::ui::text().content("+")}/>
    }).unwrap();
    assert!(html.contains("Refresh"));
}

#[test]
fn function_widget_runs_once_and_retires_derived_state_with_its_owner() {
    let app = glory::App::memory();
    let value = app.scope().cage("one".to_owned());
    let baseline = app.runtime_stats();
    let calls = Rc::new(Cell::new(0));
    app.mount(view! {<Custom value={value} calls={calls.clone()}/>})
        .unwrap();
    assert_eq!(calls.get(), 1);
    let root = app.tree().roots()[0];
    assert!(matches!(
        app.tree().node(root).unwrap().descriptor.view().props,
        Props::Button(_)
    ));
    value.revise(|v| *v = "two".into());
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.runtime_stats().nodes, baseline.nodes);
    value.revise(|v| *v = "three".into());
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
}

#[test]
fn dsl_builder_preserves_typed_controlled_events() {
    let app = glory::App::memory();
    let pressed = Rc::new(Cell::new(0));
    let count = pressed.clone();
    app.mount(view! {<Button label="press" on_press={move || count.set(count.get()+1)}/>})
        .unwrap();
    let root = app.tree().roots()[0];
    app.dispatch(root, Event::Activate).unwrap();
    assert_eq!(pressed.get(), 1);
}

fn opaque_button() -> impl Widget {
    sapid::button().label("factory opaque")
}

#[test]
fn mixed_html_widgets_and_factory_calls_share_ssr() {
    let html = glory::ssr::render_with(|scope| {
        let count = scope.cage(3usize);
        let shown = scope.cage(true);
        view! {
            <div class="mixed" class:active={shown}>
                <span>{count}</span>
                <button on:click={move||count.revise(|v|*v+=1)}>"raw"</button>
                <Button label="semantic"/>
                <{opaque_button()}/>
                <{sapid::button()} label="configured factory"/>
                <Panel><div><Button label="nested mixed"/></div></Panel>
                <Show when={move||shown.get()}><span>"visible"</span></Show>
            </div>
        }
    })
    .unwrap();
    for expected in [
        "class=\"mixed active\"",
        "<!--gly:dt-->3",
        "raw",
        "semantic",
        "factory opaque",
        "configured factory",
        "nested mixed",
        "visible",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }
    assert!(html.contains("data-gly-dclick"));
    assert!(
        !html.contains("data-gly-display-node"),
        "SSR must not serialize runtime identities"
    );
}

#[test]
fn factory_tags_defer_opaque_calls_and_support_concrete_builder_children() {
    fn factory(calls: Rc<Cell<usize>>, label: &str) -> impl Widget {
        calls.set(calls.get() + 1);
        sapid::button().label(label.to_owned())
    }
    let calls = Rc::new(Cell::new(0));
    let captured = calls.clone();
    let config = view! {<{factory(captured, "ordinary arguments")}/>};
    assert_eq!(calls.get(), 0);
    let html = glory::ssr::render(config).unwrap();
    assert_eq!(calls.get(), 1);
    assert!(html.contains("ordinary arguments"));
    let html = glory::ssr::render(view! {
        <{sapid::card()} class="factory-card">
            <div><{opaque_button()}/></div>
        </>
    })
    .unwrap();
    assert!(html.contains("factory-card") && html.contains("factory opaque"));
}
