use glory::prelude::*;
use glory::render::NodeId;
use sapid::{AnchorAlign, AnchorPlacement, AnchorSide, Theme, popover};

#[derive(Clone, Copy)]
pub struct Case {
    pub index: usize,
    pub edge: bool,
    pub flip: bool,
    pub shift: bool,
    pub revision: u32,
}
pub struct State {
    pub case: Cage<Case>,
    pub open: Cage<bool>,
    pub reject: Cage<bool>,
    pub trigger: Cage<Option<NodeId>>,
    pub dismissals: Cage<usize>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let case = scope.cage(Case {
        index: 0,
        edge: false,
        flip: true,
        shift: true,
        revision: 0,
    });
    let open = scope.cage(false);
    let reject = scope.cage(false);
    let trigger = scope.cage(None);
    let dismissals = scope.cage(0);
    let widget = view().fill((
        button().label("Before placement"),
        each(scope.bond(move || vec![case.get()]))
            .key(|case| case.revision)
            .children(move |entry| {
                let row = entry.value;
                let theme = theme.clone();
                widget(move |_scope| {
                    let config = row.get();
                    let side = [
                        AnchorSide::Top,
                        AnchorSide::Bottom,
                        AnchorSide::Left,
                        AnchorSide::Right,
                    ][config.index / 3];
                    let align = [AnchorAlign::Start, AnchorAlign::Center, AnchorAlign::End]
                        [config.index % 3];
                    let placement = AnchorPlacement {
                        side,
                        align,
                        gap: 12.,
                        viewport_padding: 16.,
                        flip: config.flip,
                        shift: config.shift,
                    };
                    view()
                        .class("placement-anchor-box")
                        .style(
                            Style::new()
                                .position(Position::Fixed)
                                .left(Length::Px(if config.edge { 0. } else { 480. }))
                                .top(Length::Px(if config.edge { 0. } else { 320. })),
                        )
                        .fill(
                            popover("placement")
                                .theme(theme.clone())
                                .label("Open placement")
                                .close_label("Close placement")
                                .trigger_ref(trigger)
                                .trigger_content(|| text().content("Visual trigger"))
                                .close_content(|| text().content("Visual close"))
                                .placement(placement)
                                .open(open)
                                .style(
                                    Style::new()
                                        .width(Length::Px(160.))
                                        .min_height(Length::Px(90.)),
                                )
                                .on_open_change(move |value| {
                                    if !reject.get() {
                                        open.revise(|old| *old = value)
                                    }
                                })
                                .on_dismiss(move |_| dismissals.revise(|value| *value += 1))
                                .content(|| input().name("Placement draft").value("draft")),
                        )
                        .show_in(_scope)
                })
            }),
        text().class("placement-status").content(
            scope.bond(move || format!("open:{} dismissals:{}", open.get(), dismissals.get())),
        ),
    ));
    (
        widget,
        State {
            case,
            open,
            reject,
            trigger,
            dismissals,
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
    pub fn mount_popover_placement(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> std::result::Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing placement root"))?;
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
    pub fn popover_placement_case(
        index: u32,
        edge: bool,
        flip: bool,
        shift: bool,
    ) -> std::result::Result<(), JsValue> {
        if index >= 12 {
            return Err(js("invalid placement index"));
        }
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("placement unmounted"))?;
            app.app().pump().map_err(js)?;
            let revision = state
                .case
                .get()
                .revision
                .checked_add(1)
                .ok_or_else(|| js("placement fixture revision exhausted"))?;
            state.open.revise(|v| *v = false);
            state.case.revise(|old| {
                *old = Case {
                    index: index as usize,
                    edge,
                    flip,
                    shift,
                    revision,
                }
            });
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn popover_placement_state(
        kind: String,
        value: bool,
    ) -> std::result::Result<(), JsValue> {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("placement unmounted"))?;
            app.app().pump().map_err(js)?;
            match kind.as_str() {
                "open" => state.open.revise(|v| *v = value),
                "reject" => state.reject.revise(|v| *v = value),
                "anchor-hidden" => {
                    let id = state
                        .trigger
                        .get()
                        .ok_or_else(|| js("missing placement trigger"))?;
                    app.app()
                        .scope()
                        .patch(
                            id,
                            glory::render::Patch::Style(if value {
                                glory::render::Change::Set(Box::new(
                                    Style::new().display(Display::None),
                                ))
                            } else {
                                glory::render::Change::Clear
                            }),
                        )
                        .map_err(js)?;
                }
                _ => return Err(js("invalid placement state")),
            }
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_popover_placement() {
        APP.with(|slot| drop(slot.borrow_mut().take()));
    }
}
