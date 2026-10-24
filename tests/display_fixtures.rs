#[path = "../examples/web/src/actions_form.rs"]
mod actions_form;
#[path = "../examples/web/src/display_form.rs"]
mod display_form;
#[path = "../examples/web/src/document_form.rs"]
mod document_form;
#[path = "../examples/web/src/timeline_form.rs"]
mod timeline_form;

use sapid::Theme;

#[test]
fn shared_display_and_action_fixtures_build_for_every_theme() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let display = glory::ssr::render(display_form::display_form(theme.clone())).unwrap();
        let timeline = glory::ssr::render(timeline_form::timeline_form(theme.clone())).unwrap();
        assert!(timeline.contains("<ol") && timeline.contains("<li"));
        assert!(
            timeline.contains("role=\"separator\"")
                && timeline.contains("aria-orientation=\"vertical\"")
        );
        assert!(
            timeline.contains("Completed")
                && timeline.contains("Current")
                && timeline.contains("Upcoming")
        );
        assert!(display.contains("Display widgets"));
        assert!(display.contains("aria-label=\"Remove local tag\""));
        assert!(display.contains("Controlled tag"));
        let actions = glory::ssr::render(actions_form::actions_form(theme)).unwrap();
        assert!(actions.contains("aria-label=\"Refresh preview\""));
        assert!(actions.contains("Horizontal actions"));
        assert!(actions.contains("Vertical actions"));
        let document =
            glory::ssr::render(document_form::document_form(Theme::blue_light())).unwrap();
        for tag in [
            "<code",
            "<kbd",
            "<mark",
            "<ol",
            "<ul",
            "<li",
            "<blockquote",
            "<dl",
            "<dt",
            "<dd",
        ] {
            assert!(document.contains(tag), "missing {tag}");
        }
    }
}
