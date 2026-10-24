//! Static entry and a small representative hydration matrix for the Rust site.
use std::{error::Error, fs, path::PathBuf};
fn document(html: &str, selection: &str, hue: &str, scheme: &str, ssr: bool) -> String {
    include_str!("../../showcase.html")
        .replace(
            "<!--BASE_CSS-->",
            &format!(
                "<style data-gly-base=\"v1\">{}</style>",
                glory::html::BASE_CSS
            ),
        )
        .replace(
            "<div id=\"app\"></div>",
            &format!("<div id=\"app\">{html}</div>"),
        )
        .replace(
            "name=\"sapid-showcase-render\" content=\"csr\"",
            &format!(
                "name=\"sapid-showcase-render\" content=\"{}\"",
                if ssr { "ssr" } else { "csr" }
            ),
        )
        .replace(
            "name=\"sapid-showcase-selection\" content=\"\"",
            &format!("name=\"sapid-showcase-selection\" content=\"{selection}\""),
        )
        .replace(
            "name=\"sapid-showcase-hue\" content=\"blue\"",
            &format!("name=\"sapid-showcase-hue\" content=\"{hue}\""),
        )
        .replace(
            "name=\"sapid-showcase-scheme\" content=\"light\"",
            &format!("name=\"sapid-showcase-scheme\" content=\"{scheme}\""),
        )
}
fn main() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("expected D-drive output directory")?,
    );
    if !directory.is_absolute()
        || !directory
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with("d:")
    {
        return Err("showcase output must be an absolute D-drive directory".into());
    }
    fs::create_dir_all(&directory)?;
    let csr = document("", "", "blue", "light", false);
    fs::write(directory.join("showcase.html"), &csr)?;
    fs::write(directory.join("index.html"), csr)?;
    // This is a deliberately representative SSR set, not 149 × four renders.
    // All 149 canonical documents remain directly addressable in the CSR shell.
    for (selection, name) in [
        ("", "home"),
        ("P0-01", "button"),
        ("P0-02", "textfield"),
        ("P9-01", "dates"),
        ("P13-01", "chat"),
    ] {
        for (hue_name, hue) in [("blue", sapid::Hue::Blue), ("green", sapid::Hue::Green)] {
            for (scheme_name, scheme) in [
                ("light", sapid::ColorScheme::Light),
                ("dark", sapid::ColorScheme::Dark),
            ] {
                let snapshot = glory::ssr::snapshot(sapid_web_example::showcase::site(
                    sapid::Theme::new(hue, scheme),
                    selection.to_owned(),
                ))?;
                let html = document(&snapshot.html, selection, hue_name, scheme_name, true);
                fs::write(
                    directory.join(format!("ssr-{name}-{hue_name}-{scheme_name}.html")),
                    html,
                )?;
            }
        }
    }
    println!(
        "Showcase: CSR entry + 20 representative SSR pages in {}",
        directory.display()
    );
    Ok(())
}
