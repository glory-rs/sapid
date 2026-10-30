//! Exact role inputs and per-index inheritance.
use super::*;
pub(super) fn role_rows(
    draft: Cage<Draft>,
    layer: Layer,
    target: Cage<Target>,
    picker: Cage<Hsva>,
    theme: Theme,
) -> impl Widget {
    widget(move |scope| {
        let mut rows: Vec<Box<dyn Widget>> = Vec::new();
        for field in Target::all() {
            let label = format!("{} {}", layer.name(), field.name());
            let current = scope.bond(move || draft.with(|d| d.text(layer, field)));
            let enabled = scope.bond(move || {
                draft.with(|d| {
                    layer.scheme().is_none_or(|s| d.record.supports(s))
                        && d.overridden(layer, field)
                })
            });
            let supported = scope
                .bond(move || draft.with(|d| layer.scheme().is_none_or(|s| d.record.supports(s))));
            let overridden = scope.bond(move || draft.with(|d| d.overridden(layer, field)));
            let style = scope.bond(move || Style {
                background: Some(draft.with(|d| d.color(layer, field))),
                ..Style::new()
            });
            let invalid = scope.bond(move || parse_invalid(&draft.with(|d| d.text(layer, field))));
            let rowclass = scope.bond(move || {
                if invalid.get() {
                    "studio-color-row is-invalid".to_owned()
                } else {
                    "studio-color-row".to_owned()
                }
            });
            let selected = field;
            let rowtheme = theme.clone();
            let inheritance: Box<dyn Widget> = if layer == Layer::Base {
                Box::new(copy("studio-inherited", "Base"))
            } else {
                Box::new(
                    sapid::checkbox()
                        .theme(theme.clone())
                        .name(format!("Override {label}"))
                        .checked(overridden)
                        .enabled(supported)
                        .on_checked_change(move |on| {
                            draft.revise(|d| d.inherit(layer, field, on));
                            picker.revise(|v| {
                                *v = Hsva::from_rgba(draft.with(|d| d.color(layer, target.get())))
                            });
                        }),
                )
            };
            // Row editing is fully controlled by exact input text, including incomplete hex.
            rows.push(Box::new(
                view().class(rowclass).fill((
                    sapid::button()
                        .unstyled()
                        .class("studio-color-dot")
                        .style(style)
                        .name(format!("Inspect {label}"))
                        .label("")
                        .on_press(move || {
                            target.revise(|t| *t = selected);
                            picker.revise(|v| {
                                *v = Hsva::from_rgba(draft.with(|d| d.color(layer, selected)))
                            });
                        }),
                    copy("studio-color-label", field.name()),
                    sapid::text_field()
                        .theme(rowtheme)
                        .name(label)
                        .value(current)
                        .invalid(invalid)
                        .enabled(enabled)
                        .on_value_change(move |value| {
                            let parsed = model::parse_hex(&value).ok();
                            draft.revise(|d| d.set_text(layer, field, value));
                            if target.get() == field
                                && let Some(c) = parsed
                            {
                                picker.revise(|p| *p = Hsva::from_rgba(c));
                            }
                        }),
                    inheritance,
                )),
            ));
        }
        view().class("studio-color-list").fill(rows).show_in(scope)
    })
}
fn parse_invalid(value: &str) -> bool {
    model::parse_hex(value).is_err()
}
