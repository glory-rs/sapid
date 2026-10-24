use glory::prelude::*;
use sapid::{CommandHandle, CommandResults, Theme, command, command_action, command_group};

pub struct State {
    pub open: Cage<bool>,
    pub reject_open: Cage<bool>,
    pub reject_search: Cage<bool>,
    pub disabled: Cage<bool>,
    pub async_handle: Cage<Option<CommandHandle>>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let open = scope.cage(false);
    let reject_open = scope.cage(false);
    let reject_search = scope.cage(false);
    let disabled = scope.cage(false);
    let query = scope.cage(String::new());
    let root = scope.cage(None);
    let shortcut = scope.cage(None::<CommandHandle>);
    let weak = scope.downgrade();
    scope.effect(move || {
        if let (Some(target), Some(handle)) = (root.get(), shortcut.get())
            && let Err(error) = handle.register_shortcut(target)
            && let Some(scope) = weak.upgrade()
        {
            scope.report_error(error)
        }
    });
    let async_handle = scope.cage(None);
    let actions = scope.cage(String::new());
    let palette = command("application-commands")
        .label("Application commands")
        .theme(theme.clone())
        .open(open)
        .search(query)
        .enabled(scope.bond(move || !disabled.get()))
        .on_open_change(move |value| {
            if !reject_open.get() {
                open.revise(|old| *old = value)
            }
        })
        .on_search_change(move |value| {
            if !reject_search.get() {
                query.revise(|old| *old = value)
            }
        })
        .on_action(move |key| actions.revise(|old| *old = key))
        .on_ready(move |handle| shortcut.revise(|old| *old = Some(handle)))
        .group(command_group("files", "Files"))
        .actions([
            command_action("new", "New document")
                .group("files")
                .keywords("fresh create"),
            command_action("blocked", "Unavailable action")
                .group("files")
                .disabled(true),
            command_action("open", "Open document").group("files"),
            command_action("settings", "Settings").description("Application preferences"),
        ]);
    let many =
        command("many-commands")
            .label("Many commands")
            .theme(theme.clone())
            .actions((0..256).map(|index| {
                command_action(format!("action-{index}"), format!("Action {index}"))
            }))
            .on_action(move |key| actions.revise(|old| *old = key));
    let asynchronous = command("remote-commands")
        .label("Async commands")
        .theme(theme)
        .on_ready(move |handle| async_handle.revise(|old| *old = Some(handle)))
        .load_with(|query| async move {
            super::menu_form::local_delay(250).await;
            if query == "error" {
                Err("Command search failed".into())
            } else {
                Ok(CommandResults::new(vec![command_action(
                    "remote",
                    format!("Remote {query}"),
                )]))
            }
        })
        .on_action(move |key| actions.revise(|old| *old = format!("async-{key}")));
    (
        view()
            .node_ref(root)
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(16.0),
            )
            .fill((
                button().label("Before command"),
                palette,
                input().name("Native shortcut editor").value("Keep draft"),
                asynchronous,
                many,
                text().content(scope.bond(move || format!("Executed: {}", actions.get()))),
                button().label("After command"),
            )),
        State {
            open,
            reject_open,
            reject_search,
            disabled,
            async_handle,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static APP:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_command(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> std::result::Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing command root"))?;
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
            let (widget, value) = form(scope, theme);
            state = Some(value);
            widget
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        APP.with(|slot| *slot.borrow_mut() = Some((app, state.unwrap())));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn command_state(kind: String, value: bool) -> std::result::Result<(), JsValue> {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("command unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "open" => state.open.revise(|old| *old = value),
                "reject-open" => state.reject_open.revise(|old| *old = value),
                "reject-search" => state.reject_search.revise(|old| *old = value),
                "disabled" => state.disabled.revise(|old| *old = value),
                "cancel" => state
                    .async_handle
                    .get()
                    .ok_or_else(|| js("missing async handle"))?
                    .cancel()
                    .map_err(js)?,
                "refresh" => state
                    .async_handle
                    .get()
                    .ok_or_else(|| js("missing async handle"))?
                    .refresh()
                    .map_err(js)?,
                _ => return Err(js("unknown command state")),
            };
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_command() {
        APP.with(|slot| drop(slot.borrow_mut().take()));
    }
}
