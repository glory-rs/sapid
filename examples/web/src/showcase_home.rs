//! The landing page is made from the same live widgets as the documentation.
use glory::render::{Role, SemanticId, Semantics};
use glory::ui::{Widget, view, widget};
use sapid::{ButtonVariant, Size, Theme};

fn copy(class: &str, content: impl Into<glory::Vary<String>>) -> impl Widget {
    sapid::text()
        .unstyled()
        .class(class.to_owned())
        .content(content)
}

fn heading(level: u8, class: &str, content: &str) -> impl Widget {
    sapid::title(level)
        .unstyled()
        .class(class.to_owned())
        .content(content.to_owned())
}

fn destination(theme: &Theme, id: &str) -> String {
    let hue = &theme.name;
    let scheme = theme.scheme.name();
    format!("showcase.html?widget={id}&theme={hue}&scheme={scheme}")
}

fn link(theme: &Theme, id: &str, label: &str, class: &str) -> impl Widget {
    sapid::anchor()
        .unstyled()
        .href(destination(theme, id))
        .on_navigate(crate::showcase_host::request)
        .label(label.to_owned())
        .class(class.to_owned())
}

fn mosaic(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let saved = scope.cage(false);
        let save_label = saved.map(|saved| {
            if *saved {
                "Changes saved ✓".to_owned()
            } else {
                "Save changes".to_owned()
            }
        });
        let progress = scope.cage(72.);
        let progress_label = progress.map(|value| format!("{}%", *value as u32));
        let palette = theme.clone();
        view()
            .class("site-hero-mosaic")
            .fill((
                view().class("site-mosaic-column site-mosaic-left").fill((
                    sapid::card()
                        .unstyled()
                        .class("site-mosaic-card site-mosaic-buttons")
                        .fill((
                            copy("site-mini-eyebrow", "MADE TO FIT"),
                            heading(3, "site-mini-title", "Your style. Your way."),
                            sapid::button()
                                .theme(theme.clone())
                                .label("Create something")
                                .size(Size::Small)
                                .on_press(move || saved.revise(|value| *value = !*value)),
                            sapid::button()
                                .theme(theme.clone())
                                .variant(ButtonVariant::Secondary)
                                .size(Size::Small)
                                .label("Make it your own")
                                .on_press(move || saved.revise(|value| *value = !*value)),
                            view().class("site-inline-tags").fill((
                                sapid::badge().theme(theme.clone()).content("Flexible"),
                                sapid::badge().theme(theme.clone()).content("Accessible"),
                            )),
                        )),
                    sapid::card()
                        .unstyled()
                        .class("site-mosaic-card site-mosaic-progress")
                        .fill((
                            sapid::ring_progress(progress)
                                .theme(theme.clone())
                                .name("Demo project progress")
                                .size(94.)
                                .thickness(8.),
                            view().class("site-progress-copy").fill((
                                copy("site-progress-number", progress_label),
                                copy("site-mini-muted", "A little closer."),
                                sapid::button()
                                    .theme(theme.clone())
                                    .variant(ButtonVariant::Secondary)
                                    .size(Size::ExtraSmall)
                                    .label("Keep going →")
                                    .on_press(move || {
                                        progress.revise(|value| {
                                            *value = if *value >= 100. {
                                                0.
                                            } else {
                                                (*value + 7.).min(100.)
                                            }
                                        })
                                    }),
                            )),
                        )),
                    view()
                        .class("site-mosaic-caption")
                        .fill(copy("", "Real widgets. Go ahead, try them.")),
                )),
                view().class("site-mosaic-column site-mosaic-right").fill((
                    sapid::card()
                        .unstyled()
                        .class("site-mosaic-card site-mosaic-form")
                        .fill((
                            view().class("site-card-kicker").fill((
                                copy("site-mini-eyebrow", "YOUR WORKSPACE"),
                                copy("site-micro-dot", "●"),
                            )),
                            heading(3, "site-mini-title", "Welcome back"),
                            copy("site-mini-muted", "Good ideas start here."),
                            sapid::field("home-mosaic-email")
                                .theme(theme.clone())
                                .label("Email address")
                                .size(Size::Small)
                                .control(|field| {
                                    sapid::text_field()
                                        .field(field)
                                        .default_value("you@studio.dev")
                                }),
                            sapid::checkbox()
                                .theme(theme.clone())
                                .size(Size::Small)
                                .label("Remember me")
                                .default_checked(true),
                            sapid::button()
                                .theme(theme.clone())
                                .size(Size::Small)
                                .class("site-mosaic-save")
                                .label(save_label)
                                .on_press(move || saved.revise(|value| *value = true)),
                        )),
                    sapid::card()
                        .unstyled()
                        .class("site-mosaic-card site-mosaic-team")
                        .fill((
                            view().class("site-team-header").fill((
                                sapid::avatar("Alex Morgan")
                                    .theme(palette.clone())
                                    .fallback("AM")
                                    .size(Size::Large),
                                view().fill((
                                    copy("site-team-name", "Alex Morgan"),
                                    copy("site-mini-muted", "Product designer"),
                                )),
                                copy("site-team-check", "✓"),
                            )),
                            copy("site-team-description", "The details make the difference."),
                            view().class("site-inline-tags").fill((
                                sapid::badge().theme(theme.clone()).content("Design"),
                                sapid::pill().theme(theme.clone()).content("Rust"),
                            )),
                        )),
                )),
            ))
            .show_in(scope)
    })
}

fn category(
    theme: &Theme,
    group: &'static str,
    id: &'static str,
    icon: &'static str,
    class: &'static str,
    description: &'static str,
) -> impl Widget {
    let count = crate::showcase_catalog::ENTRIES
        .iter()
        .filter(|entry| entry.group == group)
        .count();
    sapid::card()
        .unstyled()
        .class(format!("site-category-card {class}"))
        .fill((
            view().class("site-category-top").fill((
                copy("site-category-icon", icon),
                copy("site-category-count", format!("{count} guides")),
            )),
            link(theme, id, &format!("{group}  →"), "site-category-title"),
            copy("site-category-description", description),
        ))
}

fn playground(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let count = scope.cage(0_u32);
        let caption = count.map(|count| format!("Clicked {count} times"));
        let notifications = scope.cage(true);
        let updates = scope.cage(false);
        let picked = scope.cage("balanced".to_owned());
        view().class("site-playground-grid").fill((
            sapid::card().unstyled().class("site-playground-card").fill((
                copy("site-eyebrow", "FEELS LIKE YOUR APP"),
                heading(3, "site-playground-title", "Small interactions. Thoughtfully built."),
                copy("site-playground-description", "Click, type, and make a choice. These are the same widgets you will use in your application."),
                view().class("site-button-demo").fill((
                    sapid::button().theme(theme.clone()).label("Give it a click").on_press(move || count.revise(|count| *count += 1)),
                    sapid::button().theme(theme.clone()).variant(ButtonVariant::Secondary).label("Reset").on_press(move || count.revise(|count| *count = 0)),
                    copy("site-demo-feedback", caption),
                )),
                link(&theme, "P0-01", "Explore Button →", "site-text-link"),
            )),
            sapid::card().unstyled().class("site-playground-card site-preferences").fill((
                copy("site-eyebrow", "BUILT AROUND YOU"),
                heading(3, "site-playground-title", "Make yourself at home."),
                copy("site-preference-label", "Choose your pace"),
                sapid::segmented_control("home-preferences").theme(theme.clone()).name("Workspace pace").value(picked)
                    .on_value_change(move |value| picked.revise(|picked| *picked = value))
                    .segments([sapid::segment("relaxed").label("Relaxed"), sapid::segment("balanced").label("Balanced"), sapid::segment("focused").label("Focused")]),
                view().class("site-preference-row").fill((
                    view().fill((copy("site-preference-title", "Notifications"), copy("site-mini-muted", "Stay in the loop."))),
                    sapid::switch().theme(theme.clone()).name("Demo notifications").checked(notifications)
                        .on_checked_change(move |checked| notifications.revise(|value| *value = checked)),
                )),
                sapid::checkbox().theme(theme.clone()).label("Send me product updates").checked(updates)
                    .on_checked_change(move |checked| updates.revise(|value| *value = checked)),
            )),
        )).show_in(scope)
    })
}

pub fn home(theme: Theme) -> impl Widget {
    let count = crate::showcase_catalog::ENTRIES.len();
    let groups = crate::showcase_catalog::ENTRIES
        .iter()
        .map(|entry| entry.group)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    view().role(Role::Main).class("site-home").fill((
        view().class("site-home-container site-hero").fill((
            view().class("site-hero-copy").fill((
                copy("site-hero-eyebrow", "SAPID · BUILT WITH GLORY"),
                heading(1, "site-hero-title", "Build beautiful\nRust applications."),
                copy("site-hero-lead", format!("A thoughtfully crafted widget library for your next idea. {count} widget guides, expressive themes, and the freedom of handwritten Rust.")),
                view().class("site-hero-actions").fill((
                    link(&theme, "P0-01", "Get started  →", "site-button site-button-primary"),
                    sapid::anchor().unstyled().href("#widgets").label("Explore widgets").class("site-button site-button-secondary"),
                )),
                view().class("site-hero-facts").fill((copy("", format!("{count} widget guides")), copy("site-fact-dot", "·"), copy("", "Light & dark themes"), copy("site-fact-dot", "·"), copy("", "Pure Rust"))),
            )),
            mosaic(theme.clone()),
        )),
        view().class("site-home-container site-principles").fill((
            view().fill((copy("site-principle-symbol", "{ }"), copy("site-principle-title", "Rust, all the way"), copy("site-principle-copy", "Compose your interface with familiar, handwritten builders."))),
            view().fill((copy("site-principle-symbol", "◐"), copy("site-principle-title", "A theme that fits"), copy("site-principle-copy", "Consistent tokens, thoughtful defaults, and room for your style."))),
            view().fill((copy("site-principle-symbol", "↹"), copy("site-principle-title", "Details included"), copy("site-principle-copy", "Real controls, keyboard behavior, and accessible semantics."))),
        )),
        view().class("site-home-container site-widget-section").semantics(Some(Box::new(Semantics { id: Some(SemanticId::new("widgets").expect("static semantic id")), ..Default::default() }))).fill((
            view().class("site-section-heading").fill((
                copy("site-eyebrow", "FROM THE FIRST BUTTON TO THE FINAL DETAIL"),
                heading(2, "site-home-section-title", "Everything you need to build your next idea."),
                copy("site-section-description", format!("Explore {count} widget guides across {groups} categories. Each page includes a live example, Rust source, and a clear description of supported behavior.")),
            )),
            view().class("site-category-grid").fill((
                category(&theme, "Inputs", "P0-02", "Aa", "site-category-blue", "From a simple text field to considered, complete forms."),
                category(&theme, "Overlays", "P0-07", "▣", "site-category-violet", "Give your content a little room to step forward."),
                category(&theme, "Navigation", "P6-01", "↗", "site-category-cyan", "Help people find their way, without getting in it."),
                category(&theme, "Data display", "P8-01", "≡", "site-category-teal", "Turn your information into something easy to understand."),
                category(&theme, "Dates", "P9-02", "28", "site-category-orange", "A thoughtful set of controls for dates and schedules."),
                category(&theme, "Charts", "P11-01", "▥", "site-category-pink", "Give your numbers a voice with real, interactive charts."),
            )),
        )),
        view().class("site-home-container site-live-section").fill((
            view().class("site-section-heading").fill((copy("site-eyebrow", "MORE THAN A PRETTY PREVIEW"), heading(2, "site-home-section-title", "Good design is in the details."), copy("site-section-description", "A few small examples. A lot of possibilities."))),
            playground(theme.clone()),
        )),
        view().class("site-home-container site-get-started").semantics(Some(Box::new(Semantics { id: Some(SemanticId::new("get-started").expect("static semantic id")), ..Default::default() }))).fill((
            view().class("site-start-copy").fill((
                copy("site-eyebrow", "LESS CEREMONY. MORE CREATING."),
                heading(2, "site-home-section-title", "Your next idea\nstarts here."),
                copy("site-section-description", "Readable Rust, composable widgets, and a reactive model that stays with you from the first prototype to a complete application."),
                link(&theme, "P0-01", "Start building  →", "site-button site-button-primary"),
            )),
            view().class("site-start-code").fill((
                view().class("site-start-code-header").fill((copy("", "Hello, Sapid"), copy("site-code-language", "Rust"))),
                sapid::code_highlight().theme(theme).language(sapid::CodeLanguage::Rust).class("site-home-code").source("use sapid::{button, stack, title};\n\nstack().gap(20.0).fill((\n    title(1).content(\"Hello, possibility.\"),\n    button()\n        .label(\"Let's build something\")\n        .on_press(move || {\n            // Your next idea goes here.\n        }),\n))"),
            )),
        )),
        view().class("site-footer").fill((
            copy("", "Sapid · Made with Glory and handwritten Rust."),
            sapid::anchor().unstyled().href("https://github.com/sapid-ui/sapid").label("Explore the source ↗"),
        )),
    ))
}
