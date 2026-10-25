//! Share the build's immutable media URLs between CSR and SSR.
//! Fixture builds without the deployment environment use the ordinary assets directory.
pub fn media(name: &str) -> String {
    format!(
        "{}/{name}",
        option_env!("SAPID_SHOWCASE_MEDIA").unwrap_or("/assets")
    )
}
