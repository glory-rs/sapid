use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, ToggleMode, toggle, toggle_group, toggle_item};

    pub struct State {
        pub pressed: Cage<bool>,
        pub single: Cage<Vec<String>>,
        pub multiple: Cage<Vec<String>>,
        pub reject: Cage<bool>,
        pub disabled: Cage<bool>,
        pub changes: Cage<usize>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let pressed = scope.cage(false);
        let single = scope.cage(vec!["left".to_owned()]);
        let multiple = scope.cage(vec!["locked".to_owned()]);
        let reject = scope.cage(false);
        let disabled = scope.cage(false);
        let changes = scope.cage(0usize);
        let enabled = scope.bond(move || !disabled.get());
        let widgets = view()
            .style(
                Style::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(24.0),
            )
            .fill((
                toggle()
                    .theme(theme.clone())
                    .class("toggle-bold")
                    .label("Bold text")
                    .pressed(pressed)
                    .enabled(enabled)
                    .on_pressed_change(move |value| {
                        changes.revise(|count| *count += 1);
                        if !reject.get() {
                            pressed.revise(|old| *old = value)
                        }
                    }),
                toggle_group("text-alignment")
                    .theme(theme.clone())
                    .class("toggle-alignment")
                    .name("Text alignment")
                    .value(single)
                    .enabled(enabled)
                    .items([
                        toggle_item("left").label("Align left").class("toggle-left"),
                        toggle_item("locked")
                            .label("Unavailable alignment")
                            .class("toggle-unavailable")
                            .enabled(false),
                        toggle_item("center")
                            .label("Align center")
                            .class("toggle-center"),
                        toggle_item("right")
                            .label("Align right")
                            .class("toggle-right"),
                    ])
                    .on_value_change(move |value| {
                        changes.revise(|count| *count += 1);
                        if !reject.get() {
                            single.revise(|old| *old = value)
                        }
                    }),
                toggle_group("text-styles")
                    .theme(theme)
                    .class("toggle-formatting")
                    .name("Text styles")
                    .orientation(glory::render::Orientation::Vertical)
                    .mode(ToggleMode::Multiple)
                    .value(multiple)
                    .enabled(enabled)
                    .items([
                        toggle_item("locked")
                            .label("Inherited style")
                            .class("toggle-inherited")
                            .enabled(false),
                        toggle_item("italic")
                            .label("Italic text")
                            .class("toggle-italic"),
                        toggle_item("underline")
                            .label("Underline text")
                            .class("toggle-underline"),
                    ])
                    .on_value_change(move |value| {
                        changes.revise(|count| *count += 1);
                        if !reject.get() {
                            multiple.revise(|old| *old = value)
                        }
                    }),
                text().class("toggle-status").content(scope.bond(move || {
                    format!(
                        "pressed={}; single={:?}; multiple={:?}; changes={}",
                        pressed.get(),
                        single.get(),
                        multiple.get(),
                        changes.get()
                    )
                })),
            ));
        (
            widgets,
            State {
                pressed,
                single,
                multiple,
                reject,
                disabled,
                changes,
            },
        )
    }
}
