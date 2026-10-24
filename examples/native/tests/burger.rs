use glory::blitz::shell::WindowApp;
use glory::blitz::{Config, Raster};
use glory::render::{Event, MotionPreference};
#[test]
fn four_theme_burger_pixels_morph_with_stable_button_graphic_identity_and_no_frame_source_writes()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::with_config(Config {
            motion: MotionPreference::NoPreference,
            ..Default::default()
        });
        host.renderer().set_viewport(80, 80, 1.).unwrap();
        host.mount(sapid::burger().theme(theme).duration_millis(200))
            .unwrap();
        host.app().settle(16).unwrap();
        let button = host.app().tree().roots()[0];
        let graphic = host.app().tree().node(button).unwrap().children[0];
        let physical = host
            .renderer()
            .engine_id(graphic, glory::html::Part::Main)
            .unwrap();
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let closed = raster.pixels().to_vec();
        host.app().dispatch(button, Event::Activate).unwrap();
        let writes = host.app().runtime_stats().source_writes;
        host.renderer().advance_time(0.1).unwrap();
        raster.draw(host.renderer()).unwrap();
        let intermediate = raster.pixels().to_vec();
        assert_ne!(intermediate, closed);
        assert_eq!(host.app().runtime_stats().source_writes, writes);
        host.renderer().advance_time(0.21).unwrap();
        raster.draw(host.renderer()).unwrap();
        assert_ne!(raster.pixels(), closed);
        assert_ne!(raster.pixels(), intermediate);
        assert!(!host.renderer().inspect(|d| d.is_animating()));
        assert_eq!(
            host.renderer()
                .engine_id(graphic, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let state = host
            .app()
            .tree()
            .node(button)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .clone()
            .unwrap();
        assert_eq!((state.pressed, state.expanded), (Some(true), Some(true)));
        host.app().clear().unwrap();
        assert!(
            host.renderer()
                .engine_id(graphic, glory::html::Part::Main)
                .is_err()
        );
    }
}
#[test]
fn native_default_reduced_motion_controlled_refusal_and_disabled_activation_keep_exact_pixels()
{
    let host = WindowApp::new();
    host.renderer().set_viewport(80, 80, 1.).unwrap();
    let open = host.app().scope().cage(false);
    let enabled = host.app().scope().cage(true);
    let calls = host.app().scope().cage(0usize);
    host.mount(
        sapid::burger()
            .open(open)
            .enabled(enabled)
            .on_open_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    host.app().settle(16).unwrap();
    let button = host.app().tree().roots()[0];
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    let initial = raster.pixels().to_vec();
    host.app().dispatch(button, Event::Activate).unwrap();
    raster.draw(host.renderer()).unwrap();
    assert_eq!(raster.pixels(), initial);
    assert_eq!(calls.get(), 1);
    assert!(!open.get());
    assert!(!host.renderer().inspect(|d| d.is_animating()));
    open.revise(|v| *v = true);
    host.app().flush().unwrap();
    raster.draw(host.renderer()).unwrap();
    assert_ne!(raster.pixels(), initial);
    assert_eq!(calls.get(), 1);
    assert!(!host.renderer().inspect(|d| d.is_animating()));
    enabled.revise(|v| *v = false);
    host.app().flush().unwrap();
    assert!(!host.app().dispatch(button, Event::Activate).unwrap());
    assert_eq!(calls.get(), 1);
}
