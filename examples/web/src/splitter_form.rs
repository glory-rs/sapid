use glory::prelude::*;
use glory::render::{DragPhase, Event, EventKind, Orientation};
use glory::style::{Length, Style};
use sapid::{Theme, splitter};

pub struct State {
    pub value: Cage<f64>,
    pub accept: Cage<bool>,
    pub enabled: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let value = scope.cage(50.0);
    let accept = scope.cage(true);
    let enabled = scope.cage(true);
    let changes = scope.cage(0usize);
    let commits = scope.cage(0usize);
    let request = scope.cage(50.0);
    let phase = scope.cage(String::from("idle"));
    let stage = scope.cage(false);
    let ordered = scope.cage(true);
    let status = scope.bond(move || {
        format!(
            "value={:.2};request={:.2};changes={};commits={};phase={};ordered={}",
            value.get(),
            request.get(),
            changes.get(),
            commits.get(),
            phase.get(),
            ordered.get()
        )
    });
    let horizontal = splitter("workspace")
        .theme(theme.clone())
        .name("Workspace divider")
        .class("horizontal-splitter")
        .value(value)
        .enabled(enabled)
        .min(0.0)
        .max(100.0)
        .style(
            Style::new()
                .width(Length::Px(606.0))
                .height(Length::Px(180.0)),
        )
        .first(|| {
            glory::ui::input()
                .name("First draft")
                .class("first-draft")
                .value("Preserve the first draft")
        })
        .second(|| {
            glory::ui::input()
                .name("Second draft")
                .class("second-draft")
                .value("Preserve the second draft")
        })
        .on_value_change(move |next| {
            changes.revise(|v| *v += 1);
            request.revise(|v| *v = next);
            stage.revise(|v| *v = true);
            if accept.get_untracked() {
                value.revise(|v| *v = next);
            }
        })
        .on_value_commit(move |_| {
            commits.revise(|v| *v += 1);
        })
        .on(EventKind::Range, move |_| {
            ordered.revise(|v| *v &= stage.get_untracked());
            stage.revise(|v| *v = false);
        })
        .on(EventKind::Drag, move |event| {
            if let Event::Drag { phase: current, .. } = event {
                if matches!(current, DragPhase::Move | DragPhase::Drop) {
                    ordered.revise(|v| *v &= stage.get_untracked());
                    stage.revise(|v| *v = false);
                }
                phase.revise(|v| *v = format!("{current:?}"));
            }
        });
    let vertical = splitter("vertical")
        .theme(theme.clone())
        .name("Vertical divider")
        .class("vertical-splitter")
        .axis(Orientation::Vertical)
        .style(
            Style::new()
                .width(Length::Px(606.0))
                .height(Length::Px(206.0)),
        )
        .first(|| glory::ui::text().content("Top pane"))
        .second(|| glory::ui::text().content("Bottom pane"));
    (
        glory::ui::view().class("splitter-demo").fill((
            sapid::title(1)
                .theme(theme.clone())
                .content("Resizable Splitter"),
            horizontal,
            sapid::text()
                .theme(theme.clone())
                .class("splitter-status")
                .content(status),
            vertical,
            sapid::button().theme(theme.clone()).label("After splitters"),
        )),
        State {
            value,
            accept,
            enabled,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static SPLITTER:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_splitter(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing splitter root"))?;
        let theme = crate::themes::select(
            if hue == "green" { "green" } else { "blue" },
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
        let old = SPLITTER.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn splitter_policy(accept: bool, enabled: bool) -> Result {
        SPLITTER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.accept.revise(|v| *v = accept);
            state.enabled.revise(|v| *v = enabled);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn splitter_value(value: f64) -> Result {
        SPLITTER.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.value.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_splitter() {
        let old = SPLITTER.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
