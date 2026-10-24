use glory::prelude::*;
use glory::render::EventKind;
use sapid::{Theme, chat::*};
fn key(value: &str) -> TraceKey {
    TraceKey::new(value).unwrap()
}
fn public(value: &str) -> TracePayload {
    TracePayload::Public(TraceText::new(value).unwrap())
}
pub fn trace_data(updated: bool, child: bool) -> TraceData {
    let mut records = vec![
        TraceRecord::new(
            key("request"),
            TraceKind::Span,
            if updated {
                "Updated request"
            } else {
                "Application request"
            },
        )
        .unwrap()
        .status(TraceStatus::Running)
        .elapsed_millis(Some(120))
        .unwrap()
        .input(public(
            "Literal <script>window.traceExecuted=true</script>\nNo command executes here.",
        ))
        .output(TracePayload::Redacted),
    ];
    if child {
        records.extend([
            tool_record().parent(key("request")),
            agent_record().parent(key("request")),
            TraceRecord::new(key("review"), TraceKind::Span, "Review result")
                .unwrap()
                .parent(key("agent"))
                .status(TraceStatus::Succeeded)
                .output(TracePayload::Public(
                    TraceText::prefix(
                        "Long public detail: αβγδ repeated public application text",
                        30,
                    )
                    .unwrap(),
                )),
        ]);
    }
    TraceData::new(records).unwrap()
}
pub fn tool_record() -> TraceRecord {
    TraceRecord::new(key("lookup"), TraceKind::Tool, "Lookup repository")
        .unwrap()
        .status(TraceStatus::Succeeded)
        .elapsed_millis(Some(35))
        .unwrap()
        .input(public("tool input: {\"path\":\"src/lib.rs\"}"))
        .output(public("Application-supplied tool result"))
}
pub fn agent_record() -> TraceRecord {
    TraceRecord::new(key("agent"), TraceKind::Subagent, "Review agent")
        .unwrap()
        .status(TraceStatus::Pending)
        .input(TracePayload::Redacted)
        .output(public("Only application-provided public summary"))
}
pub fn stack_data(updated: bool) -> StackTraceData {
    StackTraceData::new(public("Error: literal <img src=x onerror=window.stackExecuted=true>\nNo automatic filesystem links"), [
        StackFrame::new(key("parse"), "parse<script>").unwrap().location(StackLocation::new("src/parser.rs", if updated {44} else {12}, Some(3)).unwrap()),
        StackFrame::new(key("render"), "render_document").unwrap().location(StackLocation::new("src/render.rs", 28, None).unwrap()),
        StackFrame::new(key("opaque"), "native_frame_without_location").unwrap(),
    ]).unwrap()
}
#[derive(Clone)]
pub struct State {
    pub data: Cage<TraceData>,
    pub stack: Cage<StackTraceData>,
    pub expanded: Cage<TraceExpansion>,
    pub enabled: Cage<bool>,
    pub accept: Cage<bool>,
    pub visible: Cage<bool>,
    pub reference: Cage<Option<glory::render::NodeId>>,
}
pub fn form(scope: &Scope, theme: Theme, stack_only: bool) -> (impl Widget + use<>, State) {
    let data = scope.cage(trace_data(false, true));
    let stack = scope.cage(stack_data(false));
    let expanded = scope.cage(TraceExpansion::new([key("request"), key("lookup")]).unwrap());
    let enabled = scope.cage(true);
    let accept = scope.cage(true);
    let visible = scope.cage(true);
    let reference = scope.cage(None);
    let events = scope.cage(String::new());
    let editor = scope.cage("Keep caret here".to_owned());
    let palette = theme.clone();
    let body = glory::ui::when(visible, move || {
        let theme = palette.clone();
        widget(move |scope| {
            if stack_only {
                stack_trace(stack)
                    .theme(theme.clone())
                    .name("Application stack")
                    .class("stack-primary")
                    .node_ref(reference)
                    .enabled(enabled)
                    .on_location(move |request| {
                        events.revise(|v| {
                            v.push_str(&format!(
                                "location:{}:{};",
                                request.key.as_str(),
                                request.location.line()
                            ))
                        })
                    })
                    .on(EventKind::Activate, move |_| {
                        events.revise(|v| v.push_str("generic;"))
                    })
                    .show_in(scope)?;
                stack_trace(stack_data(false))
                    .theme(theme.clone())
                    .class("stack-passive")
                    .name("Passive stack")
                    .show_in(scope)?;
                stack_trace(stack_data(false))
                    .theme(theme.clone())
                    .class("stack-unstyled")
                    .unstyled()
                    .name("Unstyled stack")
                    .show_in(scope)?;
            } else {
                trace("fixture-trace", data)
                    .theme(theme.clone())
                    .name("Application trace")
                    .class("trace-primary")
                    .node_ref(reference)
                    .enabled(enabled)
                    .expanded(expanded)
                    .on_expand_change(move |proposal| {
                        events.revise(|v| {
                            v.push_str(&format!(
                                "toggle:{}:{};",
                                proposal.key.as_str(),
                                proposal.open
                            ))
                        });
                        if accept.get_untracked() {
                            expanded.revise(|v| *v = proposal.expanded);
                        }
                    })
                    .on_action(move |request| {
                        events.revise(|v| {
                            v.push_str(&format!("action:{};", request.key.as_str()))
                        })
                    })
                    .on(EventKind::Activate, move |_| {
                        events.revise(|v| v.push_str("generic;"))
                    })
                    .show_in(scope)?;
                tool("fixture-tool", tool_record())
                    .theme(theme.clone())
                    .class("tool-standalone")
                    .name("Standalone tool")
                    .default_open(true)
                    .on_action(move |a| {
                        events.revise(|v| v.push_str(&format!("tool:{};", a.key.as_str())))
                    })
                    .show_in(scope)?;
                subagent("fixture-agent", agent_record())
                    .theme(theme.clone())
                    .class("agent-standalone")
                    .name("Standalone agent")
                    .default_open(false)
                    .show_in(scope)?;
                trace("fixture-plain", TraceData::new([tool_record()]).unwrap())
                    .theme(theme.clone())
                    .class("trace-unstyled")
                    .unstyled()
                    .name("Unstyled trace")
                    .show_in(scope)?;
            }
            Ok(())
        })
    });
    let result = view()
        .class("trace-fixture")
        .style(
            Style::new()
                .display(glory::style::Display::Flex)
                .flex_direction(glory::style::FlexDirection::Column)
                .gap(14.)
                .min_width(Length::Px(0.)),
        )
        .fill((
            sapid::button()
                .theme(theme.clone())
                .label("Update accepted record")
                .on_press(move || {
                    data.revise(|v| *v = trace_data(true, true));
                    stack.revise(|v| *v = stack_data(true));
                }),
            sapid::button()
                .theme(theme.clone())
                .label("Remove trace children")
                .on_press(move || data.revise(|v| *v = trace_data(false, false))),
            sapid::button()
                .theme(theme.clone())
                .label("Restore trace children")
                .on_press(move || data.revise(|v| *v = trace_data(false, true))),
            sapid::button()
                .theme(theme.clone())
                .label("Toggle trace owner")
                .on_press(move || visible.revise(|v| *v = !*v)),
            sapid::text_field()
                .theme(theme.clone())
                .name("Unrelated trace editor")
                .class("trace-editor")
                .value(editor)
                .on_value_change(move |v| editor.revise(|s| *s = v)),
            body,
            sapid::text()
                .theme(theme.clone())
                .class("trace-events")
                .style(
                    Style::new()
                        .display(glory::style::Display::Block)
                        .min_width(Length::Px(0.))
                        .max_width(glory::style::Limit::Percent(100.))
                        .overflow_x(glory::style::Overflow::Auto),
                )
                .content(events),
            sapid::text()
                .theme(theme)
                .content("Tools executed by this display: 0; files opened: 0"),
        ));
    (
        result,
        State {
            data,
            stack,
            expanded,
            enabled,
            accept,
            visible,
            reference,
        },
    )
}
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    type Result<T = ()> = std::result::Result<T, JsValue>;
    thread_local! { static APP:RefCell<Option<(glory::web::WebApp,State)>>=const{RefCell::new(None)}; }
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    #[wasm_bindgen]
    pub fn mount_trace(hydrate: bool, hue: String, scheme: String, stack_only: bool) -> Result {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
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
            let (w, s) = form(scope, theme, stack_only);
            state = Some(s);
            w
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        let old = APP.with(|s| s.borrow_mut().replace((app, state.unwrap())));
        drop(old);
        Ok(())
    }
    fn update(f: impl FnOnce(&State)) -> Result {
        APP.with(|slot| {
            let slot = slot.borrow();
            let (app, s) = slot.as_ref().ok_or_else(|| js("unmounted"))?;
            app.app().pump().map_err(js)?;
            f(s);
            app.app().flush().map_err(js)
        })
    }
    #[wasm_bindgen]
    pub fn trace_policy(accept: bool, enabled: bool) -> Result {
        update(|s| {
            s.accept.revise(|v| *v = accept);
            s.enabled.revise(|v| *v = enabled);
        })
    }
    #[wasm_bindgen]
    pub fn trace_visible(value: bool) -> Result {
        update(|s| s.visible.revise(|v| *v = value))
    }
    #[wasm_bindgen]
    pub fn trace_retired() -> bool {
        APP.with(|s| {
            s.borrow()
                .as_ref()
                .is_none_or(|(_, s)| s.reference.get().is_none())
        })
    }
}
