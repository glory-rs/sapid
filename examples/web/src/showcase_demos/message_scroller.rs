use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::message_scroller_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::render::SemanticId;
    use glory::style::{Display, FlexDirection, Style};
    use glory::ui::{Widget, view, widget};
    use sapid::{MessageData, MessageScrollerHandle, Theme};
    use std::{cell::Cell, rc::Rc};
    pub fn message_scroller_form(theme: Theme) -> impl Widget {
        message_scroller_form_with_controller(theme, None)
    }
    pub fn message_scroller_form_with_controller(
        theme: Theme,
        external: Option<glory::Cage<Option<MessageScrollerHandle>>>,
    ) -> impl Widget {
        widget(move |scope| {
            let first = scope.cage(1000usize);
            let count = scope.cage(1_000_000usize);
            let revision = scope.cage(0u64);
            let expanded = scope.cage(false);
            let current = scope.cage(None::<MessageScrollerHandle>);
            let info = scope.cage(String::new());
            let reads = Rc::new(Cell::new(0usize));
            let key_reads = reads.clone();
            let message_theme = theme.clone();
            let scroller=sapid::message_scroller(count,move |index| {
            key_reads.set(key_reads.get()+1);SemanticId::new(format!("row-{}",first.get()+index)).unwrap()
        },move |key| {
            key.as_str().strip_prefix("row-").and_then(|n|n.parse::<usize>().ok()).and_then(|n|n.checked_sub(first.get())).filter(|at|*at<count.get())
        },move |_,key| {
            let theme=message_theme.clone();
            widget(move |scope| {
                let message_key=key.clone();
                let key=key.clone();let body=scope.bond(move || {
                    let mut value=MessageData::new("Application",format!("Mounted {}",key.as_str()));
                    if expanded.get(){value.body.push_str("\nNew line of accepted application content\nA changed message height\nThe reading anchor remains mounted.");}
                    value
                });
                sapid::message(message_key.as_str(),body).theme(theme.clone()).show_in(scope)
            })
        }).theme(theme.clone()).height(320.).estimate(100.).revision(revision)
        .on_ready(move |handle| {
            if let Some(external)=external {external.revise(|value|*value=Some(handle.clone()));}
            current.revise(|value|*value=Some(handle));
        })
        .inspect(move |stats| {
            let next=format!("range {}..{}; rows {}; measured {}; reads {}",stats.range.start,stats.range.end,stats.rows,stats.measured,reads.get());
            if info.get_untracked()!=next{info.revise(|value|*value=next);}
        });
            let status = scope.bond(move || {
                current
                    .with(|handle| handle.as_ref().and_then(|handle| handle.status().ok()))
                    .map_or_else(
                        || "Retired".into(),
                        |value| {
                            format!("following {}; unread {}", value.following, value.unread)
                        },
                    )
            });
            view()
                .class("message-scroller-fixture")
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(12.),
                )
                .fill((
                    view()
                        .style(Style::new().display(Display::Flex).gap(8.))
                        .fill((
                            sapid::button()
                                .theme(theme.clone())
                                .label("Append five")
                                .on_press(move || {
                                    count.revise(|value| *value += 5);
                                    revision.revise(|value| *value += 1);
                                    if let Some(handle) = current.get() {
                                        handle.appended(5).unwrap();
                                    }
                                }),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Prepend ten")
                                .on_press(move || {
                                    if first.get_untracked() < 10 {
                                        return;
                                    }
                                    first.revise(|value| *value -= 10);
                                    count.revise(|value| *value += 10);
                                    revision.revise(|value| *value += 1);
                                }),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Change message heights")
                                .on_press(move || expanded.revise(|value| *value = !*value)),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Follow latest")
                                .on_press(move || {
                                    if let Some(handle) = current.get() {
                                        handle.follow_tail().unwrap();
                                    }
                                }),
                        )),
                    sapid::chat(scroller)
                        .theme(theme.clone())
                        .title("Application conversation"),
                    sapid::text()
                        .theme(theme.clone())
                        .class("message-scroller-status")
                        .content(status),
                    sapid::text()
                        .theme(theme.clone())
                        .class("message-scroller-stats")
                        .content(info),
                ))
                .show_in(scope)
        })
    }
}
