use glory::prelude::*;
use std::rc::Rc;
fn document(text: &str) -> Rc<sapid::RichDocument> {
    Rc::new(
        sapid::RichDocument::new(vec![sapid::RichBlock {
            kind: sapid::RichBlockKind::Paragraph,
            runs: vec![sapid::RichRun {
                text: text.into(),
                marks: Default::default(),
            }],
        }])
        .unwrap(),
    )
}
pub fn rich_editor_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let source = scope.cage(document("Hello 😀 world"));
        let controller = scope.cage(None::<sapid::EditorController>);
        let rejects = scope.cage(false);
        let enabled = scope.cage(true);
        let readonly = scope.cage(false);
        let visible = scope.cage(true);
        let events = scope.cage(0usize);
        let selections = scope.cage(0usize);
        let rejected = scope.cage(0usize);
        let order = scope.cage(String::new());
        let main_theme = theme.clone();
        let field_enabled = scope.cage(false);
        sapid::stack().gap(12.).fill((
   glory::ui::when(visible,move||sapid::rich_text_editor().theme(main_theme.clone()).class("rich-main").name("Article editor").document(source).controller_ref(controller).enabled(enabled).readonly(readonly)
    .on_document_change(move|next|{events.revise(|count|*count+=1);order.revise(|value|*value="typed".into());if !rejects.get(){source.revise(|value|*value=next);}})
    .on(glory::render::EventKind::RichEdit,move|_|order.revise(|value|value.push_str(";generic")))
    .on(glory::render::EventKind::RichSelection,move|_|selections.revise(|count|*count+=1))
    .on(glory::render::EventKind::RichRejected,move|_|rejected.revise(|count|*count+=1))),
   sapid::group().fill((
    sapid::button().label("Program editor source").on_press(move||source.revise(|value|*value=document("Program wins 😀"))),
    sapid::button().label("Reject editor proposals").on_press(move||rejects.revise(|value|*value = !*value)),
    sapid::button().label("Toggle editor readonly").on_press(move||readonly.revise(|value|*value = !*value)),
    sapid::button().label("Toggle editor enabled").on_press(move||enabled.revise(|value|*value = !*value)),
    sapid::button().label("Toggle editor lifetime").on_press(move||visible.revise(|value|*value = !*value)),
    sapid::button().label("Link selection").on_press(move||{if let Some(controller)=controller.get(){let _=controller.format(sapid::RichFormat::Link(Some("https://example.test/safe?x=1&y=2".into())));}}),
    sapid::button().label("Clear selection link").on_press(move||{if let Some(controller)=controller.get(){let _=controller.format(sapid::RichFormat::Link(None));}}),
   )),
   glory::ui::text().class("rich-status").content(scope.bond(move||{let summary=controller.get().map(|controller|(controller.history().get(),controller.composing().get()));format!("events:{}; selections:{}; rejected:{}; history:{summary:?}; order:{}; refused:{}",events.get(),selections.get(),rejected.get(),order.get(),rejects.get())})),
   glory::ui::text().class("rich-source").content(source.map(|document|document.plain_text())),
   sapid::rich_text_editor().theme(theme.clone()).class("rich-readonly").name("Readonly document").toolbar(false).readonly(true).default_document(document("Select this immutable 😀 document")),
   sapid::rich_text_editor().unstyled().class("rich-plain").name("Unstyled document").toolbar(false).default_document(document("Unstyled finite editor")),
   sapid::fieldset("rich-disabled-fieldset").legend("Disabled editor group").enabled(field_enabled).fill(sapid::rich_text_editor().theme(theme.clone()).class("rich-fieldset").name("Grouped document").default_document(document("Group controls editing"))),
   sapid::button().label("Toggle editor fieldset").on_press(move||field_enabled.revise(|value|*value = !*value)),
  )).show_in(scope)
    })
}
