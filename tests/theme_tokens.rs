use glory::render::Props;
use glory::ui::App;
use sapid::{ControlMetrics, Size, Theme, ThemeTokens, TypographyMetrics, TypographyRole};

#[test]
fn control_label_body_and_heading_styles_consume_the_declared_tokens() {
    let tokens = ThemeTokens::default()
        .with_control(
            Size::Medium,
            ControlMetrics {
                min_height: 57.,
                padding_x: 23.,
                padding_y: 9.,
                radius_scale: 2.,
            },
        )
        .unwrap()
        .with_typography(
            TypographyRole::Control,
            Size::Medium,
            TypographyMetrics {
                font_size: 19.,
                line_height: 27.,
                weight: Some(450.),
            },
        )
        .unwrap()
        .with_typography(
            TypographyRole::Label,
            Size::Medium,
            TypographyMetrics {
                font_size: 18.,
                line_height: 26.,
                weight: Some(550.),
            },
        )
        .unwrap()
        .with_typography(
            TypographyRole::Body,
            Size::Medium,
            TypographyMetrics {
                font_size: 23.,
                line_height: 31.,
                weight: Some(450.),
            },
        )
        .unwrap()
        .with_typography(
            TypographyRole::Heading2,
            Size::Medium,
            TypographyMetrics {
                font_size: 38.,
                line_height: 44.,
                weight: Some(650.),
            },
        )
        .unwrap();
    let theme = Theme::blue_light().with_tokens(tokens);
    theme.validate().unwrap();
    let app = App::memory();
    app.mount(glory::ui::group().fill((
        sapid::button().theme(theme.clone()).label("Token button"),
        sapid::text_field().theme(theme.clone()).name("Token input"),
        sapid::checkbox().theme(theme.clone()).label("Token check"),
        sapid::text().theme(theme.clone()).content("Token body"),
        sapid::title(2).theme(theme).content("Token heading"),
    )))
    .unwrap();
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut seen = [false; 5];
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        let descriptor = node.descriptor.view();
        let style = descriptor.style;
        match descriptor.props {
            Props::Button(button) if button.label == "Token button" => {
                assert_eq!(style.min_height, Some(glory::style::Length::Px(57.)));
                assert_eq!(
                    style.padding,
                    Some(glory::style::Edges {
                        top: 9.,
                        bottom: 9.,
                        left: 23.,
                        right: 23.
                    })
                );
                assert_eq!(style.radius, Some(8.));
                assert_eq!(style.font_size, Some(19.));
                assert_eq!(style.font_weight, Some(450.));
                seen[0] = true;
            }
            Props::Input(_) => {
                assert_eq!(style.font_size, Some(19.));
                assert_eq!(style.radius, Some(8.));
                seen[1] = true;
            }
            Props::Check(_) => {
                assert_eq!(style.font_size, Some(18.));
                assert_eq!(style.font_weight, Some(550.));
                seen[2] = true;
            }
            Props::Text(text) if text.content == "Token body" => {
                assert_eq!(style.font_size, Some(23.));
                assert_eq!(style.line_height, Some(31.));
                seen[3] = true;
            }
            Props::Text(text) if text.content == "Token heading" => {
                assert_eq!(style.font_size, Some(38.));
                assert_eq!(style.line_height, Some(44.));
                seen[4] = true;
            }
            _ => {}
        }
        pending.extend(node.children.iter().copied());
    }
    assert_eq!(seen, [true; 5]);
}

#[test]
fn default_token_values_preserve_existing_size_metrics_and_share_the_immutable_table() {
    let theme = Theme::blue_light();
    for (index, size) in [
        Size::ExtraSmall,
        Size::Small,
        Size::Medium,
        Size::Large,
        Size::ExtraLarge,
    ]
    .into_iter()
    .enumerate()
    {
        let control = theme.tokens.control(size);
        assert_eq!(control.min_height, [24., 30., 36., 42., 50.][index]);
        assert_eq!(control.radius_scale, 1.);
        assert_eq!(
            theme.typography(TypographyRole::Control, size).font_size,
            [11., 13., 14., 16., 20.][index]
        );
        assert_eq!(theme.typography(TypographyRole::Control, size).weight, None);
    }
    assert_eq!(
        theme
            .typography(TypographyRole::Body, Size::Medium)
            .font_size,
        16.
    );
    assert_eq!(
        theme
            .typography(TypographyRole::Heading1, Size::Medium)
            .font_size,
        32.
    );
    let customized = theme.clone().with_tokens(
        theme
            .tokens
            .clone()
            .with_spacing(Size::Medium, 33.)
            .unwrap(),
    );
    assert_eq!(theme.spacing(Size::Medium), 12.);
    assert_eq!(customized.spacing(Size::Medium), 33.);
}
