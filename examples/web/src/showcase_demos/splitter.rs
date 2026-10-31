use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::{DragPhase, Event, EventKind, Orientation};
    use glory::style::{Length, Style};
    use sapid::{Theme, splitter};

    pub struct State {
        pub value: Cage<f64>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(50.0);
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let request = scope.cage(50.0);
        let phase = scope.cage(String::from("idle"));
        let stage = scope.cage(false);
        let ordered = scope.cage(true);
        let status = scope.bond(move || {
            format!(
                "value={:.2};request={:.2};changes={};commits={};phase={};ordered={}",
                value.get(),
                request.get(),
                changes.get(),
                commits.get(),
                phase.get(),
                ordered.get()
            )
        });
        let horizontal = splitter("workspace")
            .theme(theme.clone())
            .name("Workspace divider")
            .class("horizontal-splitter")
            .value(value)
            .enabled(enabled)
            .min(0.0)
            .max(100.0)
            .style(
                Style::new()
                    .width(Length::Px(606.0))
                    .height(Length::Px(180.0)),
            )
            .first(|| {
                glory::ui::input()
                    .name("First draft")
                    .class("first-draft")
                    .value("Preserve the first draft")
            })
            .second(|| {
                glory::ui::input()
                    .name("Second draft")
                    .class("second-draft")
                    .value("Preserve the second draft")
            })
            .on_value_change(move |next| {
                changes.revise(|v| *v += 1);
                request.revise(|v| *v = next);
                stage.revise(|v| *v = true);
                if accept.get_untracked() {
                    value.revise(|v| *v = next);
                }
            })
            .on_value_commit(move |_| {
                commits.revise(|v| *v += 1);
            })
            .on(EventKind::Range, move |_| {
                ordered.revise(|v| *v &= stage.get_untracked());
                stage.revise(|v| *v = false);
            })
            .on(EventKind::Drag, move |event| {
                if let Event::Drag { phase: current, .. } = event {
                    if matches!(current, DragPhase::Move | DragPhase::Drop) {
                        ordered.revise(|v| *v &= stage.get_untracked());
                        stage.revise(|v| *v = false);
                    }
                    phase.revise(|v| *v = format!("{current:?}"));
                }
            });
        let vertical = splitter("vertical")
            .theme(theme.clone())
            .name("Vertical divider")
            .class("vertical-splitter")
            .axis(Orientation::Vertical)
            .style(
                Style::new()
                    .width(Length::Px(606.0))
                    .height(Length::Px(206.0)),
            )
            .first(|| glory::ui::text().content("Top pane"))
            .second(|| glory::ui::text().content("Bottom pane"));
        (
            glory::ui::view().class("splitter-demo").fill((
                sapid::title(1)
                    .theme(theme.clone())
                    .content("Resizable Splitter"),
                horizontal,
                sapid::text()
                    .theme(theme.clone())
                    .class("splitter-status")
                    .content(status),
                vertical,
                sapid::button()
                    .theme(theme.clone())
                    .label("After splitters"),
            )),
            State {
                value,
                accept,
                enabled,
            },
        )
    }
}
