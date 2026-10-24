//! Semantic color tokens inspired by Mantine/Open Color. Themes are mount-time
//! configuration; all colors remain portable Glory Style values.
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
    fn metrics(self) -> (f64, f64, f64, f64) {
        match self {
            Self::ExtraSmall => (24.0, 11.0, 2.0, 8.0),
            Self::Small => (30.0, 13.0, 4.0, 10.0),
            Self::Medium => (36.0, 14.0, 6.0, 12.0),
            Self::Large => (42.0, 16.0, 8.0, 14.0),
            Self::ExtraLarge => (50.0, 18.0, 8.0, 20.0),
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
    pub(crate) fn control(&self, size: Size) -> Style {
        let (height, font_size, vertical, horizontal) = size.metrics();
        Style {
            font_size: Some(font_size),
            line_height: Some(font_size * 1.55),
            color: Some(self.foreground),
            background: Some(self.input),
            ..Style::new()
                .min_height(Length::Px(height))
                .padding(Edges {
                    top: vertical,
                    bottom: vertical,
                    left: horizontal,
                    right: horizontal,
                })
                .radius(self.radius)
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
        Style {
            color: Some(self.foreground),
            font_size: Some(size.metrics().1),
            line_height: Some(size.metrics().1 * 1.55),
            font_weight: Some(500.0),
            ..Style::new().font_family([Family::System])
        }
    }
    pub(crate) fn description(&self, size: Size) -> Style {
        Style {
            color: Some(self.muted),
            font_size: Some(size.metrics().1 - 1.0),
            line_height: Some((size.metrics().1 - 1.0) * 1.5),
            ..Style::new().font_family([Family::System])
        }
    }
}
