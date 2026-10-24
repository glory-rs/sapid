use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::navigation_progress_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::ui::{Widget, widget};
    use sapid::{
        NavigationProgressHandle, NavigationProgressToken, Theme, button, navigation_progress,
    };
    use std::{cell::RefCell, rc::Rc};
    #[derive(Default)]
    struct Control {
        handle: Option<NavigationProgressHandle>,
        token: Option<NavigationProgressToken>,
        stale: Option<NavigationProgressToken>,
    }
    pub fn navigation_progress_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let control = Rc::new(RefCell::new(Control::default()));
            let ready = control.clone();
            let log = scope.cage("Idle".to_string());
            let mut controls = Vec::new();
            for (label, operation) in [
                ("Start navigation", 0),
                ("Report 60 percent", 1),
                ("Replace navigation", 2),
                ("Publish stale progress", 3),
                ("Finish navigation", 4),
                ("Cancel navigation", 5),
            ] {
                let control = control.clone();
                let weak = scope.downgrade();
                controls.push(
                    button()
                        .theme(theme.clone())
                        .label(label)
                        .on_press(move || {
                            let mut control = control.borrow_mut();
                            let Some(handle) = control.handle.clone() else {
                                return;
                            };
                            let result = (|| -> glory::ui::Result<String> {
                                if operation == 0 || operation == 2 {
                                    control.stale = control.token.take();
                                    control.token = Some(handle.start()?);
                                    return Ok(if operation == 0 {
                                        "Started"
                                    } else {
                                        "Replaced"
                                    }
                                    .into());
                                }
                                let token = if operation == 3 {
                                    control.stale.as_ref()
                                } else {
                                    control.token.as_ref()
                                };
                                let Some(token) = token else {
                                    return Ok("No operation".into());
                                };
                                let accepted = match operation {
                                    1 => handle.set(token, 60.)?,
                                    3 => handle.set(token, 90.)?,
                                    4 => handle.finish(token)?,
                                    _ => handle.cancel(token)?,
                                };
                                Ok(if accepted { "Accepted" } else { "Revoked" }.into())
                            })();
                            drop(control);
                            match result {
                                Ok(message) => log.revise(|value| *value = message),
                                Err(error) => {
                                    if let Some(scope) = weak.upgrade() {
                                        scope.report_error(error)
                                    }
                                }
                            }
                        }),
                );
            }
            sapid::stack()
                .gap(16.)
                .fill((
                    navigation_progress()
                        .theme(theme.clone())
                        .thickness(8.)
                        .name("Route loading")
                        .on_ready(move |handle| ready.borrow_mut().handle = Some(handle)),
                    sapid::group()
                        .wrap(glory::style::FlexWrap::Wrap)
                        .fill(controls),
                    glory::ui::text()
                        .content(log)
                        .class("navigation-progress-log"),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Navigation progress editor")
                        .default_value("Retained editor"),
                ))
                .show_in(scope)
        })
    }
}
