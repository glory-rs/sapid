use glory::render::{Current, NodeId, Props};
use glory::ui::{App, Widget};
use sapid::{table_of_contents, toc_context, toc_document, toc_heading};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut stack = tree.roots().to_vec();
    let mut out = Vec::new();
    while let Some(id) = stack.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id)
        }
        stack.extend(n.children.iter().rev());
    }
    out
}
#[test]
fn registered_labels_and_explicit_order_update_real_nested_links_without_rebuilding() {
    let app = App::memory();
    let root = app.scope();
    let ctx = toc_context(root, "live").unwrap();
    let label = root.cage("Nested".to_owned());
    let order = root.cage(1u32);
    app.mount(toc_document(ctx.clone(), "doc").content(move |ctx| {
        glory::ui::col().fill((
            toc_heading(ctx.clone(), "a", 1)
                .content("Overview")
                .order(order),
            toc_heading(ctx.clone(), "b", 3).content(label).order(2),
            toc_heading(ctx, "c", 2).content("Details").order(3),
        ))
    }))
    .unwrap();
    app.mount(table_of_contents(ctx.clone(), "nav")).unwrap();
    assert_eq!(ctx.len(), 3);
    let links = nodes(&app, "sapid-anchor");
    assert_eq!(links.len(), 3);
    let current = ctx.current(root);
    assert!(current.get().is_some());
    let initial = links.clone();
    label.revise(|v| *v = "Nested updated".into());
    app.flush().unwrap();
    assert_eq!(nodes(&app, "sapid-anchor"), initial);
    order.revise(|v| *v = 4);
    app.flush().unwrap();
    assert_eq!(nodes(&app, "sapid-anchor").len(), 3);
    assert!(initial.iter().all(|id| app.tree().node(*id).is_ok()));
    for id in &initial {
        assert!(matches!(
            app.tree().node(*id).unwrap().descriptor.view().props,
            Props::Link(_)
        ));
    }
    let entries = ctx.entries(root).get();
    assert_eq!(
        entries.iter().map(|e| e.label.as_str()).collect::<Vec<_>>(),
        ["Nested updated", "Details", "Overview"]
    );
    app.clear().unwrap();
    assert!(ctx.is_empty());
    assert!(ctx.refresh().is_ok());
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().queries, 0);
}
#[test]
fn four_theme_ssr_is_deterministic_measureless_and_uses_location_current() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let renderer = std::rc::Rc::new(std::cell::RefCell::new(None));
        let saved = renderer.clone();
        let app = App::new(move |runtime| {
            let r = glory::ssr::Renderer::new(runtime);
            *saved.borrow_mut() = Some(r.clone());
            r
        });
        let root = app.scope();
        let ctx = toc_context(root, "ssr").unwrap();
        let t = theme.clone();
        app.mount(
            toc_document(ctx.clone(), "doc")
                .theme(theme.clone())
                .content(move |ctx| {
                    glory::ui::col().fill((
                        toc_heading(ctx.clone(), "a", 1)
                            .theme(t.clone())
                            .content("Start"),
                        toc_heading(ctx, "b", 4).theme(t.clone()).content("Nested"),
                    ))
                }),
        )
        .unwrap();
        app.mount(table_of_contents(ctx.clone(), "nav").theme(theme))
            .unwrap();
        let html = renderer.borrow().as_ref().unwrap().html().unwrap();
        assert!(html.contains("aria-current=\"location\""));
        assert!(html.contains("<h1"));
        assert!(html.contains("<h4"));
        assert!(html.contains("href=\"#sapid-toc-"));
        assert_eq!(app.stats().queries, 0);
        assert!(ctx.reveal(&ctx.entries(root).get()[0].id).is_err());
        let links = nodes(&app, "sapid-anchor");
        assert_eq!(
            links
                .iter()
                .filter(|id| app
                    .tree()
                    .node(**id)
                    .unwrap()
                    .descriptor
                    .view()
                    .semantics
                    .as_ref()
                    .unwrap()
                    .current
                    == Some(Current::Location))
                .count(),
            1
        );
    }
}
#[test]
fn invalid_levels_keys_labels_and_foreign_contexts_are_rejected() {
    for level in [0, 7, u8::MAX] {
        let app = App::memory();
        let ctx = toc_context(app.scope(), "level").unwrap();
        assert!(
            app.mount(toc_heading(ctx, "x", level).content("X"))
                .is_err()
        );
    }
    for label in ["", " ", "bad\nline", "bad\0label"] {
        let app = App::memory();
        let ctx = toc_context(app.scope(), "label").unwrap();
        assert!(app.mount(toc_heading(ctx, "x", 1).content(label)).is_err());
    }
    let app = App::memory();
    let ctx = toc_context(app.scope(), "owner").unwrap();
    let other = App::memory();
    assert!(
        other
            .mount(toc_heading(ctx, "x", 1).content("Foreign"))
            .is_err()
    );
}
#[test]
fn heading_keys_retire_without_historical_registry_and_limits_are_live() {
    let app = App::memory();
    let ctx = toc_context(app.scope(), "bounded").unwrap();
    let mut mounts = Vec::new();
    for i in 0..256 {
        mounts.push(
            toc_heading(ctx.clone(), format!("h{i}"), 1)
                .content("Heading")
                .mount(app.scope())
                .unwrap(),
        );
    }
    assert_eq!(ctx.len(), 256);
    assert!(
        toc_heading(ctx.clone(), "extra", 1)
            .content("Extra")
            .mount(app.scope())
            .is_err()
    );
    drop(mounts.pop());
    assert_eq!(ctx.len(), 255);
    let new = toc_heading(ctx.clone(), "extra", 1)
        .content("Extra")
        .mount(app.scope())
        .unwrap();
    assert_eq!(ctx.len(), 256);
    drop(new);
    drop(mounts);
    assert_eq!(ctx.len(), 0);
}
#[derive(Default)]
struct QueryWire {
    pending: std::cell::RefCell<Vec<(glory::render::Query, glory::render::QueryToken, u64)>>,
    inputs: std::cell::RefCell<Vec<glory::render::Input>>,
}
struct QueryRenderer {
    memory: glory::render::memory::Memory,
    wire: std::rc::Rc<QueryWire>,
}
impl glory::render::Render for QueryRenderer {
    fn style_profile(&self) -> glory::style::Profile {
        glory::style::Profile::RECORD
    }
    fn capabilities(&self) -> glory::render::Capabilities {
        glory::render::Capabilities {
            measure: true,
            scroll: true,
            focus: true,
            ..Default::default()
        }
    }
    fn apply(
        &mut self,
        m: &[glory::render::Mutation],
    ) -> Result<glory::render::Receipt, glory::render::Error> {
        glory::render::Render::apply(&mut self.memory, m)
    }
    fn request(
        &mut self,
        q: glory::render::Query,
        t: glory::render::QueryToken,
    ) -> Result<(), glory::render::Error> {
        self.wire
            .pending
            .borrow_mut()
            .push((q, t, self.memory.receipt().applied));
        Ok(())
    }
    fn drain(&mut self, out: &mut Vec<glory::render::Input>) {
        out.extend(self.wire.inputs.take())
    }
}
#[test]
fn measured_index_coalesces_one_live_batch_and_retirement_cancels_every_late_reply() {
    use glory::render::*;
    let executor = std::rc::Rc::new(glory::core::executor::Executor::new(8));
    let wire = std::rc::Rc::new(QueryWire::default());
    let saved = wire.clone();
    let app = App::with_host(executor.clone(), move |runtime| QueryRenderer {
        memory: memory::Memory::new(runtime),
        wire: saved,
    });
    let ctx = toc_context(app.scope(), "queries").unwrap();
    app.mount(toc_document(ctx.clone(), "doc").content(move |ctx| {
        glory::ui::component(move |scope| {
            for index in 0..256 {
                toc_heading(ctx.clone(), format!("h{index}"), 1)
                    .content(format!("Heading {index}"))
                    .show_in(scope)?;
            }
            Ok(())
        })
    }))
    .unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(
        ctx.current(app.scope()).get(),
        Some(ctx.entries(app.scope()).get()[0].id.clone())
    );
    for _ in 0..300 {
        ctx.refresh().unwrap();
    }
    assert!(executor.poll_one());
    assert_eq!(app.stats().queries, 258);
    assert_eq!(wire.pending.borrow().len(), 258);
    for _ in 0..300 {
        ctx.refresh().unwrap();
    }
    assert!(!executor.poll_one());
    assert_eq!(wire.pending.borrow().len(), 258);
    let pending = wire.pending.take();
    app.clear().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert_eq!(app.stats().query_owners, 0);
    assert_eq!(app.stats().query_targets, 0);
    assert!(ctx.is_empty());
    for (query, token, applied) in pending {
        let value = match query {
            Query::Bounds(_) => QueryValue::Bounds(Rect {
                x: 0.,
                y: 0.,
                width: 100.,
                height: 20.,
            }),
            Query::ScrollMetrics(_) => QueryValue::ScrollMetrics(ScrollMetrics {
                x: 0.,
                y: 0.,
                viewport_width: 100.,
                viewport_height: 100.,
                content_width: 100.,
                content_height: 1000.,
            }),
            _ => unreachable!(),
        };
        wire.inputs.borrow_mut().push(Input::Response {
            token,
            applied,
            result: Ok(value),
        });
    }
    app.pump().unwrap();
    assert_eq!(app.stats().queries, 0);
    assert!(ctx.current(app.scope()).get().is_none());
    assert!(executor.is_empty());
}
#[test]
fn all_four_heading_permutations_preserve_links_and_current_nested_parents() {
    let app = App::memory();
    let ctx = toc_context(app.scope(), "orders").unwrap();
    let orders = app.scope().cage([0u32, 1, 2, 3]);
    app.mount(toc_document(ctx.clone(), "doc").content(move |ctx| {
        glory::ui::component(move |scope| {
            for (index, level) in [1, 2, 3, 1].into_iter().enumerate() {
                toc_heading(ctx.clone(), format!("h{index}"), level)
                    .content(format!("Heading {index}"))
                    .order(orders.map(move |values| values[index]))
                    .show_in(scope)?;
            }
            Ok(())
        })
    }))
    .unwrap();
    app.mount(table_of_contents(ctx.clone(), "nav")).unwrap();
    let original = nodes(&app, "sapid-anchor");
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let wanted = [a, b, c, d];
                    if (0..4).any(|i| (i + 1..4).any(|j| wanted[i] == wanted[j])) {
                        continue;
                    }
                    let mut ranks = [0; 4];
                    for (index, key) in wanted.into_iter().enumerate() {
                        ranks[key] = index as u32;
                    }
                    orders.revise(|value| *value = ranks);
                    app.flush().unwrap();
                    let current = nodes(&app, "sapid-anchor");
                    assert!(original.iter().all(|id| current.contains(id)));
                    let entries = ctx.entries(app.scope()).get();
                    let actual = current
                        .iter()
                        .map(|id| {
                            let tree = app.tree();
                            let glory::render::Props::Link(link) =
                                tree.node(*id).unwrap().descriptor.view().props
                            else {
                                panic!("link")
                            };
                            link.target.clone()
                        })
                        .collect::<Vec<_>>();
                    let expected = entries
                        .iter()
                        .map(|entry| format!("#{}", entry.id.as_str()))
                        .collect::<Vec<_>>();
                    assert_eq!(actual, expected);
                }
            }
        }
    }
}
#[test]
fn controlled_navigation_current_is_read_only_and_does_not_change_the_observed_section() {
    let app = App::memory();
    let ctx = toc_context(app.scope(), "control").unwrap();
    app.mount(toc_document(ctx.clone(), "doc").content(move |ctx| {
        glory::ui::col().fill((
            toc_heading(ctx.clone(), "a", 1).content("A"),
            toc_heading(ctx, "b", 2).content("B"),
        ))
    }))
    .unwrap();
    let entries = ctx.entries(app.scope()).get();
    let selected = app.scope().cage(Some(entries[1].id.clone()));
    app.mount(table_of_contents(ctx.clone(), "nav").current(selected))
        .unwrap();
    let marked = nodes(&app, "sapid-anchor")
        .into_iter()
        .find(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .semantics
                .as_ref()
                .is_some_and(|s| s.current == Some(Current::Location))
        })
        .unwrap();
    let tree = app.tree();
    let Props::Link(link) = tree.node(marked).unwrap().descriptor.view().props else {
        panic!("link")
    };
    assert_eq!(link.target, format!("#{}", entries[1].id.as_str()));
    drop(tree);
    assert_eq!(selected.get(), Some(entries[1].id.clone()));
    assert_eq!(ctx.current(app.scope()).get(), Some(entries[0].id.clone()));
    selected.revise(|v| *v = None);
    app.flush().unwrap();
    assert!(nodes(&app, "sapid-anchor").into_iter().all(|id| {
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_none_or(|s| s.current.is_none())
    }));
}
#[test]
fn reveal_revalidates_actual_staged_heading_ancestry_after_external_moves() {
    let app = App::memory();
    let ctx = toc_context(app.scope(), "ancestry").unwrap();
    let heading = app.scope().cage(None);
    app.mount(
        toc_document(ctx.clone(), "doc")
            .content(move |ctx| toc_heading(ctx, "a", 1).content("A").node_ref(heading)),
    )
    .unwrap();
    let id = ctx.entries(app.scope()).get()[0].id.clone();
    let node = heading.get().unwrap();
    app.scope().place(node, None, None).unwrap();
    let error = ctx.reveal(&id).unwrap_err();
    assert!(format!("{error:?}").contains("heading left its scrolling document"));
    assert_eq!(app.stats().queries, 0);
}

#[path = "../examples/web/table_of_contents_form.rs"]
mod browser_fixture;
#[test]
fn browser_fixture_ssr_contains_every_heading_editor_and_control() {
    let snapshot = glory::ssr::snapshot(browser_fixture::table_of_contents_form(
        sapid::Theme::green_dark(),
    ))
    .unwrap();
    assert_eq!(snapshot.html.matches("sapid-toc-heading").count(), 4);
    assert_eq!(snapshot.html.matches("sapid-toc-item").count(), 4);
    assert!(snapshot.html.contains("Guide notes"));
    for label in [
        "Rename section",
        "Reverse sections",
        "Remove nested",
        "Restore nested",
        "Reveal details",
    ] {
        assert!(snapshot.html.contains(label));
    }
    assert!(snapshot.html.contains("aria-current=\"location\""));
}
