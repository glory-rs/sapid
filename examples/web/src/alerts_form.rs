use glory::prelude::*;
use sapid::{AlertSeverity, Announcement, ButtonVariant, alert, alert_dialog};
pub fn alerts_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let visible = scope.cage(true);
        let open = scope.cage(false);
        let accept = scope.cage(true);
        let confirm_enabled = scope.cage(true);
        let log = scope.cage(String::new());
        let loading = scope.cage(true);
        let floating_theme = theme.clone();
        let local_theme = theme.clone();
        let controlled_theme = theme.clone();
        sapid::stack()
            .theme(theme.clone())
            .class("alerts-fixture")
            .gap(16.0)
            .fill((
                alert("saved")
                    .theme(theme.clone())
                    .title("Changes saved")
                    .content("Your workspace is up to date.")
                    .severity(AlertSeverity::Success)
                    .closable(true)
                    .close_label("Dismiss saved message")
                    .class("local-alert"),
                alert("network")
                    .theme(theme.clone())
                    .title("Connection lost")
                    .content("Changes remain on this device.")
                    .severity(AlertSeverity::Warning)
                    .announcement(Announcement::Assertive)
                    .visible(visible)
                    .closable(true)
                    .close_label("Request network dismissal")
                    .on_close(move || log.revise(|value| value.push_str("alert-close;")))
                    .class("controlled-alert"),
                alert("passive")
                    .theme(theme.clone())
                    .title("Reference")
                    .content("This message does not request an announcement.")
                    .announcement(Announcement::Off),
                alert_dialog("delete-draft")
                    .theme(theme.clone())
                    .label("Delete draft")
                    .title("Delete this draft?")
                    .description("This removes the local draft permanently.")
                    .confirm_label("Delete permanently")
                    .confirm_variant(ButtonVariant::Danger)
                    .cancel_label("Keep draft")
                    .content(move || {
                        sapid::text_field()
                            .theme(local_theme.clone())
                            .name("Decision note")
                            .default_value("Retain my note")
                    })
                    .on_confirm(move || log.revise(|value| value.push_str("local-confirm;")))
                    .on_cancel(move |_| log.revise(|value| value.push_str("local-cancel;"))),
                alert_dialog("controlled-decision")
                    .theme(theme.clone())
                    .label("Open controlled decision")
                    .title("Publish changes?")
                    .description("Publishing requires explicit approval.")
                    .open(open)
                    .confirm_enabled(confirm_enabled)
                    .confirm_label("Publish now")
                    .cancel_label("Continue editing")
                    .content_policy(sapid::ContentPolicy::KeepMounted)
                    .content(move || {
                        sapid::text_field()
                            .theme(controlled_theme.clone())
                            .name("Publication note")
                            .default_value("Preserved publication note")
                    })
                    .on_confirm(move || {
                        log.revise(|value| value.push_str("confirm;"));
                        accept.revise(|value| *value = false);
                    })
                    .on_cancel(move |_| {
                        log.revise(|value| value.push_str("cancel;"));
                        accept.revise(|value| *value = true);
                    })
                    .on_dismiss(move |_| log.revise(|value| value.push_str("dismiss;")))
                    .on_open_change(move |value| {
                        log.revise(|value| value.push_str("change;"));
                        if accept.get() {
                            open.revise(|current| *current = value)
                        }
                    }),
                sapid::floating_dialog("background-note")
                    .theme(theme.clone())
                    .label("Show floating notice")
                    .title("Background sync")
                    .description("You can keep editing while this notice is open.")
                    .content_policy(sapid::ContentPolicy::KeepMounted)
                    .close_label("Dismiss floating notice")
                    .content(move || {
                        sapid::text_field()
                            .theme(floating_theme.clone())
                            .name("Floating note")
                            .default_value("Retained floating note")
                    }),
                sapid::group().theme(theme.clone()).fill((
                    sapid::skeleton()
                        .theme(theme.clone())
                        .name("Loading profile")
                        .class("profile-skeleton")
                        .width(Length::Px(220.0))
                        .height(20.0)
                        .loading(loading),
                    sapid::skeleton()
                        .theme(theme.clone())
                        .class("avatar-skeleton")
                        .circle(40.0)
                        .loading(loading),
                )),
                text()
                    .class("alerts-log")
                    .content(scope.bond(move || log.get().replace(';', "; "))),
                sapid::group().theme(theme.clone()).fill((
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle skeleton loading")
                        .on_press(move || loading.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle alert source")
                        .on_press(move || visible.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle decision acceptance")
                        .on_press(move || accept.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Toggle confirmation enabled")
                        .on_press(move || confirm_enabled.revise(|value| *value = !*value)),
                    sapid::button()
                        .theme(theme.clone())
                        .label("Close from external source")
                        .on_press(move || open.revise(|value| *value = false)),
                )),
            ))
            .show_in(scope)
    })
}
