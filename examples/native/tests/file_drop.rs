#![cfg(all(
    feature = "files",
    any(target_os = "macos", target_os = "windows", target_os = "linux")
))]
use glory::{
    blitz::{Config, Raster, files::Files, shell::WindowApp},
    render::*,
};
use std::{
    cell::{Cell, RefCell},
    io,
    ops::ControlFlow,
    path::PathBuf,
    sync::Arc,
    task::{Context, Waker},
    time::{Duration, Instant},
};
use winit::{
    data_transfer::{DataTransfer, DataTransferId, TransferType, TypeHint, TypedData},
    dpi::PhysicalPosition,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, AsyncRequestSerial, DndAction},
};

// This is an application-owned native event source. It implements Winit's actual
// event-loop interface; no test-only production hook or business event bypass.
#[derive(Debug)]
struct Loop {
    serial: Cell<AsyncRequestSerial>,
    fetched: Cell<usize>,
    accepts: RefCell<Vec<bool>>,
}
impl Loop {
    fn new() -> Self {
        Self {
            serial: Cell::new(AsyncRequestSerial::get()),
            fetched: Cell::new(0),
            accepts: RefCell::new(Vec::new()),
        }
    }
}
#[derive(Debug)]
struct Metadata;
impl DataTransfer for Metadata {
    fn for_each_available_type<'a>(
        &'a self,
        func: &mut dyn FnMut(&'a dyn TransferType) -> ControlFlow<()>,
    ) {
        let _ = func(&TypeHint::UriList);
    }
}
impl winit::raw_window_handle::HasDisplayHandle for Loop {
    fn display_handle(
        &self,
    ) -> Result<
        winit::raw_window_handle::DisplayHandle<'_>,
        winit::raw_window_handle::HandleError,
    > {
        Err(winit::raw_window_handle::HandleError::Unavailable)
    }
}
impl ActiveEventLoop for Loop {
    fn data_transfer(
        &self,
        _: DataTransferId,
    ) -> Result<Box<dyn DataTransfer>, winit::error::TransferError> {
        Ok(Box::new(Metadata))
    }
    fn fetch_data_transfer(
        &self,
        _: DataTransferId,
        hint: &dyn TransferType,
    ) -> Result<AsyncRequestSerial, winit::error::TransferError> {
        assert_eq!(hint.hint(), Some(TypeHint::UriList));
        let serial = AsyncRequestSerial::get();
        self.serial.set(serial);
        self.fetched.set(self.fetched.get() + 1);
        Ok(serial)
    }
    fn set_valid_dnd_actions(
        &self,
        _: DataTransferId,
        actions: &[DndAction],
    ) -> Result<(), winit::error::TransferError> {
        assert!(actions.is_empty() || actions == [DndAction::Copy]);
        self.accepts.borrow_mut().push(!actions.is_empty());
        Ok(())
    }
    fn create_proxy(&self) -> winit::event_loop::EventLoopProxy {
        unreachable!()
    }
    fn create_window(
        &self,
        _: winit::window::WindowAttributes,
    ) -> Result<Box<dyn winit::window::Window>, winit::error::CreateWindowError> {
        unreachable!()
    }
    fn create_custom_cursor(
        &self,
        _: winit::cursor::CustomCursorSource,
    ) -> Result<winit::cursor::CustomCursor, winit::error::CustomCursorError> {
        unreachable!()
    }
    fn available_monitors(&self) -> Box<dyn Iterator<Item = winit::monitor::MonitorHandle>> {
        Box::new(std::iter::empty())
    }
    fn primary_monitor(&self) -> Option<winit::monitor::MonitorHandle> {
        None
    }
    fn listen_device_events(&self, _: winit::event_loop::DeviceEvents) {}
    fn system_theme(&self) -> Option<winit::window::Theme> {
        None
    }
    fn set_control_flow(&self, _: winit::event_loop::ControlFlow) {}
    fn control_flow(&self) -> winit::event_loop::ControlFlow {
        winit::event_loop::ControlFlow::Wait
    }
    fn exit(&self) {}
    fn exiting(&self) -> bool {
        false
    }
    fn owned_display_handle(&self) -> winit::event_loop::OwnedDisplayHandle {
        unreachable!()
    }
    fn rwh_06_handle(&self) -> &dyn winit::raw_window_handle::HasDisplayHandle {
        self
    }
}
#[derive(Debug)]
struct Data(Vec<String>);
impl TypedData for Data {
    fn type_(&self) -> &dyn TransferType {
        &TypeHint::UriList
    }
    fn try_read(&self) -> Option<Box<dyn io::BufRead>> {
        panic!("no text coercion or byte fallback")
    }
    fn try_as_uris(&self) -> io::Result<Vec<String>> {
        Ok(self.0.clone())
    }
    fn try_as_string(&self) -> io::Result<String> {
        unreachable!()
    }
}
fn settle(host: &WindowApp) {
    host.observe(&Context::from_waker(Waker::noop())).unwrap();
}
fn find(host: &WindowApp, class: &str) -> Vec<NodeId> {
    let tree = host.app().tree();
    let mut pending = tree.roots().to_vec();
    let mut found = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            found.push(id);
        }
        pending.extend(&node.children);
    }
    found
}
fn drop(host: &WindowApp, source: &Loop, n: i64, uris: Vec<String>) {
    settle(host);
    let node = find(host, "sapid-file-button")[0];
    let engine = host
        .renderer()
        .engine_id(node, glory::html::Part::Main)
        .unwrap();
    let rect = host
        .renderer()
        .inspect(|doc| doc.get_client_bounding_rect(engine).unwrap());
    assert!(
        rect.width > 0. && rect.height > 0.,
        "actual Dropzone target: {rect:?}"
    );
    let scale = host.renderer().inspect(|doc| doc.viewport().scale_f64());
    let position = PhysicalPosition::new(
        (rect.x + rect.width / 2.) * scale,
        (rect.y + rect.height / 2.) * scale,
    );
    let id = DataTransferId::from_raw(n);
    host.handle_native_file_event(
        &WindowEvent::DragEntered {
            id,
            position: Some(position),
        },
        source,
    );
    let fetched = source.fetched.get();
    assert_eq!(
        source.accepts.borrow().last(),
        Some(&true),
        "drop={n} rect={rect:?} position={position:?} stats={:?} hit={:?}",
        host.renderer().resource_stats(),
        host.renderer().inspect(|doc| doc
            .element_from_point((position.x / scale) as f32, (position.y / scale) as f32))
    );
    host.handle_native_file_event(
        &WindowEvent::DragDropped {
            id,
            proposed_action: Some(DndAction::Copy),
        },
        source,
    );
    assert_eq!(source.fetched.get(), fetched + 1);
    host.handle_native_file_event(
        &WindowEvent::DataTransferReceived {
            id,
            serial: source.serial.get(),
            value: Arc::new(Data(uris)),
        },
        source,
    );
    settle(host);
}
struct Directory(PathBuf);
impl Directory {
    fn new(n: usize) -> Self {
        let path =
            std::env::temp_dir().join(format!("sapid-native-drop-{}-{n}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn dropzone_reads_real_opened_files_previews_and_retires_attachments_across_four_themes() {
    for (n, theme) in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        let directory = Directory::new(n);
        let path = directory.0.join("投放-😀.png");
        let bytes = include_bytes!("../../web/assets/image-pattern.png");
        std::fs::write(&path, bytes).unwrap();
        let files = Files::desktop().unwrap();
        let source = Loop::new();
        let host = WindowApp::with_config(Config {
            files: Some(files.clone()),
            ..Default::default()
        });
        host.renderer().set_viewport(640, 480, 1.).unwrap();
        let scope = host.app().scope();
        let controller = scope.cage(None);
        let selected = scope.cage(Vec::<FileInfo>::new());
        let status = scope.cage(sapid::FileStatus::Idle);
        host.mount(
            sapid::dropzone()
                .theme(theme)
                .configuration(FileConfig {
                    preview: Some(Default::default()),
                    ..Default::default()
                })
                .previews(true)
                .files_ref(selected)
                .status_ref(status)
                .controller_ref(controller),
        )
        .unwrap();
        drop(
            &host,
            &source,
            1,
            vec![
                glory::blitz::assets::Url::from_file_path(&path)
                    .unwrap()
                    .to_string(),
            ],
        );
        let end = Instant::now() + Duration::from_secs(10);
        while status.get() != sapid::FileStatus::Selected {
            settle(&host);
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(5));
        }
        let info = selected.get()[0].clone();
        assert_eq!(info.name, "投放-😀.png");
        std::fs::rename(&path, directory.0.join("original.png")).unwrap();
        std::fs::write(&path, b"replaced path must not be reopened").unwrap();
        assert_eq!(find(&host, "sapid-file-attachment").len(), 1);
        assert_eq!(files.stats().references, 1);
        let result = scope.cage(None);
        let file = info.token;
        controller
            .get()
            .unwrap()
            .read(
                FileRead {
                    file,
                    offset: 0,
                    length: 8,
                },
                move |r| result.revise(|v| *v = Some(r)),
            )
            .unwrap();
        while result.get().is_none() {
            settle(&host);
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(result.get().unwrap().unwrap(), bytes[..8]);
        host.app()
            .dispatch(
                find(&host, "sapid-file-preview-request")[0],
                Event::Activate,
            )
            .unwrap();
        while host.renderer().resource_stats().cached_images != 1 {
            settle(&host);
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        assert_eq!(raster.pixels().len(), 640 * 480 * 4);
        drop(
            &host,
            &source,
            2,
            vec!["https://example.test/not-a-file".into()],
        );
        assert_eq!(
            status.get(),
            sapid::FileStatus::Rejected(FileRejection::Type)
        );
        assert_eq!(selected.get(), vec![info]);
        host.app()
            .dispatch(find(&host, "sapid-file-remove")[0], Event::Activate)
            .unwrap();
        settle(&host);
        assert!(selected.get().is_empty());
        assert_eq!(files.stats().references, 0);
        assert_eq!(files.stats().preview_bytes, 0);
        assert!(find(&host, "sapid-file-attachment").is_empty());
        host.app().clear().unwrap();
    }
}
