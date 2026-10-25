use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| fixture::form(scope, theme.clone()).0.show_in(scope))
}

#[allow(dead_code, unused_imports)]
mod fixture {
    //! Public test vectors only. Status exposes lengths/counts, never code contents.
    use glory::prelude::*;
    use glory::render::EventKind;
    use sapid::{PinAlphabet, Theme, input_otp, pin_input};
    pub struct State {
        pub value: Cage<String>,
        pub accept: Cage<bool>,
        pub enabled: Cage<bool>,
        pub readonly: Cage<bool>,
        pub error: Cage<String>,
        pub visible: Cage<bool>,
        pub reference: Cage<Option<glory::render::NodeId>>,
    }
    pub fn form(scope: &Scope, theme: Theme) -> (impl Widget + use<>, State) {
        let value = scope.cage(String::new());
        let accept = scope.cage(true);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let error = scope.cage(String::new());
        let visible = scope.cage(true);
        let reference = scope.cage(None);
        let changes = scope.cage(0usize);
        let commits = scope.cage(0usize);
        let complete = scope.cage(0usize);
        let rejected = scope.cage(0usize);
        let submits = scope.cage(0usize);
        let stage = scope.cage(false);
        let ordered = scope.cage(true);
        let status = scope.bond(move || {
            format!(
                "length={};changes={};commits={};complete={};rejected={};submits={};ordered={}",
                value.with(String::len),
                changes.get(),
                commits.get(),
                complete.get(),
                rejected.get(),
                submits.get(),
                ordered.get()
            )
        });
        let primary_theme = theme.clone();
        let primary = glory::ui::when(visible, move || {
            sapid::field("pin-code").theme(primary_theme.clone()).label("Verification code").description("Public fixture: six ASCII digits, adjacent glyph cells").form_name("otp").required(true).enabled(enabled).readonly(readonly).error(error).control(move|field|input_otp().field(field).class("primary-pin").value(value).node_ref(reference).on_value_change(move|next|{changes.revise(|n|*n+=1);stage.revise(|v|*v=true);if accept.get_untracked(){value.revise(|v|*v=next);}}).on_commit(move|next|{commits.revise(|n|*n+=1);if accept.get_untracked(){value.revise(|v|*v=next);}}).on_complete(move|_|complete.revise(|n|*n+=1)).on_reject(move|_|{rejected.revise(|n|*n+=1);stage.revise(|v|*v=true);}).on(EventKind::ValueChanged,move|event|{if matches!(event,Event::ValueChanged{composing:false,ref value,..} if value.len()<=6&&value.bytes().all(|v|v.is_ascii_digit())){ordered.revise(|v|*v&=stage.get_untracked());stage.revise(|v|*v=false);}}))
        });
        (
            sapid::form("pin-demo")
                .theme(theme.clone())
                .on_submit(move |_| submits.revise(|n| *n += 1))
                .fill(
                    glory::ui::view().class("pin-input-demo").fill((
                        sapid::title(1)
                            .theme(theme.clone())
                            .content("PinInput / InputOTP"),
                        primary,
                        sapid::text()
                            .theme(theme.clone())
                            .class("pin-input-status")
                            .content(status),
                        pin_input()
                            .theme(theme.clone())
                            .name("Masked public code")
                            .class("masked-pin")
                            .default_value("000000")
                            .masked(true),
                        pin_input()
                            .theme(theme.clone())
                            .name("Letters and digits")
                            .class("alphanumeric-pin")
                            .length(4)
                            .alphabet(PinAlphabet::AsciiAlphanumeric)
                            .default_value("aZ09"),
                        pin_input()
                            .theme(theme.clone())
                            .name("Readonly public code")
                            .default_value("000000")
                            .readonly(true),
                        pin_input()
                            .theme(theme.clone())
                            .name("Disabled public code")
                            .default_value("000000")
                            .enabled(false),
                        pin_input()
                            .theme(theme.clone())
                            .name("Plain public code")
                            .class("plain-pin")
                            .length(4)
                            .default_value("0000")
                            .unstyled()
                            .style(Style {
                                font_size: Some(24.),
                                ..Style::new()
                            }),
                        glory::ui::input()
                            .name("Unrelated draft")
                            .class("pin-draft")
                            .value("Stable draft"),
                    )),
                ),
            State {
                value,
                accept,
                enabled,
                readonly,
                error,
                visible,
                reference,
            },
        )
    }
}
