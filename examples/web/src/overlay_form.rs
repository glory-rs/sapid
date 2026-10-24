use glory::prelude::*;
use sapid::{ContentPolicy, dialog, drawer, popover, text_field};

pub struct State {
    pub open: Cage<bool>,
    pub proposals: Cage<usize>,
}
pub fn form(scope: &Scope) -> (impl Widget + use<>, State) {
    form_with(scope, false)
}
pub fn form_with(scope: &Scope, initial: bool) -> (impl Widget + use<>, State) {
    form_themed(scope, initial, sapid::Theme::default())
}

pub fn form_themed(
    scope: &Scope,
    initial: bool,
    theme: sapid::Theme,
) -> (impl Widget + use<>, State) {
    let open = scope.cage(initial);
    let proposals = scope.cage(0);
    let widget = col()
        .class("overlay-demo preview-card")
        .style(
            Style::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(18.0)
                .padding(Edges::all(24.0)),
        )
        .fill((
            glory::ui::button().class("outside").label("Outside action"),
            dialog("demo-dialog")
                .theme(theme.clone())
                .label("Open dialog")
                .title("Account settings")
                .description("Edit the account")
                .content({
                    let theme = theme.clone();
                    move || {
                        col().fill((
                            text_field()
                                .theme(theme.clone())
                                .class("dialog-input")
                                .name("Account name")
                                .default_value("Alice"),
                            popover("nested-popover")
                                .theme(theme.clone())
                                .label("Nested popover")
                                .title("More options")
                                .content(|| {
                                    glory::ui::button().class("nested-action").label("Option")
                                }),
                            dialog("nested-dialog")
                                .theme(theme.clone())
                                .label("Nested dialog")
                                .title("Confirm changes")
                                .content({
                                    let theme = theme.clone();
                                    move || {
                                        text_field()
                                            .theme(theme.clone())
                                            .class("nested-input")
                                            .name("Confirmation")
                                    }
                                }),
                        ))
                    }
                }),
            drawer("demo-drawer")
                .theme(theme.clone())
                .label("Open drawer")
                .title("Navigation")
                .content(|| {
                    col().fill((
                        glory::ui::button()
                            .label("Disabled destination")
                            .enabled(false),
                        glory::ui::button()
                            .class("drawer-action")
                            .label("Destination"),
                    ))
                }),
            view()
                .class("clipped")
                .style(
                    Style::new()
                        .height(Length::Px(44.0))
                        .overflow_x(Overflow::Hidden)
                        .overflow_y(Overflow::Hidden),
                )
                .fill(
                    popover("demo-popover")
                        .theme(theme.clone())
                        .label("Open popover")
                        .title("Quick edit")
                        .content_policy(ContentPolicy::KeepMounted)
                        .content({
                            let theme = theme.clone();
                            move || {
                                text_field()
                                    .theme(theme.clone())
                                    .class("popover-input")
                                    .name("Popover draft")
                                    .default_value("draft")
                            }
                        }),
                ),
            dialog("empty")
                .theme(theme.clone())
                .label("Empty dialog")
                .show_close(false),
            dialog("locked")
                .theme(theme.clone())
                .label("Controlled dialog")
                .title("Controlled")
                .open(open)
                .on_open_change(move |_| proposals.revise(|v| *v += 1)),
            dialog("initial")
                .theme(theme.clone())
                .label("Initial focus")
                .title("Focus selection")
                .initial_focus(SemanticId::new("preferred-focus").unwrap())
                .content(|| {
                    col().fill((
                        glory::ui::button().enabled(false).label("Disabled first"),
                        glory::ui::button().label("Other"),
                        glory::ui::input()
                            .class("preferred")
                            .semantics(Some(Box::new(Semantics {
                                id: Some(SemanticId::new("preferred-focus").unwrap()),
                                ..Default::default()
                            }))),
                    ))
                }),
            text()
                .class("layer-status")
                .content(proposals.map(|v| format!("proposals:{v}"))),
        ));
    (widget, State { open, proposals })
}
