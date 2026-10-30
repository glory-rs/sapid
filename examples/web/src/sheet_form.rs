use glory::render::{FocusMode, Layer, LayerPlacement, Role, SemanticId, Semantics, SheetConfig};
use glory::style::{Length, Overflow, Style};
use glory::ui::{Widget, text, view, widget};
use sapid::{
    ContentPolicy, DrawerSide, DrawerSnap, DrawerSnaps, Theme, button, drawer, text_field,
};
pub fn sheet_stream_probe() -> impl Widget {
    view()
        .class("stream-sheet")
        .role(Role::Dialog)
        .focus(FocusMode::Programmatic)
        .name("Streaming sheet")
        .semantics(Some(Box::new(Semantics {
            id: Some(SemanticId::new("stream-sheet").unwrap()),
            layer: Some(Layer {
                placement: LayerPlacement::Bottom,
                sheet: Some(SheetConfig {
                    fractions: vec![0.25, 0.5, 1.0],
                    current: 2,
                    max_extent: 640.0,
                    settle_millis: 180,
                    dismissible: true,
                    revision: 1,
                }),
                ..Default::default()
            }),
            ..Default::default()
        })))
        .fill((
            view()
                .class("stream-sheet-grip")
                .semantics(Some(Box::new(Semantics {
                    sheet_grip: true,
                    ..Default::default()
                })))
                .style(Style::new().height(Length::Px(24.0))),
            glory::ui::button().label("Stream action"),
        ))
}
pub fn sheet_form(theme: Theme) -> impl Widget {
    widget(move |scope| {
        let blocked = scope.cage(false);
        button()
            .theme(theme.clone())
            .label("Unblock sheets")
            .on_press(move || blocked.revise(|v| *v = false))
            .show_in(scope)?;
        for (name, side) in [
            ("left", DrawerSide::Left),
            ("right", DrawerSide::Right),
            ("top", DrawerSide::Top),
            ("bottom", DrawerSide::Bottom),
        ] {
            let open = scope.cage(false);
            let snap = scope.cage(1usize);
            let accept = scope.cage(true);
            let accept_close = scope.cage(true);
            let proposals = scope.cage(0usize);
            let value = scope.cage("Retained sheet value".to_string());
            let options = scope.cage(
                DrawerSnaps::new([
                    DrawerSnap::new(0.25, "Compact"),
                    DrawerSnap::new(0.5, "Half"),
                    DrawerSnap::new(1.0, "Full"),
                ])
                .max_extent(640.0),
            );
            let content_theme = theme.clone();
            view().semantics(scope.bond(move||Some(Box::new(Semantics{inert:blocked.get(),..Default::default()}))))
                .fill(drawer(format!("sheet-{name}"))
                    .theme(theme.clone()).label(format!("Open {name} sheet")).title(format!("{name} sheet")).side(side)
                    .class(format!("sheet-{name}")).open(open).snap_index(snap).snap_config(options).content_policy(ContentPolicy::KeepMounted)
                    .on_snap_change(move|index|{proposals.revise(|n|*n+=1);if accept.get(){snap.revise(|v|*v=index)}})
                    .on_open_change(move|next|{if next||accept_close.get(){open.revise(|v|*v=next)}})
                    .content(move||sapid::stack().theme(content_theme.clone()).gap(8.0).fill((
                        text_field().theme(content_theme.clone()).name(format!("{name} sheet editor")).value(value).on_value_change(move|next|value.revise(|v|*v=next)),
                        button().theme(content_theme.clone()).label("Toggle snap acceptance").on_press(move||accept.revise(|v|*v = !*v)),
                        button().theme(content_theme.clone()).label("Toggle close acceptance").on_press(move||accept_close.revise(|v|*v = !*v)),
                        button().theme(content_theme.clone()).label("Set compact").on_press(move||snap.revise(|v|*v=0)),
                        button().theme(content_theme.clone()).label("Change sheet configuration").on_press(move||options.revise(|v|v.max_extent=if v.max_extent==640.0{480.0}else{640.0})),
                        button().theme(content_theme.clone()).label("Block sheet").on_press(move||blocked.revise(|v|*v=true)),
                        text().class("sheet-accepted").content(snap.map(|v|format!("Accepted snap: {v}"))),
                        text().class("sheet-proposals").content(proposals.map(|v|format!("Proposals: {v}"))),
                        text().class("sheet-accept").content(accept.map(|v|format!("Accept snaps: {v}"))),
                        view().class("sheet-scroll").style(Style::new().height(Length::Px(180.0)).min_height(Length::Px(80.0)).overflow_y(Overflow::Scroll)).fill(view().style(Style::new().height(Length::Px(900.0))).fill(text().content("Scrollable sheet content retains native scrolling and text selection."))),
                    )))
                ).show_in(scope)?;
        }
        Ok(())
    })
}
