//! Semantic color tokens inspired by Mantine/Open Color. Themes are mount-time
//! configuration; all colors remain portable Glory Style values.
use crate::theme_tokens::{SIZES, ThemeTokens, TypographyMetrics, TypographyRole};
use glory::style::{BorderStyle, Color, Edges, Family, Length, Style};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}
impl Size {
    pub(crate) fn class(self) -> &'static str {
        match self {
            Self::ExtraSmall => "sapid-size-extra-small",
            Self::Small => "sapid-size-small",
            Self::Medium => "sapid-size-medium",
            Self::Large => "sapid-size-large",
            Self::ExtraLarge => "sapid-size-extra-large",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hue {
    #[default]
    Blue,
    Green,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}
const fn rgb(hex: u32) -> Color {
    Color((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255)
}
pub const BLUE: [Color; 10] = [
    rgb(0xe7f5ff),
    rgb(0xd0ebff),
    rgb(0xa5d8ff),
    rgb(0x74c0fc),
    rgb(0x4dabf7),
    rgb(0x339af0),
    rgb(0x228be6),
    rgb(0x1c7ed6),
    rgb(0x1971c2),
    rgb(0x1864ab),
];
pub const GREEN: [Color; 10] = [
    rgb(0xebfbee),
    rgb(0xd3f9d8),
    rgb(0xb2f2bb),
    rgb(0x8ce99a),
    rgb(0x69db7c),
    rgb(0x51cf66),
    rgb(0x40c057),
    rgb(0x37b24d),
    rgb(0x2f9e44),
    rgb(0x2b8a3e),
];
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub hue: Hue,
    pub scheme: ColorScheme,
    pub canvas: Color,
    pub surface: Color,
    pub input: Color,
    pub foreground: Color,
    pub muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_soft: Color,
    pub accent_text: Color,
    pub on_accent: Color,
    pub danger: Color,
    pub danger_fill: Color,
    pub on_danger: Color,
    pub danger_soft: Color,
    pub border: Color,
    pub disabled: Color,
    pub disabled_text: Color,
    pub radius: f64,
    pub tokens: ThemeTokens,
}
impl Default for Theme {
    fn default() -> Self {
        Self::blue_light()
    }
}
impl Theme {
    pub fn light() -> Self {
        Self::blue_light()
    }
    pub fn dark() -> Self {
        Self::blue_dark()
    }
    pub fn blue_light() -> Self {
        Self::new(Hue::Blue, ColorScheme::Light)
    }
    pub fn blue_dark() -> Self {
        Self::new(Hue::Blue, ColorScheme::Dark)
    }
    pub fn green_light() -> Self {
        Self::new(Hue::Green, ColorScheme::Light)
    }
    pub fn green_dark() -> Self {
        Self::new(Hue::Green, ColorScheme::Dark)
    }
    pub fn new(hue: Hue, scheme: ColorScheme) -> Self {
        let shades = match hue {
            Hue::Blue => BLUE,
            Hue::Green => GREEN,
        };
        let accent = match hue {
            Hue::Blue => shades[8],
            Hue::Green => rgb(0x237032),
        };
        let mut theme = Self {
            hue,
            scheme,
            canvas: rgb(0xf8f9fa),
            surface: rgb(0xffffff),
            input: rgb(0xffffff),
            foreground: rgb(0x212529),
            muted: rgb(0x5c6470),
            accent,
            accent_hover: match hue {
                Hue::Blue => shades[9],
                Hue::Green => rgb(0x1b5e2a),
            },
            accent_soft: shades[0],
            accent_text: match hue {
                Hue::Blue => shades[9],
                Hue::Green => rgb(0x237032),
            },
            on_accent: rgb(0xffffff),
            danger: rgb(0xc92a2a),
            danger_fill: rgb(0xc92a2a),
            on_danger: rgb(0xffffff),
            danger_soft: rgb(0xfff5f5),
            border: rgb(0xced4da),
            disabled: rgb(0xf1f3f5),
            disabled_text: rgb(0x868e96),
            radius: 4.0,
            tokens: ThemeTokens::default(),
        };
        if scheme == ColorScheme::Dark {
            theme.canvas = rgb(0x141517);
            theme.surface = rgb(0x1a1b1e);
            theme.input = rgb(0x25262b);
            theme.foreground = rgb(0xdee2e6);
            theme.muted = rgb(0xadb5bd);
            theme.border = rgb(0x495057);
            theme.disabled = rgb(0x25262b);
            theme.disabled_text = rgb(0x909296);
            theme.accent_text = shades[3];
            theme.accent_soft = match hue {
                Hue::Blue => rgb(0x142c41),
                Hue::Green => rgb(0x173322),
            };
            theme.danger = rgb(0xffa8a8);
            theme.danger_soft = rgb(0x3d2023);
        }
        theme
    }
    pub fn palette(&self) -> &'static [Color; 10] {
        match self.hue {
            Hue::Blue => &BLUE,
            Hue::Green => &GREEN,
        }
    }
    pub fn class(&self) -> &'static str {
        match (self.hue, self.scheme) {
            (Hue::Blue, ColorScheme::Light) => "sapid-theme-blue-light",
            (Hue::Blue, ColorScheme::Dark) => "sapid-theme-blue-dark",
            (Hue::Green, ColorScheme::Light) => "sapid-theme-green-light",
            (Hue::Green, ColorScheme::Dark) => "sapid-theme-green-dark",
        }
    }
    pub fn with_tokens(mut self, tokens: ThemeTokens) -> Self {
        self.tokens = tokens;
        self
    }
    pub fn spacing(&self, size: Size) -> f64 {
        self.tokens.spacing(size)
    }
    pub fn typography(&self, role: TypographyRole, size: Size) -> TypographyMetrics {
        self.tokens.typography(role, size)
    }
    pub fn validate(&self) -> Result<(), glory::render::Error> {
        if !self.radius.is_finite() || self.radius < 0. {
            return Err(glory::render::Error::InvalidProps("invalid theme radius"));
        }
        for size in SIZES {
            self.control(size)
                .validate()
                .map_err(glory::render::Error::InvalidProps)?;
        }
        Ok(())
    }
    pub(crate) fn control(&self, size: Size) -> Style {
        let control = self.tokens.control(size);
        let text = self.tokens.typography(TypographyRole::Control, size);
        Style {
            font_size: Some(text.font_size),
            line_height: Some(text.line_height),
            font_weight: text.weight,
            color: Some(self.foreground),
            background: Some(self.input),
            ..Style::new()
                .min_height(Length::Px(control.min_height))
                .padding(Edges {
                    top: control.padding_y,
                    bottom: control.padding_y,
                    left: control.padding_x,
                    right: control.padding_x,
                })
                .radius(self.radius * control.radius_scale)
                .border_width(Edges::all(1.0))
                .border_style(BorderStyle::Solid)
                .border_color(self.border)
                .font_family([Family::System])
        }
    }
    pub(crate) fn checkbox(&self, size: Size) -> Style {
        self.label(size)
    }
    pub(crate) fn label(&self, size: Size) -> Style {
        let text = self.tokens.typography(TypographyRole::Label, size);
        Style {
            color: Some(self.foreground),
            font_size: Some(text.font_size),
            line_height: Some(text.line_height),
            font_weight: text.weight,
            ..Style::new().font_family([Family::System])
        }
    }
    pub(crate) fn description(&self, size: Size) -> Style {
        let text = self.tokens.typography(TypographyRole::Description, size);
        Style {
            color: Some(self.muted),
            font_size: Some(text.font_size),
            line_height: Some(text.line_height),
            font_weight: text.weight,
            ..Style::new().font_family([Family::System])
        }
    }
}
