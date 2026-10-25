//! Static entry and a small representative hydration matrix for the Rust site.
use std::{error::Error, fs, path::PathBuf};
fn document(
    html: &str,
    selection: &str,
    hue: &str,
    scheme: &str,
    ssr: bool,
    assets: &str,
) -> String {
    let theme = sapid_web_example::themes::select(
        hue,
        if scheme == "dark" {
            sapid::ColorScheme::Dark
        } else {
            sapid::ColorScheme::Light
        },
    );
    include_str!("../../showcase.html")
        .replace(
            "assets/sapid-logo.svg",
            &sapid_web_example::assets::media("sapid-logo.svg"),
        )
        .replace(
            "<body>",
            &format!(
                "<body style=\"background:{};color:{};color-scheme:{}\">",
                sapid_web_example::theme_studio::model::hex(theme.canvas),
                sapid_web_example::theme_studio::model::hex(theme.foreground),
                theme.scheme.name()
            ),
        )
        .replace(
            "href=\"sapid.css\"",
            &format!("href=\"{assets}/sapid.css\""),
        )
        .replace("href=\"site.css\"", &format!("href=\"{assets}/site.css\""))
        .replace(
            "'./sapid_web_example.js'",
            &format!("'./{assets}/sapid_web_example.js'"),
        )
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
    let assets = std::env::args()
        .nth(2)
        .ok_or("expected immutable asset directory")?;
    if !assets.starts_with("assets-")
        || !assets
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid asset directory".into());
    }
    let csr = document("", "", "blue", "light", false, &assets);
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
        for hue_name in ["blue", "green"] {
            for (scheme_name, scheme) in [
                ("light", sapid::ColorScheme::Light),
                ("dark", sapid::ColorScheme::Dark),
            ] {
                let snapshot = glory::ssr::snapshot(sapid_web_example::showcase::site(
                    sapid_web_example::themes::select(hue_name, scheme),
                    selection.to_owned(),
                ))?;
                let html = document(
                    &snapshot.html,
                    selection,
                    hue_name,
                    scheme_name,
                    true,
                    &assets,
                );
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
