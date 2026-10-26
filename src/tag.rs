//! Shared dimensions for passive pills and accepted editable tags.
use crate::{Size, Theme};
use glory::style::{Align, Display, Edges, FlexDirection, Length, Limit, Style};

pub(crate) fn remove_side(size: Size) -> f64 {
    match size {
        Size::ExtraSmall => 16.0,
        Size::Small => 18.0,
        Size::Medium => 20.0,
        Size::Large => 24.0,
        Size::ExtraLarge => 28.0,
    }
}

pub(crate) fn style(theme: &Theme, size: Size, rounded: bool) -> Style {
    let mut style = theme.label(size);
    style.display = Some(Display::InlineFlex);
    style.flex_direction = Some(FlexDirection::Row);
    style.align = Some(Align::Center);
    style.gap = Some(4.0);
    style.max_width = Some(Limit::Percent(100.0));
    style.background = Some(theme.accent_soft);
    style.color = Some(theme.accent_text);
    style.font_size = Some(
        (theme
            .typography(crate::TypographyRole::Control, size)
            .font_size
            - 2.0)
            .max(0.0),
    );
    style.line_height = Some(remove_side(size));
    style.padding = Some(Edges {
        top: 2.0,
        bottom: 2.0,
        left: 8.0,
        right: 4.0,
    });
    style.radius = Some(if rounded { 999.0 } else { theme.radius });
    style
}

pub(crate) fn remove_style(size: Size) -> Style {
    let side = Length::Px(remove_side(size));
    Style::new()
        .width(side)
        .height(side)
        .min_width(side)
        .min_height(side)
        .padding(Edges::all(0.0))
        .radius(999.0)
}
