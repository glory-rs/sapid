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
            let slider = glory::ssr::snapshot_with(|scope| {
                sapid_web_example::slider_form::form(scope, theme.clone()).0
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
                ("affix-hydrate", include_str!("../../affix.html"), affix),
                ("slider-hydrate", include_str!("../../slider.html"), slider),
                (
                    "navigation-menu-hydrate",
                    include_str!("../../navigation-menu.html"),
                    navigation_menu,
                ),
                ("select-hydrate", include_str!("../../select.html"), select),
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
