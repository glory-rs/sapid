extern crate glory as g;
#[path = "../../../../glory/examples/display-web/src/mixed.rs"]
mod mixed;
#[path = "../../fixtures/dsl_demo.rs"]
mod sapid_dsl;
use blitz_dom::{Document, NodeId as EngineId};
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
    MouseEventButtons, Point, PointerCoords, UiEvent,
};
use glory::blitz::{Raster, shell::WindowApp};
use std::task::{Context, Waker};

fn settle(host: &WindowApp) {
    host.observe(&Context::from_waker(Waker::noop())).unwrap();
}
fn nodes(host: &WindowApp, class: &str) -> Vec<EngineId> {
    host.renderer().inspect(|doc| {
        let mut pending = vec![doc.root_node().id];
        let mut found = Vec::new();
        while let Some(id) = pending.pop() {
            let node = doc.get_node(id).unwrap();
            if node
                .element_data()
                .and_then(|e| e.attr(blitz_dom::LocalName::from("class")))
                .is_some_and(|value| value.split_ascii_whitespace().any(|c| c == class))
            {
                found.push(id);
            }
            pending.extend(node.children.iter().rev().copied());
        }
        found
    })
}
fn key(host: &mut WindowApp, key: &str, released: bool) {
    let event = BlitzKeyEvent {
        key: key.parse().unwrap(),
        code: Default::default(),
        modifiers: Default::default(),
        location: Default::default(),
        is_auto_repeating: false,
        is_composing: false,
        state: if released {
            KeyState::Released
        } else {
            KeyState::Pressed
        },
        text: None,
    };
    host.handle_ui_event(if released {
        UiEvent::KeyUp(event)
    } else {
        UiEvent::KeyDown(event)
    });
    settle(host);
}
fn click(host: &mut WindowApp, id: EngineId) {
    let rect = host
        .renderer()
        .inspect(|doc| doc.get_client_bounding_rect(id).unwrap());
    assert!(rect.width > 0. && rect.height > 0.);
    let x = (rect.x + rect.width / 2.) as f32;
    let y = (rect.y + rect.height / 2.) as f32;
    let pointer = |pressed| BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            client_x: x,
            client_y: y,
            screen_x: x,
            screen_y: y,
        },
        button: MouseEventButton::Main,
        buttons: if pressed {
            MouseEventButtons::Primary
        } else {
            MouseEventButtons::empty()
        },
        mods: Default::default(),
        details: Default::default(),
        element: Point::default(),
        active_pointers: Default::default(),
    };
    host.handle_ui_event(UiEvent::PointerMove(pointer(false)));
    host.handle_ui_event(UiEvent::PointerDown(pointer(true)));
    host.handle_ui_event(UiEvent::PointerUp(pointer(false)));
    settle(host);
}
#[test]
fn shared_mixed_tree_uses_real_native_layout_input_keyed_moves_paint_and_cleanup() {
    let mut host = WindowApp::new();
    host.renderer().set_viewport(1100, 900, 1.).unwrap();
    host.renderer().set_css(".mixed{padding:20px}.mixed>button{display:block;margin:4px}.row{display:flex}.active .count{color:red}");
    let model = mixed::Model::new(host.app().scope());
    host.mount(mixed::view(&model)).unwrap();
    settle(&host);
    let raw = nodes(&host, "raw")[0];
    let semantic = nodes(&host, "semantic")[0];
    let editor = nodes(&host, "editor")[0];
    let row = nodes(&host, "row-button")[0];
    let vector_row = nodes(&host, "vector-1")[0];
    let mut raster = Raster::new();
    assert!(raster.draw(host.renderer()).unwrap().rendered);
    let before = raster.pixels().to_vec();
    for id in [
        raw,
        semantic,
        nodes(&host, "opaque")[0],
        nodes(&host, "configured")[0],
    ] {
        click(&mut host, id);
    }
    assert_eq!(model.count.get(), 4);
    assert_eq!(model.bubbled.get(), 1);
    click(&mut host, raw);
    key(&mut host, "Enter", false);
    key(&mut host, " ", false);
    key(&mut host, " ", true);
    assert_eq!(model.count.get(), 7);
    key(&mut host, "Tab", false);
    assert_eq!(
        host.renderer().inspect(|doc| doc.get_focussed_node_id()),
        Some(semantic)
    );
    model.input.revise(|value| *value = "native 🦀".into());
    model.rows.swap(0, 1).unwrap();
    model.vector.revise(|rows| rows.swap(0, 1));
    model.visible.revise(|v| *v = false);
    settle(&host);
    assert_eq!(nodes(&host, "raw"), vec![raw]);
    assert_eq!(nodes(&host, "semantic"), vec![semantic]);
    assert_eq!(nodes(&host, "editor"), vec![editor]);
    assert_eq!(nodes(&host, "row-button")[1], row);
    assert_eq!(nodes(&host, "vector-1")[0], vector_row);
    assert!(nodes(&host, "branch").is_empty());
    assert_eq!(model.drops.get(), 1);
    host.renderer().inspect(|doc| {
        assert_eq!(
            doc.get_node(editor)
                .unwrap()
                .element_data()
                .unwrap()
                .text_input_data()
                .unwrap()
                .editor
                .raw_text(),
            "native 🦀"
        );
        assert!(
            !doc.get_node(nodes(&host, "mixed")[0])
                .unwrap()
                .attr(blitz_dom::LocalName::from("class"))
                .unwrap()
                .split_ascii_whitespace()
                .any(|c| c == "active")
        );
    });
    assert!(raster.draw(host.renderer()).unwrap().rendered);
    assert_ne!(raster.pixels(), before);
    host.app().clear().unwrap();
    settle(&host);
    assert!(host.app().tree().is_empty());
    assert!(nodes(&host, "mixed").is_empty());
}

#[test]
fn sapid_shared_dsl_example_uses_native_widgets_and_raw_event_targets() {
    let mut host = WindowApp::new();
    host.renderer().set_viewport(1100, 900, 1.).unwrap();
    host.mount(sapid_dsl::demo(sapid::Theme::light())).unwrap();
    settle(&host);
    let raw = nodes(&host, "dsl-raw")[0];
    let editor = nodes(&host, "dsl-editor")[0];
    for class in ["dsl-raw", "dsl-type", "dsl-factory", "dsl-concrete"] {
        let id = nodes(&host, class)[0];
        click(&mut host, id);
    }
    let tree = host.app().tree();
    let mut pending = tree.roots().to_vec();
    let mut updated = false;
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        updated |= matches!(node.descriptor.view().props,
            glory::render::Props::Button(value) if value.label == "Imported type 4");
        pending.extend(&node.children);
    }
    assert!(updated);
    drop(tree);
    let toggle = nodes(&host, "dsl-toggle")[0];
    click(&mut host, toggle);
    assert!(nodes(&host, "dsl-branch").is_empty());
    click(&mut host, toggle);
    assert_eq!(nodes(&host, "dsl-branch").len(), 1);
    assert_eq!(nodes(&host, "dsl-raw"), [raw]);
    assert_eq!(nodes(&host, "dsl-editor"), [editor]);
    assert!(Raster::new().draw(host.renderer()).unwrap().rendered);
    host.app().clear().unwrap();
    settle(&host);
    assert!(nodes(&host, "dsl-demo").is_empty());
}
