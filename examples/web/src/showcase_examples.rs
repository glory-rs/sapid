//! Widgets mounted directly by the Rust documentation site, one demo at a time.
use glory::prelude::*;
use glory::style::FlexWrap;

pub fn demo(id: &str, theme: sapid::Theme) -> Box<dyn Widget> {
    let selected: Box<dyn Widget> = match id {
        "P0-01" => Box::new(button_demo(theme)),
        "P0-02" => Box::new(text_field_demo(theme)),
        "P0-03" => Box::new(checkbox_demo(theme)),
        "P0-04" => Box::new(collapsible_demo(theme)),
        "P0-05" => Box::new(accordion_demo(theme)),
        "P0-06" => Box::new(tabs_demo(theme)),
        "P0-07" => Box::new(dialog_demo(theme)),
        "P0-08" => Box::new(drawer_demo(theme)),
        "P0-09" => Box::new(popover_demo(theme)),
        "P0-14" => Box::new(catalogue_demo(theme)),
        _ => match crate::showcase_catalog::find(id) {
            Some(entry) => family_demo(entry.example, theme),
            None => Box::new(
                sapid::text()
                    .theme(theme)
                    .content("Select a component to explore."),
            ),
        },
    };
    Box::new(view().class("site-example").fill(selected))
}

fn column() -> glory::ui::View {
    col().style(
        Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(16.0),
    )
}

// showcase-code: P0-01
fn button_demo(theme: sapid::Theme) -> impl Widget {
    component(move |scope| {
        let clicks = scope.cage(0usize);
        let variant = scope.cage(Some("filled".to_owned()));
        let size = scope.cage(Some("medium".to_owned()));
        let disabled = scope.cage(false);
        let loading = scope.cage(false);
        let style_theme = theme.clone();
        let button_style = scope.bond(move || {
            let size = match size.get().as_deref() {
                Some("small") => sapid::Size::Small,
                Some("large") => sapid::Size::Large,
                _ => sapid::Size::Medium,
            };
            let control = style_theme.tokens.control(size);
            let text = style_theme
                .tokens
                .typography(sapid::TypographyRole::Control, size);
            let (background, foreground, border) = if disabled.get() || loading.get() {
                (
                    style_theme.disabled,
                    style_theme.disabled_text,
                    style_theme.border,
                )
            } else {
                match variant.get().as_deref() {
                    Some("outline") => (
                        style_theme.input,
                        style_theme.accent_text,
                        style_theme.accent,
                    ),
                    Some("light") => (
                        style_theme.accent_soft,
                        style_theme.accent_text,
                        style_theme.accent_soft,
                    ),
                    _ => (
                        style_theme.accent,
                        style_theme.on_accent,
                        style_theme.accent,
                    ),
                }
            };
            Style {
                background: Some(background),
                color: Some(foreground),
                border_color: Some(border),
                font_size: Some(text.font_size),
                line_height: Some(text.line_height),
                font_weight: text.weight,
                ..Style::new()
                    .min_height(Length::Px(control.min_height))
                    .padding(Edges {
                        top: control.padding_y,
                        bottom: control.padding_y,
                        left: control.padding_x,
                        right: control.padding_x,
                    })
                    .radius(style_theme.radius * control.radius_scale)
            }
        });
        let label = scope.bond(move || {
            if loading.get() {
                return "Loading…".to_owned();
            }
            match variant.get().as_deref() {
                Some("outline") => "Outline button",
                Some("light") => "Light button",
                _ => "Filled button",
            }
            .to_owned()
        });
        view()
            .class("site-button-playground")
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_wrap(FlexWrap::Wrap)
                    .gap(24.0),
            )
            .fill((
                column()
                    .class("site-playground-preview")
                    .style(Style {
                        grow: Some(1.0),
                        align: Some(Align::Center),
                        justify: Some(Justify::Center),
                        ..Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .gap(16.0)
                            .min_width(Length::Px(180.0))
                    })
                    .fill((
                        sapid::button()
                            .theme(theme.clone())
                            .class("site-playground-button")
                            .label(label)
                            .style(button_style)
                            .enabled(disabled.map(|v| !v))
                            .loading(loading)
                            .on_press(move || clicks.revise(|n| *n += 1)),
                        sapid::text()
                            .theme(theme.clone())
                            .content(clicks.map(|n| format!("Clicked {n} times"))),
                    )),
                column()
                    .class("site-playground-controls")
                    .style(
                        Style::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .gap(16.0)
                            .width(Length::Px(208.0))
                            .max_width(Limit::Percent(100.0)),
                    )
                    .fill((
                        sapid::field("showcase-button-variant")
                            .theme(theme.clone())
                            .label("Variant")
                            .control({
                                let theme = theme.clone();
                                move |field| {
                                    sapid::native_select("showcase-variant")
                                        .theme(theme.clone())
                                        .field(field)
                                        .option(sapid::select_option("filled", "Filled"))
                                        .option(sapid::select_option("outline", "Outline"))
                                        .option(sapid::select_option("light", "Light"))
                                        .value(variant)
                                        .on_value_change(move |next| {
                                            if next.is_some() {
                                                variant.revise(|v| *v = next);
                                            }
                                        })
                                }
                            }),
                        sapid::field("showcase-button-size")
                            .theme(theme.clone())
                            .label("Size")
                            .control({
                                let theme = theme.clone();
                                move |field| {
                                    sapid::native_select("showcase-size")
                                        .theme(theme.clone())
                                        .field(field)
                                        .option(sapid::select_option("small", "Sm"))
                                        .option(sapid::select_option("medium", "Md"))
                                        .option(sapid::select_option("large", "Lg"))
                                        .value(size)
                                        .on_value_change(move |next| {
                                            if next.is_some() {
                                                size.revise(|v| *v = next);
                                            }
                                        })
                                }
                            }),
                        sapid::checkbox()
                            .theme(theme.clone())
                            .label("Disabled")
                            .checked(disabled)
                            .on_checked_change(move |next| disabled.revise(|v| *v = next)),
                        sapid::checkbox()
                            .theme(theme.clone())
                            .label("Loading")
                            .checked(loading)
                            .on_checked_change(move |next| loading.revise(|v| *v = next)),
                    )),
            ))
            .show_in(scope)
    })
}
// /showcase-code

// showcase-code: P0-02
fn text_field_demo(theme: sapid::Theme) -> impl Widget {
    component(move |scope| {
        let value = scope.cage(String::new());
        column()
            .fill((
                sapid::field("showcase-name")
                    .theme(theme.clone())
                    .label("Your name")
                    .description("This value is controlled by a Cage.")
                    .control({
                        let theme = theme.clone();
                        move |field| {
                            sapid::text_field()
                                .theme(theme.clone())
                                .field(field)
                                .value(value)
                                .on_value_change(move |next| value.revise(|v| *v = next))
                        }
                    }),
                sapid::text().theme(theme.clone()).content(value.map(|v| {
                    if v.is_empty() {
                        "Start typing above".to_owned()
                    } else {
                        format!("Hello, {v}!")
                    }
                })),
            ))
            .show_in(scope)
    })
}
// /showcase-code

// showcase-code: P0-03
fn checkbox_demo(theme: sapid::Theme) -> impl Widget {
    component(move |scope| {
        let checked = scope.cage(false);
        column()
            .fill((
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("I agree to the terms and conditions")
                    .checked(checked)
                    .on_checked_change(move |next| checked.revise(|v| *v = next)),
                sapid::checkbox()
                    .theme(theme.clone())
                    .label("Disabled checkbox")
                    .enabled(false),
                sapid::text().theme(theme.clone()).content(checked.map(|v| {
                    if *v {
                        "Thank you for confirming."
                    } else {
                        "Your confirmation is optional in this demo."
                    }
                    .to_owned()
                })),
            ))
            .show_in(scope)
    })
}
// /showcase-code

// showcase-code: P0-04
fn collapsible_demo(theme: sapid::Theme) -> impl Widget {
    sapid::collapsible("showcase-details")
        .theme(theme.clone())
        .label("Show more information")
        .content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("Build accessible interfaces with handwritten Rust builders.")
        })
}
// /showcase-code

// showcase-code: P0-05
fn accordion_demo(theme: sapid::Theme) -> impl Widget {
    sapid::accordion("showcase-faq")
        .theme(theme.clone())
        .default_open(["shipping"])
        .item(
            sapid::accordion_item("shipping")
                .label("How long does shipping take?")
                .content({
                    let theme = theme.clone();
                    move || {
                        sapid::text()
                            .theme(theme.clone())
                            .content("Orders usually arrive within 3–5 business days.")
                    }
                }),
        )
        .item(
            sapid::accordion_item("returns")
                .label("What is your return policy?")
                .content({
                    let theme = theme.clone();
                    move || {
                        sapid::text()
                            .theme(theme.clone())
                            .content("You can request a return within 30 days.")
                    }
                }),
        )
        .item(
            sapid::accordion_item("support")
                .label("How can I contact support?")
                .content(move || {
                    sapid::text()
                        .theme(theme.clone())
                        .content("Send a message to the support team from your account.")
                }),
        )
}
// /showcase-code

// showcase-code: P0-06
fn tabs_demo(theme: sapid::Theme) -> impl Widget {
    sapid::tabs("showcase-tabs")
        .theme(theme.clone())
        .name("Account settings")
        .default_value("gallery")
        .tab(sapid::tab("gallery").label("Gallery").content({
            let theme = theme.clone();
            move || {
                sapid::text()
                    .theme(theme.clone())
                    .content("Gallery tab content")
            }
        }))
        .tab(sapid::tab("messages").label("Messages").content({
            let theme = theme.clone();
            move || {
                sapid::text()
                    .theme(theme.clone())
                    .content("Messages tab content")
            }
        }))
        .tab(sapid::tab("settings").label("Settings").content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("Settings tab content")
        }))
}
// /showcase-code

// showcase-code: P0-07
fn dialog_demo(theme: sapid::Theme) -> impl Widget {
    sapid::dialog("showcase-dialog")
        .theme(theme.clone())
        .label("Open modal")
        .title("Edit your profile")
        .description("Update your display name and close the dialog when finished.")
        .content(move || {
            sapid::field("showcase-profile")
                .theme(theme.clone())
                .label("Display name")
                .control({
                    let theme = theme.clone();
                    move |field| {
                        sapid::text_field()
                            .theme(theme.clone())
                            .field(field)
                            .default_value("Alex Morgan")
                    }
                })
        })
}
// /showcase-code

// showcase-code: P0-08
fn drawer_demo(theme: sapid::Theme) -> impl Widget {
    sapid::drawer("showcase-drawer")
        .theme(theme.clone())
        .label("Open drawer")
        .title("Your workspace")
        .content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("Keep related information and actions within reach.")
        })
}
// /showcase-code

// showcase-code: P0-09
fn popover_demo(theme: sapid::Theme) -> impl Widget {
    sapid::popover("showcase-popover")
        .theme(theme.clone())
        .label("Toggle popover")
        .title("Popover title")
        .content(move || {
            sapid::text()
                .theme(theme.clone())
                .content("A popover keeps additional information beside its trigger.")
        })
}
// /showcase-code

// showcase-code: P0-14
fn catalogue_demo(theme: sapid::Theme) -> impl Widget {
    column().fill((
        sapid::title(3).theme(theme.clone()).content("A complete component catalogue"),
        sapid::text().theme(theme).content(format!("Explore {} documented work items from the navigation. Each preview is a real Glory Widget.", crate::showcase_catalog::ENTRIES.len())),
    ))
}
// /showcase-code

// BEGIN GENERATED FAMILIES
#[rustfmt::skip]
fn family_demo(page: &str, theme: sapid::Theme) -> Box<dyn Widget> {
    match page {
        "actions" => Box::new(crate::actions_form::actions_form(theme)),
        "affix" => Box::new(component(move |scope| crate::affix_form::form(scope, theme.clone()).0.show_in(scope))),
        "alerts" => Box::new(crate::alerts_form::alerts_form(theme)),
        "angle-slider" => Box::new(component(move |scope| crate::angle_slider_form::form(scope, theme.clone()).0.show_in(scope))),
        "avatar" => Box::new(crate::avatar_form::avatar_form(theme)),
        "calendar" => Box::new(component(move |scope| crate::calendar_form::form(scope, theme.clone()).0.show_in(scope))),
        "carousel" => Box::new(crate::carousel_form::carousel_form(theme)),
        "cartesian" => Box::new(crate::charts_form::charts_form(theme, true)),
        "cascader" => Box::new(crate::tree_choices_form::tree_choices_form(theme, true)),
        "charts" => Box::new(crate::charts_form::charts_form(theme, false)),
        "choice-extras" => Box::new(component(move |scope| crate::choice_extras_form::form(scope, theme.clone()).0.show_in(scope))),
        "choices" => Box::new(component(move |scope| crate::choices_form::form(scope, theme.clone()).0.show_in(scope))),
        "citations" => Box::new(crate::chat_evidence_form::chat_evidence_form(theme, true)),
        "code" => Box::new(crate::code_form::code_form(theme)),
        "color-picker" => Box::new(component(move |scope| crate::color_picker_form::form(scope, theme.clone()).0.show_in(scope))),
        "color-sliders" => Box::new(component(move |scope| crate::color_sliders_form::form(scope, theme.clone()).0.show_in(scope))),
        "color-swatch" => Box::new(crate::color_swatch_form::color_swatch_form(theme)),
        "combobox" => Box::new(component(move |scope| crate::combobox_form::form(scope, theme.clone()).0.show_in(scope))),
        "command" => Box::new(component(move |scope| crate::command_form::form(scope, theme.clone()).0.show_in(scope))),
        "confirmation" => Box::new(crate::confirmation_form::confirmation_form(theme)),
        "context-menu" => Box::new(component(move |scope| crate::context_menu_form::form(scope, theme.clone()).0.show_in(scope))),
        "copy-button" => Box::new(crate::copy_button_form::copy_button_form(theme)),
        "data-table" => Box::new(crate::data_table_form::data_table_form(theme)),
        "date-input" => Box::new(component(move |scope| crate::date_input_form::form(scope, theme.clone()).0.show_in(scope))),
        "date-picker" => Box::new(component(move |scope| crate::date_picker_form::form(scope, theme.clone()).0.show_in(scope))),
        "date-time-picker" => Box::new(component(move |scope| crate::date_time_picker_form::form(scope, theme.clone()).0.show_in(scope))),
        "display" => Box::new(crate::display_form::display_form(theme)),
        "document" => Box::new(crate::document_form::document_form(theme)),
        "environment" => Box::new(crate::environment_form::environment_form(theme)),
        "fields" => Box::new(component(move |scope| crate::fields_form::form(scope, theme.clone()).0.show_in(scope))),
        "files" => Box::new(crate::files_form::files_form(theme)),
        "floating-indicator" => Box::new(crate::floating_indicator_form::floating_indicator_form(theme)),
        "floating-window" => Box::new(crate::floating_window_form::floating_window_form(theme)),
        "focus-scope" => Box::new(crate::focus_scope_form::focus_scope_form(theme)),
        "forms" => Box::new(component(move |scope| crate::forms_form::form(scope, theme.clone()).0.show_in(scope))),
        "graphics" => Box::new(crate::graphics_form::graphics_form(theme)),
        "grid" => Box::new(crate::grid_form::grid_form(theme)),
        "hierarchy-charts" => Box::new(crate::plots_form::plots_form(theme, 5)),
        "hints" => Box::new(component(move |scope| crate::hints_form::form(scope, theme.clone()).0.show_in(scope))),
        "image" => Box::new(crate::image_form::image_form(theme)),
        "json-input" => Box::new(component(move |scope| crate::json_input_form::form(scope, theme.clone()).0.show_in(scope))),
        "json-viewer" => Box::new(crate::json_tree_example::json_tree_example(theme)),
        "layout" => Box::new(crate::layout_form::layout_form(theme)),
        "lightbox" => Box::new(crate::lightbox_form::lightbox_form(theme)),
        "loading-overlay" => Box::new(crate::loading_overlay_form::loading_overlay_form(theme)),
        "mask-input" => Box::new(component(move |scope| crate::mask_input_form::form(scope, theme.clone()).0.show_in(scope))),
        "matrix-charts" => Box::new(crate::plots_form::plots_form(theme, 3)),
        "menu" => Box::new(component(move |scope| crate::menu_form::form(scope, theme.clone()).0.show_in(scope))),
        "menubar" => Box::new(component(move |scope| crate::menubar_form::form(scope, theme.clone()).0.show_in(scope))),
        "message-scroller" => Box::new(crate::message_scroller_form::message_scroller_form(theme)),
        "messages" => Box::new(crate::messages_form::messages_form(theme)),
        "modals" => Box::new(crate::modals_form::modals_form(theme)),
        "model-suggestions" => Box::new(crate::model_suggestions_form::model_suggestions_form(theme)),
        "native-select" => Box::new(component(move |scope| crate::native_select_form::form(scope, theme.clone()).0.show_in(scope))),
        "navigation" => Box::new(component(move |scope| crate::navigation_form::form(scope, theme.clone()).0.show_in(scope))),
        "navigation-menu" => Box::new(component(move |scope| crate::navigation_menu_form::form(scope, theme.clone()).0.show_in(scope))),
        "navigation-progress" => Box::new(crate::navigation_progress_form::navigation_progress_form(theme)),
        "notifications" => Box::new(crate::notifications_form::notifications_form(theme)),
        "number-input" => Box::new(component(move |scope| crate::number_input_form::form(scope, theme.clone()).0.show_in(scope))),
        "output" => Box::new(crate::output_form::output_form(theme)),
        "overflow-list" => Box::new(crate::overflow_list_form::overflow_list_form(theme)),
        "period-picker" => Box::new(component(move |scope| crate::period_picker_form::form(scope, theme.clone()).0.show_in(scope))),
        "pin-input" => Box::new(component(move |scope| crate::pin_input_form::form(scope, theme.clone()).0.show_in(scope))),
        "point-charts" => Box::new(crate::plots_form::plots_form(theme, 1)),
        "polar-charts" => Box::new(crate::plots_form::plots_form(theme, 0)),
        "progress" => Box::new(crate::progress_form::progress_form(theme)),
        "prompt" => Box::new(crate::prompt_form::prompt_form(theme)),
        "public-explanations" => Box::new(crate::chat_evidence_form::chat_evidence_form(theme, false)),
        "questionnaire" => Box::new(crate::questionnaire_form::questionnaire_form(theme)),
        "range-charts" => Box::new(crate::plots_form::plots_form(theme, 4)),
        "range-slider" => Box::new(component(move |scope| crate::range_slider_form::form(scope, theme.clone()).0.show_in(scope))),
        "rating" => Box::new(component(move |scope| crate::rating_form::form(scope, theme.clone()).0.show_in(scope))),
        "resource-schedule" => Box::new(component(move |scope| crate::resource_schedule_form::form(scope, theme.clone()).0.show_in(scope))),
        "responsive" => Box::new(crate::responsive_form::responsive_form(theme)),
        "rich-editor" => Box::new(crate::rich_editor_form::rich_editor_form(theme)),
        "schedule" => Box::new(component(move |scope| crate::schedule_form::form(scope, theme.clone()).0.show_in(scope))),
        "schedule-calendar" => Box::new(component(move |scope| crate::schedule_calendar_form::form(scope, theme.clone()).0.show_in(scope))),
        "schedule-timeline" => Box::new(component(move |scope| crate::schedule_timeline_form::form(scope, theme.clone()).0.show_in(scope))),
        "scroll-area" => Box::new(component(move |scope| crate::scroll_area_form::form(scope, theme.clone()).0.show_in(scope))),
        "select" => Box::new(component(move |scope| crate::select_form::form(scope, theme.clone()).0.show_in(scope))),
        "shape-charts" => Box::new(crate::plots_form::plots_form(theme, 2)),
        "sheet" => Box::new(crate::sheet_form::sheet_form(theme)),
        "shell" => Box::new(component(move |scope| crate::shell_form::form(scope, theme.clone()).0.show_in(scope))),
        "slider" => Box::new(component(move |scope| crate::slider_form::form(scope, theme.clone()).0.show_in(scope))),
        "spinner" => Box::new(crate::spinner_form::spinner_form(theme)),
        "splitter" => Box::new(component(move |scope| crate::splitter_form::form(scope, theme.clone()).0.show_in(scope))),
        "spoiler" => Box::new(crate::spoiler_form::spoiler_form(theme)),
        "stepper" => Box::new(component(move |scope| crate::stepper_form::form(scope, theme.clone()).0.show_in(scope))),
        "table" => Box::new(crate::table_form::table_form(theme)),
        "table-of-contents" => Box::new(crate::table_of_contents_form::table_of_contents_form(theme)),
        "tags" => Box::new(component(move |scope| crate::tags_form::form(scope, theme.clone()).0.show_in(scope))),
        "tasks" => Box::new(crate::tasks_form::tasks_form(theme)),
        "text-motion" => Box::new(crate::text_motion_form::text_motion_form(theme)),
        "time-input" => Box::new(component(move |scope| crate::time_input_form::form(scope, theme.clone()).0.show_in(scope))),
        "time-picker" => Box::new(component(move |scope| crate::time_picker_form::form(scope, theme.clone()).0.show_in(scope))),
        "timeline" => Box::new(crate::timeline_form::timeline_form(theme)),
        "toggles" => Box::new(component(move |scope| crate::toggles_form::form(scope, theme.clone()).0.show_in(scope))),
        "toolbar" => Box::new(crate::toolbar_form::toolbar_form(theme)),
        "tour" => Box::new(crate::tour_form::tour_form(theme)),
        "trace" => Box::new(component(move |scope| crate::trace_form::form(scope, theme.clone(), false).0.show_in(scope))),
        "tree" => Box::new(crate::tree_form::tree_form(theme)),
        "tree-select" => Box::new(crate::tree_choices_form::tree_choices_form(theme, false)),
        "typography" => Box::new(component(move |scope| crate::typography_form::form(scope, theme.clone()).0.show_in(scope))),
        "visually-hidden" => Box::new(crate::visually_hidden_form::visually_hidden_form(theme)),
        _ => Box::new(sapid::text().theme(theme).content("This demo is not registered.")),
    }
}
// END GENERATED FAMILIES

#[cfg(test)]
mod tests {
    use super::*;
    use glory::render::{Event, Props};

    #[test]
    fn clean_examples_build_real_widgets_and_retire_their_owned_state() {
        for id in [
            "P0-01", "P0-02", "P0-03", "P0-04", "P0-05", "P0-06", "P0-07", "P0-08", "P0-09",
            "P0-14",
        ] {
            let app = App::memory();
            app.mount(demo(id, sapid::Theme::blue_light())).unwrap();
            assert!(!app.tree().is_empty(), "empty demo: {id}");
            if id == "P0-01" {
                let mut pending = app.tree().roots().to_vec();
                let mut button = None;
                while let Some(id) = pending.pop() {
                    let tree = app.tree();
                    let node = tree.node(id).unwrap();
                    if matches!(node.descriptor.view().props, Props::Button(p) if p.label == "Filled button")
                    {
                        button = Some(id);
                    }
                    pending.extend(node.children.iter().copied());
                }
                app.dispatch(button.unwrap(), Event::Activate).unwrap();
                let mut pending = app.tree().roots().to_vec();
                let mut updated = false;
                while let Some(id) = pending.pop() {
                    let tree = app.tree();
                    let node = tree.node(id).unwrap();
                    updated |= matches!(node.descriptor.view().props, Props::Text(p) if p.content == "Clicked 1 times");
                    pending.extend(node.children.iter().copied());
                }
                assert!(
                    updated,
                    "the displayed counter must respond to native activation"
                );
            }
            app.clear().unwrap();
            assert_eq!(app.stats().nodes, 0, "retired demo: {id}");
            assert_eq!(app.runtime_stats().tasks, 0, "retired tasks: {id}");
        }
    }
}
