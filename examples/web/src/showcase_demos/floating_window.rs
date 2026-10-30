use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::floating_window_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{FloatingWindowHandle, Theme, WindowBounds, WindowCommand, floating_window};

    pub fn floating_window_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let bounds = scope.cage(WindowBounds {
                x: 100.,
                y: 120.,
                width: 560.,
                height: 420.,
            });
            let open = scope.cage(true);
            let enabled = scope.cage(true);
            let reject = scope.cage(false);
            let mounted = scope.cage(true);
            let proposals = scope.cage(0usize);
            let primary = scope.cage(None::<FloatingWindowHandle>);
            let secondary = scope.cage(None::<FloatingWindowHandle>);
            let stale = scope.cage(String::new());
            let modal = scope.cage(false);
            let draft = scope.cage("Keep this editor".to_owned());
            let outside_theme = theme.clone();
            let control_theme = theme.clone();
            let window_theme = theme.clone();
            view()
                .fill((
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Before windows"),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Raise working window")
                        .on_press(move || {
                            if let Some(handle) = primary.get() {
                                let _ = handle.raise();
                            }
                        }),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Raise secondary window")
                        .on_press(move || {
                            if let Some(handle) = secondary.get() {
                                let _ = handle.raise();
                            }
                        }),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Toggle geometry refusal")
                        .on_press(move || reject.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Toggle window enabled")
                        .on_press(move || enabled.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Toggle working window")
                        .on_press(move || open.revise(|value| *value = !*value)),
                ))
                .fill((
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Reset window bounds")
                        .on_press(move || {
                            bounds.revise(|value| {
                                *value = WindowBounds {
                                    x: 100.,
                                    y: 120.,
                                    width: 560.,
                                    height: 420.,
                                }
                            })
                        }),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Retire windows")
                        .on_press(move || mounted.revise(|value| *value = false)),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Rebuild windows")
                        .on_press(move || mounted.revise(|value| *value = true)),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("Invoke retired window command")
                        .on_press(move || {
                            let rejected = primary.get().is_some_and(|handle| {
                                handle.command(WindowCommand::MoveRight).is_err()
                            });
                            stale.revise(|value| {
                                *value = if rejected { "retired denied" } else { "live" }.into()
                            });
                        }),
                    text()
                        .class("window-bounds-status")
                        .content(scope.bond(move || {
                            let value = bounds.get();
                            format!(
                                "x={} y={} width={} height={} proposals={} reject={} enabled={} {}",
                                value.x,
                                value.y,
                                value.width,
                                value.height,
                                proposals.get(),
                                reject.get(),
                                enabled.get(),
                                stale.get()
                            )
                        })),
                    when(mounted, move || {
                        let theme = window_theme.clone();
                        view().fill((
                            floating_window("working")
                                .theme(theme.clone())
                                .title("Working window")
                                .bounds(bounds)
                                .open(open)
                                .enabled(enabled)
                                .on_ready(move |handle| {
                                    primary.revise(|value| *value = Some(handle))
                                })
                                .on_bounds_change(move |next| {
                                    proposals.revise(|value| *value += 1);
                                    if !reject.get() {
                                        bounds.revise(|value| *value = next)
                                    }
                                })
                                .on_open_change(move |next| open.revise(|value| *value = next))
                                .content(move || {
                                    sapid::text_field()
                                        .theme(theme.clone())
                                        .name("Window draft")
                                        .value(draft)
                                        .on_value_change(move |value| {
                                            draft.revise(|current| *current = value)
                                        })
                                }),
                            floating_window("secondary")
                                .theme(window_theme.clone())
                                .title("Secondary window")
                                .default_bounds(WindowBounds {
                                    x: 320.,
                                    y: 200.,
                                    width: 440.,
                                    height: 340.,
                                })
                                .on_ready(move |handle| {
                                    secondary.revise(|value| *value = Some(handle))
                                })
                                .content(|| text().content("Independent nonmodal window")),
                        ))
                    }),
                    sapid::dialog("window-blocker")
                        .theme(control_theme.clone())
                        .title("Modal blocker")
                        .label("Open modal blocker")
                        .open(modal)
                        .on_open_change(move |value| modal.revise(|current| *current = value))
                        .content(move || {
                            view().fill((
                                sapid::text_field()
                                    .theme(control_theme.clone())
                                    .name("Modal editor"),
                                sapid::button()
                                    .theme(control_theme.clone())
                                    .label("Try blocked window raise")
                                    .on_press(move || {
                                        if let Some(handle) = primary.get() {
                                            let _ = handle.raise();
                                        }
                                    }),
                            ))
                        }),
                    sapid::button()
                        .theme(outside_theme.clone())
                        .label("After windows"),
                ))
                .show_in(scope)
        })
    }
}
