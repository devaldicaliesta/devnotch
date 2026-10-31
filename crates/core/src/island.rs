use crate::Ms;
use crate::activity::Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    Always,
    #[default]
    ActivityOnly,
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

    pub fn duration(&self) -> Ms {
        match self {
            AlertKind::Volume { .. } => 1_500,
            AlertKind::Charging { .. } | AlertKind::Unplugged { .. } => 3_000,
            AlertKind::LowBattery { .. } | AlertKind::TimerDone => 5_000,
        }
    }

    pub fn is_hud(&self) -> bool {
        matches!(self, AlertKind::Volume { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveAlert {
    kind: AlertKind,
    until: Ms,
}

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
    autohide: bool,
    peek_until: Option<Ms>,
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
            autohide: false,
            peek_until: None,
        }
    }

    pub fn set_autohide(&mut self, on: bool) {
        self.autohide = on;
    }

    pub fn autohide(&self) -> bool {
        self.autohide
    }

    pub fn peek(&mut self, until: Ms) {
        self.peek_until = Some(self.peek_until.map_or(until, |p| p.max(until)));
    }

    pub fn is_retracted(&self, now: Ms) -> bool {
        if !self.autohide || self.hover || self.peek_until.is_some_and(|t| now < t) {
            return false;
        }
        matches!(self.presentation(now), Presentation::Idle | Presentation::Compact | Presentation::Minimal)
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
        if let Some(a) = &self.alert
            && now < a.until {
                return Presentation::Alert(a.kind.clone());
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

    pub fn click(&mut self, now: Ms) {
        if self.alert_active(now) {
            self.alert = None;
            self.expanded = true;
        } else {
            self.expanded = !self.expanded;
        }
    }

    pub fn long_press(&mut self) {
        self.expanded = true;
    }

    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    pub fn hotkey(&mut self) {
        self.snoozed_until = None;
        self.alert = None;
        self.expanded = !self.expanded;
    }

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

    pub fn push_alert(&mut self, kind: AlertKind, now: Ms) -> bool {
        if let Some(cur) = &self.alert
            && now < cur.until && kind.priority() > cur.kind.priority() {
                return false;
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
        if self.peek_until.is_some_and(|t| now >= t) {
            self.peek_until = None;
            changed = true;
        }
        changed
    }

    pub fn next_deadline(&self) -> Option<Ms> {
        [self.alert.as_ref().map(|a| a.until), self.snoozed_until, self.peek_until].into_iter().flatten().min()
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
    fn sembunyi_otomatis_ke_tepi_atas() {
        let mut i = Island::new(DisplayMode::Always);
        i.set_activity_count(1);
        assert!(!i.is_retracted(0), "tanpa autohide tidak pernah masuk");
        i.set_autohide(true);
        assert!(i.is_retracted(0));
        assert_eq!(i.presentation(0), Presentation::Compact, "isi tetap sama, hanya posisinya");
        i.set_hover(true);
        assert!(!i.is_retracted(0), "kursor mendekat: turun");
        i.set_hover(false);
        i.click(0);
        assert!(!i.is_retracted(0), "kartu terbuka selalu tampil");
        i.collapse();
        i.push_alert(AlertKind::Charging { percent: None }, 0);
        assert!(!i.is_retracted(100), "alert selalu menyembul");
        assert!(i.is_retracted(3_000));
    }

    #[test]
    fn peek_sebentar_lalu_masuk_lagi() {
        let mut i = Island::new(DisplayMode::Always);
        i.set_autohide(true);
        i.peek(2_500);
        i.peek(1_000);
        assert!(!i.is_retracted(2_000), "peek tidak dipersingkat");
        assert_eq!(i.next_deadline(), Some(2_500));
        assert!(i.tick(2_500));
        assert!(i.is_retracted(2_500));
    }

    #[test]
    fn autohide_tidak_menampilkan_yang_tersembunyi() {
        let mut i = Island::new(DisplayMode::ActivityOnly);
        i.set_autohide(true);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        assert!(!i.is_retracted(0), "tidak ada yang perlu disembunyikan");
    }

    #[test]
    fn kunci_mode() {
        assert_eq!(DisplayMode::from_key("HOVER"), Some(DisplayMode::Hover));
        assert_eq!(DisplayMode::from_key("activity"), Some(DisplayMode::ActivityOnly));
        assert_eq!(DisplayMode::from_key("kadang"), None);
    }
}
