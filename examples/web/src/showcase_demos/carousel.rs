use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::carousel_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
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
                    .autoplay(||Box::pin(super::menu_form::local_delay(180)))
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
}
#[allow(dead_code)]
mod menu_form {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn local_delay(_millis: i32) {}
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn local_delay(millis: i32) -> impl std::future::Future<Output = ()> {
        browser::Delay::new(millis)
    }
    #[cfg(target_arch = "wasm32")]
    mod browser {
        use std::cell::{Cell, RefCell};
        use std::future::Future;
        use std::pin::Pin;
        use std::rc::Rc;
        use std::task::{Context, Poll, Waker};
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;
        struct TimerState {
            done: Cell<bool>,
            waker: RefCell<Option<Waker>>,
        }
        pub(super) struct Delay {
            millis: i32,
            state: Rc<TimerState>,
            timer: Option<i32>,
            callback: Option<Closure<dyn FnMut()>>,
        }
        impl Delay {
            pub fn new(millis: i32) -> Self {
                Self {
                    millis,
                    state: Rc::new(TimerState {
                        done: Cell::new(false),
                        waker: RefCell::new(None),
                    }),
                    timer: None,
                    callback: None,
                }
            }
        }
        impl Future for Delay {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.state.done.get() {
                    return Poll::Ready(());
                }
                let previous = self.state.waker.borrow_mut().replace(cx.waker().clone());
                drop(previous);
                if self.timer.is_none() {
                    let state = self.state.clone();
                    let callback = Closure::new(move || {
                        state.done.set(true);
                        let waker = state.waker.borrow_mut().take();
                        if let Some(waker) = waker {
                            waker.wake();
                        }
                    });
                    self.timer = Some(
                        web_sys::window()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                callback.as_ref().unchecked_ref(),
                                self.millis,
                            )
                            .unwrap(),
                    );
                    self.callback = Some(callback);
                }
                Poll::Pending
            }
        }
        impl Drop for Delay {
            fn drop(&mut self) {
                if let Some(timer) = self.timer.take()
                    && let Some(window) = web_sys::window()
                {
                    window.clear_timeout_with_handle(timer);
                }
            }
        }
    }
}
