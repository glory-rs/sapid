//! Bounded, host-independent studio data. Browser storage lives in the host module.
use glory::style::Color;
use sapid::{ColorScheme, Palette, PaletteOverrides, Theme, ThemeDefinition};
const ACCENT_ROLES: [&str; 7] = [
    "accent",
    "accent_hover",
    "accent_soft",
    "accent_text",
    "on_accent",
    "focus_ring",
    "input_focus",
];
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_THEMES: usize = 8;
pub const MAX_STORAGE_BYTES: usize = 131_072;
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeRecord {
    pub id: String,
    pub title: String,
    pub base: Palette,
    pub light: Option<PaletteOverrides>,
    pub dark: Option<PaletteOverrides>,
    pub default_scheme: ColorScheme,
}
impl ThemeRecord {
    pub fn from_definition(def: &ThemeDefinition) -> Self {
        Self {
            id: String::new(),
            title: format!("My {}", def.name),
            base: def.base,
            light: def.light,
            dark: def.dark,
            default_scheme: def.default_scheme,
        }
    }
    pub fn resolve(&self, scheme: ColorScheme) -> Option<Palette> {
        let mut d =
            ThemeDefinition::new("stored", self.base).with_default_scheme(self.default_scheme);
        d.light = self.light;
        d.dark = self.dark;
        d.resolve(scheme)
    }
    pub fn supports(&self, scheme: ColorScheme) -> bool {
        self.resolve(scheme).is_some()
    }
    pub fn theme(&self, scheme: ColorScheme) -> Result<Theme, String> {
        let p = self
            .resolve(scheme)
            .ok_or("Theme does not support this mode")?;
        Theme::from_palette(
            if self.id.is_empty() {
                "studio-preview".to_owned()
            } else {
                self.id.clone()
            },
            scheme,
            p,
        )
        .map_err(|e| e.to_string())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty()
            || self.title.chars().count() > 40
            || self.title.contains(['\n', '\r', '\0'])
        {
            return Err("Use a theme name of 1–40 characters".into());
        }
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            if self.supports(scheme) {
                self.theme(scheme)?;
            }
        }
        Ok(())
    }
    pub fn set_default(&mut self, scheme: ColorScheme) {
        let old = self.default_scheme;
        if old != scheme {
            let slot = match old {
                ColorScheme::Light => &mut self.light,
                ColorScheme::Dark => &mut self.dark,
            };
            if slot.is_none() {
                *slot = Some(PaletteOverrides::NONE)
            }
            self.default_scheme = scheme
        }
    }
    pub fn set_support(&mut self, scheme: ColorScheme, enabled: bool) {
        if scheme == self.default_scheme {
            return;
        }
        let slot = match scheme {
            ColorScheme::Light => &mut self.light,
            ColorScheme::Dark => &mut self.dark,
        };
        *slot = if enabled {
            Some(slot.unwrap_or(PaletteOverrides::NONE))
        } else {
            None
        }
    }
    pub fn code(&self) -> String {
        let title = self
            .title
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let symbol = if !title.bytes().any(|b| b.is_ascii_alphanumeric()) {
            "MY_THEME".to_owned()
        } else if title.starts_with(|c: char| c.is_ascii_digit()) {
            format!("THEME_{title}")
        } else {
            title
        };
        let id = if self.id.is_empty() {
            "my-theme"
        } else {
            &self.id
        };
        let mut imports = vec!["Palette", "ThemeDefinition"];
        if self.light.is_some() || self.dark.is_some() {
            imports.push("PaletteOverrides");
        }
        if self.default_scheme == ColorScheme::Dark {
            imports.push("ColorScheme");
        }
        let imports = imports.join(", ");
        let mut out = format!(
            "use glory::style::Color;\nuse sapid::{{{imports}}};\n\npub const {symbol}: ThemeDefinition = ThemeDefinition::new(\n    \"{id}\",\n    Palette {{\n        shades: {},\n        series: {},\n",
            rust_array(&self.base.shades),
            rust_array(&self.base.series)
        );
        for name in Palette::COLOR_NAMES {
            out.push_str(&format!(
                "        {name}: {},\n",
                rust_color(self.base.color(name).unwrap())
            ))
        }
        out.push_str("    },\n)");
        for (mode, ov) in [("light", self.light), ("dark", self.dark)] {
            if let Some(ov) = ov {
                out.push_str(&format!(".with_{mode}(PaletteOverrides {{\n"));
                if ov.shades.iter().any(Option::is_some) {
                    out.push_str(&format!("    shades: {},\n", rust_options(&ov.shades)))
                }
                if ov.series.iter().any(Option::is_some) {
                    out.push_str(&format!("    series: {},\n", rust_options(&ov.series)))
                }
                for name in Palette::COLOR_NAMES {
                    if let Some(c) = ov.color(name) {
                        out.push_str(&format!("    {name}: Some({}),\n", rust_color(c)))
                    }
                }
                out.push_str("    ..PaletteOverrides::NONE\n})");
            }
        }
        if self.default_scheme == ColorScheme::Dark {
            out.push_str(".with_default_scheme(ColorScheme::Dark)")
        }
        out.push_str(";\n");
        out
    }
}
fn rust_color(Color(r, g, b, a): Color) -> String {
    format!("Color({r}, {g}, {b}, {a})")
}
fn rust_array<const N: usize>(values: &[Color; N]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|c| rust_color(*c))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn rust_options<const N: usize>(values: &[Option<Color>; N]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|c| c
                .map(|c| format!("Some({})", rust_color(c)))
                .unwrap_or_else(|| "None".into()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
pub fn hex(Color(r, g, b, a): Color) -> String {
    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
}
pub fn parse_hex(text: &str) -> Result<Color, String> {
    let s = text.strip_prefix('#').unwrap_or(text);
    if !matches!(s.len(), 6 | 8) || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Use #RRGGBB or #RRGGBBAA".into());
    }
    let channel = |i| u8::from_str_radix(&s[i..i + 2], 16).unwrap();
    Ok(Color(
        channel(0),
        channel(2),
        channel(4),
        if s.len() == 8 { channel(6) } else { 255 },
    ))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    Base,
    Light,
    Dark,
}
impl Layer {
    pub fn name(self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
    pub fn scheme(self) -> Option<ColorScheme> {
        match self {
            Self::Base => None,
            Self::Light => Some(ColorScheme::Light),
            Self::Dark => Some(ColorScheme::Dark),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    Role(usize),
    Shade(usize),
    Series(usize),
}
impl Target {
    pub fn all() -> Vec<Self> {
        (0..10)
            .map(Self::Shade)
            .chain((0..Palette::COLOR_NAMES.len()).map(Self::Role))
            .chain((0..8).map(Self::Series))
            .collect()
    }
    pub fn name(self) -> String {
        match self {
            Self::Role(i) => Palette::COLOR_NAMES[i].into(),
            Self::Shade(i) => format!("shade {i}"),
            Self::Series(i) => format!("series {i}"),
        }
    }
    pub fn color(self, p: &Palette) -> Color {
        match self {
            Self::Role(i) => p.color(Palette::COLOR_NAMES[i]).unwrap(),
            Self::Shade(i) => p.shades[i],
            Self::Series(i) => p.series[i],
        }
    }
    fn set(self, p: &mut Palette, c: Color) {
        match self {
            Self::Role(i) => {
                p.set_color(Palette::COLOR_NAMES[i], c);
            }
            Self::Shade(i) => p.shades[i] = c,
            Self::Series(i) => p.series[i] = c,
        }
    }
    fn override_color(self, p: &PaletteOverrides) -> Option<Color> {
        match self {
            Self::Role(i) => p.color(Palette::COLOR_NAMES[i]),
            Self::Shade(i) => p.shades[i],
            Self::Series(i) => p.series[i],
        }
    }
    fn set_override(self, p: &mut PaletteOverrides, c: Option<Color>) {
        match self {
            Self::Role(i) => {
                p.set_color(Palette::COLOR_NAMES[i], c);
            }
            Self::Shade(i) => p.shades[i] = c,
            Self::Series(i) => p.series[i] = c,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Draft {
    pub record: ThemeRecord,
    pub seed: String,
    pub epoch: u64,
    texts: BTreeMap<(Layer, Target), String>,
}
impl Draft {
    pub fn new(record: ThemeRecord) -> Self {
        Self {
            seed: hex(record.base.shades[6]),
            record,
            epoch: 0,
            texts: BTreeMap::new(),
        }
    }
    pub fn color(&self, layer: Layer, target: Target) -> Color {
        target.color(
            &layer
                .scheme()
                .and_then(|s| self.record.resolve(s))
                .unwrap_or(self.record.base),
        )
    }
    pub fn text(&self, layer: Layer, target: Target) -> String {
        self.texts
            .get(&(layer, target))
            .cloned()
            .unwrap_or_else(|| hex(self.color(layer, target)))
    }
    pub fn overridden(&self, layer: Layer, target: Target) -> bool {
        match layer {
            Layer::Base => true,
            Layer::Light => self
                .record
                .light
                .as_ref()
                .and_then(|o| target.override_color(o))
                .is_some(),
            Layer::Dark => self
                .record
                .dark
                .as_ref()
                .and_then(|o| target.override_color(o))
                .is_some(),
        }
    }
    pub fn set_text(&mut self, layer: Layer, target: Target, text: String) {
        let text = text.chars().take(32).collect::<String>();
        let color = parse_hex(&text);
        self.texts.insert((layer, target), text);
        if let Ok(c) = color {
            self.set_color(layer, target, c, false)
        }
    }
    pub fn set_color(&mut self, layer: Layer, target: Target, c: Color, canonical: bool) {
        match layer {
            Layer::Base => target.set(&mut self.record.base, c),
            Layer::Light | Layer::Dark => {
                let slot = if layer == Layer::Light {
                    &mut self.record.light
                } else {
                    &mut self.record.dark
                };
                target.set_override(slot.get_or_insert(PaletteOverrides::NONE), Some(c))
            }
        }
        if canonical {
            self.texts.insert((layer, target), hex(c));
        }
        self.epoch = self.epoch.saturating_add(1)
    }
    pub fn inherit(&mut self, layer: Layer, target: Target, override_on: bool) {
        if layer == Layer::Base {
            return;
        }
        let c = self.color(layer, target);
        let slot = if layer == Layer::Light {
            &mut self.record.light
        } else {
            &mut self.record.dark
        };
        target.set_override(
            slot.get_or_insert(PaletteOverrides::NONE),
            if override_on { Some(c) } else { None },
        );
        self.texts.remove(&(layer, target));
        self.epoch = self.epoch.saturating_add(1)
    }
    pub fn set_support(&mut self, scheme: ColorScheme, enabled: bool) {
        self.record.set_support(scheme, enabled);
        if !self.record.supports(scheme) {
            self.texts
                .retain(|(layer, _), _| layer.scheme() != Some(scheme));
        }
        self.epoch = self.epoch.saturating_add(1);
    }
    pub fn valid(&self) -> bool {
        self.record.validate().is_ok() && self.texts.values().all(|s| parse_hex(s).is_ok())
    }
    pub fn generate(&mut self) -> Result<(), String> {
        let seed = parse_hex(&self.seed)?;
        let shades = super::color::ramp(seed);
        self.record.base.shades = shades;
        self.record.base.accent = shades[7];
        self.record.base.accent_hover = shades[8];
        self.record.base.accent_soft = shades[0];
        self.record.base.accent_text = shades[9];
        self.record.base.on_accent = super::color::on_color(shades[7]);
        self.record.base.focus_ring = shades[3];
        self.record.base.input_focus = shades[6];
        if let Some(light) = &mut self.record.light {
            for role in ACCENT_ROLES {
                light.set_color(role, None);
            }
        }
        if let Some(d) = &mut self.record.dark {
            d.accent = None;
            d.accent_hover = None;
            d.on_accent = None;
            d.accent_soft = Some(super::color::at_lightness(seed, 0.23));
            d.accent_text = Some(shades[2]);
            d.focus_ring = Some(shades[3]);
            d.input_focus = Some(shades[4]);
        }
        self.texts.retain(|(layer, target), _| match target {
            Target::Role(index) => !ACCENT_ROLES.contains(&Palette::COLOR_NAMES[*index]),
            Target::Shade(_) => *layer != Layer::Base,
            Target::Series(_) => true,
        });
        self.epoch = self.epoch.saturating_add(1);
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ColorsDto {
    colors: BTreeMap<String, [u8; 4]>,
    shades: Vec<[u8; 4]>,
    series: Vec<[u8; 4]>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OverridesDto {
    colors: BTreeMap<String, [u8; 4]>,
    shades: Vec<Option<[u8; 4]>>,
    series: Vec<Option<[u8; 4]>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordDto {
    id: String,
    title: String,
    default_scheme: String,
    base: ColorsDto,
    light: Option<OverridesDto>,
    dark: Option<OverridesDto>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreDto {
    version: u32,
    next_id: u64,
    themes: Vec<RecordDto>,
}
fn bytes(Color(r, g, b, a): Color) -> [u8; 4] {
    [r, g, b, a]
}
fn color([r, g, b, a]: [u8; 4]) -> Color {
    Color(r, g, b, a)
}
impl From<&ThemeRecord> for RecordDto {
    fn from(r: &ThemeRecord) -> Self {
        let ov = |p: PaletteOverrides| OverridesDto {
            colors: Palette::COLOR_NAMES
                .iter()
                .filter_map(|n| p.color(n).map(|c| ((*n).into(), bytes(c))))
                .collect(),
            shades: p.shades.iter().map(|c| c.map(bytes)).collect(),
            series: p.series.iter().map(|c| c.map(bytes)).collect(),
        };
        Self {
            id: r.id.clone(),
            title: r.title.clone(),
            default_scheme: r.default_scheme.name().into(),
            base: ColorsDto {
                colors: Palette::COLOR_NAMES
                    .iter()
                    .map(|n| ((*n).into(), bytes(r.base.color(n).unwrap())))
                    .collect(),
                shades: r.base.shades.map(bytes).to_vec(),
                series: r.base.series.map(bytes).to_vec(),
            },
            light: r.light.map(ov),
            dark: r.dark.map(ov),
        }
    }
}
impl RecordDto {
    fn decode(mut self, legacy: bool) -> Result<ThemeRecord, String> {
        if self
            .id
            .strip_prefix("custom-")
            .and_then(|n| n.parse::<u64>().ok())
            .is_none_or(|n| n == 0 || self.id != format!("custom-{n}"))
        {
            return Err("Invalid saved theme ID".into());
        }
        if legacy
            && self.base.colors.len() + 1 == Palette::COLOR_NAMES.len()
            && !self.base.colors.contains_key("backdrop")
            && self
                .base
                .colors
                .keys()
                .all(|name| Palette::COLOR_NAMES.contains(&name.as_str()))
        {
            self.base
                .colors
                .insert("backdrop".into(), bytes(sapid::BLUE.base.backdrop));
            if let Some(dark) = &mut self.dark {
                dark.colors
                    .entry("backdrop".into())
                    .or_insert(bytes(sapid::BLUE.dark().expect("built-in dark").backdrop));
            }
        }
        let mut base = sapid::BLUE.base;
        if self.base.colors.len() != Palette::COLOR_NAMES.len()
            || self.base.shades.len() != 10
            || self.base.series.len() != 8
        {
            return Err("Incomplete saved palette".into());
        }
        for (name, value) in self.base.colors {
            if !base.set_color(&name, color(value)) {
                return Err("Unknown saved color role".into());
            }
        }
        base.shades = self
            .base
            .shades
            .into_iter()
            .map(color)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        base.series = self
            .base
            .series
            .into_iter()
            .map(color)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let ov = |p: OverridesDto| -> Result<PaletteOverrides, String> {
            if p.shades.len() != 10 || p.series.len() != 8 {
                return Err("Invalid saved array".into());
            }
            let mut o = PaletteOverrides::NONE;
            for (name, value) in p.colors {
                if !o.set_color(&name, Some(color(value))) {
                    return Err("Unknown saved override".into());
                }
            }
            o.shades = p
                .shades
                .into_iter()
                .map(|v| v.map(color))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            o.series = p
                .series
                .into_iter()
                .map(|v| v.map(color))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            Ok(o)
        };
        let record = ThemeRecord {
            id: self.id,
            title: self.title,
            base,
            light: self.light.map(ov).transpose()?,
            dark: self.dark.map(ov).transpose()?,
            default_scheme: match self.default_scheme.as_str() {
                "light" => ColorScheme::Light,
                "dark" => ColorScheme::Dark,
                _ => return Err("Unknown default mode".into()),
            },
        };
        record.validate()?;
        Ok(record)
    }
}
#[derive(Clone, Debug, Default)]
pub struct ThemeStore {
    pub next_id: u64,
    pub themes: Vec<ThemeRecord>,
}
impl ThemeStore {
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > MAX_STORAGE_BYTES {
            return Err("Saved themes exceed local storage budget".into());
        }
        let dto: StoreDto =
            serde_json::from_str(text).map_err(|_| "Saved themes have an invalid format")?;
        if !matches!(dto.version, 1 | 2) || dto.themes.len() > MAX_THEMES {
            return Err("Unsupported saved theme version or count".into());
        }
        let mut themes = Vec::new();
        let next_id = dto.next_id;
        let legacy = dto.version == 1;
        for dto in dto.themes {
            let r = dto.decode(legacy)?;
            if themes.iter().any(|x: &ThemeRecord| x.id == r.id) {
                return Err("Duplicate saved theme ID".into());
            }
            if r.id
                .strip_prefix("custom-")
                .unwrap()
                .parse::<u64>()
                .unwrap()
                > next_id
            {
                return Err("Invalid saved ID sequence".into());
            }
            themes.push(r)
        }
        Ok(Self {
            next_id: dto.next_id,
            themes,
        })
    }
    pub fn needs_upgrade(text: &str) -> bool {
        serde_json::from_str::<StoreDto>(text).is_ok_and(|s| s.version == 1)
    }
    pub fn json(&self) -> Result<String, String> {
        let text = serde_json::to_string(&StoreDto {
            version: 2,
            next_id: self.next_id,
            themes: self.themes.iter().map(RecordDto::from).collect(),
        })
        .map_err(|_| "Could not encode theme")?;
        if text.len() > MAX_STORAGE_BYTES {
            return Err("Saved themes exceed local storage budget".into());
        }
        Ok(text)
    }
    pub fn save(&mut self, mut record: ThemeRecord) -> Result<String, String> {
        record.validate()?;
        if let Some(i) = self.themes.iter().position(|r| r.id == record.id) {
            self.themes[i] = record.clone();
            return Ok(record.id);
        }
        if self.themes.len() >= MAX_THEMES {
            return Err(
                "Eight previews are saved. Edit an existing preview or remove one first."
                    .into(),
            );
        }
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or("Saved theme IDs exhausted")?;
        record.id = format!("custom-{}", self.next_id);
        let id = record.id.clone();
        record.validate()?;
        self.themes.push(record);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_hex_incomplete_input_and_independent_inheritance() {
        let mut draft = Draft::new(ThemeRecord::from_definition(&sapid::BLUE));
        let target = Target::Role(0);
        let before = draft.color(Layer::Base, target);
        draft.set_text(Layer::Base, target, "#1".into());
        assert_eq!(draft.color(Layer::Base, target), before);
        assert!(!draft.valid());
        draft.set_text(Layer::Base, target, "#12345600".into());
        assert_eq!(draft.color(Layer::Base, target), Color(18, 52, 86, 0));
        assert!(draft.valid());
        draft.inherit(Layer::Dark, target, false);
        assert_eq!(draft.color(Layer::Dark, target), Color(18, 52, 86, 0));
        draft.inherit(Layer::Dark, target, true);
        draft.set_color(Layer::Base, target, Color(30, 40, 50, 60), true);
        assert_eq!(draft.color(Layer::Dark, target), Color(18, 52, 86, 0));
        draft.inherit(Layer::Dark, target, false);
        assert_eq!(draft.color(Layer::Dark, target), Color(30, 40, 50, 60));
        assert_eq!(parse_hex("ffffff"), Ok(Color(255, 255, 255, 255)));
        assert!(parse_hex("#ffffff00x").is_err());
    }
    #[test]
    fn alternate_mode_is_explicit_and_default_switch_preserves_old_support() {
        let mut record = ThemeRecord::from_definition(&sapid::BLUE);
        record.dark = None;
        assert!(!record.supports(ColorScheme::Dark));
        assert!(record.theme(ColorScheme::Dark).is_err());
        record.set_default(ColorScheme::Dark);
        assert!(record.supports(ColorScheme::Dark));
        assert!(record.light.is_some());
        record.set_support(ColorScheme::Dark, false);
        assert!(record.supports(ColorScheme::Dark));
        record.set_support(ColorScheme::Light, false);
        assert!(!record.supports(ColorScheme::Light));
        assert!(!record.code().contains(".with_light("));
        assert!(
            record
                .code()
                .contains(".with_default_scheme(ColorScheme::Dark)")
        );
    }
    #[test]
    fn storage_roundtrip_update_budget_and_strict_validation() {
        let mut store = ThemeStore::default();
        let mut record = ThemeRecord::from_definition(&sapid::GREEN);
        record.base.canvas = Color(9, 8, 7, 6);
        record.light = Some(PaletteOverrides::NONE);
        record.light.as_mut().unwrap().series[7] = Some(Color(4, 3, 2, 1));
        let id = store.save(record).unwrap();
        assert_eq!(id, "custom-1");
        let json = store.json().unwrap();
        let mut reloaded = ThemeStore::parse(&json).unwrap();
        assert_eq!(reloaded.themes, store.themes);
        let mut edited = reloaded.themes[0].clone();
        edited.title = "Edited".into();
        reloaded.save(edited).unwrap();
        assert_eq!(reloaded.themes.len(), 1);
        for _ in 1..MAX_THEMES {
            reloaded
                .save(ThemeRecord::from_definition(&sapid::BLUE))
                .unwrap();
        }
        assert!(
            reloaded
                .save(ThemeRecord::from_definition(&sapid::BLUE))
                .is_err()
        );
        for invalid in [
            json.replace("\"version\":2", "\"version\":99"),
            json.replace("custom-1", "custom-evil"),
            json.replace("\"next_id\":1", "\"next_id\":0"),
            json.replace("\"canvas\"", "\"unknown\""),
        ] {
            assert!(ThemeStore::parse(&invalid).is_err(), "{invalid}");
        }
        assert!(ThemeStore::parse(&" ".repeat(MAX_STORAGE_BYTES + 1)).is_err());
    }
    #[test]
    fn old_preview_upgrade_preserves_user_colors_and_rejects_unrelated_missing_fields() {
        let mut store = ThemeStore::default();
        let mut record = ThemeRecord::from_definition(&sapid::GREEN);
        record.base.canvas = Color(3, 4, 5, 6);
        store.save(record).unwrap();
        let mut legacy: serde_json::Value =
            serde_json::from_str(&store.json().unwrap()).unwrap();
        legacy["version"] = 1.into();
        legacy["themes"][0]["base"]["colors"]
            .as_object_mut()
            .unwrap()
            .remove("backdrop");
        legacy["themes"][0]["dark"]["colors"]
            .as_object_mut()
            .unwrap()
            .remove("backdrop");
        let json = legacy.to_string();
        let migrated = ThemeStore::parse(&json).unwrap();
        assert_eq!(migrated.themes[0].base.canvas, Color(3, 4, 5, 6));
        assert_eq!(migrated.themes[0].id, "custom-1");
        assert_eq!(migrated.themes[0].base.backdrop, sapid::BLUE.base.backdrop);
        assert_eq!(
            migrated.themes[0]
                .resolve(ColorScheme::Dark)
                .unwrap()
                .backdrop,
            sapid::BLUE.dark().unwrap().backdrop
        );
        assert!(ThemeStore::needs_upgrade(&json));
        assert!(!ThemeStore::needs_upgrade(&migrated.json().unwrap()));
        legacy["themes"][0]["base"]["colors"]
            .as_object_mut()
            .unwrap()
            .remove("canvas");
        assert!(ThemeStore::parse(&legacy.to_string()).is_err());
    }
    #[test]
    fn generation_preserves_neutrals_and_updates_both_modes() {
        let mut d = Draft::new(ThemeRecord::from_definition(&sapid::GREEN));
        let canvas = d.record.base.canvas;
        d.seed = "#7c3aed".into();
        d.generate().unwrap();
        assert_eq!(d.record.base.shades[6], Color(124, 58, 237, 255));
        assert_eq!(d.record.base.canvas, canvas);
        assert_eq!(
            d.record.resolve(ColorScheme::Dark).unwrap().accent,
            d.record.base.accent
        );
        let code = d.record.code();
        for name in Palette::COLOR_NAMES {
            assert!(code.contains(&format!("{name}: Color(")));
        }
        assert!(code.contains("..PaletteOverrides::NONE"));
    }
}
