use glory::render::SemanticId;
use glory::style::{Display, FlexDirection, Style};
use glory::ui::{Widget, view, widget};
use sapid::{MessageAlignment, MessageData, MessageGroup, MessageQuote, MessageStatus, Theme};

pub fn messages_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let mut initial = MessageData::new("Application", "A public answer 🙂\nSecond line");
        initial.timestamp = Some("10:00 UTC".into());
        initial.status = MessageStatus::Sending;
        initial.quote = Some(MessageQuote {
            source: "Original question".into(),
            body: "Quoted application content".into(),
            target: Some(sapid::message_id("question").unwrap()),
        });
        let data = scope.cage(initial);
        let actions = scope.cage(0usize);
        let mut question = MessageData::new("You", "What did the source say?");
        question.alignment = MessageAlignment::End;
        question.status = MessageStatus::Sent;
        let update_theme = theme.clone();
        view()
            .class("messages-fixture")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(16.),
            )
            .fill((
                sapid::message("question", question).theme(theme.clone()),
                sapid::message("answer", data).theme(theme.clone()).actions(
                    sapid::button()
                        .theme(theme.clone())
                        .label("Application retry")
                        .on_press(move || actions.revise(|value| *value += 1)),
                ),
                view()
                    .style(Style::new().display(Display::Flex).gap(8.))
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .label("Complete message")
                            .on_press(move || {
                                data.revise(|value| {
                                    value.status = MessageStatus::Sent;
                                    value.body = "Completed answer 🙂".into();
                                })
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Group middle")
                            .on_press(move || {
                                data.revise(|value| value.group = MessageGroup::Middle)
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Group single")
                            .on_press(move || {
                                data.revise(|value| value.group = MessageGroup::Single)
                            }),
                        sapid::button()
                            .theme(theme.clone())
                            .label("Remove quote")
                            .on_press(move || data.revise(|value| value.quote = None)),
                    )),
                sapid::text()
                    .theme(update_theme)
                    .class("messages-action-count")
                    .content(scope.bond(move || format!("Retries {}", actions.get()))),
                sapid::marker(SemanticId::new("sapid-message-8_question").unwrap())
                    .theme(theme.clone())
                    .label("Jump to question"),
                sapid::bubble("<script>display only</script>")
                    .theme(theme.clone())
                    .unstyled()
                    .class("messages-unstyled"),
            ))
            .show_in(scope)
    })
}
