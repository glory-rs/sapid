//! Regenerate only the data-driven preset rules in the shared stylesheet.
use std::{error::Error, fs, path::Path};
fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("styles/sapid.css");
    let old = fs::read_to_string(&path)?;
    let start = old
        .find("/* BEGIN SAPID PALETTES */")
        .ok_or("missing palette start")?;
    let end = old
        .find("/* END SAPID PALETTES */")
        .ok_or("missing palette end")?
        + "/* END SAPID PALETTES */".len();
    let mut rules = String::from("/* BEGIN SAPID PALETTES */\n");
    for definition in [&sapid::BLUE, &sapid::GREEN] {
        for scheme in [sapid::ColorScheme::Light, sapid::ColorScheme::Dark] {
            if definition.resolve(scheme).is_some() {
                rules.push_str(&sapid::Theme::new(definition, scheme)?.stylesheet()?);
            }
        }
    }
    rules.push_str("/* END SAPID PALETTES */");
    let new = format!("{}{}{}", &old[..start], rules, &old[end..]);
    if std::env::args().any(|arg| arg == "--check") {
        if old != new {
            return Err("stale palette CSS; run cargo run --example palette_css".into());
        }
    } else {
        fs::write(path, new)?;
    }
    Ok(())
}
