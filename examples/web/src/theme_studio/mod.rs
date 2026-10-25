//! Rust palette workbench: exact colors, independent overrides, live widgets and export.
pub mod color;
pub mod host;
pub mod model;
use glory::prelude::*;
use glory::render::{Role, TextKind};
use model::{Draft, Layer, Target, ThemeRecord};
use sapid::{ColorScheme, Hsva, Theme};

fn copy(class: &str, value: impl Into<Lotus<String>>) -> impl Widget {
    text().class(class.to_owned()).content(value)
}
fn section(title: &str) -> impl Widget {
    text()
        .kind(TextKind::Heading(2))
        .class("studio-heading")
        .content(title.to_owned())
}
fn update_picker(
    draft: Cage<Draft>,
    layer: Cage<Layer>,
    target: Cage<Target>,
    picker: Cage<Hsva>,
) {
    picker.revise(|v| *v = Hsva::from_rgba(draft.with(|d| d.color(layer.get(), target.get()))));
}
mod fields;
mod preview;
mod settings;

/// One bounded workspace, owned by the application rather than a route view.
#[derive(Clone, Copy)]
pub struct EditorState {
    pub draft: Cage<Draft>,
    pub layer: Cage<Layer>,
    pub mode: Cage<ColorScheme>,
    pub target: Cage<Target>,
    pub picker: Cage<Hsva>,
    pub status: Cage<String>,
    pub initialized: Cage<bool>,
    pub edit_key: Cage<String>,
    pub clicks: Cage<usize>,
    pub sample_name: Cage<String>,
    pub checked: Cage<bool>,
}
impl EditorState {
    pub fn forget_saved(self, id: &str) {
        if self.draft.with(|d| d.record.id == id) {
            self.draft.revise(|d| d.record.id.clear());
            self.edit_key.revise(String::clear);
            self.status.revise(|v| {
                *v = "Saved preview removed. These colors remain as an unsaved draft.".into()
            });
        }
    }
    pub fn new(scope: &Scope, initial: ThemeRecord, scheme: ColorScheme) -> Self {
        let accent = initial.base.accent;
        Self {
            draft: scope.cage(Draft::new(initial)),
            layer: scope.cage(Layer::Base),
            mode: scope.cage(scheme),
            target: scope.cage(Target::Role(5)),
            picker: scope.cage(Hsva::from_rgba(accent)),
            status: scope.cage(String::new()),
            initialized: scope.cage(false),
            edit_key: scope.cage(String::new()),
            clicks: scope.cage(0),
            sample_name: scope.cage("Ada".into()),
            checked: scope.cage(true),
        }
    }
    pub fn prepare(self, initial: ThemeRecord, scheme: ColorScheme, explicit: Option<&str>) {
        if !self.initialized.get()
            || explicit.is_some_and(|key| self.edit_key.with(|old| old != key))
        {
            let mode = if initial.supports(scheme) {
                scheme
            } else {
                initial.default_scheme
            };
            let key = explicit.unwrap_or("").to_owned();
            let accent = initial.base.accent;
            self.draft.revise(|d| *d = Draft::new(initial));
            self.layer.revise(|v| *v = Layer::Base);
            self.mode.revise(|v| *v = mode);
            self.target.revise(|v| *v = Target::Role(5));
            self.picker.revise(|v| *v = Hsva::from_rgba(accent));
            self.status.revise(String::clear);
            self.edit_key.revise(|v| *v = key);
            self.initialized.revise(|v| *v = true);
        }
    }
}
pub fn editor(theme: Theme, state: EditorState, return_to: String) -> impl Widget {
    widget(move |scope| {
        let EditorState {
            draft,
            layer,
            mode,
            target,
            picker,
            status,
            ..
        } = state;
        let valid = scope.bond(move || draft.with(Draft::valid));
        let source = scope.bond(move || draft.with(|d| d.record.code()));
        scope.effect(move || {
            let mut resolved = draft
                .with(|d| d.record.theme(mode.get()))
                .expect("supported preview");
            resolved.name = "studio-preview".into();
            if let Err(error) = host::preview(&resolved) {
                status.revise(|s| *s = error);
            }
        });
        let title = scope.bond(move || draft.with(|d| d.record.title.clone()));
        let help = scope.bond(move || {
            if valid.get() {
                "Draft is valid. Save to add it to the top-right picker.".to_owned()
            } else {
                "Fix the theme name or incomplete color values before saving or copying."
                    .to_owned()
            }
        });
        let save_return = return_to.clone();
        let toolbar = view().class("studio-toolbar").fill((
            sapid::text_field()
                .theme(theme.clone())
                .name("Theme name")
                .value(title)
                .on_value_change(move |v| {
                    draft.revise(|d| d.record.title = v.chars().take(40).collect())
                }),
            sapid::copy_button()
                .theme(theme.clone())
                .label("Copy Rust definition")
                .source(source)
                .enabled(valid),
            sapid::button()
                .theme(theme.clone())
                .label("Save & preview")
                .enabled(valid)
                .on_press(move || {
                    let record = draft.with(|d| d.record.clone());
                    match host::save(record) {
                        Ok(id) => {
                            draft.revise(|d| d.record.id = id.clone());
                            state.edit_key.revise(|key| *key = id.clone());
                            let url = format!(
                                "showcase.html?widget={save_return}&theme={id}&scheme={}",
                                mode.get().name()
                            );
                            if let Err(error) = host::navigate(&url) {
                                status.revise(|s| *s = error);
                            }
                        }
                        Err(error) => status.revise(|s| *s = error),
                    }
                }),
        ));
        let saved = scope.bond(move || draft.with(|d| !d.record.id.is_empty()));
        let remove = when(saved, {
            let theme = theme.clone();
            move || {
                sapid::button()
                    .theme(theme.clone())
                    .label("Remove saved preview")
                    .on_press(move || {
                        let id = draft.with(|d| d.record.id.clone());
                        let result = host::remove(&id).and_then(|()| {
                            state.forget_saved(&id);
                            host::navigate("showcase.html?widget=theme-studio")
                        });
                        if let Err(error) = result {
                            status.revise(|s| *s = error);
                        }
                    })
            }
        });
        let inspector = view().class("studio-card").fill((
            section("Color inspector"),
            copy(
                "studio-inspecting",
                scope.bond(move || format!("{} · {}", layer.get().name(), target.get().name())),
            ),
            sapid::color_picker("studio-inspector")
                .theme(theme.clone())
                .name("Inspect selected color")
                .value(picker)
                .enabled(scope.bond(move || {
                    draft.with(|d| {
                        layer.get().scheme().is_none_or(|s| d.record.supports(s))
                            && d.overridden(layer.get(), target.get())
                    })
                }))
                .on_value_change(move |value| {
                    picker.revise(|v| *v = value);
                    if let Ok(color) = value.to_rgba() {
                        draft.revise(|d| d.set_color(layer.get(), target.get(), color, true));
                    }
                }),
        ));
        let layout = view().class("studio-layout").fill((
            view().class("studio-settings").fill((
                settings::controls(draft, layer, mode, target, picker, status, theme.clone()),
                view().class("studio-card").fill((
                    section("Rust definition"),
                    copy("studio-help", "A complete ThemeDefinition: every base value and only your explicit overrides."),
                    sapid::code_highlight().theme(theme.clone()).language(sapid::CodeLanguage::Rust).source(source),
                )),
            )),
            view().class("studio-live").fill((inspector, preview::preview_panel(state, theme.clone()))),
        ));
        let heading = view().class("studio-page-heading").fill((
            copy("site-breadcrumb", "THEME STUDIO"),
            text().kind(TextKind::Heading(1)).class("studio-title").content("A palette, down to every detail."),
            copy("studio-lead", "Start with a color. Shape the base, then override only what each mode needs. Save a local preview and try it across the entire widget library."),
        ));
        view()
            .role(Role::Main)
            .class("studio-main")
            .fill((
                heading,
                toolbar,
                remove,
                copy("studio-validation", help),
                copy("studio-status", status),
                layout,
            ))
            .show_in(scope)
    })
}
