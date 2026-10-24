use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::table_of_contents_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::{Theme, button, table_of_contents, toc_context, toc_document, toc_heading};
    pub fn table_of_contents_form(theme: Theme) -> impl Widget {
        widget(move |scope| {
            let ctx = toc_context(scope, "guide")?;
            let keys = scope.cage(
                ["overview", "nested", "details", "finish"]
                    .map(str::to_owned)
                    .to_vec(),
            );
            let label = scope.cage("Nested topic".to_owned());
            let draft = scope.cage("Keep this draft".to_owned());
            let current = ctx.current(scope);
            let entries = ctx.entries(scope);
            let reveal_entries = ctx.entries(scope);
            let c = ctx.clone();
            let t = theme.clone();
            let status = scope.bond(move || {
                format!(
                    "Headings: {}; current: {}",
                    entries.get().len(),
                    current
                        .get()
                        .map_or_else(|| "none".to_owned(), |id| id.as_str().to_owned())
                )
            });
            let doc=toc_document(ctx.clone(),"article").theme(theme.clone()).name("Guide document").offset(8.).style(Style::new().height(Length::Px(180.)).width(Length::Percent(100.))).content(move|ctx|{
   let t=t.clone();each(keys,|key|key.clone(),move|key,index|{
    let key=key.get();let level=match key.as_str(){"nested"=>3,"details"=>2,_=>1};let heading_label:Lotus<String>=match key.as_str(){"overview"=>"Overview".into(),"nested"=>label.into(),"details"=>"Details".into(),_=>"Finish".into()};
    glory::ui::view().class(format!("toc-section toc-section-{key}")).style(Style::new().height(Length::Px(220.))).fill(toc_heading(ctx.clone(),key,level).theme(t.clone()).content(heading_label).order(index.map(|value|*value as u32))).fill(sapid::text().theme(t.clone()).content("A registered live section. Scroll the guide or follow its actual section link."))
   })
  });
            glory::ui::col()
                .style(
                    Style::new()
                        .display(Display::Flex)
                        .flex_direction(glory::style::FlexDirection::Column)
                        .gap(12.)
                        .width(Length::Percent(100.)),
                )
                .fill(
                    sapid::text_field()
                        .class("toc-draft")
                        .theme(theme.clone())
                        .name("Guide notes")
                        .value(draft)
                        .on_value_change(move |value| draft.revise(|v| *v = value)),
                )
                .fill(
                    table_of_contents(ctx.clone(), "navigation")
                        .name("Guide sections")
                        .theme(theme.clone()),
                )
                .fill(doc)
                .fill(glory::ui::text().class("toc-status").content(status))
                .fill(
                    glory::ui::row()
                        .style(Style {
                            flex_wrap: Some(glory::style::FlexWrap::Wrap),
                            ..Style::new().display(Display::Flex).gap(8.)
                        })
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Rename section")
                                .on_press(move || {
                                    label.revise(|v| {
                                        *v = if v == "Nested topic" {
                                            "Renamed topic".into()
                                        } else {
                                            "Nested topic".into()
                                        }
                                    })
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Reverse sections")
                                .on_press(move || keys.revise(|v| v.reverse())),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Remove nested")
                                .on_press(move || {
                                    keys.revise(|v| v.retain(|key| key != "nested"))
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Restore nested")
                                .on_press(move || {
                                    keys.revise(|v| {
                                        if !v.iter().any(|key| key == "nested") {
                                            v.insert(1, "nested".into())
                                        }
                                    })
                                }),
                        )
                        .fill(
                            button()
                                .theme(theme.clone())
                                .label("Reveal details")
                                .on_press(move || {
                                    let entry = reveal_entries
                                        .get()
                                        .into_iter()
                                        .find(|e| e.label == "Details");
                                    if let Some(entry) = entry {
                                        c.reveal(&entry.id).expect("mounted guide")
                                    }
                                }),
                        ),
                )
                .show_in(scope)
        })
    }
}
