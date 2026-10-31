#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(hex: u32) -> Self {
        Self { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8, a: 255 }
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub fn with_opacity(self, fraction: f32) -> Self {
        self.with_alpha((fraction.clamp(0.0, 1.0) * 255.0).round() as u8)
    }

    pub fn over(self, bg: Color) -> Color {
        let a = self.a as f32 / 255.0;
        let mix = |f: u8, b: u8| (f as f32 * a + b as f32 * (1.0 - a)).round() as u8;
        Color { r: mix(self.r, bg.r), g: mix(self.g, bg.g), b: mix(self.b, bg.b), a: 255 }
    }

    pub fn relative_luminance(self) -> f32 {
        fn channel(c: u8) -> f32 {
            let s = c as f32 / 255.0;
            if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }

    pub fn contrast_ratio(self, other: Color) -> f32 {
        let (a, b) = (self.relative_luminance(), other.relative_luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }
}

pub const ISLAND_BG: Color = Color::rgb(0x000000);
pub const TEXT_PRIMARY: Color = Color::rgb(0xFFFFFF);
pub const STATUS_CRITICAL: Color = Color::rgb(0xFF5A5F);

pub fn text_secondary() -> Color {
    TEXT_PRIMARY.with_opacity(0.6)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Skin {
    #[default]
    Langit,
    Karang,
    Mint,
    Lemon,
    Lavender,
    Permen,
}

impl Skin {
    pub const ALL: [Skin; 6] =
        [Skin::Langit, Skin::Karang, Skin::Mint, Skin::Lemon, Skin::Lavender, Skin::Permen];

    pub fn color(self) -> Color {
        match self {
            Skin::Langit => Color::rgb(0x4DA8FF),
            Skin::Karang => Color::rgb(0xFF7A66),
            Skin::Mint => Color::rgb(0x3DD6A0),
            Skin::Lemon => Color::rgb(0xFFD23F),
            Skin::Lavender => Color::rgb(0xA58BFF),
            Skin::Permen => Color::rgb(0xFF8FC7),
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Skin::Langit => "langit",
            Skin::Karang => "karang",
            Skin::Mint => "mint",
            Skin::Lemon => "lemon",
            Skin::Lavender => "lavender",
            Skin::Permen => "permen",
        }
    }

    pub fn from_key(key: &str) -> Option<Skin> {
        Skin::ALL.into_iter().find(|s| s.key().eq_ignore_ascii_case(key))
    }
}

pub fn accent_charging() -> Color {
    Skin::Mint.color()
}

pub fn accent_alert() -> Color {
    Skin::Lemon.color()
}

#[cfg(test)]
mod tests {
    use super::*;

    const AA: f32 = 4.5;

    #[test]
    fn semua_token_lulus_wcag_aa_di_atas_hitam() {
        for skin in Skin::ALL {
            let ratio = skin.color().contrast_ratio(ISLAND_BG);
            assert!(ratio >= AA, "{skin:?} hanya {ratio:.2}:1");
        }
        assert!(STATUS_CRITICAL.contrast_ratio(ISLAND_BG) >= AA);
        assert!(TEXT_PRIMARY.contrast_ratio(ISLAND_BG) >= AA);
        let secondary = text_secondary().over(ISLAND_BG);
        assert!(secondary.contrast_ratio(ISLAND_BG) >= AA);
    }

    #[test]
    fn kontras_hitam_putih_21() {
        let r = TEXT_PRIMARY.contrast_ratio(ISLAND_BG);
        assert!((r - 21.0).abs() < 0.01);
    }

    #[test]
    fn kunci_skin_bolak_balik() {
        for s in Skin::ALL {
            assert_eq!(Skin::from_key(s.key()), Some(s));
        }
        assert_eq!(Skin::from_key("MINT"), Some(Skin::Mint));
        assert_eq!(Skin::from_key("ungu"), None);
    }
}
