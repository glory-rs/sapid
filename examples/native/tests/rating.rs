use glory::blitz::{Raster, Renderer, shell::WindowApp};
use glory::prelude::*;
use glory::render::{
    CheckMode, CheckProps, Descriptor, Error, FormControl, Input, Mutation, NodeId, NodeIds,
    Props, Query, QueryToken, QueryValue, Rect, Render, RuntimeId, SemanticId, Semantics,
};
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &App, id: NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let reply = result.clone();
    app.request(Query::Bounds(id), move |v| {
        *reply.borrow_mut() = Some(v.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
fn parts(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending: Vec<_> = tree.roots().iter().rev().copied().collect();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|c| c == "sapid-rating-display-part")
        {
            result.push(id)
        }
        pending.extend(node.children.iter().rev().copied())
    }
    result
}
fn pixel(raster: &Raster, x: f64, y: f64) -> [u8; 4] {
    let offset =
        (y.round() as usize * raster.frame().unwrap().width as usize + x.round() as usize) * 4;
    raster.pixels()[offset..offset + 4].try_into().unwrap()
}
fn rgba(color: glory::style::Color) -> [u8; 4] {
    [color.0, color.1, color.2, color.3]
}
fn close(actual: [u8; 4], expected: [u8; 4]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            actual.abs_diff(expected) <= 2,
            "color channel {actual} != {expected}"
        );
    }
}
#[test]
fn native_display_rating_four_themes_paints_exact_half_star_pixels_with_stable_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let value = app.scope().cage(2.5);
        let reference = app.scope().cage(None);
        host.mount(
            sapid::rating("display")
                .name("Quality")
                .theme(theme.clone())
                .precision(sapid::RatingPrecision::Half)
                .display_only()
                .value(value)
                .node_ref(reference),
        )
        .unwrap();
        app.settle(16).unwrap();
        let members = parts(app);
        assert_eq!(members.len(), 10);
        let boxes: Vec<_> = members.iter().map(|id| bounds(app, *id)).collect();
        for rect in &boxes {
            assert!(
                (rect.width - 15.).abs() < 0.1 && (rect.height - 30.).abs() < 0.1,
                "{rect:?}"
            );
        }
        for adjacent in boxes.windows(2) {
            assert!(
                (adjacent[1].x - adjacent[0].x - adjacent[0].width).abs() < 0.1,
                "half-star gap: {boxes:?}"
            );
        }
        let root = reference.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(root, glory::html::Part::Main)
            .unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        close(
            pixel(&raster, boxes[4].x + 10.5, boxes[4].y + 15.),
            rgba(theme.accent),
        );
        close(
            pixel(&raster, boxes[5].x + 4.5, boxes[5].y + 15.),
            rgba(theme.surface),
        );
        value.revise(|v| *v = 3.);
        app.settle(16).unwrap();
        raster.draw(host.renderer()).unwrap();
        close(
            pixel(&raster, boxes[5].x + 4.5, boxes[5].y + 15.),
            rgba(theme.accent),
        );
        assert_eq!(
            app.tree()
                .node(root)
                .unwrap()
                .descriptor
                .view()
                .name
                .as_deref(),
            Some("Quality: 3 of 5")
        );
        assert_eq!(
            host.renderer()
                .engine_id(root, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        app.clear().unwrap();
        assert!(reference.get().is_none());
    }
}
#[test]
fn native_interactive_and_readonly_rating_keep_the_explicit_radio_rejection() {
    for readonly in [false, true] {
        let host = WindowApp::new();
        let error = host
            .mount(sapid::rating("interactive").readonly(readonly))
            .unwrap_err();
        assert!(
            matches!(
                error,
                glory::ui::Error::Render(Error::Unsupported("native radio groups"))
            ),
            "{error:?}"
        );
        host.app().clear().unwrap();
    }
}
#[test]
fn native_radio_preflight_and_direct_rejection_preserve_prefix_query_watermark() {
    let runtime = RuntimeId::new();
    let mut ids = NodeIds::new(runtime);
    let first = ids.allocate();
    let radio_id = ids.allocate();
    let mut host = Renderer::new(runtime);
    let mut radio = Descriptor::new(Props::Check(CheckProps {
        mode: CheckMode::Radio,
        checked: true,
        ..Default::default()
    }));
    radio.semantics = Some(Box::new(Semantics {
        check_group: Some(SemanticId::new("quality").unwrap()),
        form_control: Some(FormControl {
            name: "quality".into(),
            value: Some("2.5".into()),
        }),
        ..Default::default()
    }));
    assert_eq!(
        host.preflight(radio.view(), None),
        Err(Error::Unsupported("native radio groups"))
    );
    assert_eq!(
        host.apply(&[
            Mutation::Create {
                id: first,
                node: Box::new(Descriptor::default())
            },
            Mutation::Place {
                id: first,
                parent: None,
                before: None
            },
            Mutation::Create {
                id: radio_id,
                node: Box::new(radio)
            }
        ]),
        Err(Error::Unsupported("native radio groups"))
    );
    assert_eq!(host.stats().nodes, 1);
    assert_eq!(host.stats().applied, 2);
    host.request(Query::Bounds(first), QueryToken(7341))
        .unwrap();
    let mut inputs = Vec::new();
    host.drain(&mut inputs);
    assert!(matches!(
        inputs.as_slice(),
        [Input::Response {
            applied: 2,
            token: QueryToken(7341),
            result: Ok(QueryValue::Bounds(_))
        }]
    ));
}
