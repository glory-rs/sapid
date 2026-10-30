use glory::prelude::*;
use glory::render::{Event, EventKind};
use glory::style::{Length, Style};
use sapid::{
    AffixHorizontal, AffixOffset, AffixVertical, ShellLayout, Theme, affix, affix_container,
};

pub struct State {
    pub offset: Cage<Option<AffixOffset>>,
    pub visible: Cage<bool>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let layout = ShellLayout::new(scope, 1000.0, 700.0).unwrap();
    let driver = layout.clone();
    let offset = scope.cage(None::<AffixOffset>);
    let visible = scope.cage(true);
    let mobile = layout.mobile();
    let position = scope.bond(move || {
        offset.get().unwrap_or_else(|| {
            let inset = if mobile.get() { 8.0 } else { 24.0 };
            AffixOffset {
                horizontal: AffixHorizontal::Right(inset),
                vertical: AffixVertical::Bottom(inset),
            }
        })
    });
    let status = scope.bond(move || format!("mobile={};visible={}", mobile.get(), visible.get()));
    let fixed = affix("viewport")
        .theme(theme.clone())
        .name("Viewport affix")
        .class("viewport-affix")
        .offset(position)
        .visible(visible)
        .fill(
            glory::ui::input()
                .name("Floating draft")
                .class("affix-draft")
                .value("Keep floating draft"),
        );
    let container_theme = theme.clone();
    let container = affix_container("container")
        .theme(theme.clone())
        .name("Affix container")
        .class("demo-affix-container")
        .style(
            Style::new()
                .width(Length::Px(500.0))
                .height(Length::Px(240.0)),
        )
        .content(move |context| {
            glory::ui::view()
                .style(Style::new().height(Length::Px(1000.0)))
                .fill((
                    glory::ui::view().style(Style::new().height(Length::Px(80.0))),
                    affix("sticky")
                        .theme(container_theme.clone())
                        .name("Sticky affix")
                        .class("sticky-affix")
                        .sticky(context)
                        .offset(AffixOffset {
                            horizontal: AffixHorizontal::Left(12.0),
                            vertical: AffixVertical::Top(12.0),
                        })
                        .style(Style {
                            background: Some(container_theme.surface),
                            ..Style::new()
                                .width(Length::Px(160.0))
                                .height(Length::Px(34.0))
                        })
                        .fill(glory::ui::button().label("Sticky action")),
                    affix("absolute")
                        .theme(container_theme.clone())
                        .name("Container affix")
                        .class("absolute-affix")
                        .within(context)
                        .style(
                            Style::new()
                                .width(Length::Px(120.0))
                                .height(Length::Px(34.0)),
                        )
                        .fill(glory::ui::button().label("Container action")),
                    glory::ui::text()
                        .content("Explicit scroll content; sticky remains within this container."),
                ))
        });
    (
        glory::ui::view()
            .class("affix-demo")
            .on(EventKind::Resize, move |event| {
                if let Event::Resize { size, .. } = event {
                    driver.observe_width(size.width).unwrap();
                }
            })
            .fill((
                sapid::title(1).theme(theme.clone()).content("Affix"),
                sapid::text()
                    .theme(theme.clone())
                    .class("affix-status")
                    .content(status),
                container,
                fixed,
                glory::ui::view()
                    .style(Style::new().height(Length::Px(1500.0)))
                    .fill(
                        sapid::text()
                            .theme(theme)
                            .content("Scroll the page to verify viewport fixation."),
                    ),
            )),
        State { offset, visible },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static AFFIX:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_affix(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing affix root"))?;
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
        let old = AFFIX.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn affix_offset(right: f64, bottom: f64) -> Result {
        AFFIX.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            state.offset.revise(|v| {
                *v = Some(AffixOffset {
                    horizontal: AffixHorizontal::Right(right),
                    vertical: AffixVertical::Bottom(bottom),
                })
            });
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn affix_visible(visible: bool) -> Result {
        AFFIX.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            state.visible.revise(|v| *v = visible);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_affix() {
        let old = AFFIX.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
