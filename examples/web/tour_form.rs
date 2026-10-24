use glory::prelude::*;
use sapid::{Theme, TourHandle, tour, tour_context, tour_step, tour_target};
pub fn tour_form(theme: Theme) -> impl Widget {
    component(move |scope| {
        let ctx = tour_context(scope, "example-tour")?;
        let open = scope.cage(false);
        let current = scope.cage(0usize);
        let present = scope.cage(true);
        let rejected = scope.cage(false);
        let close_rejected = scope.cage(false);
        let changes = scope.cage(0usize);
        let losses = scope.cage(0usize);
        let completions = scope.cage(0usize);
        let handle = scope.cage(None::<TourHandle>);
        let draft = scope.cage("Retain my draft".to_string());
        let target_context = ctx.clone();
        let target_theme = theme.clone();
        view().fill((
            sapid::button().theme(theme.clone()).label("Start tour").on_press(move || { current.revise(|v| *v=0); open.revise(|v| *v=true); }),
            sapid::button().theme(theme.clone()).label("Go to second target").on_press(move || { current.revise(|v| *v=1); open.revise(|v| *v=true); }),
            sapid::button().theme(theme.clone()).label("Toggle target presence").on_press(move || present.revise(|v| *v= !*v)),
            sapid::button().theme(theme.clone()).label("Toggle step refusal").on_press(move || rejected.revise(|v| *v= !*v)),
            sapid::button().theme(theme.clone()).label("Toggle close refusal").on_press(move || close_rejected.revise(|v| *v= !*v)),
            sapid::button().theme(theme.clone()).label("Refresh target").on_press(move || { if let Some(handle) = handle.get() { let _ = handle.refresh(); } }),
            text().class("tour-status").content(scope.bond(move || format!("step={} open={} changes={} losses={} completed={} reject={} close-reject={}", current.get(), open.get(), changes.get(), losses.get(), completions.get(), rejected.get(), close_rejected.get()))),
            tour_target(&ctx, "first").fill(sapid::text_field().theme(theme.clone()).name("Tour draft").value(draft).on_value_change(move |v| draft.revise(|draft| *draft=v))),
            view().style(Style::new().height(Length::Px(1150.))).fill(text().content("The second target is below the fold.")),
            when(present, move || tour_target(&target_context, "second").fill(sapid::button().theme(target_theme.clone()).label("Second real target"))).otherwise(|| text().content("Second target retired")),
            tour(&ctx, "guide").theme(theme.clone()).current(current).open(open)
                .steps([tour_step("first", "First step", "Keep your draft while the guide moves."), tour_step("second", "Second step", "This target was physically revealed.")])
                .on_current_change(move |next| { changes.revise(|v| *v+=1); if !rejected.get() { current.revise(|v| *v=next); } })
                .on_open_change(move |next| { if !close_rejected.get() { open.revise(|v| *v=next); } })
                .on_target_lost(move |_| losses.revise(|v| *v+=1))
                .on_complete(move || completions.revise(|v| *v+=1))
                .on_ready(move |value| handle.revise(|v| *v=Some(value))),
        )).show_in(scope)
    })
}
