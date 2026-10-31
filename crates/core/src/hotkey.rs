//! Parser hotkey seperti `"Ctrl+Alt+N"`.
//!
//! Kode tombol memakai nilai virtual-key Windows sebagai angka biasa, jadi
//! modul ini tetap Rust murni.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    /// Kode virtual-key Windows.
    pub vk: u16,
}

impl Hotkey {
    pub fn parse(text: &str) -> Result<Hotkey, String> {
        let mut hk = Hotkey { ctrl: false, alt: false, shift: false, win: false, vk: 0 };
        let parts: Vec<&str> = text.split('+').map(str::trim).collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(format!("format hotkey tidak valid: {text:?}"));
        }
        let (key, mods) = parts.split_last().ok_or("hotkey kosong")?;
        for m in mods {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => hk.ctrl = true,
                "alt" => hk.alt = true,
                "shift" => hk.shift = true,
                "win" | "super" | "meta" => hk.win = true,
                other => return Err(format!("pengubah tidak dikenal: {other:?}")),
            }
        }
        hk.vk = vk_from_name(key).ok_or_else(|| format!("tombol tidak dikenal: {key:?}"))?;
        if !(hk.ctrl || hk.alt || hk.win) && !(0x70..=0x87).contains(&hk.vk) {
            return Err("hotkey global perlu Ctrl, Alt, atau Win (kecuali F1–F24)".into());
        }
        Ok(hk)
    }

    pub fn display(&self) -> String {
        let mut s = String::new();
        for (on, name) in [(self.ctrl, "Ctrl"), (self.alt, "Alt"), (self.shift, "Shift"), (self.win, "Win")] {
            if on {
                s.push_str(name);
                s.push('+');
            }
        }
        s.push_str(&vk_name(self.vk));
        s
    }
}

fn vk_from_name(name: &str) -> Option<u16> {
    let upper = name.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    if bytes.len() == 1 && (bytes[0].is_ascii_uppercase() || bytes[0].is_ascii_digit()) {
        return Some(bytes[0] as u16);
    }
    if let Some(n) = upper.strip_prefix('F').and_then(|n| n.parse::<u16>().ok()) {
        return (1..=24).contains(&n).then_some(0x6F + n);
    }
    Some(match upper.as_str() {
        "SPACE" => 0x20,
        "ENTER" | "RETURN" => 0x0D,
        "TAB" => 0x09,
        "ESC" | "ESCAPE" => 0x1B,
        "BACKSPACE" => 0x08,
        "INSERT" => 0x2D,
        "DELETE" | "DEL" => 0x2E,
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" => 0x21,
        "PAGEDOWN" => 0x22,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        _ => return None,
    })
}

fn vk_name(vk: u16) -> String {
    match vk {
        0x30..=0x39 | 0x41..=0x5A => (vk as u8 as char).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0x20 => "Space".into(),
        0x0D => "Enter".into(),
        0x09 => "Tab".into(),
        0x1B => "Esc".into(),
        _ => format!("VK{vk:#04X}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dasar() {
        let h = Hotkey::parse("Ctrl+Alt+N").unwrap();
        assert!(h.ctrl && h.alt && !h.shift && !h.win);
        assert_eq!(h.vk, b'N' as u16);
        assert_eq!(h.display(), "Ctrl+Alt+N");
    }

    #[test]
    fn parse_variasi() {
        assert_eq!(Hotkey::parse("win + shift + space").unwrap().vk, 0x20);
        assert_eq!(Hotkey::parse("F9").unwrap().vk, 0x78);
        assert_eq!(Hotkey::parse("Alt+F24").unwrap().vk, 0x87);
        assert_eq!(Hotkey::parse("ctrl+alt+shift+t").unwrap().display(), "Ctrl+Alt+Shift+T");
    }

    #[test]
    fn tolak_yang_tidak_valid() {
        for bad in ["", "N", "Shift+N", "Ctrl+", "Ctrl+Hyper+N", "Ctrl+F25", "Ctrl+Alt+Ä"] {
            assert!(Hotkey::parse(bad).is_err(), "{bad:?} seharusnya ditolak");
        }
    }
}
