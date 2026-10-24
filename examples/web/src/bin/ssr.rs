fn page(template: &str, html: &str) -> String {
    template
        .replace(
            "<div id=\"app\"></div>",
            &format!("<div id=\"app\">{html}</div>"),
        )
        .replace(
            "<!--BASE_CSS-->",
            &format!(
                "<style data-gly-base=\"v1\">{}</style>",
                glory::html::BASE_CSS
            ),
        )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory =
        std::path::PathBuf::from(std::env::args().nth(1).ok_or("output directory")?);
    for (hue_name, hue) in [("blue", sapid::Hue::Blue), ("green", sapid::Hue::Green)] {
        for (scheme_name, scheme) in [
            ("light", sapid::ColorScheme::Light),
            ("dark", sapid::ColorScheme::Dark),
        ] {
            let theme = sapid::Theme::new(hue, scheme);
            let fields = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::fields_form::form(scope, theme.clone()).0
            })?;
            let choices = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::choices_form::form(scope, theme.clone()).0
            })?;
            let forms = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::forms_form::form(scope, theme.clone()).0
            })?;
            let display = glory::ssr::snapshot(sapid_web_example::display_form::display_form(
                theme.clone(),
            ))?;
            let actions = glory::ssr::snapshot(sapid_web_example::actions_form::actions_form(
                theme.clone(),
            ))?;
            let document = glory::ssr::snapshot(
                sapid_web_example::document_form::document_form(theme.clone()),
            )?;
            let toggles = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::toggles_form::form(scope, theme.clone()).0
            })?;
            let layout = glory::ssr::snapshot(sapid_web_example::layout_form::layout_form(
                theme.clone(),
            ))?;
            let typography = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::typography_form::form(scope, theme.clone()).0
            })?;
            let timeline = glory::ssr::snapshot(
                sapid_web_example::timeline_form::timeline_form(theme.clone()),
            )?;
            let spoiler = glory::ssr::snapshot(sapid_web_example::spoiler_form::spoiler_form(
                theme.clone(),
            ))?;
            let navigation = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::navigation_form::form(scope, theme.clone()).0
            })?;
            let grid =
                glory::ssr::snapshot(sapid_web_example::grid_form::grid_form(theme.clone()))?;
            let hidden = glory::ssr::snapshot(
                sapid_web_example::visually_hidden_form::visually_hidden_form(theme.clone()),
            )?;
            let avatar = glory::ssr::snapshot(sapid_web_example::avatar_form::avatar_form(
                theme.clone(),
            ))?;
            let menu = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::menu_form::form(scope, theme.clone()).0
            })?;
            let stepper = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::stepper_form::form(scope, theme.clone()).0
            })?;
            let shell = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::shell_form::form(scope, theme.clone()).0
            })?;
            let hints = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::hints_form::form(scope, theme.clone()).0
            })?;
            let image =
                glory::ssr::snapshot(sapid_web_example::image_form::image_form(theme.clone()))?;
            let buttons = glory::ssr::snapshot(sapid_web_example::button_form::button_form(
                theme.clone(),
            ))?;
            let scroll_area = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::scroll_area_form::form(scope, theme.clone()).0
            })?;
            let alerts = glory::ssr::snapshot(sapid_web_example::alerts_form::alerts_form(
                theme.clone(),
            ))?;
            let graphics = glory::ssr::snapshot(
                sapid_web_example::graphics_form::graphics_form(theme.clone()),
            )?;
            let native_select = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::native_select_form::form(scope, theme.clone()).0
            })?;
            let select = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::select_form::form(scope, theme.clone()).0
            })?;
            let combobox = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::combobox_form::form(scope, theme.clone()).0
            })?;
            let choice_extras = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::choice_extras_form::form(scope, theme.clone()).0
            })?;
            let tags = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::tags_form::form(scope, theme.clone()).0
            })?;
            let command = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::command_form::form(scope, theme.clone()).0
            })?;
            let popover_placement = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::popover_placement_form::form(scope, theme.clone()).0
            })?;
            let layer_a = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::layer_document_form::form(scope, "a", theme.clone()).0
            })?;
            let layer_b = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::layer_document_form::form(scope, "b", theme.clone()).0
            })?;
            let layer_document = include_str!("../../layer-document.html")
                .replace(
                    "<div id=\"app-a\"></div>",
                    &format!("<div id=\"app-a\">{}</div>", layer_a.html),
                )
                .replace(
                    "<div id=\"app-b\"></div>",
                    &format!("<div id=\"app-b\">{}</div>", layer_b.html),
                );
            let layer_document = page(&layer_document, "");
            std::fs::write(
                directory.join(format!(
                    "layer-document-hydrate-{hue_name}-{scheme_name}.html"
                )),
                &layer_document,
            )?;
            if hue_name == "blue" && scheme_name == "light" {
                std::fs::write(
                    directory.join("layer-document-hydrate.html"),
                    &layer_document,
                )?;
            }
            let menubar = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::menubar_form::form(scope, theme.clone()).0
            })?;
            let context_menu = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::context_menu_form::form(scope, theme.clone()).0
            })?;
            let progress = glory::ssr::snapshot(
                sapid_web_example::progress_form::progress_form(theme.clone()),
            )?;
            let spinner = glory::ssr::snapshot(sapid_web_example::spinner_form::spinner_form(
                theme.clone(),
            ))?;
            let sheet =
                glory::ssr::snapshot(sapid_web_example::sheet_form::sheet_form(theme.clone()))?;
            let loading_overlay = glory::ssr::snapshot(
                sapid_web_example::loading_overlay_form::loading_overlay_form(theme.clone()),
            )?;
            let modals = glory::ssr::snapshot(sapid_web_example::modals_form::modals_form(
                theme.clone(),
            ))?;
            let notifications = glory::ssr::snapshot(
                sapid_web_example::notifications_form::notifications_form(theme.clone()),
            )?;
            let toolbar = glory::ssr::snapshot(sapid_web_example::toolbar_form::toolbar_form(
                theme.clone(),
            ))?;
            let accordion = glory::ssr::snapshot(
                sapid_web_example::accordion_form::accordion_form(theme.clone()),
            )?;
            let tabs_dynamic = glory::ssr::snapshot(
                sapid_web_example::tabs_dynamic_form::tabs_dynamic_form(theme.clone()),
            )?;
            let data_table_server = glory::ssr::snapshot(
                sapid_web_example::data_table_server_form::data_table_server_form(
                    theme.clone(),
                ),
            )?;
            let data_table = glory::ssr::snapshot(
                sapid_web_example::data_table_form::data_table_form(theme.clone()),
            )?;
            let table =
                glory::ssr::snapshot(sapid_web_example::table_form::table_form(theme.clone()))?;
            let table_of_contents = glory::ssr::snapshot(
                sapid_web_example::table_of_contents_form::table_of_contents_form(
                    theme.clone(),
                ),
            )?;
            let text_motion = glory::ssr::snapshot(
                sapid_web_example::text_motion_form::text_motion_form(theme.clone()),
            )?;
            let code =
                glory::ssr::snapshot(sapid_web_example::code_form::code_form(theme.clone()))?;
            let translation = glory::ssr::snapshot(
                sapid_web_example::translation_form::translation_form(theme.clone()),
            )?;
            let floating_indicator = glory::ssr::snapshot(
                sapid_web_example::floating_indicator_form::floating_indicator_form(
                    theme.clone(),
                ),
            )?;
            let overflow_list = glory::ssr::snapshot(
                sapid_web_example::overflow_list_form::overflow_list_form(theme.clone()),
            )?;
            let collapse_motion = glory::ssr::snapshot(
                sapid_web_example::collapse_motion_form::collapse_motion_form(theme.clone()),
            )?;
            let color_swatch = glory::ssr::snapshot(
                sapid_web_example::color_swatch_form::color_swatch_form(theme.clone()),
            )?;
            let navigation_progress = glory::ssr::snapshot(
                sapid_web_example::navigation_progress_form::navigation_progress_form(
                    theme.clone(),
                ),
            )?;
            let splitter = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::splitter_form::form(scope, theme.clone()).0
            })?;
            let navigation_menu = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::navigation_menu_form::form(scope, theme.clone()).0
            })?;
            let range_slider = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::range_slider_form::form(scope, theme.clone()).0
            })?;
            let slider = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::slider_form::form(scope, theme.clone()).0
            })?;
            let angle_slider = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::angle_slider_form::form(scope, theme.clone()).0
            })?;
            let number_input = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::number_input_form::form(scope, theme.clone()).0
            })?;
            let pin_input = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::pin_input_form::form(scope, theme.clone()).0
            })?;
            let mask_input = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::mask_input_form::form(scope, theme.clone()).0
            })?;
            let calendar = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::calendar_form::form(scope, theme.clone()).0
            })?;
            let rating = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::rating_form::form(scope, theme.clone()).0
            })?;
            let json_input = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::json_input_form::form(scope, theme.clone()).0
            })?;
            let color_picker = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::color_picker_form::form(scope, theme.clone()).0
            })?;
            let color_sliders = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::color_sliders_form::form(scope, theme.clone()).0
            })?;
            let affix = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::affix_form::form(scope, theme.clone()).0
            })?;
            let basic = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::form::form_themed(scope, theme.clone()).0
            })?;
            let composites = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::composite_form::form_themed(scope, theme.clone()).0
            })?;
            let overlays = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::overlay_form::form_themed(scope, false, theme.clone()).0
            })?;
            let initial = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::overlay_form::form_themed(scope, true, theme.clone()).0
            })?;
            let initial_template = include_str!("../../overlays.html")
                .replace("demo.mount_overlays(", "demo.mount_overlays_initial(");
            for (stem, template, snapshot) in [
                (
                    "color-picker-hydrate",
                    include_str!("../../color-picker.html"),
                    color_picker,
                ),
                ("affix-hydrate", include_str!("../../affix.html"), affix),
                (
                    "color-sliders-hydrate",
                    include_str!("../../color-sliders.html"),
                    color_sliders,
                ),
                ("slider-hydrate", include_str!("../../slider.html"), slider),
                (
                    "angle-slider-hydrate",
                    include_str!("../../angle-slider.html"),
                    angle_slider,
                ),
                ("rating-hydrate", include_str!("../../rating.html"), rating),
                (
                    "number-input-hydrate",
                    include_str!("../../number-input.html"),
                    number_input,
                ),
                (
                    "pin-input-hydrate",
                    include_str!("../../pin-input.html"),
                    pin_input,
                ),
                (
                    "mask-input-hydrate",
                    include_str!("../../mask-input.html"),
                    mask_input,
                ),
                (
                    "calendar-hydrate",
                    include_str!("../../calendar.html"),
                    calendar,
                ),
                (
                    "json-input-hydrate",
                    include_str!("../../json-input.html"),
                    json_input,
                ),
                (
                    "range-slider-hydrate",
                    include_str!("../../range-slider.html"),
                    range_slider,
                ),
                (
                    "navigation-menu-hydrate",
                    include_str!("../../navigation-menu.html"),
                    navigation_menu,
                ),
                ("select-hydrate", include_str!("../../select.html"), select),
                ("tags-hydrate", include_str!("../../tags.html"), tags),
                (
                    "popover-placement-hydrate",
                    include_str!("../../popover-placement.html"),
                    popover_placement,
                ),
                (
                    "command-hydrate",
                    include_str!("../../command.html"),
                    command,
                ),
                (
                    "menubar-hydrate",
                    include_str!("../../menubar.html"),
                    menubar,
                ),
                (
                    "context-menu-hydrate",
                    include_str!("../../context-menu.html"),
                    context_menu,
                ),
                (
                    "choice-extras-hydrate",
                    include_str!("../../choice-extras.html"),
                    choice_extras,
                ),
                (
                    "combobox-hydrate",
                    include_str!("../../combobox.html"),
                    combobox,
                ),
                (
                    "scroll-area-hydrate",
                    include_str!("../../scroll-area.html"),
                    scroll_area,
                ),
                ("alerts-hydrate", include_str!("../../alerts.html"), alerts),
                (
                    "graphics-hydrate",
                    include_str!("../../graphics.html"),
                    graphics,
                ),
                (
                    "native-select-hydrate",
                    include_str!("../../native-select.html"),
                    native_select,
                ),
                (
                    "progress-hydrate",
                    include_str!("../../progress.html"),
                    progress,
                ),
                (
                    "navigation-progress-hydrate",
                    include_str!("../../navigation-progress.html"),
                    navigation_progress,
                ),
                (
                    "spinner-hydrate",
                    include_str!("../../spinner.html"),
                    spinner,
                ),
                ("modals-hydrate", include_str!("../../modals.html"), modals),
                ("sheet-hydrate", include_str!("../../sheet.html"), sheet),
                (
                    "notifications-hydrate",
                    include_str!("../../notifications.html"),
                    notifications,
                ),
                (
                    "toolbar-hydrate",
                    include_str!("../../toolbar.html"),
                    toolbar,
                ),
                (
                    "accordion-hydrate",
                    include_str!("../../accordion.html"),
                    accordion,
                ),
                (
                    "tabs-dynamic-hydrate",
                    include_str!("../../tabs-dynamic.html"),
                    tabs_dynamic,
                ),
                (
                    "data-table-server-hydrate",
                    include_str!("../../data-table-server.html"),
                    data_table_server,
                ),
                (
                    "data-table-hydrate",
                    include_str!("../../data-table.html"),
                    data_table,
                ),
                ("table-hydrate", include_str!("../../table.html"), table),
                (
                    "table-of-contents-hydrate",
                    include_str!("../../table-of-contents.html"),
                    table_of_contents,
                ),
                (
                    "floating-indicator-hydrate",
                    include_str!("../../floating-indicator.html"),
                    floating_indicator,
                ),
                (
                    "translation-hydrate",
                    include_str!("../../translation.html"),
                    translation,
                ),
                (
                    "text-motion-hydrate",
                    include_str!("../../text-motion.html"),
                    text_motion,
                ),
                ("code-hydrate", include_str!("../../code.html"), code),
                (
                    "overflow-list-hydrate",
                    include_str!("../../overflow-list.html"),
                    overflow_list,
                ),
                (
                    "collapse-motion-hydrate",
                    include_str!("../../collapse-motion.html"),
                    collapse_motion,
                ),
                (
                    "color-swatch-hydrate",
                    include_str!("../../color-swatch.html"),
                    color_swatch,
                ),
                (
                    "loading-overlay-hydrate",
                    include_str!("../../loading-overlay.html"),
                    loading_overlay,
                ),
                (
                    "splitter-hydrate",
                    include_str!("../../splitter.html"),
                    splitter,
                ),
                ("image-hydrate", include_str!("../../image.html"), image),
                (
                    "buttons-hydrate",
                    include_str!("../../buttons.html"),
                    buttons,
                ),
                ("hints-hydrate", include_str!("../../hints.html"), hints),
                ("shell-hydrate", include_str!("../../shell.html"), shell),
                ("avatar-hydrate", include_str!("../../avatar.html"), avatar),
                (
                    "visually-hidden-hydrate",
                    include_str!("../../visually-hidden.html"),
                    hidden,
                ),
                ("menu-hydrate", include_str!("../../menu.html"), menu),
                (
                    "stepper-hydrate",
                    include_str!("../../stepper.html"),
                    stepper,
                ),
                ("grid-hydrate", include_str!("../../grid.html"), grid),
                (
                    "navigation-hydrate",
                    include_str!("../../navigation.html"),
                    navigation,
                ),
                (
                    "spoiler-hydrate",
                    include_str!("../../spoiler.html"),
                    spoiler,
                ),
                (
                    "timeline-hydrate",
                    include_str!("../../timeline.html"),
                    timeline,
                ),
                (
                    "typography-hydrate",
                    include_str!("../../typography.html"),
                    typography,
                ),
                (
                    "toggles-hydrate",
                    include_str!("../../toggles.html"),
                    toggles,
                ),
                ("layout-hydrate", include_str!("../../layout.html"), layout),
                (
                    "document-hydrate",
                    include_str!("../../document.html"),
                    document,
                ),
                ("forms-hydrate", include_str!("../../forms.html"), forms),
                (
                    "choices-hydrate",
                    include_str!("../../choices.html"),
                    choices,
                ),
                (
                    "display-hydrate",
                    include_str!("../../display.html"),
                    display,
                ),
                (
                    "actions-hydrate",
                    include_str!("../../actions.html"),
                    actions,
                ),
                ("fields-hydrate", include_str!("../../fields.html"), fields),
                ("hydrate", include_str!("../../index.html"), basic),
                (
                    "composites-hydrate",
                    include_str!("../../composites.html"),
                    composites,
                ),
                (
                    "overlays-hydrate",
                    include_str!("../../overlays.html"),
                    overlays,
                ),
                (
                    "overlays-initial-hydrate",
                    initial_template.as_str(),
                    initial,
                ),
            ] {
                let html = page(template, &snapshot.html);
                std::fs::write(
                    directory.join(format!("{stem}-{hue_name}-{scheme_name}.html")),
                    &html,
                )?;
                if hue_name == "blue" && scheme_name == "light" {
                    std::fs::write(directory.join(format!("{stem}.html")), &html)?;
                }
            }
        }
    }
    Ok(())
}
