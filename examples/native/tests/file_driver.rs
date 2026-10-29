#![cfg(feature = "files")]
use glory::{
    apace::{
        Config, Raster,
        files::{FilePicker, FileReader, Files, Reply, SelectedFile},
        shell::WindowApp,
    },
    render::*,
};
use std::{
    sync::{Arc, Mutex},
    task::{Context, Waker},
    time::{Duration, Instant},
};
struct Reader(Arc<Vec<u8>>);
impl FileReader for Reader {
    fn read(&self, offset: u64, length: u32) -> Reply<Vec<u8>> {
        let bytes = self.0.clone();
        Box::pin(async move {
            bytes
                .get(offset as usize..offset as usize + length as usize)
                .map(<[u8]>::to_vec)
                .ok_or(FileRejection::Bytes)
        })
    }
}
#[derive(Default)]
struct Picker(Mutex<Option<Result<Option<Vec<SelectedFile>>, FileRejection>>>);
impl FilePicker for Picker {
    fn pick(&self, _: FileConfig) -> Reply<Option<Vec<SelectedFile>>> {
        Box::pin(std::future::ready(self.0.lock().unwrap().take().unwrap()))
    }
}
fn themes() -> [sapid::Theme; 4] {
    [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ]
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
fn settle(host: &WindowApp) {
    host.observe(&Context::from_waker(Waker::noop())).unwrap();
}
fn pixels(host: &WindowApp) -> Vec<u8> {
    let mut raster = Raster::new();
    raster.draw(host.renderer()).unwrap();
    assert_eq!(raster.pixels().len(), 640 * 480 * 4);
    raster.pixels().to_vec()
}
fn png() -> Vec<u8> {
    include_bytes!("../../web/assets/image-pattern.png").to_vec()
}
fn pick(host: &WindowApp) {
    let node = find(host, "sapid-file-button")[0];
    host.app().dispatch(node, Event::Activate).unwrap();
    settle(host);
}
#[test]
fn actual_file_input_reads_previews_removes_and_clears_across_four_themes() {
    for theme in themes() {
        let picker = Arc::new(Picker::default());
        let files = Files::new(picker.clone());
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
            sapid::file_input()
                .theme(theme)
                .label("Choose image")
                .configuration(FileConfig {
                    preview: Some(Default::default()),
                    ..Default::default()
                })
                .previews(true)
                .controller_ref(controller)
                .files_ref(selected)
                .status_ref(status),
        )
        .unwrap();
        let before = pixels(&host);
        let bytes = Arc::new(png());
        *picker.0.lock().unwrap() = Some(Ok(Some(vec![SelectedFile {
            name: "中文-😀.png".into(),
            media_type: "image/png".into(),
            size: bytes.len() as u64,
            reader: Arc::new(Reader(bytes.clone())),
        }])));
        let control = controller.get().unwrap();
        pick(&host);
        assert_eq!(status.get(), sapid::FileStatus::Selected);
        let file = selected.get()[0].token;
        let read = scope.cage(None);
        control
            .read(
                FileRead {
                    file,
                    offset: 0,
                    length: 8,
                },
                move |result| read.revise(|v| *v = Some(result)),
            )
            .unwrap();
        settle(&host);
        assert_eq!(read.get().unwrap().unwrap(), bytes[..8]);
        let preview = find(&host, "sapid-file-preview-request")[0];
        host.app().dispatch(preview, Event::Activate).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            settle(&host);
            if host.renderer().resource_stats().cached_images == 1 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "actual file attachment preview did not decode"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(find(&host, "sapid-file-preview").len(), 1);
        assert_ne!(before, pixels(&host));
        let remove = find(&host, "sapid-file-remove")[0];
        host.app().dispatch(remove, Event::Activate).unwrap();
        settle(&host);
        assert!(selected.get().is_empty());
        assert!(find(&host, "sapid-file-preview").is_empty());
        assert_eq!(files.stats().references, 0);
        assert_eq!(files.stats().preview_bytes, 0);
        assert!(
            control
                .read(
                    FileRead {
                        file,
                        offset: 0,
                        length: 1
                    },
                    |_| {}
                )
                .is_err()
        );
        host.app().clear().unwrap();
        assert!(host.app().tree().is_empty());
    }
}
#[test]
fn file_button_cancellation_and_refusal_preserve_the_current_controller_selection() {
    for theme in themes() {
        let picker = Arc::new(Picker::default());
        let files = Files::new(picker.clone());
        let host = WindowApp::with_config(Config {
            files: Some(files.clone()),
            ..Default::default()
        });
        let scope = host.app().scope();
        let controller = scope.cage(None);
        let selected = scope.cage(Vec::<FileInfo>::new());
        host.mount(
            sapid::file_button()
                .theme(theme)
                .controller_ref(controller)
                .files_ref(selected),
        )
        .unwrap();
        let control = controller.get().unwrap();
        *picker.0.lock().unwrap() = Some(Ok(Some(vec![SelectedFile {
            name: "note.txt".into(),
            media_type: "text/plain".into(),
            size: 1,
            reader: Arc::new(Reader(Arc::new(vec![7]))),
        }])));
        pick(&host);
        let previous = selected.get();
        assert_eq!(previous.len(), 1);
        for outcome in [
            Ok(None),
            Err(FileRejection::Busy),
            Err(FileRejection::Unavailable),
        ] {
            *picker.0.lock().unwrap() = Some(outcome);
            pick(&host);
            assert_eq!(selected.get(), previous);
            assert_eq!(files.stats().references, 1);
        }
        control.clear().unwrap();
        settle(&host);
        assert!(selected.get().is_empty());
        assert_eq!(files.stats().references, 0);
        host.app().clear().unwrap();
    }
}
