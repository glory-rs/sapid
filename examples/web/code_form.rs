use glory::prelude::*;
pub const RUST_SOURCE: &str = "pub fn greet<'a>(name: &'a str) -> String {\n    // Unicode and HTML remain plain code: 😀 <script>\n    let message = format!(\"Hello, {name}!\");\n    message\n}\n";
pub const JSON_SOURCE: &str = "{\n  \"name\": \"Sapid 😀 <script>\",\n  \"ready\": true,\n  \"values\": [1, -2.5e+3, null]\n}\n";
pub fn code_form(theme: sapid::Theme) -> impl Widget {
    component(move |scope| {
        let content = scope.cage(RUST_SOURCE.to_string());
        let language = scope.cage(sapid::CodeLanguage::Rust);
        let document = scope.cage(
            "{\"branch\":{\"needle\":\"😀 <script>\",\"list\":[1,2]},\"version\":1}"
                .to_string(),
        );
        let expanded = scope.cage(vec![String::new()]);
        let query = scope.cage(String::new());
        let accepted = scope.cage(true);
        let proposals = scope.cage(0usize);
        glory::ui::col()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(12.)
                    .min_width(Length::Px(0.)),
            )
            .fill((
                sapid::code_highlight()
                    .theme(theme.clone())
                    .class("code-demo-highlight")
                    .source(content)
                    .language(language)
                    .line_numbers(true)
                    .highlighted_lines([2, 3]),
                sapid::code_block()
                    .theme(theme.clone())
                    .class("code-demo-plain")
                    .source("  exact spaces\n\tand a tab\n<script>& 😀\n")
                    .line_numbers(true)
                    .start_line(40),
                sapid::code_highlight()
                    .unstyled()
                    .class("code-demo-unstyled")
                    .source("fn unstyled() { 42 }"),
                sapid::text_field()
                    .theme(theme.clone())
                    .name("Code draft")
                    .default_value("Keep code draft"),
                glory::ui::view()
                    .style(Style {
                        flex_wrap: Some(glory::style::FlexWrap::Wrap),
                        ..Style::new().display(Display::Flex).gap(8.)
                    })
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .label("Show JSON code")
                            .on_press(move || {
                                content.revise(|v| *v = JSON_SOURCE.into());
                                language.revise(|v| *v = sapid::CodeLanguage::Json);
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Show Rust code")
                            .on_press(move || {
                                content.revise(|v| *v = RUST_SOURCE.into());
                                language.revise(|v| *v = sapid::CodeLanguage::Rust);
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Show plain code")
                            .on_press(move || {
                                language.revise(|v| *v = sapid::CodeLanguage::Plain)
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Show long code")
                            .on_press(move || {
                                content.revise(|v| {
                                    *v = format!("// {}\n", "finite long source ".repeat(50))
                                })
                            }),
                    )),
                sapid::text_field().theme(theme.clone()).name("Search JSON structure").value(query).on_value_change(move|value|query.revise(|q|*q=value)),
                sapid::json_viewer("code-viewer").theme(theme.clone()).name("JSON structure").source(document).expanded(expanded).search(query).on_expanded_change(move|next| {
                    proposals.revise(|count|*count+=1);
                    if accepted.get(){expanded.revise(|paths|*paths=next)}
                }),
                glory::ui::text().class("json-demo-proposals").content(proposals.map(|count|format!("Expansion proposals: {count}"))),
                glory::ui::view().style(Style { flex_wrap:Some(glory::style::FlexWrap::Wrap),..Style::new().display(Display::Flex).gap(8.) }).fill((
                    sapid::button().theme(theme.clone()).label("Reject or accept JSON expansion").on_press(move||accepted.revise(|value|*value = !*value)),
                    sapid::button().theme(theme.clone()).label("Reorder JSON properties").on_press(move||document.revise(|value|*value="{\"version\":2,\"branch\":{\"needle\":\"😀 <script>\",\"list\":[1,2]}}".into())),
                )),
            ))
            .show_in(scope)
    })
}
