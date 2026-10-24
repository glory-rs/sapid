use glory::prelude::*;
use glory::render::{CollapseConfig, Event, EventKind, Semantics};

pub fn collapse_stream_probe() -> impl Widget {
    component(|scope| {
        let config = scope.cage(CollapseConfig {
            open: true,
            duration_millis: 240,
            revision: 0,
        });
        let completed = scope.cage(0usize);
        let draft = scope.cage("stream draft".to_owned());
        let weak = scope.downgrade();
        let panel = glory::ui::view()
            .class("stream-collapse-panel")
            .semantics(scope.bond(move || {
                let config = config.get();
                Some(Box::new(Semantics {
                    collapse: Some(config),
                    hidden: !config.open,
                    inert: !config.open,
                    ..Default::default()
                }))
            }))
            .on(EventKind::CollapseComplete, move |event| {
                if let Event::CollapseComplete { revision } = event {
                    let current = config.get();
                    if !current.open && current.revision == revision {
                        completed.revise(|value| *value += 1);
                    }
                }
            })
            .fill(
                glory::ui::view()
                    .style(Style::new().height(Length::Px(96.)))
                    .fill(
                        glory::ui::input()
                            .class("stream-collapse-editor")
                            .name("Streaming draft")
                            .value(draft)
                            .on(EventKind::ValueChanged, move |event| {
                                if let Event::ValueChanged { value, .. } = event {
                                    draft.revise(|draft| *draft = value);
                                }
                            }),
                    ),
            );
        let change = move |open: bool| {
            let result = (|| -> glory::ui::Result {
                let current = config.try_get()?;
                if current.open != open {
                    let revision = current
                        .revision
                        .checked_add(1)
                        .ok_or(glory::render::Error::CollapseRevisionExhausted)?;
                    config.try_revise(|value| {
                        *value = CollapseConfig {
                            open,
                            revision,
                            ..current
                        }
                    })?;
                }
                Ok(())
            })();
            if let Err(error) = result
                && let Some(scope) = weak.upgrade()
            {
                scope.report_error(error);
            }
        };
        let change = std::rc::Rc::new(change);
        let close = change.clone();
        glory::ui::view()
            .class("stream-collapse-container")
            .fill((
                panel,
                glory::ui::button()
                    .label("Close streamed collapse")
                    .on(EventKind::Activate, move |_| close(false)),
                glory::ui::button()
                    .label("Open streamed collapse")
                    .on(EventKind::Activate, move |_| change(true)),
                glory::ui::text()
                    .class("stream-collapse-completed")
                    .content(completed.map(|value| format!("Completed: {value}"))),
            ))
            .show_in(scope)
    })
}
