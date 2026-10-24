use std::{cell::Cell, rc::Rc};
#[test]
fn unsupported_modal_manager_fails_before_admission_and_keeps_native_app_usable() {
    let host = glory::blitz::shell::WindowApp::new();
    let ready = Rc::new(Cell::new(false));
    let saved = ready.clone();
    assert!(
        host.mount(sapid::modals_manager("unsupported").on_ready(move |_| saved.set(true)))
            .is_err()
    );
    assert!(!ready.get());
    assert!(host.app().tree().is_empty());
    host.mount(sapid::button().label("After unsupported manager"))
        .unwrap();
    host.app().settle(8).unwrap();
    assert!(!host.app().tree().is_empty());
}
