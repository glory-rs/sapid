use glory::ui::{Widget, component, view};
use sapid::{ModelRecord, Theme, model_selector, suggestion, suggestion_group};
pub fn model_suggestions_form(theme: Theme) -> impl Widget {
    model_suggestions_form_with_selector(theme, true)
}
pub fn model_suggestions_form_with_selector(
    theme: Theme,
    include_selector: bool,
) -> impl Widget {
    component(move |scope| {
        let selected = scope.cage(Some("atlas".to_owned()));
        let search = scope.cage("Atlas".to_owned());
        let accept_model = scope.cage(false);
        let accept_search = scope.cage(true);
        let enabled = scope.cage(true);
        let model_result = scope.cage("No model proposal".to_owned());
        let suggestion_result = scope.cage("No suggestion chosen".to_owned());
        let count = scope.cage(0usize);
        let mut first = ModelRecord::new("atlas", "North examples", "Atlas");
        first.value = "north/atlas-v1".into();
        let mut second = ModelRecord::new("boreal", "South examples", "Boreal");
        second.value = "south/boreal-v2".into();
        let mut disabled = ModelRecord::new("retired", "South examples", "Retired example");
        disabled.available = false;
        let selector = sapid::field("selected-model")
            .theme(theme.clone())
            .label("Model")
            .form_name("model")
            .control(move |field| {
                model_selector("models")
                    .field(field)
                    .models([first.clone(), second.clone(), disabled.clone()])
                    .value(selected)
                    .search(search)
                    .on_change(move |proposal| {
                        model_result.revise(|value| {
                            *value = proposal.as_ref().map_or_else(
                                || "Model proposal: none".into(),
                                |model| {
                                    format!("Model proposal: {} = {}", model.key, model.value)
                                },
                            )
                        });
                        if accept_model.get() {
                            selected.revise(|value| *value = proposal.map(|model| model.key));
                        }
                    })
                    .on_search_change(move |proposal| {
                        if accept_search.get() {
                            search.revise(|value| *value = proposal)
                        }
                    })
            });
        let actions = suggestion_group("suggested-actions")
            .theme(theme.clone())
            .name("Suggested actions")
            .enabled(enabled)
            .suggestion(
                suggestion("explain")
                    .label("Explain")
                    .value("Application explanation request")
                    .source("Application notes"),
            )
            .suggestion(
                suggestion("unavailable")
                    .label("Unavailable suggestion")
                    .enabled(false),
            )
            .suggestion(
                suggestion("summarize")
                    .label("Summarize")
                    .value("Application summary request"),
            )
            .on_choose(move |choice| {
                count.revise(|value| *value += 1);
                suggestion_result.revise(|value| {
                    *value = format!(
                        "Chosen {}: {} [{}]",
                        choice.key,
                        choice.value,
                        choice.source.as_deref().unwrap_or("no source")
                    )
                });
            });
        let standalone = suggestion("single")
            .theme(theme.clone())
            .label("Standalone action")
            .value("Standalone application payload")
            .source("Application")
            .on_choose(move |choice| {
                suggestion_result
                    .revise(|value| *value = format!("Standalone: {}", choice.value))
            });
        let mut root = view().class("model-suggestions-fixture").style(
            glory::style::Style::new()
                .width(glory::style::Length::Percent(100.))
                .max_width(glory::style::Limit::Px(760.))
                .display(glory::style::Display::Flex)
                .flex_direction(glory::style::FlexDirection::Column)
                .gap(12.),
        );
        if include_selector {
            root = root.fill(
                sapid::form("model-fixture")
                    .theme(theme.clone())
                    .fill(selector),
            );
        }
        root.fill((
            actions,
            standalone,
            sapid::button()
                .theme(theme.clone())
                .label("Accept model proposals")
                .on_press(move || accept_model.revise(|value| *value = true)),
            sapid::button()
                .theme(theme.clone())
                .label("Refuse model proposals")
                .on_press(move || accept_model.revise(|value| *value = false)),
            sapid::button()
                .theme(theme.clone())
                .label("Refuse model search")
                .on_press(move || accept_search.revise(|value| *value = false)),
            sapid::button()
                .theme(theme.clone())
                .label("Toggle suggestions enabled")
                .on_press(move || enabled.revise(|value| *value = !*value)),
            sapid::text()
                .theme(theme.clone())
                .class("model-proposal-result")
                .content(model_result),
            sapid::text()
                .theme(theme.clone())
                .class("suggestion-result")
                .content(suggestion_result),
            sapid::text()
                .theme(theme.clone())
                .class("suggestion-count")
                .content(scope.bond(move || format!("Chosen count {}", count.get()))),
        ))
        .show_in(scope)
    })
}
