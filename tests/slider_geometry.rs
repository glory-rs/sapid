use glory::prelude::*;
use glory::style::{Length, Style};
use sapid::{Size, angle_slider, range_slider, slider};

#[test]
fn unstyled_ranges_keep_default_geometry_for_all_five_sizes_and_both_axes() {
    for (size, cross) in [
        (Size::ExtraSmall, 24.),
        (Size::Small, 30.),
        (Size::Medium, 36.),
        (Size::Large, 42.),
        (Size::ExtraLarge, 50.),
    ] {
        for axis in [Orientation::Horizontal, Orientation::Vertical] {
            let app = App::memory();
            let single = app.scope().cage(None);
            let pair = app.scope().cage(None);
            app.mount(
                glory::ui::group().fill((
                    slider("single")
                        .unstyled()
                        .size(size)
                        .orientation(axis)
                        .node_ref(single),
                    range_slider("pair")
                        .unstyled()
                        .size(size)
                        .orientation(axis)
                        .node_ref(pair),
                )),
            )
            .unwrap();
            let expected = if axis == Orientation::Horizontal {
                (240., cross)
            } else {
                (cross, 240.)
            };
            for id in [single.get().unwrap(), pair.get().unwrap()] {
                let tree = app.tree();
                let style = tree.node(id).unwrap().descriptor.view().style;
                assert_eq!(style.width, Some(Length::Px(expected.0)));
                assert_eq!(style.height, Some(Length::Px(expected.1)));
                assert!(style.background.is_none());
            }
        }
    }
}
#[test]
fn unstyled_angles_keep_five_default_diameters() {
    for (size, cross) in [
        (Size::ExtraSmall, 24.),
        (Size::Small, 30.),
        (Size::Medium, 36.),
        (Size::Large, 42.),
        (Size::ExtraLarge, 50.),
    ] {
        let app = App::memory();
        let reference = app.scope().cage(None);
        app.mount(
            angle_slider("angle")
                .unstyled()
                .size(size)
                .node_ref(reference),
        )
        .unwrap();
        let tree = app.tree();
        let style = tree
            .node(reference.get().unwrap())
            .unwrap()
            .descriptor
            .view()
            .style;
        assert_eq!(style.width, Some(Length::Px(cross * 3. + 12.)));
        assert_eq!(style.height, style.width);
    }
}
#[test]
fn explicit_auto_and_dynamic_dimensions_are_preserved_while_absent_dimensions_recover_defaults() {
    let app = App::memory();
    let single = app.scope().cage(None);
    let pair = app.scope().cage(None);
    let angle = app.scope().cage(None);
    let dimensions = app
        .scope()
        .cage(Style::new().width(Length::Auto).height(Length::Px(77.)));
    app.mount(
        glory::ui::group().fill((
            slider("single")
                .unstyled()
                .style(dimensions)
                .node_ref(single),
            range_slider("pair")
                .unstyled()
                .style(dimensions)
                .node_ref(pair),
            angle_slider("angle")
                .unstyled()
                .style(dimensions)
                .node_ref(angle),
        )),
    )
    .unwrap();
    for id in [
        single.get().unwrap(),
        pair.get().unwrap(),
        angle.get().unwrap(),
    ] {
        let tree = app.tree();
        let style = tree.node(id).unwrap().descriptor.view().style;
        assert_eq!(style.width, Some(Length::Auto));
        assert_eq!(style.height, Some(Length::Px(77.)));
    }
    dimensions.revise(|v| *v = Style::new());
    app.flush().unwrap();
    for (id, width, height) in [
        (single.get().unwrap(), 240., 36.),
        (pair.get().unwrap(), 240., 36.),
        (angle.get().unwrap(), 120., 120.),
    ] {
        let tree = app.tree();
        let style = tree.node(id).unwrap().descriptor.view().style;
        assert_eq!(style.width, Some(Length::Px(width)));
        assert_eq!(style.height, Some(Length::Px(height)));
    }
}
