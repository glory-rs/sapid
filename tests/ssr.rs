use glory::ui::{Purpose, col};
use sapid::{button, checkbox, text_field};

#[test]
fn widgets_keep_native_control_shapes_and_escape_user_text() {
    let html = glory::ssr::render(
        col().fill((
            button().label("Save <draft>").enabled(false),
            text_field()
                .name("Name <required>")
                .value("\"<&")
                .purpose(Purpose::Email),
            checkbox().label("Allow <mail>").checked(true),
        )),
    )
    .unwrap();
    assert!(html.contains("<button"));
    assert!(html.contains("type=\"button\""));
    assert!(html.contains("disabled"));
    assert!(html.contains("Save &lt;draft&gt;"));
    assert!(html.contains("aria-label=\"Name &lt;required&gt;\""));
    assert!(html.contains("value=\"&quot;&lt;&amp;\""));
    assert!(html.contains("type=\"checkbox\""));
    assert!(html.contains("checked"));
    assert!(html.contains("Allow &lt;mail&gt;"));
    assert!(html.contains("sapid-button"));
    assert!(!html.contains("<script"));
}

#[test]
fn multiline_password_and_unstyled_output_share_glory_mapping() {
    let html = glory::ssr::render(
        col().fill((
            text_field()
                .multiline(true)
                .default_value("first\r\nsecond"),
            text_field().purpose(Purpose::Password).value("secret"),
            checkbox().unstyled().label("Native"),
        )),
    )
    .unwrap();
    assert!(html.contains("<textarea"));
    assert!(html.contains("first\nsecond"));
    assert!(html.contains("type=\"password\""));
    assert!(html.contains("sapid-checkbox"));
}
