use crate::color::{Color, TEXT_PRIMARY};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Style {
    #[default]
    Neumorphism,
    Clay,
    Classic,
}

impl Style {
    pub const ALL: [Style; 3] = [Style::Neumorphism, Style::Clay, Style::Classic];

    pub fn key(self) -> &'static str {
        match self {
            Style::Neumorphism => "neumorphism",
            Style::Clay => "clay",
            Style::Classic => "classic",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.key().eq_ignore_ascii_case(key))
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Relief {
    pub fill: Color,
    pub fill_end: Color,
    pub outer_tl: Color,
    pub outer_br: Color,
    pub inner_tl: Color,
    pub inner_br: Color,
    pub depth: f32,
    pub blur: f32,
}

impl Relief {
    pub const fn flat(fill: Color) -> Relief {
        let none = Color::rgb(0).with_alpha(0);
        Relief { fill, fill_end: fill, outer_tl: none, outer_br: none, inner_tl: none, inner_br: none, depth: 0.0, blur: 0.0 }
    }

    pub fn pad(&self) -> f32 {
        if self.outer_tl.a == 0 && self.outer_br.a == 0 { 1.0 } else { (self.depth + self.blur * 1.5).ceil() + 1.0 }
    }

    pub fn has_inner(&self) -> bool {
        self.inner_tl.a > 0 || self.inner_br.a > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub style: Style,
    pub base: Color,
    pub card: Color,
    pub well: Color,
    pub light: Color,
    pub dark: Color,
    pub text: Color,
    pub text_dim: Color,
    pub divider: Color,
}

pub fn palette(style: Style) -> Palette {
    match style {
        Style::Neumorphism => Palette {
            style,
            base: Color::rgb(0x111215),
            card: Color::rgb(0x111215),
            well: Color::rgb(0x0B0C0E),
            light: Color::rgb(0x23262D),
            dark: Color::rgb(0x000000),
            text: TEXT_PRIMARY,
            text_dim: TEXT_PRIMARY.with_opacity(0.62),
            divider: TEXT_PRIMARY.with_opacity(0.06),
        },
        Style::Clay => Palette {
            style,
            base: Color::rgb(0x0F0E13),
            card: Color::rgb(0x18171E),
            well: Color::rgb(0x0A090D),
            light: TEXT_PRIMARY.with_opacity(0.22),
            dark: Color::rgb(0).with_opacity(0.65),
            text: TEXT_PRIMARY,
            text_dim: TEXT_PRIMARY.with_opacity(0.64),
            divider: TEXT_PRIMARY.with_opacity(0.07),
        },
        Style::Classic => Palette {
            style,
            base: Color::rgb(0x000000),
            card: Color::rgb(0x1C1C1E),
            well: TEXT_PRIMARY.with_opacity(0.10).over(Color::rgb(0)),
            light: Color::rgb(0).with_alpha(0),
            dark: Color::rgb(0).with_alpha(0),
            text: TEXT_PRIMARY,
            text_dim: TEXT_PRIMARY.with_opacity(0.6),
            divider: TEXT_PRIMARY.with_opacity(0.08),
        },
    }
}

fn shade(c: Color, f: f32) -> Color {
    let k = |v: u8| ((v as f32) * f).round().clamp(0.0, 255.0) as u8;
    Color { r: k(c.r), g: k(c.g), b: k(c.b), a: c.a }
}

fn tint(c: Color, f: f32) -> Color {
    let k = |v: u8| (v as f32 + (255.0 - v as f32) * f).round().clamp(0.0, 255.0) as u8;
    Color { r: k(c.r), g: k(c.g), b: k(c.b), a: c.a }
}

impl Palette {
    pub fn is_flat(&self) -> bool {
        self.style == Style::Classic
    }

    pub fn raised(&self, fill: Color, depth: f32) -> Relief {
        match self.style {
            Style::Neumorphism => Relief {
                fill: tint(fill, 0.03),
                fill_end: shade(fill, 0.92),
                outer_tl: self.light,
                outer_br: self.dark,
                inner_tl: Color::rgb(0).with_alpha(0),
                inner_br: Color::rgb(0).with_alpha(0),
                depth,
                blur: depth * 2.0,
            },
            Style::Clay => self.clay(fill, depth),
            Style::Classic => Relief::flat(fill),
        }
    }

    pub fn clay(&self, fill: Color, depth: f32) -> Relief {
        Relief {
            fill: tint(fill, 0.08),
            fill_end: shade(fill, 0.86),
            outer_tl: Color::rgb(0).with_alpha(0),
            outer_br: Color::rgb(0).with_opacity(0.5),
            inner_tl: TEXT_PRIMARY.with_opacity(0.34),
            inner_br: Color::rgb(0).with_opacity(0.30),
            depth,
            blur: depth * 2.2,
        }
    }

    pub fn inset(&self, fill: Color, depth: f32) -> Relief {
        match self.style {
            Style::Classic => Relief::flat(fill),
            Style::Neumorphism => Relief {
                fill,
                fill_end: fill,
                outer_tl: Color::rgb(0).with_alpha(0),
                outer_br: Color::rgb(0).with_alpha(0),
                inner_tl: self.dark,
                inner_br: self.light,
                depth,
                blur: depth * 1.8,
            },
            Style::Clay => Relief {
                fill,
                fill_end: fill,
                outer_tl: Color::rgb(0).with_alpha(0),
                outer_br: Color::rgb(0).with_alpha(0),
                inner_tl: Color::rgb(0).with_opacity(0.6),
                inner_br: TEXT_PRIMARY.with_opacity(0.08),
                depth,
                blur: depth * 1.8,
            },
        }
    }

    pub fn control(&self, accent: Color) -> Color {
        match self.style {
            Style::Clay => accent,
            Style::Neumorphism => self.base,
            Style::Classic => TEXT_PRIMARY.with_opacity(0.14).over(self.base),
        }
    }

    pub fn on(&self, bg: Color) -> Color {
        on_color(bg)
    }
}

pub fn on_color(bg: Color) -> Color {
    let dark = Color::rgb(0x15161A);
    if TEXT_PRIMARY.contrast_ratio(bg) >= dark.contrast_ratio(bg) { TEXT_PRIMARY } else { dark }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Skin;

    #[test]
    fn teks_terbaca_di_semua_gaya() {
        for s in Style::ALL {
            let p = palette(s);
            assert!(p.text.contrast_ratio(p.base) >= 7.0, "{s:?} teks utama");
            assert!(p.text_dim.over(p.card).contrast_ratio(p.card) >= 4.5, "{s:?} teks sekunder");
            assert!(p.text.contrast_ratio(p.well) >= 7.0, "{s:?} teks di sumur");
        }
    }

    #[test]
    fn warna_di_atas_aksen_terbaca() {
        for skin in Skin::ALL {
            let bg = skin.color();
            assert!(on_color(bg).contrast_ratio(bg) >= 4.5, "{skin:?}");
        }
    }

    #[test]
    fn kunci_gaya() {
        assert_eq!(Style::from_key("CLAY"), Some(Style::Clay));
        assert_eq!(Style::from_key("glass"), None);
        assert_eq!(Style::default(), Style::Neumorphism);
    }

    #[test]
    fn klasik_datar_tanpa_bayangan() {
        let p = palette(Style::Classic);
        let r = p.raised(Color::rgb(0x333333), 4.0);
        assert_eq!(r.pad(), 1.0);
        assert!(!r.has_inner());
        let n = palette(Style::Neumorphism).raised(Color::rgb(0x111215), 4.0);
        assert!(n.pad() > 4.0);
    }
}
