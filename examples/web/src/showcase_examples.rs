//! Widgets mounted directly by the Rust documentation site, one demo at a time.
use glory::prelude::*;

pub fn demo(id: &str, theme: sapid::Theme) -> Box<dyn Widget> {
    let selected = crate::showcase_catalog::find(id).expect("catalogued widget");
    Box::new(view().class("site-example").fill((selected.build)(theme)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glory::render::{Event, Props};

    #[test]
    fn clean_examples_build_real_widgets_and_retire_their_owned_state() {
        for id in [
            "P0-01", "P0-02", "P0-03", "P0-04", "P0-05", "P0-06", "P0-07", "P0-08", "P0-09",
            "P0-14",
        ] {
            let app = App::memory();
            app.mount(demo(id, sapid::Theme::blue_light())).unwrap();
            assert!(!app.tree().is_empty(), "empty demo: {id}");
            if id == "P0-01" {
                let mut pending = app.tree().roots().to_vec();
                let mut button = None;
                while let Some(id) = pending.pop() {
                    let tree = app.tree();
                    let node = tree.node(id).unwrap();
                    if matches!(node.descriptor.view().props, Props::Button(p) if p.label == "Filled button")
                    {
                        button = Some(id);
                    }
                    pending.extend(node.children.iter().copied());
                }
                app.dispatch(button.unwrap(), Event::Activate).unwrap();
                let mut pending = app.tree().roots().to_vec();
                let mut updated = false;
                while let Some(id) = pending.pop() {
                    let tree = app.tree();
                    let node = tree.node(id).unwrap();
                    updated |= matches!(node.descriptor.view().props, Props::Text(p) if p.content == "Clicked 1 times");
                    pending.extend(node.children.iter().copied());
                }
                assert!(
                    updated,
                    "the displayed counter must respond to native activation"
                );
            }
            app.clear().unwrap();
            assert_eq!(app.stats().nodes, 0, "retired demo: {id}");
            assert_eq!(app.runtime_stats().tasks, 0, "retired tasks: {id}");
        }
    }
}
