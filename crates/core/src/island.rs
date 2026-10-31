//! State machine Island (PRD §3.1).
//!
//! State normal (Idle, Compact, Minimal, Expanded) disimpan terpisah dari
//! interupsi (Alert, layar penuh), sehingga setelah interupsi selesai Island
//! otomatis kembali ke state sebelumnya.

use crate::Ms;
use crate::activity::Priority;

/// Mode tampil Island (PRD §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// Selalu tampil, termasuk pil kosong saat idle.
    Always,
    /// Hanya saat ada aktivitas (default).
    #[default]
    ActivityOnly,
    /// Muncul saat kursor berada di area Island.
    Hover,
}

impl DisplayMode {
    pub const ALL: [DisplayMode; 3] =
        [DisplayMode::Always, DisplayMode::ActivityOnly, DisplayMode::Hover];

    pub fn key(self) -> &'static str {
        match self {
            DisplayMode::Always => "always",
            DisplayMode::ActivityOnly => "activity",
            DisplayMode::Hover => "hover",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.key().eq_ignore_ascii_case(key))
    }
}

/// Peristiwa singkat yang menyela state normal.
#[derive(Debug, Clone, PartialEq)]
pub enum AlertKind {
    Charging { percent: Option<u8> },
    Unplugged { percent: Option<u8> },
    LowBattery { percent: u8, critical: bool },
    TimerDone,
    Volume { level: u8, muted: bool },
}

impl AlertKind {
    pub fn priority(&self) -> Priority {
        match self {
            AlertKind::Volume { .. } => Priority::P4,
            AlertKind::TimerDone => Priority::P2,
            _ => Priority::P1,
        }
    }

    /// Lama alert tampil (PRD: 3–5 detik; HUD volume lebih singkat).
    pub fn duration(&self) -> Ms {
        match self {
            AlertKind::Volume { .. } => 1_500,
            AlertKind::Charging { .. } | AlertKind::Unplugged { .. } => 3_000,
            AlertKind::LowBattery { .. } | AlertKind::TimerDone => 5_000,
        }
    }

    /// HUD kecil (bentuk compact) atau kartu alert besar.
    pub fn is_hud(&self) -> bool {
        matches!(self, AlertKind::Volume { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveAlert {
    kind: AlertKind,
    until: Ms,
}

/// Apa yang harus digambar saat ini.
#[derive(Debug, Clone, PartialEq)]
pub enum Presentation {
    Hidden,
    Idle,
    Compact,
    Minimal,
    Expanded,
    Alert(AlertKind),
}

#[derive(Debug, Clone)]
pub struct Island {
    mode: DisplayMode,
    expanded: bool,
    activity_count: usize,
    alert: Option<ActiveAlert>,
    fullscreen: bool,
    hover: bool,
    snoozed_until: Option<Ms>,
}

impl Island {
    pub fn new(mode: DisplayMode) -> Self {
        Self {
            mode,
            expanded: false,
            activity_count: 0,
            alert: None,
            fullscreen: false,
            hover: false,
            snoozed_until: None,
        }
    }

    pub fn mode(&self) -> DisplayMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: DisplayMode) {
        self.mode = mode;
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn presentation(&self, now: Ms) -> Presentation {
        if self.fullscreen || self.snoozed_until.is_some_and(|t| now < t) {
            return Presentation::Hidden;
        }
        if let Some(a) = &self.alert {
            if now < a.until {
                return Presentation::Alert(a.kind.clone());
            }
        }
        if self.expanded {
            return Presentation::Expanded;
        }
        match self.activity_count {
            0 => match self.mode {
                DisplayMode::Always => Presentation::Idle,
                DisplayMode::ActivityOnly => Presentation::Hidden,
                DisplayMode::Hover if self.hover => Presentation::Idle,
                DisplayMode::Hover => Presentation::Hidden,
            },
            1 => Presentation::Compact,
            _ => Presentation::Minimal,
        }
    }

    pub fn set_activity_count(&mut self, n: usize) {
        self.activity_count = n;
    }

    pub fn set_hover(&mut self, hover: bool) {
        self.hover = hover;
    }

    pub fn is_hover(&self) -> bool {
        self.hover
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
        if fullscreen {
            self.expanded = false;
        }
    }

    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Klik: expand/collapse. Mengklik alert menutupnya lalu membuka kartu.
    pub fn click(&mut self, now: Ms) {
        if self.alert_active(now) {
            self.alert = None;
            self.expanded = true;
        } else {
            self.expanded = !self.expanded;
        }
    }

    /// Tahan 350 ms.
    pub fn long_press(&mut self) {
        self.expanded = true;
    }

    /// Esc, klik di luar, atau kursor lama meninggalkan kartu.
    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    /// Hotkey global: buka/tutup tanpa mouse, juga membatalkan snooze.
    pub fn hotkey(&mut self) {
        self.snoozed_until = None;
        self.alert = None;
        self.expanded = !self.expanded;
    }

    /// "Sembunyikan 1 jam".
    pub fn snooze(&mut self, until: Ms) {
        self.snoozed_until = Some(until);
        self.expanded = false;
    }

    pub fn is_snoozed(&self, now: Ms) -> bool {
        self.snoozed_until.is_some_and(|t| now < t)
    }

    pub fn unsnooze(&mut self) {
        self.snoozed_until = None;
    }

    /// Menampilkan alert. Alert baru hanya menggantikan alert aktif yang
    /// prioritasnya sama atau lebih tinggi.
    pub fn push_alert(&mut self, kind: AlertKind, now: Ms) -> bool {
        if let Some(cur) = &self.alert {
            if now < cur.until && kind.priority() > cur.kind.priority() {
                return false;
            }
        }
        let until = now + kind.duration();
        self.alert = Some(ActiveAlert { kind, until });
        true
    }

    pub fn dismiss_alert(&mut self) {
        self.alert = None;
    }

    fn alert_active(&self, now: Ms) -> bool {
        self.alert.as_ref().is_some_and(|a| now < a.until)
    }

    /// Membuang alert dan snooze yang kedaluwarsa. Mengembalikan `true` bila
    /// presentasi berubah karenanya.
    pub fn tick(&mut self, now: Ms) -> bool {
        let mut changed = false;
        if self.alert.as_ref().is_some_and(|a| now >= a.until) {
            self.alert = None;
            changed = true;
        }
        if self.snoozed_until.is_some_and(|t| now >= t) {
            self.snoozed_until = None;
            changed = true;
        }
        changed
    }

    /// Waktu terdekat ketika [`Island::tick`] perlu dipanggil.
    pub fn next_deadline(&self) -> Option<Ms> {
        let a = self.alert.as_ref().map(|a| a.until);
        match (a, self.snoozed_until) {
            (Some(a), Some(s)) => Some(a.min(s)),
            (a, s) => a.or(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn charging() -> AlertKind {
        AlertKind::Charging { percent: Some(50) }
    }

    #[test]
    fn idle_mengikuti_mode() {
        let mut i = Island::new(DisplayMode::Always);
        assert_eq!(i.presentation(0), Presentation::Idle);
        i.set_mode(DisplayMode::ActivityOnly);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        i.set_mode(DisplayMode::Hover);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        i.set_hover(true);
        assert_eq!(i.presentation(0), Presentation::Idle);
    }

    #[test]
    fn jumlah_aktivitas_menentukan_bentuk() {
        let mut i = Island::new(DisplayMode::ActivityOnly);
        i.set_activity_count(1);
        assert_eq!(i.presentation(0), Presentation::Compact);
        i.set_activity_count(2);
        assert_eq!(i.presentation(0), Presentation::Minimal);
        i.set_activity_count(5);
        assert_eq!(i.presentation(0), Presentation::Minimal);
        i.set_activity_count(0);
        assert_eq!(i.presentation(0), Presentation::Hidden);
    }

    #[test]
    fn klik_expand_dan_collapse() {
        let mut i = Island::new(DisplayMode::Always);
        i.set_activity_count(1);
        i.click(0);
        assert_eq!(i.presentation(0), Presentation::Expanded);
        i.click(0);
        assert_eq!(i.presentation(0), Presentation::Compact);
        i.long_press();
        assert_eq!(i.presentation(0), Presentation::Expanded);
        i.collapse();
        assert_eq!(i.presentation(0), Presentation::Compact);
    }

    #[test]
    fn alert_menyela_lalu_kembali_ke_state_semula() {
        let mut i = Island::new(DisplayMode::Always);
        i.set_activity_count(1);
        i.click(0);
        assert!(i.push_alert(charging(), 1_000));
        assert_eq!(i.presentation(1_500), Presentation::Alert(charging()));
        assert_eq!(i.next_deadline(), Some(4_000));
        assert!(i.tick(4_000));
        assert_eq!(i.presentation(4_000), Presentation::Expanded);
    }

    #[test]
    fn alert_prioritas_rendah_tidak_menimpa() {
        let mut i = Island::new(DisplayMode::Always);
        assert!(i.push_alert(AlertKind::LowBattery { percent: 10, critical: true }, 0));
        assert!(!i.push_alert(AlertKind::Volume { level: 30, muted: false }, 100));
        assert!(i.push_alert(charging(), 200));
        // Setelah alert habis, alert prioritas rendah boleh tampil lagi.
        assert!(i.push_alert(AlertKind::Volume { level: 30, muted: false }, 10_000));
    }

    #[test]
    fn klik_alert_membuka_kartu() {
        let mut i = Island::new(DisplayMode::Always);
        i.push_alert(AlertKind::TimerDone, 0);
        i.click(10);
        assert_eq!(i.presentation(10), Presentation::Expanded);
    }

    #[test]
    fn layar_penuh_menyembunyikan_dan_kembali() {
        let mut i = Island::new(DisplayMode::Always);
        i.set_activity_count(1);
        i.set_fullscreen(true);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        i.push_alert(charging(), 0);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        i.set_fullscreen(false);
        assert_eq!(i.presentation(10), Presentation::Alert(charging()));
        assert_eq!(i.presentation(5_000), Presentation::Compact);
    }

    #[test]
    fn snooze_satu_jam() {
        let mut i = Island::new(DisplayMode::Always);
        i.snooze(3_600_000);
        assert_eq!(i.presentation(10), Presentation::Hidden);
        assert_eq!(i.next_deadline(), Some(3_600_000));
        assert!(i.tick(3_600_000));
        assert_eq!(i.presentation(3_600_000), Presentation::Idle);
        i.snooze(10_000_000);
        i.hotkey();
        assert_eq!(i.presentation(5), Presentation::Expanded);
    }

    #[test]
    fn kunci_mode() {
        assert_eq!(DisplayMode::from_key("HOVER"), Some(DisplayMode::Hover));
        assert_eq!(DisplayMode::from_key("activity"), Some(DisplayMode::ActivityOnly));
        assert_eq!(DisplayMode::from_key("kadang"), None);
    }
}
