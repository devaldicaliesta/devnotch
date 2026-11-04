use crate::color::{Color, TEXT_PRIMARY};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Style {
    #[default]
    Neumorphism,
    Clay,
    /// Kaca modern: permukaan tembus pandang dengan tepi bercahaya.
    Glass,
    Classic,
}

impl Style {
    pub const ALL: [Style; 4] = [Style::Neumorphism, Style::Clay, Style::Glass, Style::Classic];

    pub fn key(self) -> &'static str {
        match self {
            Style::Neumorphism => "neumorphism",
            Style::Clay => "clay",
            Style::Glass => "glass",
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

/// Pilihan tema di pengaturan: mengikuti Windows, selalu gelap, atau selalu terang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Dark, Theme::Light];

    pub fn key(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key().eq_ignore_ascii_case(key))
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    /// Mode yang benar-benar dipakai; `system_light` dibaca dari Windows.
    pub fn resolve(self, system_light: bool) -> Mode {
        match self {
            Theme::Dark => Mode::Dark,
            Theme::Light => Mode::Light,
            Theme::System if system_light => Mode::Light,
            Theme::System => Mode::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Dark,
    Light,
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
    pub mode: Mode,
    /// Isi pil Island. Pada gaya kaca warnanya tembus pandang.
    pub base: Color,
    /// Latar jendela Pengaturan, selalu pekat.
    pub backdrop: Color,
    pub card: Color,
    pub well: Color,
    pub light: Color,
    pub dark: Color,
    pub text: Color,
    pub text_dim: Color,
    pub divider: Color,
    /// Latar tombol netral.
    pub neutral: Color,
    /// Kenop sakelar dan penanda segmen terpilih di Pengaturan.
    pub knob: Color,
    pub pill: Color,
}

const NONE: Color = Color::rgb(0).with_alpha(0);
const INK: Color = Color::rgb(0x16181D);
const WHITE: Color = TEXT_PRIMARY;
const BLACK: Color = Color::rgb(0);

pub fn palette(style: Style, mode: Mode) -> Palette {
    let dim = |text: Color, o: f32| text.with_opacity(o);
    match (style, mode) {
        (Style::Neumorphism, Mode::Dark) => Palette {
            style,
            mode,
            base: Color::rgb(0x111215),
            backdrop: Color::rgb(0x111215),
            card: Color::rgb(0x111215),
            well: Color::rgb(0x0B0C0E),
            light: Color::rgb(0x23262D),
            dark: BLACK,
            text: WHITE,
            text_dim: dim(WHITE, 0.62),
            divider: dim(WHITE, 0.06),
            neutral: Color::rgb(0x111215),
            knob: Color::rgb(0x2A2D35),
            pill: Color::rgb(0x1E2026),
        },
        (Style::Neumorphism, Mode::Light) => Palette {
            style,
            mode,
            base: Color::rgb(0xE4E8EE),
            backdrop: Color::rgb(0xE4E8EE),
            card: Color::rgb(0xE4E8EE),
            well: Color::rgb(0xD5DAE2),
            light: WHITE,
            dark: Color::rgb(0xA7B3C4),
            text: INK,
            text_dim: dim(INK, 0.66),
            divider: dim(INK, 0.08),
            neutral: Color::rgb(0xE4E8EE),
            knob: Color::rgb(0xF4F6F9),
            pill: Color::rgb(0xEEF1F5),
        },
        (Style::Clay, Mode::Dark) => Palette {
            style,
            mode,
            base: Color::rgb(0x0F0E13),
            backdrop: Color::rgb(0x0F0E13),
            card: Color::rgb(0x18171E),
            well: Color::rgb(0x0A090D),
            light: dim(WHITE, 0.22),
            dark: BLACK.with_opacity(0.65),
            text: WHITE,
            text_dim: dim(WHITE, 0.64),
            divider: dim(WHITE, 0.07),
            neutral: Color::rgb(0x2A2833),
            knob: Color::rgb(0x4A4857),
            pill: Color::rgb(0x2A2833),
        },
        (Style::Clay, Mode::Light) => Palette {
            style,
            mode,
            base: Color::rgb(0xF2EFF7),
            backdrop: Color::rgb(0xECE8F2),
            card: Color::rgb(0xFAF8FD),
            well: Color::rgb(0xE1DCEA),
            light: dim(WHITE, 0.9),
            dark: Color::rgb(0x3A3050).with_opacity(0.22),
            text: INK,
            text_dim: dim(INK, 0.66),
            divider: dim(INK, 0.08),
            neutral: WHITE,
            knob: WHITE,
            pill: WHITE,
        },
        (Style::Glass, Mode::Dark) => Palette {
            style,
            mode,
            base: Color::rgb(0x1B1F28).with_opacity(0.78),
            backdrop: Color::rgb(0x12151B),
            card: dim(WHITE, 0.06),
            well: BLACK.with_opacity(0.28),
            light: dim(WHITE, 0.30),
            dark: BLACK.with_opacity(0.45),
            text: WHITE,
            text_dim: dim(WHITE, 0.68),
            divider: dim(WHITE, 0.09),
            neutral: dim(WHITE, 0.10),
            knob: dim(WHITE, 0.92),
            pill: dim(WHITE, 0.16),
        },
        (Style::Glass, Mode::Light) => Palette {
            style,
            mode,
            base: Color::rgb(0xF6F8FC).with_opacity(0.80),
            backdrop: Color::rgb(0xE9EDF4),
            card: dim(WHITE, 0.62),
            well: Color::rgb(0x0B1530).with_opacity(0.07),
            light: dim(WHITE, 0.95),
            dark: Color::rgb(0x0B1530).with_opacity(0.14),
            text: INK,
            text_dim: dim(INK, 0.66),
            divider: dim(INK, 0.08),
            neutral: dim(WHITE, 0.70),
            knob: WHITE,
            pill: dim(WHITE, 0.92),
        },
        (Style::Classic, Mode::Dark) => Palette {
            style,
            mode,
            base: BLACK,
            backdrop: BLACK,
            card: Color::rgb(0x1C1C1E),
            well: dim(WHITE, 0.10).over(BLACK),
            light: NONE,
            dark: NONE,
            text: WHITE,
            text_dim: dim(WHITE, 0.6),
            divider: dim(WHITE, 0.08),
            neutral: dim(WHITE, 0.14).over(BLACK),
            knob: WHITE,
            pill: Color::rgb(0x3A3A3C),
        },
        (Style::Classic, Mode::Light) => Palette {
            style,
            mode,
            base: WHITE,
            backdrop: Color::rgb(0xF2F2F7),
            card: WHITE,
            well: dim(INK, 0.07).over(WHITE),
            light: NONE,
            dark: NONE,
            text: INK,
            text_dim: dim(INK, 0.62),
            divider: dim(INK, 0.08),
            neutral: dim(INK, 0.08).over(WHITE),
            knob: WHITE,
            pill: WHITE,
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

    pub fn is_light(&self) -> bool {
        self.mode == Mode::Light
    }

    /// Isi pil di atas blur acrylic: lebih tembus pandang daripada `base` karena blur
    /// sudah meredam latar yang ramai. Hanya berbeda untuk gaya Kaca.
    pub fn blur_tint(&self) -> Color {
        if self.style == Style::Glass {
            self.base.with_opacity(self.base.a as f32 / 255.0 * 0.62)
        } else {
            self.base
        }
    }

    /// Warna pekat permukaan pil, untuk menghitung kontras.
    pub fn surface(&self) -> Color {
        self.base.over(self.backdrop)
    }

    /// Warna aksen yang tetap terbaca di atas permukaan: digelapkan pada tema terang,
    /// dicerahkan pada tema gelap bila perlu.
    pub fn ink(&self, c: Color) -> Color {
        let bg = self.surface();
        let mut out = c.with_alpha(255);
        for _ in 0..24 {
            if out.contrast_ratio(bg) >= 4.5 {
                break;
            }
            out = if self.is_light() { shade(out, 0.9) } else { tint(out, 0.12) };
        }
        out.with_alpha(c.a)
    }

    pub fn raised(&self, fill: Color, depth: f32) -> Relief {
        match self.style {
            Style::Neumorphism => Relief {
                fill: tint(fill, 0.03),
                fill_end: shade(fill, if self.is_light() { 0.97 } else { 0.92 }),
                outer_tl: self.light,
                outer_br: self.dark,
                inner_tl: NONE,
                inner_br: NONE,
                depth,
                blur: depth * 2.0,
            },
            Style::Clay => self.clay(fill, depth),
            Style::Glass => self.glass(fill, depth),
            Style::Classic => Relief::flat(fill),
        }
    }

    pub fn clay(&self, fill: Color, depth: f32) -> Relief {
        let light = self.is_light();
        Relief {
            fill: tint(fill, 0.08),
            fill_end: shade(fill, if light { 0.93 } else { 0.86 }),
            outer_tl: NONE,
            outer_br: BLACK.with_opacity(if light { 0.20 } else { 0.5 }),
            inner_tl: WHITE.with_opacity(if light { 0.75 } else { 0.34 }),
            inner_br: BLACK.with_opacity(if light { 0.12 } else { 0.30 }),
            depth,
            blur: depth * 2.2,
        }
    }

    /// Kaca: isi tembus pandang bergradasi, tepi atas bercahaya, dan bayangan jatuh tipis.
    pub fn glass(&self, fill: Color, depth: f32) -> Relief {
        let light = self.is_light();
        Relief {
            fill: tint(fill, if light { 0.25 } else { 0.10 }),
            fill_end: shade(fill, if light { 0.97 } else { 0.85 }),
            outer_tl: NONE,
            outer_br: BLACK.with_opacity(if light { 0.10 } else { 0.30 }),
            inner_tl: WHITE.with_opacity(if light { 0.95 } else { 0.32 }),
            inner_br: if light { BLACK.with_opacity(0.05) } else { WHITE.with_opacity(0.06) },
            depth: depth * 0.6,
            blur: depth * 1.2,
        }
    }

    pub fn inset(&self, fill: Color, depth: f32) -> Relief {
        let light = self.is_light();
        match self.style {
            Style::Classic => Relief::flat(fill),
            Style::Neumorphism => Relief {
                fill,
                fill_end: fill,
                outer_tl: NONE,
                outer_br: NONE,
                inner_tl: self.dark,
                inner_br: self.light,
                depth,
                blur: depth * 1.8,
            },
            Style::Clay => Relief {
                fill,
                fill_end: fill,
                outer_tl: NONE,
                outer_br: NONE,
                inner_tl: BLACK.with_opacity(if light { 0.18 } else { 0.6 }),
                inner_br: WHITE.with_opacity(if light { 0.7 } else { 0.08 }),
                depth,
                blur: depth * 1.8,
            },
            Style::Glass => Relief {
                fill,
                fill_end: fill,
                outer_tl: NONE,
                outer_br: NONE,
                inner_tl: BLACK.with_opacity(if light { 0.10 } else { 0.35 }),
                inner_br: WHITE.with_opacity(if light { 0.6 } else { 0.12 }),
                depth: depth * 0.8,
                blur: depth * 1.4,
            },
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

    fn all() -> impl Iterator<Item = Palette> {
        Style::ALL.into_iter().flat_map(|s| [Mode::Dark, Mode::Light].map(|m| palette(s, m)))
    }

    #[test]
    fn teks_terbaca_di_semua_gaya() {
        for p in all() {
            let base = p.surface();
            let card = p.card.over(p.backdrop);
            let well = p.well.over(base);
            assert_eq!(p.backdrop.a, 255, "{p:?}: latar jendela pekat");
            assert!(p.text.contrast_ratio(base) >= 7.0, "{:?}/{:?} teks utama", p.style, p.mode);
            assert!(p.text.contrast_ratio(p.backdrop) >= 7.0, "{:?}/{:?} teks di latar", p.style, p.mode);
            assert!(p.text_dim.over(card).contrast_ratio(card) >= 4.5, "{:?}/{:?} teks sekunder", p.style, p.mode);
            assert!(p.text.contrast_ratio(well) >= 7.0, "{:?}/{:?} teks di sumur", p.style, p.mode);
        }
    }

    #[test]
    fn aksen_tetap_terbaca_di_tema_terang() {
        for p in all() {
            for skin in Skin::ALL {
                let c = p.ink(skin.color());
                assert!(c.contrast_ratio(p.surface()) >= 4.5, "{:?}/{:?} {skin:?}", p.style, p.mode);
            }
        }
        let dark = palette(Style::Neumorphism, Mode::Dark);
        assert_eq!(dark.ink(Skin::Langit.color()), Skin::Langit.color(), "aksen yang sudah terbaca tidak diubah");
    }

    #[test]
    fn tema_mengikuti_windows() {
        assert_eq!(Theme::System.resolve(true), Mode::Light);
        assert_eq!(Theme::System.resolve(false), Mode::Dark);
        assert_eq!(Theme::Dark.resolve(true), Mode::Dark);
        assert_eq!(Theme::Light.resolve(false), Mode::Light);
        assert_eq!(Theme::from_key("LIGHT"), Some(Theme::Light));
        assert_eq!(Theme::default(), Theme::System);
    }

    #[test]
    fn tint_blur_lebih_tipis_hanya_untuk_kaca() {
        let g = palette(Style::Glass, Mode::Dark);
        assert!(g.blur_tint().a < g.base.a && g.blur_tint().a > 80);
        let n = palette(Style::Neumorphism, Mode::Light);
        assert_eq!(n.blur_tint(), n.base);
    }

    #[test]
    fn kaca_tembus_pandang() {
        for m in [Mode::Dark, Mode::Light] {
            let p = palette(Style::Glass, m);
            assert!(p.base.a < 255 && p.base.a > 160, "tembus pandang tapi tetap terbaca");
            let r = p.raised(p.neutral, 4.0);
            assert!(r.has_inner() && r.pad() > 1.0, "tepi bercahaya");
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
        assert_eq!(Style::from_key("glass"), Some(Style::Glass));
        assert_eq!(Style::from_key("kaca"), None);
        assert_eq!(Style::default(), Style::Neumorphism);
    }

    #[test]
    fn klasik_datar_tanpa_bayangan() {
        for m in [Mode::Dark, Mode::Light] {
            let p = palette(Style::Classic, m);
            let r = p.raised(Color::rgb(0x333333), 4.0);
            assert_eq!(r.pad(), 1.0);
            assert!(!r.has_inner());
        }
        let n = palette(Style::Neumorphism, Mode::Dark).raised(Color::rgb(0x111215), 4.0);
        assert!(n.pad() > 4.0);
    }
}
