fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory =
        std::path::PathBuf::from(std::env::args().nth(1).ok_or("output directory")?);
    let snapshot =
        glory::ssr::snapshot(sapid_web_example::dsl::fixture::demo(sapid::Theme::light()))?;
    let template = include_str!("../../dsl.html").replace(
        "<!--BASE_CSS-->",
        &format!(
            "<style data-gly-base=\"v1\">{}</style>",
            glory::html::BASE_CSS
        ),
    );
    std::fs::write(directory.join("dsl.html"), &template)?;
    std::fs::write(
        directory.join("dsl-hydrate.html"),
        template.replace(
            "<div id=\"app\"></div>",
            &format!("<div id=\"app\">{}</div>", snapshot.html),
        ),
    )?;
    Ok(())
}
