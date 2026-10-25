//! A theme owns its base colors and explicitly supported light/dark overrides.
use super::{ColorScheme, Palette, PaletteOverrides};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeDefinition {
    pub name: &'static str,
    pub base: Palette,
    pub default_scheme: ColorScheme,
    /// None still supports Light when it is the default scheme.
    pub light: Option<PaletteOverrides>,
    /// A non-default mode requires Some, even when its overrides are empty.
    pub dark: Option<PaletteOverrides>,
}
impl ThemeDefinition {
    /// The base palette initially defines a Light-only theme.
    pub const fn new(name: &'static str, base: Palette) -> Self {
        Self {
            name,
            base,
            default_scheme: ColorScheme::Light,
            light: None,
            dark: None,
        }
    }
    pub const fn with_default_scheme(mut self, scheme: ColorScheme) -> Self {
        self.default_scheme = scheme;
        self
    }
    pub const fn with_light(mut self, overrides: PaletteOverrides) -> Self {
        self.light = Some(overrides);
        self
    }
    pub const fn with_dark(mut self, overrides: PaletteOverrides) -> Self {
        self.dark = Some(overrides);
        self
    }
    /// Default mode is always supported; its optional overrides are applied to base.
    pub const fn default(&self) -> (ColorScheme, Palette) {
        let overrides = match self.default_scheme {
            ColorScheme::Light => self.light,
            ColorScheme::Dark => self.dark,
        };
        (self.default_scheme, self.apply(overrides))
    }
    pub const fn light(&self) -> Option<Palette> {
        self.resolve(ColorScheme::Light)
    }
    pub const fn dark(&self) -> Option<Palette> {
        self.resolve(ColorScheme::Dark)
    }
    pub const fn resolve(&self, scheme: ColorScheme) -> Option<Palette> {
        let overrides = match scheme {
            ColorScheme::Light => self.light,
            ColorScheme::Dark => self.dark,
        };
        let is_default = matches!(
            (scheme, self.default_scheme),
            (ColorScheme::Light, ColorScheme::Light) | (ColorScheme::Dark, ColorScheme::Dark)
        );
        if is_default || overrides.is_some() {
            Some(self.apply(overrides))
        } else {
            None
        }
    }
    const fn apply(&self, overrides: Option<PaletteOverrides>) -> Palette {
        match overrides {
            Some(value) => value.apply(self.base),
            None => self.base,
        }
    }
}
