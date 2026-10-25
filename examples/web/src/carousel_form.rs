use glory::prelude::*;
pub fn carousel_form(theme: sapid::Theme) -> impl Widget {
    widget(move |scope| {
        let current = scope.cage(5usize);
        let count = scope.cage(1_000_000usize);
        let reject = scope.cage(false);
        let enabled = scope.cage(true);
        let visible = scope.cage(true);
        let proposals = scope.cage(0usize);
        let builds = scope.cage(0usize);
        let cleaned = scope.cage(0usize);
        let order = scope.cage(String::new());
        let paused = scope.cage(true);
        let main_theme = theme.clone();
        let swipe_revision = scope.cage(4u64);
        let swipe_capture = scope.cage(Some(glory::render::SwipeCapture {
            axis: glory::render::Orientation::Horizontal,
            revision: 4,
        }));
        let swipe_log = scope.cage(String::new());
        sapid::stack().theme(theme.clone()).gap(12.).fill((
            glory::ui::group().fill((
                glory::ui::when(visible,move||{
                    sapid::carousel().theme(main_theme.clone()).class("carousel-main").count(count).current(current).overscan(3).enabled(enabled)
                        .style(Style::new().width(Length::Percent(100.)))
                        .on_current_change(move|next|{
                            proposals.revise(|n|*n+=1);order.revise(|s|*s=format!("typed:{next}"));
                            if !reject.get(){current.revise(|n|*n=next);}
                        })
                        .on(glory::render::EventKind::Navigate,move|_|order.revise(|s|s.push_str(";generic")))
                        .render_slide(move|index|{
                            builds.revise(|n|*n+=1);
                            widget(move|scope|{
                                scope.on_cleanup(move||{let _=cleaned.try_revise(|n|*n+=1);})?;
                                glory::ui::view().class("carousel-slide-body").style(Style::new().height(Length::Px(170.)).padding(Edges::all(12.))).fill((
                                    glory::ui::text().content(format!("Logical slide {index} of a million")),
                                    glory::ui::input().class(format!("carousel-editor-{index}")).value(format!("Draft {index}")),
                                    glory::ui::text().content("Swipe the empty area; editors keep native text selection."),
                                )).show_in(scope)
                            })
                        })
                }),
                sapid::group().theme(theme.clone()).fill((
                    sapid::button().theme(theme.clone()).label("Toggle controlled rejection").on_press(move||reject.revise(|v|*v = !*v)),
                    sapid::button().theme(theme.clone()).label("Program next slide").on_press(move||current.revise(|n|*n=(*n+1).min(count.get().saturating_sub(1)))),
                    sapid::button().theme(theme.clone()).label("Toggle carousel enabled").on_press(move||enabled.revise(|v|*v = !*v)),
                    sapid::button().theme(theme.clone()).label("Toggle carousel lifetime").on_press(move||visible.revise(|v|*v = !*v)),
                    sapid::button().theme(theme.clone()).label("Reduce carousel count").on_press(move||{current.revise(|n|*n=0);count.revise(|n|*n=2);}),
                )),
                glory::ui::text().class("carousel-fixture-status").content(scope.bond(move||format!("Current: {}; proposals: {}; factories: {}; cleaned: {}; reject: {}; order: {}",current.get(),proposals.get(),builds.get(),cleaned.get(),reject.get(),order.get()))),
            )),
            glory::ui::group().fill((
                sapid::carousel().theme(theme.clone()).class("carousel-rtl").name("RTL carousel").count(3usize).looping(true).direction(glory::render::Direction::Rtl)
                    .render_slide(|index|glory::ui::view().style(Style::new().height(Length::Px(70.)).padding(Edges::all(8.))).fill(glory::ui::text().content(format!("RTL slide {index}")))),
                sapid::carousel().theme(theme.clone()).class("carousel-auto").name("Autoplay carousel").count(3usize).looping(true).paused(paused)
                    .autoplay(||Box::pin(crate::menu_form::local_delay(180)))
                    .render_slide(|index|glory::ui::view().style(Style::new().height(Length::Px(70.)).padding(Edges::all(8.))).fill(glory::ui::text().content(format!("Auto slide {index}")))),
                sapid::button().theme(theme.clone()).label("Toggle autoplay source pause").on_press(move||paused.revise(|v|*v = !*v)),
                sapid::carousel().unstyled().class("carousel-plain").name("Unstyled carousel").count(1usize).render_slide(|_|glory::ui::text().content("No theme declarations")),
                glory::ui::view().class("swipe-protocol").name("Finite swipe probe").style(Style::new().width(Length::Percent(100.)).height(Length::Px(160.)).border_width(Edges::all(1.)).border_style(glory::style::BorderStyle::Solid))
                    .semantics(swipe_capture.map(|capture|Some(Box::new(glory::render::Semantics{swipe_capture:*capture,..Default::default()}))))
                    .on(glory::render::EventKind::Swipe,move|event|{if let glory::render::Event::Swipe{phase,revision,..}=event{swipe_log.revise(|s|{if s.len()<4096{s.push_str(&format!("{phase:?}:{revision};"));}});}})
                    .fill(glory::ui::text().content("Raw captured-revision swipe; blank area below is draggable.")),
                sapid::group().theme(theme.clone()).fill((
                    sapid::button().theme(theme.clone()).label("Set vertical swipe configuration").on_press(move||{swipe_revision.revise(|n|*n+=1);swipe_capture.revise(|s|*s=Some(glory::render::SwipeCapture{axis:glory::render::Orientation::Vertical,revision:swipe_revision.get()}));}),
                    sapid::button().theme(theme.clone()).label("Clear swipe configuration").on_press(move||swipe_capture.revise(|s|*s=None)),
                    sapid::button().theme(theme.clone()).label("Restore horizontal swipe configuration").on_press(move||{swipe_revision.revise(|n|*n+=1);swipe_capture.revise(|s|*s=Some(glory::render::SwipeCapture{axis:glory::render::Orientation::Horizontal,revision:swipe_revision.get()}));}),
                )),
                glory::ui::text().class("swipe-protocol-log").content(swipe_log),

            )),
        )).show_in(scope)
    })
}

#[cfg(test)]
#[test]
fn ssr_preserves_bounded_carousel_window_and_typed_swipe_shape() {
    let snapshot = glory::ssr::snapshot(carousel_form(sapid::Theme::blue_light())).unwrap();
    assert_eq!(snapshot.html.matches("carousel-slide-body").count(), 7);
    assert!(snapshot.html.contains("data-gly-swipe-axis=\"horizontal\""));
    assert!(snapshot.html.contains("carousel-fixture-status"));
    assert!(
        snapshot
            .html
            .contains("Current: 5; proposals: 0; factories: 7; cleaned: 0;")
    );
}
