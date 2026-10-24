use glory::prelude::*;

fn column() -> glory::ui::View {
    col().style(
        Style::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(16.0),
    )
}

pub fn demo(theme: sapid::Theme) -> impl Widget {
    column().fill((
        sapid::title(3).theme(theme.clone()).content("A complete widget catalogue"),
        sapid::text().theme(theme).content(format!("Explore {} documented work items from the navigation. Each preview is a real Glory Widget.", 149usize)),
    ))
}
