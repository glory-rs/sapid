use glory::prelude::*;
use glory::render::{Current, EventKind};
use glory::style::{Edges, Length, Style};
use sapid::{ShellLayout, Theme, app_shell, sidebar};

pub struct State {
    pub accept: Cage<bool>,
    pub collapsed: Cage<bool>,
    pub open: Cage<bool>,
}

pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let accept = scope.cage(true);
    let collapsed = scope.cage(false);
    let open = scope.cage(false);
    let requests = scope.cage(0_usize);
    let creations = scope.cage(0_usize);
    let stage = scope.cage(false);
    let ordered = scope.cage(true);
    let resize_ordered = scope.cage(true);
    let width_stage = scope.cage(false);
    let widths = scope.cage(0_usize);
    let layout = ShellLayout::new(scope, 1024.0, 768.0).expect("finite shell layout");
    let nav_theme = theme.clone();
    let main_theme = theme.clone();
    let shell = app_shell("demo")
        .theme(theme)
        .class("shell-demo")
        .layout(layout)
        .style(Style::new().min_height(Length::Px(640.0)))
        .sidebar(
            sidebar("primary")
                .name("Workspace navigation")
                .collapsed(collapsed)
                .mobile_open(open)
                .on_collapsed_change(move |next| {
                    stage.revise(|value| *value = true);
                    requests.revise(|value| *value += 1);
                    if accept.get_untracked() {
                        collapsed.revise(|value| *value = next);
                    }
                })
                .on_mobile_open_change(move |next| {
                    stage.revise(|value| *value = true);
                    requests.revise(|value| *value += 1);
                    if accept.get_untracked() {
                        open.revise(|value| *value = next);
                    }
                })
                .on(EventKind::Activate, move |_| {
                    ordered.revise(|value| *value &= stage.get_untracked());
                    stage.revise(|value| *value = false);
                })
                .on(EventKind::Dismiss, move |_| {
                    ordered.revise(|value| *value &= stage.get_untracked());
                    stage.revise(|value| *value = false);
                })
                .content(move |_| {
                    creations.revise(|value| *value += 1);
                    glory::ui::view()
                        .style(Style::new().padding(Edges::all(12.0)))
                        .fill((
                            sapid::anchor()
                                .theme(nav_theme.clone())
                                .label("Overview")
                                .href("/overview")
                                .current(Some(Current::Page))
                                .on_navigate(|_| {}),
                            sapid::nav_link("projects")
                                .theme(nav_theme.clone())
                                .label("Projects")
                                .href("/projects")
                                .on_navigate(|_| {})
                                .children(|| {
                                    sapid::anchor()
                                        .label("Recent projects")
                                        .href("/projects/recent")
                                        .on_navigate(|_| {})
                                }),
                            glory::ui::input()
                                .name("Sidebar draft")
                                .class("shell-draft")
                                .value("Persistent sidebar note"),
                        ))
                }),
        )
        .header(move |context| {
            let handle = context.sidebar.expect("configured sidebar");
            let width = context.layout.width();
            let mobile = handle.mobile();
            let current_collapsed = handle.collapsed();
            let current_open = handle.open();
            glory::ui::component(move |scope| {
                let status = scope.bond(move || {
                    format!(
                        "width={:.0};mobile={};collapsed={};open={};requests={};creations={};ordered={};resize_ordered={};widths={}",
                        width.get(), mobile.get(), current_collapsed.get(), current_open.get(),
                        requests.get(), creations.get(), ordered.get(), resize_ordered.get(), widths.get()
                    )
                });
                glory::ui::text().content(status).class("shell-status").show_in(scope)
            })
        })
        .main(move |_| {
            glory::ui::view()
                .style(Style::new().padding(Edges::all(20.0)))
                .fill((
                    sapid::title(1).theme(main_theme.clone()).content("Workspace"),
                    sapid::text().content("The same navigation tree moves between its desktop dock and the host-owned mobile drawer."),
                    glory::ui::input().name("Main note").value("Main content stays mounted"),
                ))
        })
        .footer(|_| sapid::text().content("Sapid application footer"))
        .on_width_change(move |_| {
            width_stage.revise(|value| *value = true);
            widths.revise(|value| *value += 1);
        })
        .on(EventKind::Resize, move |_| {
            // Repeated equal measurements need no typed notification.
            if width_stage.get_untracked() {
                resize_ordered.revise(|value| *value &= widths.get_untracked() > 0);
                width_stage.revise(|value| *value = false);
            }
        });
    (
        shell,
        State {
            accept,
            collapsed,
            open,
        },
    )
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static SHELL:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(value: impl std::fmt::Display) -> JsValue {
        value.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_shell(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing shell root"))?;
        let theme = Theme::new(
            if hue == "green" {
                sapid::Hue::Green
            } else {
                sapid::Hue::Blue
            },
            if scheme == "dark" {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        );
        let mut state = None;
        let build = |scope: &Scope| {
            let (widget, source) = form(scope, theme);
            state = Some(source);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = SHELL.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn shell_policy(accept: bool) -> Result {
        SHELL.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted shell"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|value| *value = accept);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn shell_controls(collapsed: bool, open: bool) -> Result {
        SHELL.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted shell"))?;
            app.app().pump().map_err(js)?;
            state.collapsed.revise(|value| *value = collapsed);
            state.open.revise(|value| *value = open);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_shell() {
        let old = SHELL.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
