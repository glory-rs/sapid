use glory::prelude::*;
use glory::render::{FileConfig, FileFilter, FilePreviewLimits, FileRead};
pub fn files_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let files = scope.cage(Vec::<glory::render::FileInfo>::new());
        let drop_files = scope.cage(Vec::<glory::render::FileInfo>::new());
        let status = scope.cage(sapid::FileStatus::Idle);
        let control = scope.cage(None::<sapid::FileController>);
        let drop_control = scope.cage(None::<sapid::FileController>);
        let enabled = scope.cage(true);
        let visible = scope.cage(true);
        let calls = scope.cage(0usize);
        let results = scope.cage(String::new());
        let bytes = scope.cage(String::new());
        let log = scope.cage(String::new());
        let configuration = scope.cage(FileConfig {
            multiple: true,
            max_files: 3,
            max_file_bytes: 1024 * 1024,
            max_total_bytes: 2 * 1024 * 1024,
            preview: Some(FilePreviewLimits::default()),
            filters: vec![
                FileFilter::Mime("text/plain".into()),
                FileFilter::Mime("image/png".into()),
                FileFilter::Mime("image/jpeg".into()),
            ],
            ..Default::default()
        });
        let inner = theme.clone();
        let weak = scope.downgrade();
        let weak_clear = scope.downgrade();
        let weak_drop = scope.downgrade();
        sapid::stack()
            .gap(12.)
            .fill((
                glory::ui::group().fill((
                    glory::ui::when(visible, move || {
                        sapid::file_input()
                            .theme(inner.clone())
                            .class("files-demo")
                            .label("Choose documents")
                            .configuration(configuration)
                            .enabled(enabled)
                            .files_ref(files)
                            .status_ref(status)
                            .controller_ref(control)
                            .previews(true)
                            .description(|| {
                                glory::ui::text()
                                    .class("files-description")
                                    .content("Up to 3 text, PNG or JPEG files, 1 MiB each")
                            })
                            .on_files(move |_| calls.revise(|value| *value += 1))
                            .on_result(move |result| {
                                results.revise(|value| *value = format!("{result:?}"))
                            })
                            .on(glory::render::EventKind::Files, move |_| {
                                log.revise(|value| {
                                    *value = format!(
                                        "generic:{:?}:{}",
                                        status.get(),
                                        files.get().len()
                                    )
                                })
                            })
                    }),
                    sapid::dropzone()
                        .theme(theme.clone())
                        .class("drop-demo")
                        .configuration(FileConfig {
                            multiple: true,
                            max_files: 3,
                            max_file_bytes: 1024 * 1024,
                            max_total_bytes: 2 * 1024 * 1024,
                            preview: Some(FilePreviewLimits::default()),
                            ..Default::default()
                        })
                        .files_ref(drop_files)
                        .controller_ref(drop_control)
                        .previews(true)
                        .description(|| {
                            glory::ui::text()
                                .content("Drop files here or use the keyboard to choose")
                        }),
                    sapid::file_button()
                        .unstyled()
                        .class("files-unstyled")
                        .label("Unstyled file picker"),
                    sapid::file_button()
                        .theme(theme.clone())
                        .enabled(false)
                        .label("Disabled file picker")
                        .class("files-disabled"),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Read selected bytes")
                        .on_press(move || {
                            let Some(scope) = weak.upgrade() else {
                                return;
                            };
                            let Some(handle) = control.get() else {
                                return;
                            };
                            let Some(file) = handle.files().get().first().cloned() else {
                                return;
                            };
                            let request = FileRead {
                                file: file.token,
                                offset: 0,
                                length: file.size.min(65536) as u32,
                            };
                            if let Err(error) = handle.read(request, move |result| {
                                bytes.revise(|value| {
                                    *value = match result {
                                        Ok(bytes) => format!(
                                            "Bytes:{}:{}",
                                            bytes.len(),
                                            bytes
                                                .iter()
                                                .map(|value| format!("{value:02x}"))
                                                .collect::<String>()
                                        ),
                                        Err(error) => format!("Read error:{error:?}"),
                                    }
                                })
                            }) {
                                scope.report_error(error);
                            }
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Read five concurrent chunks")
                        .on_press(move || {
                            let Some(handle) = control.get() else {
                                return;
                            };
                            let Some(file) = handle.files().get().first().cloned() else {
                                return;
                            };
                            bytes.revise(String::clear);
                            for index in 0..5 {
                                let request = FileRead {
                                    file: file.token,
                                    offset: 0,
                                    length: file.size.min(65536) as u32,
                                };
                                let _ = handle.read(request, move |result| {
                                    bytes.revise(|value| {
                                        value.push_str(&format!(
                                            "{index}:{};",
                                            if result.is_ok() { "ok" } else { "error" }
                                        ))
                                    })
                                });
                            }
                        }),
                )),
                glory::ui::group().fill((
                    sapid::button()
                        .theme(theme.clone())
                        .label("Clear selected files")
                        .on_press(move || {
                            if let Some(handle) = control.get()
                                && let Err(error) = handle.clear()
                                && let Some(scope) = weak_clear.upgrade()
                            {
                                scope.report_error(error);
                            }
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Clear dropped files")
                        .on_press(move || {
                            if let Some(handle) = drop_control.get()
                                && let Err(error) = handle.clear()
                                && let Some(scope) = weak_drop.upgrade()
                            {
                                scope.report_error(error);
                            }
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle files enabled")
                        .on_press(move || enabled.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle files owner")
                        .on_press(move || visible.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Restrict file size")
                        .on_press(move || {
                            configuration.revise(|value| value.max_file_bytes = 3)
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Reset file configuration")
                        .on_press(move || {
                            configuration.revise(|value| value.max_file_bytes = 1024 * 1024)
                        }),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Overwrite file output only")
                        .on_press(move || files.revise(Vec::clear)),
                )),
                glory::ui::group().fill((
                    glory::ui::text()
                        .class("files-current")
                        .style(
                            glory::style::Style::new()
                                .display(glory::style::Display::Block)
                                .max_width(glory::style::Limit::Percent(100.))
                                .overflow_x(glory::style::Overflow::Auto),
                        )
                        .content(scope.bond(move || {
                            files
                                .get()
                                .iter()
                                .map(|value| value.name.clone())
                                .collect::<Vec<_>>()
                                .join(" | ")
                        })),
                    glory::ui::text()
                        .class("files-drop-current")
                        .style(
                            glory::style::Style::new()
                                .display(glory::style::Display::Block)
                                .max_width(glory::style::Limit::Percent(100.))
                                .overflow_x(glory::style::Overflow::Auto),
                        )
                        .content(scope.bond(move || {
                            drop_files
                                .get()
                                .iter()
                                .map(|value| value.name.clone())
                                .collect::<Vec<_>>()
                                .join(" | ")
                        })),
                    glory::ui::text()
                        .class("files-state")
                        .content(scope.bond(move || format!("{:?}", status.get()))),
                    glory::ui::text()
                        .class("files-calls")
                        .content(scope.bond(move || format!("Calls:{}", calls.get()))),
                    glory::ui::text()
                        .class("files-result")
                        .style(
                            glory::style::Style::new()
                                .display(glory::style::Display::Block)
                                .max_width(glory::style::Limit::Percent(100.))
                                .overflow_x(glory::style::Overflow::Auto),
                        )
                        .content(results),
                    glory::ui::text()
                        .class("files-bytes")
                        .style(
                            glory::style::Style::new()
                                .display(glory::style::Display::Block)
                                .max_width(glory::style::Limit::Percent(100.))
                                .overflow_x(glory::style::Overflow::Auto),
                        )
                        .content(bytes),
                    glory::ui::text()
                        .class("files-log")
                        .style(
                            glory::style::Style::new()
                                .display(glory::style::Display::Block)
                                .max_width(glory::style::Limit::Percent(100.))
                                .overflow_x(glory::style::Overflow::Auto),
                        )
                        .content(log),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("File neighbor draft")
                        .default_value("Retained file draft"),
                )),
            ))
            .show_in(scope)
    })
}
/// Matches the final classic stream shape; fresh Node/revision lifetime on adoption.
pub fn files_stream_probe() -> impl Widget {
    widget(move |scope| {
        let node = scope.cage(None);
        let weak = scope.downgrade();
        let observed = scope.downgrade();
        glory::ui::file()
            .label("Stream file picker")
            .class("stream-file")
            .configuration(FileConfig {
                multiple: true,
                drop: true,
                max_files: 2,
                filters: vec![FileFilter::Extension(".txt".into())],
                ..Default::default()
            })
            .node_ref(node)
            .on(glory::render::EventKind::Activate, move |_| {
                if let Some(scope) = weak.upgrade()
                    && let Some(node) = node.get()
                    && let Err(error) = scope.pick_files(node)
                {
                    scope.report_error(error);
                }
            })
            .on(glory::render::EventKind::Files, move |event| {
                if let glory::render::Event::Files(event) = event
                    && let glory::render::FileOutcome::Selected(files) = event.outcome
                    && let Some(scope) = observed.upgrade()
                    && let Some(node) = node.get()
                    && let Err(error) = scope.patch(
                        node,
                        glory::render::Patch::Label(glory::render::Change::Set(format!(
                            "Selected {} files",
                            files.len()
                        ))),
                    )
                {
                    scope.report_error(error);
                }
            })
            .show_in(scope)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn files_fixture_keeps_all_three_groups_and_real_picker_siblings() {
        let html = glory::ssr::snapshot(super::files_form(sapid::Theme::blue_light()))
            .unwrap()
            .html;
        for class in [
            "files-demo",
            "drop-demo",
            "files-unstyled",
            "files-disabled",
            "files-current",
            "files-bytes",
        ] {
            assert!(html.contains(class), "{class}");
        }
        for label in [
            "Read selected bytes",
            "Read five concurrent chunks",
            "Clear selected files",
            "Toggle files owner",
        ] {
            assert!(html.contains(label), "{label}");
        }
        assert_eq!(html.matches("type=\"file\"").count(), 4);
        assert!(!html.contains("<button><input"));
    }
}
