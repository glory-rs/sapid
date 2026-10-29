//! Native engine acceptance kept outside Sapid's default dependency graph.
#[cfg(test)]
mod tests {
    use glory::apace::shell::WindowApp;
    use glory::prelude::*;
    use glory::render::{Event, NodeId, Props, Query, QueryValue};
    use std::{cell::RefCell, rc::Rc};

    fn find(app: &App, class: &str) -> NodeId {
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        while let Some(id) = pending.pop() {
            let node = tree.node(id).unwrap();
            if node.descriptor.view().classes.iter().any(|c| c == class) {
                return id;
            }
            pending.extend(node.children.iter().copied());
        }
        panic!("missing {class}")
    }
    fn bounds(app: &App, id: NodeId) -> glory::render::Rect {
        let result = Rc::new(RefCell::new(None));
        let reply = result.clone();
        app.request(Query::Bounds(id), move |value| {
            *reply.borrow_mut() = Some(value.unwrap())
        })
        .unwrap();
        app.pump().unwrap();
        let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
            panic!("bounds")
        };
        rect
    }
    #[test]
    fn fixed_rows_and_group_actions_use_real_native_layout_and_retire() {
        let host = WindowApp::new();
        host.mount(
            col().style(Style::new().width(Length::Px(480.0))).fill((
                sapid::field("bio")
                    .label("Biography")
                    .control(|context| sapid::textarea().field(context).rows(4)),
                sapid::input_group()
                    .start(sapid::input_addon().content("https://"))
                    .input(sapid::text_field().name("Website").default_value("example"))
                    .end(sapid::input_section().fill(sapid::button().label("Visit"))),
            )),
        )
        .unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        let input = find(app, "sapid-textarea");
        let style = app
            .tree()
            .node(input)
            .unwrap()
            .descriptor
            .view()
            .style
            .clone();
        let Some(Length::Px(expected)) = style.height else {
            panic!("row height")
        };
        assert!((bounds(app, input).height - expected).abs() < 0.1);
        assert!(bounds(app, find(app, "sapid-input-addon")).width > 0.0);
        assert!(bounds(app, find(app, "sapid-button")).width > 0.0);
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
    #[test]
    fn password_toggle_preserves_native_engine_identity_and_raw_value() {
        let host = WindowApp::new();
        host.mount(sapid::field("secret").label("Secret").control(|context| {
            sapid::password_input()
                .field(context)
                .default_value("秘密🦀")
        }))
        .unwrap();
        let app = host.app();
        app.settle(8).unwrap();
        let input = find(app, "sapid-password-input");
        let toggle = find(app, "sapid-password-toggle");
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        let before = app
            .tree()
            .node(input)
            .unwrap()
            .descriptor
            .view()
            .props
            .clone();
        for expected in [Purpose::Text, Purpose::Password] {
            app.dispatch(toggle, Event::Activate).unwrap();
            app.settle(8).unwrap();
            assert_eq!(
                host.renderer()
                    .engine_id(input, glory::html::Part::Main)
                    .unwrap(),
                physical
            );
            assert!(
                matches!(&app.tree().node(input).unwrap().descriptor.view().props,Props::Input(p) if p.purpose==expected)
            );
            host.renderer().inspect(|doc| {
                assert_eq!(
                    doc.get_node(physical)
                        .unwrap()
                        .element_data()
                        .unwrap()
                        .text_input_data()
                        .unwrap()
                        .editor
                        .raw_text(),
                    "秘密🦀"
                )
            });
        }
        assert_eq!(
            app.tree()
                .node(input)
                .unwrap()
                .descriptor
                .view()
                .props
                .clone(),
            before
        );
        app.clear().unwrap();
        assert!(app.tree().is_empty());
    }
    #[test]
    fn autosize_has_an_explicit_native_refusal_and_does_not_leave_partial_controls() {
        let host = WindowApp::new();
        assert!(matches!(
            host.mount(sapid::textarea().autosize(true)),
            Err(glory::ui::Error::Render(
                glory::render::Error::UnsupportedStyle { .. }
            ))
        ));
        assert!(host.app().tree().is_empty());
        host.mount(sapid::textarea().rows(2)).unwrap();
        host.app().settle(8).unwrap();
        assert!(bounds(host.app(), find(host.app(), "sapid-textarea")).height > 0.0);
    }
}
