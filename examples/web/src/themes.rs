//! The showcase offers the light/dark modes of two theme definitions.
use sapid::{BLUE, ColorScheme, GREEN, Theme};
pub fn select(name: &str, scheme: ColorScheme) -> Theme {
    let definition = if name == "green" { &GREEN } else { &BLUE };
    Theme::new(definition, scheme).expect("showcase definitions support both schemes")
}
