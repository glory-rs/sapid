use glory::prelude::*;
use sapid::{ModelRecord, Theme};

pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
    match id {
        "P13-05" => Box::new(widget(move |scope| {
            let selected = scope.cage(Some("atlas".to_owned()));
            let search = scope.cage("Atlas".to_owned());
            let mut atlas = ModelRecord::new("atlas", "North examples", "Atlas");
            atlas.value = "north/atlas-v1".into();
            let mut boreal = ModelRecord::new("boreal", "South examples", "Boreal");
            boreal.value = "south/boreal-v2".into();
            sapid::field("focused-model")
                .theme(theme.clone())
                .label("Model")
                .description("Search the application-provided model catalogue.")
                .control(move |field| {
                    sapid::model_selector("focused-models")
                        .field(field)
                        .models([atlas.clone(), boreal.clone()])
                        .value(selected)
                        .search(search)
                        .on_change(move |proposal| {
                            selected.revise(|current| *current = proposal.map(|model| model.key));
                        })
                        .on_search_change(move |value| search.revise(|current| *current = value))
                })
                .show_in(scope)
        })),
        "P13-06" => Box::new(
            sapid::suggestion_group("focused-suggestions")
                .theme(theme)
                .name("Suggested actions")
                .suggestion(
                    sapid::suggestion("explain")
                        .label("Explain this code")
                        .value("Explain the selected code")
                        .source("Editor selection"),
                )
                .suggestion(
                    sapid::suggestion("summarize")
                        .label("Summarize the document")
                        .value("Summarize the current document")
                        .source("Current document"),
                )
                .suggestion(
                    sapid::suggestion("unavailable")
                        .label("Unavailable action")
                        .enabled(false),
                ),
        ),
        _ => unreachable!("unsupported model suggestions demo"),
    }
}
