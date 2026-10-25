//! The two shipped theme definitions; colors never live in the resolver.
use super::{Palette, PaletteOverrides, ThemeDefinition};
use glory::style::Color;

const fn rgb(hex: u32) -> Color {
    Color((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255)
}
pub const BLUE_SHADES: [Color; 10] = [
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
pub const GREEN_SHADES: [Color; 10] = [
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

const DARK: PaletteOverrides = PaletteOverrides {
    backdrop: Some(Color(0, 0, 0, 150)),
    series: [
        Some(rgb(0x74c0fc)),
        Some(rgb(0xff8787)),
        Some(rgb(0x8ce99a)),
        Some(rgb(0xe599f7)),
        Some(rgb(0xffd43b)),
        Some(rgb(0x66d9e8)),
        Some(rgb(0xb197fc)),
        Some(rgb(0xf783ac)),
    ],
    success: Some(GREEN_SHADES[3]),
    success_soft: Some(rgb(0x173322)),
    warning: Some(rgb(0xffe066)),
    warning_soft: Some(rgb(0x382b0d)),
    syntax_string: Some(GREEN_SHADES[3]),
    shadow: Some(Color(0, 0, 0, 70)),
    shadow_small: Some(Color(0, 0, 0, 52)),
    shadow_medium: Some(Color(0, 0, 0, 62)),
    shadow_large: Some(Color(0, 0, 0, 70)),
    checker_light: Some(rgb(0x495057)),
    checker_dark: Some(rgb(0x343a40)),
    canvas: Some(rgb(0x141517)),
    surface: Some(rgb(0x1a1b1e)),
    input: Some(rgb(0x25262b)),
    foreground: Some(rgb(0xdee2e6)),
    muted: Some(rgb(0xadb5bd)),
    border: Some(rgb(0x495057)),
    disabled: Some(rgb(0x25262b)),
    disabled_text: Some(rgb(0x909296)),
    danger: Some(rgb(0xffa8a8)),
    danger_soft: Some(rgb(0x3d2023)),
    ..PaletteOverrides::NONE
};

pub const BLUE: ThemeDefinition = ThemeDefinition::new(
    "blue",
    Palette {
        shades: BLUE_SHADES,
        backdrop: Color(0, 0, 0, 90),
        series: [
            rgb(0x1971c2),
            rgb(0xc92a2a),
            rgb(0x2b8a3e),
            rgb(0x9c36b5),
            rgb(0xe67700),
            rgb(0x0b7285),
            rgb(0x5f3dc4),
            rgb(0x862e9c),
        ],
        success: rgb(0x237032),
        success_soft: rgb(0xebfbee),
        warning: rgb(0x664d03),
        warning_soft: rgb(0xfff9db),
        syntax_string: GREEN_SHADES[9],
        shadow: Color(0, 0, 0, 40),
        shadow_small: Color(0, 0, 0, 26),
        shadow_medium: Color(0, 0, 0, 31),
        shadow_large: Color(0, 0, 0, 35),
        checker_light: rgb(0xffffff),
        checker_dark: rgb(0xc0c0c0),
        control_thumb: rgb(0xffffff),
        swatch_mark_light: rgb(0x000000),
        swatch_mark_dark: rgb(0xffffff),
        canvas: rgb(0xf8f9fa),
        surface: rgb(0xffffff),
        input: rgb(0xffffff),
        foreground: rgb(0x212529),
        muted: rgb(0x5c6470),
        accent: BLUE_SHADES[8],
        accent_hover: BLUE_SHADES[9],
        accent_soft: BLUE_SHADES[0],
        accent_text: BLUE_SHADES[9],
        on_accent: rgb(0xffffff),
        focus_ring: BLUE_SHADES[3],
        input_focus: BLUE_SHADES[6],
        danger: rgb(0xc92a2a),
        danger_fill: rgb(0xc92a2a),
        on_danger: rgb(0xffffff),
        danger_soft: rgb(0xfff5f5),
        border: rgb(0xced4da),
        disabled: rgb(0xf1f3f5),
        disabled_text: rgb(0x868e96),
    },
)
.with_dark(PaletteOverrides {
    accent_soft: Some(rgb(0x142c41)),
    accent_text: Some(BLUE_SHADES[3]),
    ..DARK
});

pub const GREEN: ThemeDefinition = ThemeDefinition::new(
    "green",
    Palette {
        shades: GREEN_SHADES,
        accent: rgb(0x237032),
        accent_hover: rgb(0x1b5e2a),
        accent_soft: GREEN_SHADES[0],
        accent_text: rgb(0x237032),
        focus_ring: GREEN_SHADES[3],
        input_focus: GREEN_SHADES[8],
        ..BLUE.base
    },
)
.with_dark(PaletteOverrides {
    accent_soft: Some(rgb(0x173322)),
    accent_text: Some(GREEN_SHADES[3]),
    input_focus: Some(GREEN_SHADES[5]),
    ..DARK
});
