use glory::routing::{Bootstrap, Location};
use sapid_web_example::showcase_navigation::{self as navigation, Route, SSR_PAGES};

#[test]
fn query_targets_round_trip_through_glory_without_losing_unicode_or_fragment_encoding() {
    let mut route = Route::new("P0-01".into(), &sapid::Theme::green_dark());
    route.palette = "custom-中文 & preview".into();
    route.return_to = "P0-02".into();
    route.edit = Some("custom-中文 & preview".into());
    route.fragment = "composition/中文".into();
    let location = Location::parse(&format!("/{}", route.url())).unwrap();
    assert_eq!(Route::from_location(&location), route);
    let routes = navigation::routes();
    let seed = Bootstrap::new(&routes, location.clone(), navigation::SCHEMA).unwrap();
    assert_eq!(
        seed.validate(&routes, navigation::SCHEMA).unwrap(),
        location
    );
}

#[test]
fn ssr_aliases_and_query_overrides_share_the_browser_decoder() {
    for &(widget, name) in SSR_PAGES {
        for palette in ["blue", "green"] {
            for scheme in ["light", "dark"] {
                let location =
                    Location::parse(&format!("/ssr-{name}-{palette}-{scheme}.html")).unwrap();
                let route = Route::from_location(&location);
                assert_eq!(
                    (&*route.widget, &*route.palette, &*route.scheme),
                    (widget, palette, scheme)
                );
            }
        }
    }
    let location = Location::parse(
        "/ssr-button-blue-dark.html?widget=P0-02&theme=green&scheme=light&unknown=kept#usage",
    )
    .unwrap();
    let route = Route::from_location(&location);
    assert_eq!(
        (
            &*route.widget,
            &*route.palette,
            &*route.scheme,
            &*route.fragment
        ),
        ("P0-02", "green", "light", "usage")
    );
    assert_eq!(
        location.query().required::<String>("unknown").unwrap(),
        "kept"
    );
}

#[test]
fn canonical_navigation_keeps_defaults_and_unknown_query_pairs() {
    let requested =
        Location::parse("/showcase.html?widget=theme-studio&foreign=first&foreign=second#usage")
            .unwrap();
    let route = Route::from_location(&requested);
    let canonical = route.canonical_location(&requested);
    assert_eq!(
        canonical.to_string(),
        "/showcase.html?widget=theme-studio&theme=blue&scheme=light&foreign=first&foreign=second#usage"
    );
    let mut fallback = route;
    fallback.scheme = "dark".into();
    assert_eq!(
        fallback
            .canonical_location(&requested)
            .query()
            .required::<String>("scheme")
            .unwrap(),
        "dark"
    );
}
