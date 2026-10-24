use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::toolbar_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, action_bar, button, toolbar, toolbar_group};
    pub fn toolbar_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let enabled = scope.cage(false);
            let count = scope.cage(0usize);
            let accept = scope.cage(false);
            let requests = scope.cage(0usize);
            sapid::stack()
                .gap(16.)
                .class("toolbar-fixture")
                .fill((
                    toolbar("editing")
                        .name("Editing tools")
                        .theme(theme.clone())
                        .group(
                            toolbar_group("clipboard")
                                .name("Clipboard")
                                .button(button().label("Cut draft"))
                                .button(button().label("Copy draft").enabled(enabled)),
                        )
                        .group(
                            toolbar_group("history")
                                .name("History")
                                .button(button().label("Undo draft"))
                                .button(button().label("Redo draft")),
                        ),
                    toolbar("vertical")
                        .name("Vertical tools")
                        .theme(theme.clone())
                        .orientation(glory::render::Orientation::Vertical)
                        .loop_focus(false)
                        .group(
                            toolbar_group("view")
                                .button(button().label("Zoom in"))
                                .button(button().label("Zoom out")),
                        ),
                    toolbar("rtl")
                        .name("RTL tools")
                        .theme(theme.clone())
                        .direction(glory::render::Direction::Rtl)
                        .group(
                            toolbar_group("format")
                                .button(button().label("Align start"))
                                .button(button().label("Align center")),
                        ),
                    action_bar("selection")
                        .name("Selection actions")
                        .theme(theme.clone())
                        .selected_count(count)
                        .on_clear_selection(move || {
                            requests.revise(|v| *v += 1);
                            if accept.get() {
                                count.revise(|v| *v = 0)
                            }
                        })
                        .group(
                            toolbar_group("bulk")
                                .name("Bulk operations")
                                .button(button().label("Archive selected"))
                                .button(button().label("Delete selected")),
                        ),
                    sapid::group().fill((
                        button()
                            .theme(theme.clone())
                            .label("Toggle copy enabled")
                            .on_press(move || enabled.revise(|v| *v = !*v)),
                        button()
                            .theme(theme.clone())
                            .label("Select three rows")
                            .on_press(move || count.revise(|v| *v = 3)),
                        button()
                            .theme(theme.clone())
                            .label("Accept clear selection")
                            .on_press(move || accept.revise(|v| *v = true)),
                        button()
                            .theme(theme.clone())
                            .label("Remove selection")
                            .on_press(move || count.revise(|v| *v = 0)),
                    )),
                    glory::ui::text().class("toolbar-clear-requests").content(
                        scope.bond(move || format!("Clear requests: {}", requests.get())),
                    ),
                    sapid::text_field()
                        .theme(theme.clone())
                        .name("Outside toolbar editor")
                        .default_value("Retained draft"),
                ))
                .show_in(scope)
        })
    }
}
