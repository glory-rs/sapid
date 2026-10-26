use glory::prelude::*;
use sapid::{TagDuplicatePolicy, TagsInputState, Theme};

pub fn demo(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let skills = scope.cage(TagsInputState {
            values: vec!["Rust".into(), "WebAssembly".into()],
            search: String::new(),
        });
        let labels = scope.cage(TagsInputState {
            values: vec!["stable".into(), "release".into()],
            search: String::new(),
        });
        let readonly = scope.cage(false);
        let enabled = scope.cage(true);
        sapid::stack().theme(theme.clone()).gap(22.0).fill((
            sapid::field("showcase-skills").theme(theme.clone()).label("Skills")
                .description("Press Enter or type a comma to add a skill. Backspace removes the last empty-search tag.")
                .control(move |field| {
                    sapid::tags_input("showcase-skills").field(field).state(skills)
                        .placeholder("Add a skill")
                        .separators(glory::render::TagSeparators::EnterCommaSemicolon)
                        .readonly(readonly).enabled(enabled)
                        .on_change(move |proposal| skills.revise(|current| *current = proposal))
                }),
            sapid::field("showcase-labels").theme(theme.clone()).label("Labels")
                .description("Rounded pills use the same editing behavior. Duplicate labels are allowed.")
                .control(move |field| {
                    sapid::pills_input("showcase-labels").field(field).state(labels)
                        .placeholder("Add a label")
                        .duplicate_policy(TagDuplicatePolicy::Allow)
                        .readonly(readonly).enabled(enabled)
                        .on_change(move |proposal| labels.revise(|current| *current = proposal))
                }),
            sapid::group().theme(theme.clone()).fill((
                sapid::checkbox().theme(theme.clone()).label("Disabled")
                    .checked(enabled.map(|value| !value))
                    .on_checked_change(move |value| enabled.revise(|current| *current = !value)),
                sapid::checkbox().theme(theme.clone()).label("Read only").checked(readonly)
                    .on_checked_change(move |value| readonly.revise(|current| *current = value)),
            )),
        )).show_in(scope)
    })
}
