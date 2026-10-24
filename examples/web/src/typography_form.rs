use glory::prelude::*;
use glory::render::{EventKind, LinkOpen};
use sapid::{TextVariant, Theme};

pub struct State {
    pub content: Cage<String>,
    pub href: Cage<String>,
    pub open: Cage<LinkOpen>,
    pub navigations: Cage<usize>,
    pub last: Cage<String>,
}
pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
    let content = scope.cage("A concise paragraph with reactive content.".to_owned());
    let href = scope.cage("https://example.invalid/documentation".to_owned());
    let open = scope.cage(LinkOpen::NewContext);
    let navigations = scope.cage(0usize);
    let last = scope.cage(String::new());
    let ordered = scope.cage(true);
    let stage = scope.cage(false);
    let status = scope.bond(move || {
        format!(
            "requests={}; ordered={}; {}",
            navigations.get(),
            ordered.get(),
            last.get()
        )
    });
    let widget=sapid::typography().theme(theme).class("typography-demo").content(move|context|(
  context.title(1).class("type-title").content("Readable by design"),
  context.text().class("type-lead").variant(TextVariant::Lead).content("Clear hierarchy with semantic text and links."),
  context.title(2).class("type-subtitle").content("Typography and navigation"),
  context.text().class("type-body").content(content),
  context.text().class("type-muted").variant(TextVariant::Muted).content("Muted supporting content remains accessible."),
  context.text().class("type-small").variant(TextVariant::Small).content("A smaller supporting paragraph."),
  context.text().class("type-caption").variant(TextVariant::Caption).content("Caption · concise context"),
  context.text().class("type-truncate").truncate(true).style(Style::new().width(Length::Px(220.0))).content("This full sentence remains accessible while its visual presentation uses a real ellipsis."),
  context.anchor().class("type-request").href(href).open_in(open).label("Open documentation").on_navigate(move|request|{
   navigations.revise(|count|*count+=1);last.revise(|last|*last=format!("{:?}:{}",request.open,request.href));stage.revise(|value|*value=true);
  }).on(EventKind::Activate,move|_|ordered.revise(|value|*value&=stage.get_untracked())),
  context.anchor().class("type-passive").href("#passive").label("Native fragment navigation"),
  context.text().class("type-status").variant(TextVariant::Small).content(status),
  context.text().class("type-unstyled").unstyled().font_size(17.0).content("Unstyled text keeps explicit size."),
  ));
    (
        widget,
        State {
            content,
            href,
            open,
            navigations,
            last,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use glory::web::WebApp;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! {static TYPOGRAPHY:RefCell<Option<(WebApp,State)>>=const{RefCell::new(None)};}
    type Result = std::result::Result<(), JsValue>;
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_typography(hydrate: bool, hue: String, scheme: String) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing typography root"))?;
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
        let old = TYPOGRAPHY.with(|slot| slot.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    #[wasm_bindgen]
    pub fn typography_text(value: String) -> Result {
        TYPOGRAPHY.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted typography"))?;
            app.app().pump().map_err(js)?;
            state.content.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn typography_href(value: String) -> Result {
        TYPOGRAPHY.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted typography"))?;
            app.app().pump().map_err(js)?;
            state.href.revise(|v| *v = value);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn typography_external(value: bool) -> Result {
        TYPOGRAPHY.with(|slot| {
            let slot = slot.borrow();
            let (app, state) = slot.as_ref().ok_or_else(|| js("unmounted typography"))?;
            app.app().pump().map_err(js)?;
            state.open.revise(|v| {
                *v = if value {
                    LinkOpen::NewContext
                } else {
                    LinkOpen::Current
                }
            });
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn unmount_typography() {
        let old = TYPOGRAPHY.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
