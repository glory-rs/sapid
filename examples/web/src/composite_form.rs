use glory::prelude::*;
use sapid::{
    ContentPolicy, TabActivation, accordion, accordion_item, collapsible, tab, tabs, text_field,
};

pub struct State {
    pub open: Cage<bool>,
    pub selected: Cage<String>,
    pub disabled: Cage<bool>,
}

fn column() -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(12.0)
}
fn title(label: &'static str) -> glory::ui::Text {
    text()
        .content(label)
        .kind(TextKind::Heading(2))
        .style(Style {
            font_size: Some(18.0),
            font_weight: Some(600.0),
            ..Default::default()
        })
}

pub fn form(scope: &Scope) -> (impl Widget + use<>, State) {
    form_themed(scope, sapid::Theme::default())
}

pub fn form_themed(scope: &Scope, theme: sapid::Theme) -> (impl Widget + use<>, State) {
    let open = scope.cage(false);
    let selected = scope.cage("a".to_string());
    let disabled = scope.cage(false);
    let proposals = scope.cage(0_usize);
    let widget = col().class("composite-demo").style(column()).fill((
        col()
            .class("preview-card")
            .style(column().padding(Edges::all(24.0)))
            .fill((
                title("Collapsible"),
                collapsible("demo-details")
                    .theme(theme.clone())
                    .class("demo-disclosure")
                    .label("Details")
                    .open(open)
                    .content_policy(ContentPolicy::KeepMounted)
                    .on_open_change(move |value| open.revise(|current| *current = value))
                    .content(draft(&theme, "details-input", "Details draft")),
            )),
        col()
            .class("preview-card")
            .style(column().padding(Edges::all(24.0)))
            .fill((
                title("Accordion"),
                accordion("demo-faq")
                    .theme(theme.clone())
                    .class("demo-accordion")
                    .default_open(["a"])
                    .item(accordion_item("a").label("First question").content(draft(
                        &theme,
                        "faq-input",
                        "FAQ draft",
                    )))
                    .item(
                        accordion_item("b")
                            .label("Disabled question")
                            .enabled(false),
                    )
                    .item(
                        accordion_item("c")
                            .label("Last question")
                            .content(|| text().content("Another answer")),
                    ),
            )),
        col()
            .class("preview-card")
            .style(column().padding(Edges::all(24.0)))
            .fill((
                title("Automatic tabs"),
                tabs("demo-auto")
                    .theme(theme.clone())
                    .class("auto-tabs")
                    .name("Account")
                    .value(selected)
                    .on_value_change(move |value| selected.revise(|current| *current = value))
                    .tab(
                        tab("a")
                            .label("Profile")
                            .enabled(disabled.map(|value| !value))
                            .content(draft(&theme, "profile-input", "Profile draft")),
                    )
                    .tab(tab("b").label("Unavailable").enabled(false))
                    .tab(tab("c").label("Security").content(draft(
                        &theme,
                        "security-input",
                        "Security draft",
                    ))),
            )),
        col()
            .class("preview-card")
            .style(column().padding(Edges::all(24.0)))
            .fill((
                title("Manual tabs"),
                tabs("demo-manual")
                    .theme(theme.clone())
                    .class("manual-tabs")
                    .name("Manual pages")
                    .activation(TabActivation::Manual)
                    .tab(
                        tab("a")
                            .label("Overview")
                            .content(|| text().content("Overview content")),
                    )
                    .tab(
                        tab("b")
                            .label("History")
                            .content(|| text().content("History content")),
                    ),
            )),
        col()
            .class("preview-card")
            .style(column().padding(Edges::all(24.0)))
            .fill((
                title("Vertical tabs"),
                tabs("demo-vertical")
                    .theme(theme.clone())
                    .class("vertical-tabs")
                    .name("Vertical pages")
                    .orientation(Orientation::Vertical)
                    .loop_focus(false)
                    .tab(
                        tab("a")
                            .label("Top")
                            .content(|| text().content("Top content")),
                    )
                    .tab(
                        tab("b")
                            .label("Bottom")
                            .content(|| text().content("Bottom content")),
                    ),
            )),
        col()
            .class("preview-card")
            .style(column().padding(Edges::all(24.0)))
            .fill((
                title("Controlled selection, changes declined"),
                tabs("demo-readonly")
                    .theme(theme.clone())
                    .class("readonly-tabs")
                    .name("Fixed pages")
                    .value("a")
                    .on_value_change(move |_| proposals.revise(|value| *value += 1))
                    .tab(tab("a").label("Fixed"))
                    .tab(tab("b").label("Propose another")),
            )),
        text()
            .class("composite-status")
            .content(scope.bond(move || {
                format!("selected:{} proposals:{}", selected.get(), proposals.get())
            })),
    ));
    (
        widget,
        State {
            open,
            selected,
            disabled,
        },
    )
}

fn draft(
    theme: &sapid::Theme,
    class: &'static str,
    name: &'static str,
) -> impl FnMut() -> sapid::TextField + use<> {
    let theme = theme.clone();
    move || {
        text_field()
            .theme(theme.clone())
            .class(class)
            .name(name)
            .default_value(name)
    }
}
