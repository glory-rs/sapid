use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::visually_hidden_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::SemanticId;
    pub fn visually_hidden_form(theme: sapid::Theme) -> impl Widget {
        widget(move |scope| {
            let detail = scope.cage("Changes are stored in this workspace.".to_string());
            let id = SemanticId::new("hidden-detail").unwrap();
            sapid::stack()
                .class("hidden-fixture")
                .gap(14.0)
                .fill((
                    view()
                        .class("hidden-row")
                        .style(
                            Style::new()
                                .position(glory::style::Position::Relative)
                                .height(Length::Px(40.0)),
                        )
                        .fill((
                            text().content("Visible content"),
                            sapid::visually_hidden()
                                .semantic_id(id.clone())
                                .class("hidden-detail")
                                .content(detail),
                        )),
                    button()
                        .class("hidden-described-button")
                        .label("Save")
                        .semantics(Some(Box::new(Semantics {
                            described_by: Some(id),
                            ..Default::default()
                        }))),
                    button()
                        .class("hidden-name-button")
                        .fill(sapid::visually_hidden().content("Invisible action name")),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Update hidden detail")
                        .on_press(move || {
                            detail.revise(|value| *value = "New accessible detail.".into())
                        }),
                    input().class("hidden-neighbor").value("Neighbor"),
                ))
                .show_in(scope)
        })
    }
}
