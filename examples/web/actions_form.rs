use glory::render::Orientation;
use glory::ui::{Widget, component, text};
use sapid::{Theme, action_icon, burger, button, button_group, close_button};

pub fn actions_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let open = scope.cage(false);
        let proposals = scope.cage(0_u32);
        let enabled = scope.cage(true);
        let toolbar_calls = scope.cage(0_u32);
        let toolbar_status =
            scope.bond(move || format!("Toolbar activations: {}", toolbar_calls.get()));
        let caption = scope.bond(move || format!("Disclosure proposals: {}", proposals.get()));
        glory::ui::col()
            .class("actions-fixture")
            .fill((
                action_icon("Refresh preview")
                    .theme(theme.clone())
                    .icon(text().content("↻")),
                close_button().theme(theme.clone()).name("Dismiss preview"),
                burger().theme(theme.clone()).class("local-burger"),
                burger()
                    .theme(theme.clone())
                    .class("controlled-burger")
                    .open(open)
                    .on_open_change(move |_| proposals.revise(|v| *v += 1)),
                button()
                    .theme(theme.clone())
                    .label("Toggle disclosure source")
                    .on_press(move || open.revise(|v| *v = !*v)),
                text().class("action-proposals").content(caption),
                button_group()
                    .theme(theme.clone())
                    .name("Horizontal actions")
                    .enabled(enabled)
                    .buttons([
                        button().label("Group first"),
                        button().label("Group disabled").enabled(false),
                        button().label("Group last"),
                    ]),
                button_group()
                    .theme(theme.clone())
                    .name("Vertical actions")
                    .orientation(Orientation::Vertical)
                    .buttons([button().label("Above"), button().label("Below")]),
                button()
                    .theme(theme.clone())
                    .label("Toggle group enabled")
                    .on_press(move || enabled.revise(|v| *v = !*v)),
                text().class("toolbar-status").content(toolbar_status),
                button_group()
                    .theme(theme.clone())
                    .toolbar(true)
                    .name("RTL editing tools")
                    .direction(glory::render::Direction::Rtl)
                    .enabled(enabled)
                    .buttons([
                        button()
                            .label("Toolbar first")
                            .on_press(move || toolbar_calls.revise(|value| *value += 1)),
                        button().label("Toolbar disabled").enabled(false),
                        button().label("Toolbar last"),
                    ]),
                button_group()
                    .theme(theme.clone())
                    .toolbar(true)
                    .name("Vertical tools")
                    .orientation(Orientation::Vertical)
                    .loop_focus(false)
                    .buttons([
                        button().label("Toolbar above"),
                        button().label("Toolbar below"),
                    ]),
            ))
            .show_in(scope)
    })
}
