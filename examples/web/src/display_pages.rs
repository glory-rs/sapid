#[cfg(target_arch = "wasm32")]
mod browser {
    use glory::web::WebApp;
    use glory::{Scope, Widget};
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    thread_local! { static APP:RefCell<Option<WebApp>>=const {RefCell::new(None)}; }
    fn js(error: impl std::fmt::Display) -> JsValue {
        error.to_string().into()
    }
    fn mount(hydrate: bool, family: u8, hue: String, scheme: String) -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| js("missing root"))?;
        let theme = crate::themes::select(
            if hue == "green" { "green" } else { "blue" },
            if scheme == "dark" {
                sapid::ColorScheme::Dark
            } else {
                sapid::ColorScheme::Light
            },
        );
        let build = move |_: &Scope| -> Box<dyn Widget> {
            if family == 98 {
                Box::new(crate::confirmation_form::confirmation_form(theme.clone()))
            } else if family == 99 {
                Box::new(crate::tasks_form::tasks_form(theme.clone()))
            } else if family == 82 {
                Box::new(crate::rich_editor_form::rich_editor_form(theme.clone()))
            } else if family == 101 {
                Box::new(crate::output_form::output_form(theme.clone()))
            } else if family == 96 || family == 97 {
                Box::new(crate::chat_evidence_form::chat_evidence_form(
                    theme.clone(),
                    family == 97,
                ))
            } else if family == 94 {
                Box::new(crate::model_suggestions_form::model_suggestions_form(
                    theme.clone(),
                ))
            } else if family == 93 {
                Box::new(crate::questionnaire_form::questionnaire_form(theme.clone()))
            } else if family == 92 {
                Box::new(crate::prompt_form::prompt_form(theme.clone()))
            } else if family == 91 {
                Box::new(crate::message_scroller_form::message_scroller_form(
                    theme.clone(),
                ))
            } else if family == 90 {
                Box::new(crate::messages_form::messages_form(theme.clone()))
            } else if family == 81 {
                Box::new(crate::lightbox_form::lightbox_form(theme.clone()))
            } else if family == 80 {
                Box::new(crate::carousel_form::carousel_form(theme.clone()))
            } else if family == 74 {
                Box::new(crate::environment_form::environment_form(theme.clone()))
            } else if family == 75 {
                Box::new(crate::responsive_form::responsive_form(theme.clone()))
            } else if (68..=73).contains(&family) {
                Box::new(crate::plots_form::plots_form(
                    theme.clone(),
                    usize::from(family - 68),
                ))
            } else if family == 66 || family == 67 {
                Box::new(crate::charts_form::charts_form(theme.clone(), family == 67))
            } else if family == 51 {
                Box::new(crate::files_form::files_form(theme.clone()))
            } else if family == 1 {
                Box::new(crate::actions_form::actions_form(theme.clone()))
            } else if family == 2 {
                Box::new(crate::document_form::document_form(theme.clone()))
            } else if family == 3 {
                Box::new(crate::layout_form::layout_form(theme.clone()))
            } else if family == 4 {
                Box::new(crate::timeline_form::timeline_form(theme.clone()))
            } else if family == 5 {
                Box::new(crate::spoiler_form::spoiler_form(theme.clone()))
            } else if family == 6 {
                Box::new(crate::grid_form::grid_form(theme.clone()))
            } else if family == 7 {
                Box::new(crate::visually_hidden_form::visually_hidden_form(
                    theme.clone(),
                ))
            } else if family == 8 {
                Box::new(crate::avatar_form::avatar_form(theme.clone()))
            } else if family == 9 {
                Box::new(crate::image_form::image_form(theme.clone()))
            } else if family == 10 {
                Box::new(crate::button_form::button_form(theme.clone()))
            } else if family == 11 {
                Box::new(crate::alerts_form::alerts_form(theme.clone()))
            } else if family == 12 {
                Box::new(crate::graphics_form::graphics_form(theme.clone()))
            } else if family == 13 {
                Box::new(crate::progress_form::progress_form(theme.clone()))
            } else if family == 14 {
                Box::new(crate::navigation_progress_form::navigation_progress_form(
                    theme.clone(),
                ))
            } else if family == 15 {
                Box::new(crate::spinner_form::spinner_form(theme.clone()))
            } else if family == 16 {
                Box::new(crate::modals_form::modals_form(theme.clone()))
            } else if family == 17 {
                Box::new(crate::loading_overlay_form::loading_overlay_form(
                    theme.clone(),
                ))
            } else if family == 19 {
                Box::new(crate::sheet_form::sheet_form(theme.clone()))
            } else if family == 23 {
                Box::new(crate::accordion_form::accordion_form(theme.clone()))
            } else if family == 24 {
                Box::new(crate::tabs_dynamic_form::tabs_dynamic_form(theme.clone()))
            } else if family == 25 {
                Box::new(crate::collapse_motion_form::collapse_motion_form(
                    theme.clone(),
                ))
            } else if family == 22 {
                Box::new(crate::color_swatch_form::color_swatch_form(theme.clone()))
            } else if family == 21 {
                Box::new(crate::toolbar_form::toolbar_form(theme.clone()))
            } else if family == 57 {
                Box::new(crate::portal_form::portal_missing_form(theme.clone()))
            } else if family == 55 {
                Box::new(crate::focus_scope_form::focus_scope_form(theme.clone()))
            } else if family == 54 {
                Box::new(crate::portal_form::portal_form(theme.clone()))
            } else if family == 39 {
                Box::new(crate::tour_form::tour_form(theme.clone()))
            } else if family == 38 {
                Box::new(crate::floating_window_form::floating_window_form(
                    theme.clone(),
                ))
            } else if family == 37 {
                Box::new(crate::data_table_server_form::data_table_server_form(
                    theme.clone(),
                ))
            } else if family == 35 {
                Box::new(crate::data_table_form::data_table_form(theme.clone()))
            } else if family == 31 {
                Box::new(crate::table_form::table_form(theme.clone()))
            } else if family == 30 {
                Box::new(crate::table_of_contents_form::table_of_contents_form(
                    theme.clone(),
                ))
            } else if family == 27 {
                Box::new(crate::floating_indicator_form::floating_indicator_form(
                    theme.clone(),
                ))
            } else if family == 26 {
                Box::new(crate::overflow_list_form::overflow_list_form(theme.clone()))
            } else if family == 32 {
                Box::new(crate::translation_form::translation_form(theme.clone()))
            } else if family == 33 {
                Box::new(crate::text_motion_form::text_motion_form(theme.clone()))
            } else if family == 45 {
                Box::new(crate::tree_form::tree_form(theme.clone()))
            } else if family == 46 || family == 47 {
                Box::new(crate::tree_choices_form::tree_choices_form(
                    theme.clone(),
                    family == 47,
                ))
            } else if family == 44 {
                Box::new(crate::copy_button_form::copy_button_form(theme.clone()))
            } else if family == 34 {
                Box::new(crate::code_form::code_form(theme.clone()))
            } else if family == 18 {
                Box::new(crate::notifications_form::notifications_form(theme.clone()))
            } else {
                Box::new(crate::display_form::display_form(theme.clone()))
            }
        };
        let app = if hydrate {
            glory::web::hydrate_with(root, build).map_err(js)?
        } else {
            glory::web::launch_with(root, build).map_err(js)?
        };
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_display(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 0, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_actions(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 1, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_document(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 2, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_layout(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 3, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_timeline(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 4, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_spoiler(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 5, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_grid(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 6, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_visually_hidden(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 7, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_avatar(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 8, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_image(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 9, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_buttons(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 10, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_alerts(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 11, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_graphics(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 12, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_progress(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 13, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_navigation_progress(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 14, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_spinner(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 15, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_loading_overlay(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 17, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_inert_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("inert-stream-root"))
            .ok_or_else(|| js("missing inert stream root"))?;
        let app = glory::web::hydrate(root, crate::loading_overlay_form::inert_stream_probe())
            .map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_modals(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 16, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_notifications(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 18, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_toolbar(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 21, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_color_swatch(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 22, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_sheet(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 19, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_accordion(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 23, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_tabs_dynamic(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 24, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_collapse_motion(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 25, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_data_table_server(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 37, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_data_table(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 35, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_table(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 31, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_table_of_contents(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 30, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_overflow_list(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 26, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_floating_indicator(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 27, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_floating_window(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 38, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_tour(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 39, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_focus_scope(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 55, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_environment(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 74, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_messages(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 90, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_message_scroller(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 91, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_model_suggestions(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 94, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_questionnaire(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 93, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_prompt(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 92, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_responsive(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 75, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_portal(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 54, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_portal_missing(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 57, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_translation(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 32, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_text_motion(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 33, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_tree(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 45, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_tree_select(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 46, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_cascader(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 47, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_charts(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 66, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_cartesian(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 67, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_polar_charts(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 68, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_point_charts(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 69, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_shape_charts(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 70, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_matrix_charts(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 71, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_range_charts(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 72, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_hierarchy_charts(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 73, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_rich_editor(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 82, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_carousel(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 80, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_lightbox(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 81, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_public_explanations(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 96, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_citations(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 97, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_output(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 101, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_tasks(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 99, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_confirmation(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 98, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_files(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 51, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_files_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("files-stream-root"))
            .ok_or_else(|| js("missing files stream root"))?;
        let app =
            glory::web::hydrate(root, crate::files_form::files_stream_probe()).map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_copy_button(
        hydrate: bool,
        hue: String,
        scheme: String,
    ) -> Result<(), JsValue> {
        mount(hydrate, 44, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_code(hydrate: bool, hue: String, scheme: String) -> Result<(), JsValue> {
        mount(hydrate, 34, hue, scheme)
    }
    #[wasm_bindgen]
    pub fn mount_collapse_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("collapse-stream-root"))
            .ok_or_else(|| js("missing collapse stream root"))?;
        let app =
            glory::web::hydrate(root, crate::collapse_stream_probe::collapse_stream_probe())
                .map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn mount_sheet_stream_probe() -> Result<(), JsValue> {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("sheet-stream-root"))
            .ok_or_else(|| js("missing sheet stream root"))?;
        let app =
            glory::web::hydrate(root, crate::sheet_form::sheet_stream_probe()).map_err(js)?;
        APP.with(|slot| *slot.borrow_mut() = Some(app));
        Ok(())
    }
    #[wasm_bindgen]
    pub fn unmount_display() {
        let old = APP.with(|slot| slot.borrow_mut().take());
        drop(old);
    }
}
