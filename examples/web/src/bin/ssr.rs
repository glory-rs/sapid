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
    for hue_name in ["blue", "green"] {
        for (scheme_name, scheme) in [
            ("light", sapid::ColorScheme::Light),
            ("dark", sapid::ColorScheme::Dark),
        ] {
            let theme = sapid_web_example::themes::select(hue_name, scheme);
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
            let floating_window = glory::ssr::snapshot(
                sapid_web_example::floating_window_form::floating_window_form(theme.clone()),
            )?;
            let tour =
                glory::ssr::snapshot(sapid_web_example::tour_form::tour_form(theme.clone()))?;
            let focus_scope = glory::ssr::snapshot(
                sapid_web_example::focus_scope_form::focus_scope_form(theme.clone()),
            )?;
            let environment = glory::ssr::snapshot(
                sapid_web_example::environment_form::environment_form(theme.clone()),
            )?;
            let messages = glory::ssr::snapshot(
                sapid_web_example::messages_form::messages_form(theme.clone()),
            )?;
            let message_scroller = glory::ssr::snapshot(
                sapid_web_example::message_scroller_form::message_scroller_form(theme.clone()),
            )?;
            let prompt = glory::ssr::snapshot(sapid_web_example::prompt_form::prompt_form(
                theme.clone(),
            ))?;
            let questionnaire = glory::ssr::snapshot(
                sapid_web_example::questionnaire_form::questionnaire_form(theme.clone()),
            )?;
            let model_suggestions = glory::ssr::snapshot(
                sapid_web_example::model_suggestions_form::model_suggestions_form(
                    theme.clone(),
                ),
            )?;
            let responsive = glory::ssr::snapshot(
                sapid_web_example::responsive_form::responsive_form(theme.clone()),
            )?;
            let portal = glory::ssr::snapshot(sapid_web_example::portal_form::portal_form(
                theme.clone(),
            ))?;
            let portal_missing = glory::ssr::snapshot(
                sapid_web_example::portal_form::portal_missing_form(theme.clone()),
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
            let tree =
                glory::ssr::snapshot(sapid_web_example::tree_form::tree_form(theme.clone()))?;
            let charts = glory::ssr::snapshot(sapid_web_example::charts_form::charts_form(
                theme.clone(),
                false,
            ))?;
            let cartesian = glory::ssr::snapshot(sapid_web_example::charts_form::charts_form(
                theme.clone(),
                true,
            ))?;
            let plot_0 = glory::ssr::snapshot(sapid_web_example::plots_form::plots_form(
                theme.clone(),
                0,
            ))?;
            let plot_1 = glory::ssr::snapshot(sapid_web_example::plots_form::plots_form(
                theme.clone(),
                1,
            ))?;
            let plot_2 = glory::ssr::snapshot(sapid_web_example::plots_form::plots_form(
                theme.clone(),
                2,
            ))?;
            let plot_3 = glory::ssr::snapshot(sapid_web_example::plots_form::plots_form(
                theme.clone(),
                3,
            ))?;
            let plot_4 = glory::ssr::snapshot(sapid_web_example::plots_form::plots_form(
                theme.clone(),
                4,
            ))?;
            let plot_5 = glory::ssr::snapshot(sapid_web_example::plots_form::plots_form(
                theme.clone(),
                5,
            ))?;
            let lightbox = glory::ssr::snapshot(
                sapid_web_example::lightbox_form::lightbox_form(theme.clone()),
            )?;
            let public_explanations = glory::ssr::snapshot(
                sapid_web_example::chat_evidence_form::chat_evidence_form(theme.clone(), false),
            )?;
            let tasks =
                glory::ssr::snapshot(sapid_web_example::tasks_form::tasks_form(theme.clone()))?;
            let confirmation = glory::ssr::snapshot(
                sapid_web_example::confirmation_form::confirmation_form(theme.clone()),
            )?;
            let output = glory::ssr::snapshot(sapid_web_example::output_form::output_form(
                theme.clone(),
            ))?;
            let citations = glory::ssr::snapshot(
                sapid_web_example::chat_evidence_form::chat_evidence_form(theme.clone(), true),
            )?;
            let tree_select = glory::ssr::snapshot(
                sapid_web_example::tree_choices_form::tree_choices_form(theme.clone(), false),
            )?;
            let cascader = glory::ssr::snapshot(
                sapid_web_example::tree_choices_form::tree_choices_form(theme.clone(), true),
            )?;
            let rich_editor = glory::ssr::snapshot(
                sapid_web_example::rich_editor_form::rich_editor_form(theme.clone()),
            )?;
            let carousel = glory::ssr::snapshot(
                sapid_web_example::carousel_form::carousel_form(theme.clone()),
            )?;
            let files =
                glory::ssr::snapshot(sapid_web_example::files_form::files_form(theme.clone()))?;
            let copy_button = glory::ssr::snapshot(
                sapid_web_example::copy_button_form::copy_button_form(theme.clone()),
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
            let time_picker = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::time_picker_form::form(scope, theme.clone()).0
            })?;
            let date_time_picker = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::date_time_picker_form::form(scope, theme.clone()).0
            })?;
            let schedule = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::schedule_form::form(scope, theme.clone()).0
            })?;
            let schedule_timeline = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::schedule_timeline_form::form(scope, theme.clone()).0
            })?;
            let schedule_calendar = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::schedule_calendar_form::form(scope, theme.clone()).0
            })?;
            let resource_schedule = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::resource_schedule_form::form(scope, theme.clone()).0
            })?;
            let trace = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::trace_form::form(scope, theme.clone(), false).0
            })?;
            let stack_trace = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::trace_form::form(scope, theme.clone(), true).0
            })?;
            let time_input = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::time_input_form::form(scope, theme.clone()).0
            })?;
            let period_picker = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::period_picker_form::form(scope, theme.clone()).0
            })?;
            let date_input = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::date_input_form::form(scope, theme.clone()).0
            })?;
            let date_picker = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::date_picker_form::form(scope, theme.clone()).0
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
                    "time-picker-hydrate",
                    include_str!("../../time-picker.html"),
                    time_picker,
                ),
                (
                    "date-time-picker-hydrate",
                    include_str!("../../date-time-picker.html"),
                    date_time_picker,
                ),
                (
                    "schedule-hydrate",
                    include_str!("../../schedule.html"),
                    schedule,
                ),
                (
                    "schedule-timeline-hydrate",
                    include_str!("../../schedule-timeline.html"),
                    schedule_timeline,
                ),
                (
                    "schedule-calendar-hydrate",
                    include_str!("../../schedule-calendar.html"),
                    schedule_calendar,
                ),
                (
                    "resource-schedule-hydrate",
                    include_str!("../../resource-schedule.html"),
                    resource_schedule,
                ),
                ("trace-hydrate", include_str!("../../trace.html"), trace),
                (
                    "stack-trace-hydrate",
                    include_str!("../../stack-trace.html"),
                    stack_trace,
                ),
                (
                    "time-input-hydrate",
                    include_str!("../../time-input.html"),
                    time_input,
                ),
                (
                    "period-picker-hydrate",
                    include_str!("../../period-picker.html"),
                    period_picker,
                ),
                (
                    "date-input-hydrate",
                    include_str!("../../date-input.html"),
                    date_input,
                ),
                (
                    "date-picker-hydrate",
                    include_str!("../../date-picker.html"),
                    date_picker,
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
                    "floating-window-hydrate",
                    include_str!("../../floating-window.html"),
                    floating_window,
                ),
                ("tour-hydrate", include_str!("../../tour.html"), tour),
                (
                    "questionnaire-hydrate",
                    include_str!("../../questionnaire.html"),
                    questionnaire,
                ),
                (
                    "model-suggestions-hydrate",
                    include_str!("../../model-suggestions.html"),
                    model_suggestions,
                ),
                ("prompt-hydrate", include_str!("../../prompt.html"), prompt),
                (
                    "message-scroller-hydrate",
                    include_str!("../../message-scroller.html"),
                    message_scroller,
                ),
                (
                    "messages-hydrate",
                    include_str!("../../messages.html"),
                    messages,
                ),
                (
                    "environment-hydrate",
                    include_str!("../../environment.html"),
                    environment,
                ),
                (
                    "responsive-hydrate",
                    include_str!("../../responsive.html"),
                    responsive,
                ),
                (
                    "focus-scope-hydrate",
                    include_str!("../../focus-scope.html"),
                    focus_scope,
                ),
                ("portal-hydrate", include_str!("../../portal.html"), portal),
                (
                    "portal-missing-hydrate",
                    include_str!("../../portal-missing.html"),
                    portal_missing,
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
                ("tree-hydrate", include_str!("../../tree.html"), tree),
                ("charts-hydrate", include_str!("../../charts.html"), charts),
                (
                    "public-explanations-hydrate",
                    include_str!("../../public-explanations.html"),
                    public_explanations,
                ),
                (
                    "citations-hydrate",
                    include_str!("../../citations.html"),
                    citations,
                ),
                ("output-hydrate", include_str!("../../output.html"), output),
                ("tasks-hydrate", include_str!("../../tasks.html"), tasks),
                (
                    "confirmation-hydrate",
                    include_str!("../../confirmation.html"),
                    confirmation,
                ),
                (
                    "rich-editor-hydrate",
                    include_str!("../../rich-editor.html"),
                    rich_editor,
                ),
                (
                    "lightbox-hydrate",
                    include_str!("../../lightbox.html"),
                    lightbox,
                ),
                (
                    "polar-charts-hydrate",
                    include_str!("../../polar-charts.html"),
                    plot_0,
                ),
                (
                    "point-charts-hydrate",
                    include_str!("../../point-charts.html"),
                    plot_1,
                ),
                (
                    "shape-charts-hydrate",
                    include_str!("../../shape-charts.html"),
                    plot_2,
                ),
                (
                    "matrix-charts-hydrate",
                    include_str!("../../matrix-charts.html"),
                    plot_3,
                ),
                (
                    "range-charts-hydrate",
                    include_str!("../../range-charts.html"),
                    plot_4,
                ),
                (
                    "hierarchy-charts-hydrate",
                    include_str!("../../hierarchy-charts.html"),
                    plot_5,
                ),
                (
                    "cartesian-hydrate",
                    include_str!("../../cartesian.html"),
                    cartesian,
                ),
                (
                    "tree-select-hydrate",
                    include_str!("../../tree-select.html"),
                    tree_select,
                ),
                (
                    "cascader-hydrate",
                    include_str!("../../cascader.html"),
                    cascader,
                ),
                ("files-hydrate", include_str!("../../files.html"), files),
                (
                    "carousel-hydrate",
                    include_str!("../../carousel.html"),
                    carousel,
                ),
                (
                    "copy-button-hydrate",
                    include_str!("../../copy-button.html"),
                    copy_button,
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
