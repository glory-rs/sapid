use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::prompt_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::render::{FileConfig, FileRead};
    use glory::style::{Display, FlexDirection, FlexWrap, Style};
    use glory::ui::{Widget, view, widget};
    use sapid::{PromptSubmitMode, PromptSuggestion, Theme};

    pub fn prompt_form(theme: Theme) -> impl Widget {
        prompt_form_with_files(theme, true)
    }
    pub fn prompt_form_with_files(theme: Theme, attachments: bool) -> impl Widget {
        widget(move |scope| {
            let busy = scope.cage(false);
            let readonly = scope.cage(false);
            let controlled = scope.cage("Accepted controlled draft".to_owned());
            let allow = scope.cage(false);
            let sends = scope.cage(0usize);
            let cancels = scope.cage(0usize);
            let result = scope.cage("No request".to_owned());
            let bytes = scope.cage(0usize);
            let reject = scope.cage("No rejection".to_owned());
            let weak = scope.downgrade();
            let mut prompt = sapid::prompt_input("primary-prompt")
                .theme(theme.clone())
                .class("prompt-primary")
                .busy(busy)
                .readonly(readonly)
                .submit_mode(PromptSubmitMode::EnterToSend)
                .default_value("Draft 🙂")
                .suggestion(PromptSuggestion {
                    label: "Explain this source".into(),
                    value: "Please explain the application source.\nUse public information.".into(),
                })
                .on_send(move |value| {
                    sends.revise(|n| *n += 1);
                    result.revise(|result| {
                        *result = format!("Text [{}]; files {}", value.text, value.files.len())
                    });
                    if let (Some(controller), Some(file), Some(scope)) =
                        (value.file_controller, value.files.first(), weak.upgrade())
                    {
                        controller
                            .read_in(
                                &scope,
                                FileRead {
                                    file: file.token,
                                    offset: 0,
                                    length: 4,
                                },
                                move |result| {
                                    if let Ok(value) = result {
                                        bytes.revise(|count| *count = value.len());
                                    }
                                },
                            )
                            .unwrap();
                    }
                })
                .on_cancel(move || cancels.revise(|n| *n += 1))
                .on_reject(move |value| reject.revise(|current| *current = format!("{value:?}")));
            if attachments {
                prompt = prompt.attachments(FileConfig {
                    multiple: true,
                    max_files: 3,
                    max_file_bytes: 1024,
                    max_total_bytes: 2048,
                    ..Default::default()
                });
            }
            let accepted = controlled;
            let controlled_prompt = sapid::prompt_input("controlled-prompt")
                .theme(theme.clone())
                .class("prompt-controlled")
                .label("Controlled prompt")
                .form_name("controlled_prompt")
                .value(controlled)
                .send_label("Send controlled")
                .cancel_label("Cancel controlled")
                .on_value_change(move |proposal| {
                    if allow.get() {
                        accepted.revise(|value| *value = proposal);
                    }
                })
                .on_send(move |value| {
                    result.revise(|result| *result = format!("Controlled [{}]", value.text))
                });
            view()
                .class("prompt-fixture")
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(18.),
                )
                .fill((
                    view()
                        .style(
                            Style::new()
                                .display(Display::Flex)
                                .flex_wrap(FlexWrap::Wrap)
                                .gap(8.),
                        )
                        .fill((
                            sapid::button()
                                .theme(theme.clone())
                                .label("Mark request busy")
                                .on_press(move || busy.revise(|value| *value = true)),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Release request busy")
                                .on_press(move || busy.revise(|value| *value = false)),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Toggle prompt readonly")
                                .on_press(move || readonly.revise(|value| *value = !*value)),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Accept controlled edits")
                                .on_press(move || allow.revise(|value| *value = true)),
                        )),
                    prompt,
                    controlled_prompt,
                    sapid::text()
                        .theme(theme.clone())
                        .class("prompt-sends")
                        .content(scope.bond(move || {
                            format!(
                                "Sent {}; cancelled {}; read {}",
                                sends.get(),
                                cancels.get(),
                                bytes.get()
                            )
                        })),
                    sapid::text()
                        .theme(theme.clone())
                        .class("prompt-result")
                        .content(result),
                    sapid::text()
                        .theme(theme.clone())
                        .class("prompt-rejection")
                        .content(reject),
                ))
                .show_in(scope)
        })
    }
}
