//! One finite color schema for resolution, editing, export and CSS.
use super::{ColorScheme, ThemeTokens};
use glory::style::Color;
macro_rules! palette {
    ($($field:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct Palette { pub shades: [Color;10], pub series: [Color;8], $(pub $field:Color,)+ }
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct PaletteOverrides { pub shades: [Option<Color>;10], pub series: [Option<Color>;8], $(pub $field:Option<Color>,)+ }
        impl Palette {
            pub const COLOR_NAMES: &'static [&'static str] = &[$(stringify!($field),)+];
            pub fn color(&self,name:&str)->Option<Color> { match name { $(stringify!($field)=>Some(self.$field),)+ _=>None } }
            pub fn set_color(&mut self,name:&str,value:Color)->bool { match name { $(stringify!($field)=>{self.$field=value;true},)+ _=>false } }
        }
        impl PaletteOverrides {
            pub const NONE:Self=Self { shades:[None;10],series:[None;8],$($field:None,)+ };
            pub const fn apply(self,base:Palette)->Palette {
                let mut shades=base.shades; let mut i=0;
                while i<10 { if let Some(value)=self.shades[i] {shades[i]=value;} i+=1; }
                let mut series=base.series; let mut i=0;
                while i<8 { if let Some(value)=self.series[i] {series[i]=value;} i+=1; }
                Palette { shades,series,$($field:match self.$field {Some(value)=>value,None=>base.$field},)+ }
            }
            pub fn color(&self,name:&str)->Option<Color> {match name {$(stringify!($field)=>self.$field,)+ _=>None}}
            pub fn set_color(&mut self,name:&str,value:Option<Color>)->bool {match name {$(stringify!($field)=>{self.$field=value;true},)+ _=>false}}
        }
        #[derive(Clone,Debug,PartialEq)]
        pub struct Theme {
            pub name:String, pub scheme:ColorScheme, pub shades:[Color;10],pub series:[Color;8],
            $(pub $field:Color,)+ pub radius:f64,pub tokens:ThemeTokens,
        }
        impl Theme {
            /// Apply an already resolved palette; no mode-dependent colors are synthesized.
            pub fn from_palette(name:impl Into<String>,scheme:ColorScheme,colors:Palette)->Result<Self,glory::render::Error> {
                let theme=Self { name:name.into(),scheme,shades:colors.shades,series:colors.series,
                    $($field:colors.$field,)+radius:4.,tokens:ThemeTokens::default() };
                theme.validate()?; Ok(theme)
            }
            pub fn colors(&self)->Palette {Palette {shades:self.shades,series:self.series,$($field:self.$field,)+}}
        }
    };
}
palette! {
    canvas,
    surface,
    input,
    foreground,
    muted,
    accent,
    accent_hover,
    accent_soft,
    accent_text,
    on_accent,
    focus_ring,
    input_focus,
    danger,
    danger_fill,
    on_danger,
    danger_soft,
    border,
    disabled,
    disabled_text,
    success,
    success_soft,
    warning,
    warning_soft,
    syntax_string,
    shadow,
    shadow_small,
    shadow_medium,
    shadow_large,
    checker_light,
    checker_dark,
    control_thumb,
    swatch_mark_light,
    swatch_mark_dark,
    backdrop
}
