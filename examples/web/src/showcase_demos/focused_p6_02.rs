use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    focused::demo_for("P6-02", theme)
}

#[allow(dead_code, unused_imports)]
mod focused {
    use glory::prelude::*;
    use sapid::Theme;

    pub fn demo_for(id: &str, theme: Theme) -> Box<dyn Widget> {
        match id {
            "P6-01" => Box::new(
                sapid::breadcrumbs("focused-trail")
                    .theme(theme.clone())
                    .current("article")
                    .items([
                        sapid::breadcrumb_item("home", "Home").href("#home"),
                        sapid::breadcrumb_item("guides", "Guides").href("#guides"),
                        sapid::breadcrumb_item("rust", "Rust").href("#rust"),
                        sapid::breadcrumb_item("article", "Semantic widgets"),
                    ]),
            ),
            "P6-02" => Box::new(widget(move |scope| {
                let page = scope.cage(3_u64);
                sapid::pagination("focused-pages")
                    .theme(theme.clone())
                    .name("Example pages")
                    .page(page)
                    .total(Some(8))
                    .on_page_change(move |next| page.revise(|current| *current = next))
                    .show_in(scope)
            })),
            _ => unreachable!("unsupported navigation demo"),
        }
    }
}
