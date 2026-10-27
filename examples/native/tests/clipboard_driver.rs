#![cfg(feature = "clipboard")]
use glory::{
    blitz::{
        Config, Raster,
        clipboard::{Clipboard, ClipboardDriver, ClipboardError, Reply},
        shell::WindowApp,
    },
    render::*,
    ui::Widget,
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct Gate {
    result: Option<Result<String, ClipboardError>>,
    wake: Option<Waker>,
}
struct Wait(Arc<Mutex<Gate>>);
impl Future for Wait {
    type Output = Result<String, ClipboardError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut gate = self.0.lock().unwrap();
        if let Some(result) = gate.result.take() {
            return Poll::Ready(result);
        }
        let previous = gate.wake.replace(cx.waker().clone());
        drop(gate);
        drop(previous);
        Poll::Pending
    }
}
impl Drop for Wait {
    fn drop(&mut self) {
        let wake = self.0.lock().unwrap().wake.take();
        drop(wake);
    }
}
type RecordedCall = (Option<String>, Arc<Mutex<Gate>>);
#[derive(Default)]
struct Driver {
    calls: Mutex<Vec<RecordedCall>>,
}
impl Driver {
    fn call(&self, text: Option<String>) -> Wait {
        let gate = Arc::new(Mutex::new(Gate::default()));
        self.calls.lock().unwrap().push((text, gate.clone()));
        Wait(gate)
    }
    fn finish(&self, n: usize, result: Result<String, ClipboardError>) {
        let gate = self.calls.lock().unwrap()[n].1.clone();
        let wake = {
            let mut gate = gate.lock().unwrap();
            gate.result = Some(result);
            gate.wake.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }
    fn written(&self, n: usize) -> String {
        self.calls.lock().unwrap()[n].0.clone().unwrap()
    }
}
impl ClipboardDriver for Driver {
    fn write(&self, text: String) -> Reply<()> {
        let wait = self.call(Some(text));
        Box::pin(async move { wait.await.map(|_| ()) })
    }
    fn read(&self, limit: usize) -> Reply<String> {
        assert_eq!(limit, MAX_TAG_PASTE_BYTES);
        Box::pin(self.call(None))
    }
}
fn host() -> (WindowApp, Arc<Driver>) {
    let driver = Arc::new(Driver::default());
    let host = WindowApp::with_config(Config {
        clipboard: Some(Clipboard::new(driver.clone())),
        ..Default::default()
    });
    host.renderer().set_viewport(640, 480, 1.).unwrap();
    (host, driver)
}
fn settle(host: &WindowApp) {
    host.observe(&Context::from_waker(Waker::noop())).unwrap();
}
fn themes() -> [sapid::Theme; 4] {
    [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ]
}
fn find(host: &WindowApp, predicate: impl Fn(DescriptorView<'_>) -> bool) -> NodeId {
    let tree = host.app().tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if predicate(node.descriptor.view()) {
            return id;
        }
        pending.extend(&node.children);
    }
    panic!("missing native consumer")
}
fn pixels(host: &WindowApp) -> Vec<u8> {
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    assert_eq!(raster.pixels().len(), 640 * 480 * 4);
    raster.pixels().to_vec()
}
fn activate(host: &WindowApp, id: NodeId) {
    host.app().dispatch(id, Event::Activate).unwrap();
    settle(host);
}

#[test]
fn four_theme_copy_button_reports_actual_results_and_source_replacement_cancels_old_feedback() {
    for theme in themes() {
        let (host, driver) = host();
        let scope = host.app().scope();
        let source = scope.cage("Private 中😀\nsource".to_owned());
        let status = scope.cage(sapid::CopyState::Idle);
        let reference = scope.cage(None);
        host.mount(
            sapid::copy_button()
                .theme(theme.clone())
                .source(source)
                .state_ref(status)
                .node_ref(reference),
        )
        .unwrap();
        settle(&host);
        let id = reference.get().unwrap();
        let before = pixels(&host);
        activate(&host, id);
        assert_eq!(status.get(), sapid::CopyState::Pending);
        assert_eq!(driver.written(0), source.get());
        driver.finish(0, Ok(String::new()));
        settle(&host);
        assert_eq!(status.get(), sapid::CopyState::Written);
        assert_ne!(before, pixels(&host));
        for (index, error, expected) in [
            (1, ClipboardError::Denied, sapid::CopyState::Denied),
            (
                2,
                ClipboardError::Unavailable,
                sapid::CopyState::Unavailable,
            ),
            (3, ClipboardError::Failed, sapid::CopyState::Failed),
        ] {
            activate(&host, id);
            driver.finish(index, Err(error));
            settle(&host);
            assert_eq!(status.get(), expected);
        }
        activate(&host, id);
        source.revise(|s| *s = "replacement".into());
        host.app().flush().unwrap();
        driver.finish(4, Ok(String::new()));
        settle(&host);
        assert_eq!(status.get(), sapid::CopyState::Idle);
        activate(&host, id);
        assert_eq!(driver.written(5), "replacement");
        driver.finish(5, Ok(String::new()));
        settle(&host);
        assert_eq!(status.get(), sapid::CopyState::Written);
        host.app().clear().unwrap();
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}

#[test]
fn code_highlight_and_json_modes_copy_the_complete_accepted_source_in_four_themes() {
    for theme in themes() {
        for mode in 0..3 {
            let (host, driver) = host();
            let state = host.app().scope().cage(sapid::CopyState::Idle);
            let source = if mode == 0 {
                "let 中文 = \"😀\";\n// exact source\n"
            } else {
                "{\"中文\": [1, \"😀\"], \"exact\": true}"
            };
            let copy = sapid::copy_button().theme(theme.clone()).state_ref(state);
            let widget: Box<dyn Widget> = if mode == 0 {
                Box::new(
                    sapid::code_highlight()
                        .theme(theme.clone())
                        .source(source)
                        .line_numbers(true)
                        .copy_button(copy),
                )
            } else {
                Box::new(
                    sapid::json_viewer("json")
                        .theme(theme.clone())
                        .source(source)
                        .tree_navigation(mode == 2)
                        .copy_button(copy),
                )
            };
            host.mount(widget).unwrap();
            settle(&host);
            let before = pixels(&host);
            let id = find(&host, |d| {
                d.semantics.as_ref().is_some_and(|s| s.clipboard.is_some())
            });
            activate(&host, id);
            assert_eq!(driver.written(0), source);
            assert_eq!(state.get(), sapid::CopyState::Pending);
            driver.finish(0, Ok(String::new()));
            settle(&host);
            assert_eq!(state.get(), sapid::CopyState::Written);
            assert_ne!(before, pixels(&host));
            host.app().clear().unwrap();
            assert_eq!(host.renderer().stats().nodes, 0);
        }
    }
}
fn editor(host: &WindowApp) -> NodeId {
    find(host, |d| matches!(d.props, Props::Input(_)))
}
fn select(host: &WindowApp, id: NodeId, start: usize, end: usize) {
    host.app().scope().action(Action::Focus(id)).unwrap();
    host.app().flush().unwrap();
    settle(host);
    host.app()
        .scope()
        .set_input_selection(
            id,
            TextSelection {
                start,
                end,
                direction: SelectionDirection::Backward,
            },
        )
        .unwrap();
    host.app().flush().unwrap();
    settle(host);
}
fn paste(host: &mut WindowApp) {
    use blitz_dom::Document;
    host.handle_ui_event(blitz_traits::events::UiEvent::AppleStandardKeybinding(
        "paste:".into(),
    ));
    settle(host);
}

#[test]
fn actual_tags_consumer_accepts_atomic_async_paste_and_preserves_its_budget_rejection() {
    for theme in themes() {
        let (mut host, driver) = host();
        let source = host.app().scope().cage(sapid::TagsInputState {
            values: vec!["Rust".into()],
            search: "A😀B".into(),
        });
        let reject = host.app().scope().cage(None);
        host.mount(
            sapid::tags_input("native-clipboard")
                .theme(theme.clone())
                .state(source)
                .on_change(move |p| source.revise(|s| *s = p))
                .on_reject(move |r| reject.revise(|s| *s = Some(r))),
        )
        .unwrap();
        settle(&host);
        let id = editor(&host);
        select(&host, id, 1, 3);
        paste(&mut host);
        driver.finish(0, Ok("中,第二".into()));
        settle(&host);
        assert_eq!(source.get().values, ["Rust", "A中", "第二B"]);
        assert_eq!(source.get().search, "");
        paste(&mut host);
        driver.finish(1, Err(ClipboardError::TooLong));
        settle(&host);
        assert_eq!(reject.get(), Some(sapid::TagRejection::PasteTooLong));
        paste(&mut host);
        driver.finish(2, Ok("plain 中".into()));
        settle(&host);
        assert_eq!(source.get().search, "plain 中");
        let _ = pixels(&host);
        host.app().clear().unwrap();
    }
}
