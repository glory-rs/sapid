use glory::ui::{Widget, text, widget};
use sapid::{
    KbdKey, KbdPlatform, Theme, blockquote, button, code, data_list, data_list_item, highlight,
    item, kbd, mark, text_list, text_list_item,
};

pub fn document_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let source = scope.cage("let glory = \"<&>\";".to_string());
        glory::ui::col()
            .class("document-fixture")
            .fill((
                code()
                    .theme(theme.clone())
                    .class("document-code")
                    .content(source),
                button()
                    .theme(theme.clone())
                    .label("Update code source")
                    .on_press(move || source.revise(|text| *text = "let sapid = 42;".into())),
                kbd()
                    .theme(theme.clone())
                    .platform(KbdPlatform::Mac)
                    .key(KbdKey::Meta),
                mark().theme(theme.clone()).content("Marked text"),
                text_list()
                    .theme(theme.clone())
                    .ordered(true)
                    .name("Document steps")
                    .item(text_list_item().fill(text().content("First step")))
                    .item(
                        text_list_item().fill((
                            text().content("Second step"),
                            text_list()
                                .theme(theme.clone())
                                .item(text_list_item().fill(text().content("Nested step"))),
                        )),
                    ),
                blockquote()
                    .theme(theme.clone())
                    .fill(text().content("A quotation <&>"))
                    .citation(text().content("Document author")),
                data_list()
                    .theme(theme.clone())
                    .name("Project metadata")
                    .item(
                        data_list_item()
                            .term("Project")
                            .fill(text().content("Glory")),
                    )
                    .item(
                        data_list_item()
                            .term("Status")
                            .fill(mark().theme(theme.clone()).content("Ready")),
                    ),
                glory::ui::view().class("document-highlight").fill(
                    highlight()
                        .theme(theme.clone())
                        .content("Find the Glory text 😀 without changing it.")
                        .query("Glory")
                        .query("😀"),
                ),
                item()
                    .theme(theme.clone())
                    .title("Repository")
                    .description("Semantic UI project")
                    .media(text().content("Project"))
                    .actions(button().theme(theme.clone()).label("Open repository")),
            ))
            .show_in(scope)
    })
}
