use glory::prelude::*;
use glory::render::{EventKind, ScrollAxes};
use glory::style::{Length, Style};
use sapid::{ScrollHandle, Theme, scroll_area, scroller};
use std::{cell::RefCell, rc::Rc};
pub struct State {
    pub handle: Rc<RefCell<Option<ScrollHandle>>>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let handle = Rc::new(RefCell::new(None));
    let saved = handle.clone();
    let observation = scope.cage(String::from("metrics pending"));
    let scrolls = scope.cage(0_usize);
    let stage = scope.cage(false);
    let ordered = scope.cage(true);
    let status = scope.bond(move || {
        format!(
            "{};scrolls={};ordered={}",
            observation.get(),
            scrolls.get(),
            ordered.get()
        )
    });
    let nested_theme = theme.clone();
    let area = scroll_area("documents")
        .theme(theme.clone())
        .name("Documents viewport")
        .axes(ScrollAxes::Both)
        .class("main-scroll-area")
        .style(Style::new().height(Length::Px(260.0)))
        .on_ready(move |value| *saved.borrow_mut() = Some(value))
        .on_metrics_change(move |metrics| {
            observation.revise(|value| {
                *value = format!(
                    "x={:.0};y={:.0};viewport={:.0}x{:.0};content={:.0}x{:.0}",
                    metrics.x,
                    metrics.y,
                    metrics.viewport_width,
                    metrics.viewport_height,
                    metrics.content_width,
                    metrics.content_height
                )
            })
        })
        .on_scroll_change(move |_| {
            scrolls.revise(|value| *value += 1);
            stage.revise(|value| *value = true);
        })
        .on(EventKind::Scroll, move |_| {
            ordered.revise(|value| *value &= stage.get_untracked());
            stage.revise(|value| *value = false);
        })
        .content(move |_| {
            glory::ui::view()
                .style(
                    Style::new()
                        .width(Length::Px(1400.0))
                        .height(Length::Px(1600.0)),
                )
                .fill((
                    glory::ui::input()
                        .name("Document draft")
                        .class("scroll-draft")
                        .value("Keep this draft while scrolling"),
                    sapid::text()
                        .theme(nested_theme.clone())
                        .content("Outer viewport content"),
                    scroll_area("nested")
                        .theme(nested_theme.clone())
                        .name("Nested viewport")
                        .class("nested-scroll-area")
                        .style(
                            Style::new()
                                .width(Length::Px(280.0))
                                .height(Length::Px(120.0)),
                        )
                        .content(|context| {
                            glory::ui::view()
                                .style(Style::new().height(Length::Px(700.0)))
                                .fill(sapid::text().theme(context.theme).content("Nested content"))
                        }),
                    sapid::text()
                        .theme(nested_theme.clone())
                        .content("End of the explicit outer content box"),
                ))
        });
    let strip = scroller("strip")
        .area(|area| {
            area.theme(theme.clone())
                .name("Horizontal pages")
                .class("horizontal-scroll-area")
                .style(
                    Style::new()
                        .width(Length::Px(480.0))
                        .height(Length::Px(100.0)),
                )
        })
        .content(|context| {
            let page = |label| {
                sapid::text()
                    .theme(context.theme.clone())
                    .content(label)
                    .style(Style::new().width(Length::Px(600.0)))
            };
            glory::ui::view()
                .style(
                    Style::new()
                        .width(Length::Px(2400.0))
                        .height(Length::Px(80.0))
                        .display(glory::style::Display::Flex),
                )
                .fill((
                    page("First content page"),
                    page("Second content page"),
                    page("Third content page"),
                    page("Fourth content page"),
                ))
        });
    (
        glory::ui::view().class("scroll-demo").fill((
            sapid::title(1)
                .theme(theme.clone())
                .content("ScrollArea and Scroller"),
            area,
            sapid::text()
                .theme(theme)
                .class("scroll-status")
                .content(status),
            strip,
        )),
        State { handle },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use wasm_bindgen::prelude::*;
    thread_local! {static AREA:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(value: impl std::fmt::Display) -> JsValue {
        value.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_scroll_area(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing scroll root"))?;
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
        let old = AREA.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn scroll_area_to(x: f64, y: f64) -> Result {
        AREA.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted scroll area"))?;
            app.app().pump().map_err(js)?;
            state
                .handle
                .borrow()
                .as_ref()
                .unwrap()
                .scroll_to(x, y)
                .map_err(js)?;
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn scroll_area_batch() -> Result {
        AREA.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted scroll area"))?;
            app.app().pump().map_err(js)?;
            let handle = state.handle.borrow();
            let handle = handle.as_ref().unwrap();
            handle.scroll_by(0.0, -100.0).map_err(js)?;
            handle.scroll_by(0.0, 50.0).map_err(js)?;
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_scroll_area() {
        let old = AREA.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
