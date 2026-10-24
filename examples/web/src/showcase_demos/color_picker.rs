use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use glory::render::Direction;
    use sapid::{Hsva, Theme, color_input, color_picker};
    pub struct State {
        pub color: Cage<Hsva>,
        pub draft: Cage<String>,
        pub accept: Cage<bool>,
        pub readonly: Cage<bool>,
        pub enabled: Cage<bool>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let color = scope.cage(Hsva {
            hue: 120.,
            saturation: 0.5,
            value: 0.75,
            alpha: 0.4,
        });
        let draft = scope.cage("#6496c880".to_owned());
        let accept = scope.cage(true);
        let readonly = scope.cage(false);
        let enabled = scope.cage(true);
        let changes = scope.cage(0u32);
        let commits = scope.cage(0u32);
        let field = sapid::field("accent")
            .label("Accent hex")
            .theme(theme.clone())
            .required(true)
            .form_name("accent")
            .readonly(readonly)
            .enabled(enabled)
            .control(move |field| {
                color_input("accent")
                    .field(field)
                    .value(draft)
                    .on_value_change(move |value| {
                        if accept.get_untracked() {
                            draft.revise(|old| *old = value);
                        }
                    })
            });
        let widget = view().class("color-picker-demo").fill((
            sapid::text_field()
                .theme(theme.clone())
                .name("Picker notes")
                .default_value("Retain 😀 caret")
                .class("picker-notes"),
            color_picker("main")
                .theme(theme.clone())
                .name("Main color")
                .value(color)
                .readonly(readonly)
                .enabled(enabled)
                .class("main-picker")
                .on_value_change(move |value| {
                    changes.revise(|count| *count += 1);
                    if accept.get_untracked() {
                        color.revise(|old| *old = value);
                    }
                })
                .on_value_commit(move |_| commits.revise(|count| *count += 1)),
            field,
            color_picker("rtl")
                .theme(theme.clone())
                .name("RTL color")
                .direction(Direction::Rtl)
                .plane_size(128.)
                .show_alpha(false)
                .class("rtl-picker"),
            color_picker("bare")
                .theme(theme.clone())
                .name("Unstyled color")
                .plane_size(128.)
                .show_alpha(false)
                .unstyled()
                .class("bare-picker"),
            sapid::text_field()
                .theme(theme)
                .name("After picker")
                .default_value("after"),
            text().class("picker-status").content(scope.bond(move || {
                let c = color.get();
                format!(
                    "hue={} saturation={} value={} alpha={} changes={} commits={}",
                    c.hue,
                    c.saturation,
                    c.value,
                    c.alpha,
                    changes.get(),
                    commits.get()
                )
            })),
        ));
        (
            widget,
            State {
                color,
                draft,
                accept,
                readonly,
                enabled,
            },
        )
    }
}
