use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::modals_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{
        ModalToken, ModalsHandle, Theme, confirmation_request, modal_request, modals_manager,
    };
    use std::{cell::RefCell, rc::Rc};
    #[derive(Default)]
    struct Control {
        handle: Option<ModalsHandle>,
        first: Option<ModalToken>,
        second: Option<ModalToken>,
    }
    pub fn modals_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let control = Rc::new(RefCell::new(Control::default()));
            let ready = control.clone();
            let log = scope.cage("None".to_string());
            let count = scope.cage(0usize);
            let mut controls = Vec::new();
            for (label, operation) in [
                ("Open managed dialog", 0),
                ("Queue confirmation", 1),
                ("Update managed dialog", 2),
                ("Close queued confirmation", 3),
                ("Close all managed dialogs", 4),
                ("Open managed sequence", 5),
            ] {
                let control = control.clone();
                let weak = scope.downgrade();
                let content_theme = theme.clone();
                controls.push(sapid::button().theme(theme.clone()).label(label).on_press(
                    move || {
                        if let Err(error) =
                            command(&control, operation, log, count, &content_theme)
                            && let Some(scope) = weak.upgrade()
                        {
                            scope.report_error(error);
                        }
                    },
                ));
            }
            sapid::stack()
                .gap(16.)
                .fill((
                    modals_manager("demo")
                        .theme(theme.clone())
                        .on_ready(move |handle| ready.borrow_mut().handle = Some(handle)),
                    sapid::group().fill(controls),
                    glory::ui::text()
                        .class("modals-pending")
                        .content(scope.bond(move || format!("Pending: {}", count.get()))),
                    glory::ui::text().class("modals-result").content(log),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Outside managed editor")
                        .default_value("Outside draft"),
                ))
                .show_in(scope)
        })
    }
    fn command(
        control: &Rc<RefCell<Control>>,
        operation: u8,
        log: Cage<String>,
        count: Cage<usize>,
        theme: &Theme,
    ) -> glory::ui::Result {
        let Some(handle) = control.borrow().handle.clone() else {
            return Ok(());
        };
        match operation {
            0 | 1 | 5 => {
                let choices: &[bool] = if operation == 5 {
                    &[false, true]
                } else if operation == 1 {
                    &[true]
                } else {
                    &[false]
                };
                for &confirmation in choices {
                    let observed = handle.clone();
                    let callback = move |result| {
                        log.revise(|value| *value = format!("{result:?}"));
                        if let Ok(pending) = observed.pending() {
                            count.revise(|value| *value = pending);
                        }
                    };
                    let content = control.clone();
                    let theme = theme.clone();
                    let request = if confirmation {
                        confirmation_request(
                            "Publish draft?",
                            "This confirmation waits for the current dialog.",
                        )
                        .confirm_label("Publish now")
                        .close_label("Keep draft")
                        .on_result(callback)
                    } else {
                        modal_request("Managed draft", "A centrally opened dialog.")
                            .close_label("Close managed draft")
                            .content(move || managed_content(content.clone(), theme.clone()))
                            .on_result(callback)
                    };
                    let token = handle.open(request)?;
                    if confirmation {
                        control.borrow_mut().second = Some(token);
                    } else {
                        control.borrow_mut().first = Some(token);
                    }
                }
            }
            2 => {
                let token = control.borrow().first.clone();
                if let Some(token) = token {
                    handle.update(
                        &token,
                        "Updated managed draft",
                        "Metadata updates preserve the content editor.",
                    )?;
                }
            }
            3 => {
                let token = control.borrow().second.clone();
                if let Some(token) = token {
                    handle.close(&token)?;
                }
            }
            _ => {
                handle.close_all()?;
            }
        }
        count.revise(|value| *value = handle.pending().unwrap_or(0));
        Ok(())
    }
    fn managed_content(control: Rc<RefCell<Control>>, theme: Theme) -> impl Widget {
        widget(move |scope| {
            let update = control.clone();
            let weak = scope.downgrade();
            let cancel = control.clone();
            let weak_cancel = scope.downgrade();
            sapid::stack()
                .gap(12.)
                .fill((
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Managed editor")
                        .default_value("Keep this draft"),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Update current modal")
                        .on_press(move || {
                            let command = {
                                let c = update.borrow();
                                c.handle.clone().zip(c.first.clone())
                            };
                            if let Some((handle, token)) = command
                                && let Err(error) = handle.update(
                                    &token,
                                    "Updated managed draft",
                                    "Metadata updates preserve the content editor.",
                                )
                                && let Some(scope) = weak.upgrade()
                            {
                                scope.report_error(error)
                            }
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Cancel waiting confirmation")
                        .on_press(move || {
                            let command = {
                                let c = cancel.borrow();
                                c.handle.clone().zip(c.second.clone())
                            };
                            if let Some((handle, token)) = command
                                && let Err(error) = handle.close(&token)
                                && let Some(scope) = weak_cancel.upgrade()
                            {
                                scope.report_error(error)
                            }
                        }),
                ))
                .show_in(scope)
        })
    }
}
