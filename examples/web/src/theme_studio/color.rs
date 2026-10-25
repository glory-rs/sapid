//! Oklab ramp generation with bounded chroma reduction to the sRGB gamut.
use glory::style::Color;
fn linear(x: u8) -> f64 {
    let x = x as f64 / 255.;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
fn lab(Color(r, g, b, _): Color) -> [f64; 3] {
    let (r, g, b) = (linear(r), linear(g), linear(b));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
    ]
}
fn rgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    let x = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let y = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let z = (l - 0.0894841775 * a - 1.291485548 * b).powi(3);
    [
        4.0767416621 * x - 3.3077115913 * y + 0.2309699292 * z,
        -1.2684380046 * x + 2.6097574011 * y - 0.3413193965 * z,
        -0.0041960863 * x - 0.7034186147 * y + 1.707614701 * z,
    ]
}
pub fn at_lightness(seed: Color, l: f64) -> Color {
    let [_, a, b] = lab(seed);
    let mut low = 0.;
    let mut high = 1.;
    for _ in 0..16 {
        let t = (low + high) / 2.;
        let c = rgb(l, a * t, b * t);
        if c.iter().all(|x| (0.0..=1.0).contains(x)) {
            low = t
        } else {
            high = t
        }
    }
    let c = rgb(l, a * low, b * low);
    let byte = |v: f64| {
        let x = if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1. / 2.4) - 0.055
        };
        (x * 255.).round().clamp(0., 255.) as u8
    };
    Color(byte(c[0]), byte(c[1]), byte(c[2]), seed.3)
}
pub fn ramp(seed: Color) -> [Color; 10] {
    let l = lab(seed)[0];
    std::array::from_fn(|i| {
        if i == 6 {
            return seed;
        }
        let light = if i < 6 {
            let top = (l + 0.45).min(0.98).max(l);
            top + (l - top) * i as f64 / 6.
        } else {
            l + (0.12_f64.min(l) - l) * (i - 6) as f64 / 3.
        };
        at_lightness(seed, light)
    })
}
fn luminance(c: Color) -> f64 {
    0.2126 * linear(c.0) + 0.7152 * linear(c.1) + 0.0722 * linear(c.2)
}
pub fn contrast(a: Color, b: Color) -> f64 {
    let blend = |c: u8, b: u8| {
        ((c as f64 * a.3 as f64 + b as f64 * (255 - a.3) as f64) / 255.).round() as u8
    };
    let a = Color(blend(a.0, b.0), blend(a.1, b.1), blend(a.2, b.2), 255);
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
/// Black/white are analytic endpoints for generated color DATA, never locked UI colors.
pub fn on_color(c: Color) -> Color {
    let black = Color(0, 0, 0, 255);
    let white = Color(255, 255, 255, 255);
    if contrast(black, c) > contrast(white, c) {
        black
    } else {
        white
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ramps_preserve_seed_and_descend_in_perceptual_lightness_including_extremes() {
        for seed in [
            Color(124, 58, 237, 255),
            Color(0, 255, 0, 255),
            Color(255, 255, 255, 255),
            Color(0, 0, 0, 255),
            Color(7, 19, 31, 0),
        ] {
            let shades = ramp(seed);
            assert_eq!(shades[6], seed);
            for pair in shades.windows(2) {
                assert!(
                    lab(pair[0])[0] + 0.005 >= lab(pair[1])[0],
                    "{seed:?}: {pair:?}"
                );
            }
            assert!(shades.iter().all(|c| c.3 == seed.3));
        }
    }
    #[test]
    fn contrast_matches_endpoints_and_alpha_compositing() {
        let white = Color(255, 255, 255, 255);
        let black = Color(0, 0, 0, 255);
        assert!((contrast(black, white) - 21.).abs() < 1e-8);
        assert_eq!(contrast(Color(0, 0, 0, 0), white), 1.);
        assert_eq!(on_color(black), white);
        assert_eq!(on_color(white), black);
    }
}
