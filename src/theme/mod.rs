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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}
impl ColorScheme {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}
mod definition;
mod palette;
mod presets;
pub use definition::ThemeDefinition;
pub use palette::{Palette, PaletteOverrides, Theme};
pub use presets::{BLUE, GREEN};

impl Default for Theme {
    fn default() -> Self {
        Self::from_default(&BLUE).expect("built-in blue default")
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
        Self::new(&BLUE, ColorScheme::Light).expect("built-in blue light")
    }
    pub fn blue_dark() -> Self {
        Self::new(&BLUE, ColorScheme::Dark).expect("built-in blue dark")
    }
    pub fn green_light() -> Self {
        Self::new(&GREEN, ColorScheme::Light).expect("built-in green light")
    }
    pub fn green_dark() -> Self {
        Self::new(&GREEN, ColorScheme::Dark).expect("built-in green dark")
    }

    /// Resolve only the definition's declared data. Unsupported modes are errors.
    /// No default colors or mode-specific color substitutions belong here.
    pub fn new(
        definition: &ThemeDefinition,
        scheme: ColorScheme,
    ) -> Result<Self, glory::render::Error> {
        let palette = definition
            .resolve(scheme)
            .ok_or(glory::render::Error::InvalidProps(
                "theme does not support the requested color scheme",
            ))?;
        Self::from_palette(definition.name, scheme, palette)
    }
    pub fn from_default(definition: &ThemeDefinition) -> Result<Self, glory::render::Error> {
        Self::new(definition, definition.default_scheme)
    }
    pub fn palette(&self) -> &[Color; 10] {
        &self.shades
    }
    pub fn class(&self) -> String {
        format!("sapid-theme-{}-{}", self.name, self.scheme.name())
    }
    /// Install alongside Sapid's structural CSS when using an application palette.
    /// The returned CSS reflects current values, including explicit Theme overrides.
    /// Widgets never inject it into a host themselves.
    pub fn stylesheet(&self) -> Result<String, glory::render::Error> {
        self.validate()?;
        fn css(Color(r, g, b, a): Color) -> String {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
        let mut rule = format!(".{class},.{class}::backdrop{{", class = self.class());
        for name in Palette::COLOR_NAMES {
            let variable = match *name {
                "foreground" => "text".to_owned(),
                "focus_ring" => "ring".to_owned(),
                "accent_hover" => "hover".to_owned(),
                other => other.replace('_', "-"),
            };
            rule.push_str(&format!(
                "--sp-{variable}:{};",
                css(self.colors().color(name).expect("finite palette color"))
            ));
        }
        for (index, color) in self.shades.iter().enumerate() {
            rule.push_str(&format!("--sp-shade-{index}:{};", css(*color)));
        }
        for (index, color) in self.series.iter().enumerate() {
            rule.push_str(&format!("--sp-series-{index}:{};", css(*color)));
        }
        rule.push_str(&format!("color-scheme:{}}}\n", self.scheme.name()));
        Ok(rule)
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
        if self.name.is_empty()
            || self.name.len() > 64
            || !self.name.as_bytes()[0].is_ascii_lowercase()
            || !self
                .name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(glory::render::Error::InvalidProps("invalid palette name"));
        }
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
