use crate::Ms;
use crate::activity::Priority;
use crate::bluetooth::DeviceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Behavior {
    #[default]
    Autohide,
    Sticky,
    Hide,
}

impl Behavior {
    pub const ALL: [Behavior; 3] = [Behavior::Autohide, Behavior::Sticky, Behavior::Hide];

    pub fn key(self) -> &'static str {
        match self {
            Behavior::Autohide => "autohide",
            Behavior::Sticky => "sticky",
            Behavior::Hide => "hide",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|b| *b == self).unwrap_or(0)
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
    Brightness { level: u8 },
    DownloadDone { name: String, bytes: u64 },
    Bluetooth { name: String, kind: DeviceKind, connected: bool, battery: Option<u8> },
}

impl AlertKind {
    pub fn priority(&self) -> Priority {
        match self {
            AlertKind::Volume { .. } | AlertKind::Brightness { .. } => Priority::P4,
            AlertKind::DownloadDone { .. } | AlertKind::Bluetooth { .. } => Priority::P3,
            AlertKind::TimerDone => Priority::P2,
            _ => Priority::P1,
        }
    }

    pub fn duration(&self) -> Ms {
        match self {
            AlertKind::Volume { .. } | AlertKind::Brightness { .. } => 1_500,
            AlertKind::Charging { .. } | AlertKind::Unplugged { .. } | AlertKind::Bluetooth { .. } => 3_000,
            AlertKind::DownloadDone { .. } => 4_000,
            AlertKind::LowBattery { .. } | AlertKind::TimerDone => 5_000,
        }
    }

    pub fn is_hud(&self) -> bool {
        matches!(self, AlertKind::Volume { .. } | AlertKind::Brightness { .. })
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
    behavior: Behavior,
    expanded: bool,
    activity_count: usize,
    alert: Option<ActiveAlert>,
    fullscreen: bool,
    covered: bool,
    hover: bool,
    snoozed_until: Option<Ms>,
    peek_until: Option<Ms>,
    idle_pinned: bool,
}

impl Island {
    pub fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            expanded: false,
            activity_count: 0,
            alert: None,
            fullscreen: false,
            covered: false,
            hover: false,
            snoozed_until: None,
            peek_until: None,
            idle_pinned: false,
        }
    }

    pub fn behavior(&self) -> Behavior {
        self.behavior
    }

    pub fn set_behavior(&mut self, behavior: Behavior) {
        self.behavior = behavior;
    }

    pub fn autohide(&self) -> bool {
        self.behavior == Behavior::Autohide
    }

    pub fn set_covered(&mut self, covered: bool) {
        self.covered = covered;
        if covered {
            self.peek_until = None;
        }
    }

    pub fn is_covered(&self) -> bool {
        self.covered
    }

    /// Menempel berarti selalu tampil, jadi jendela di bawahnya tidak membuatnya menyingkir.
    pub fn tucks(&self) -> bool {
        self.autohide() || (self.covered && self.behavior != Behavior::Sticky)
    }

    pub fn peek(&mut self, until: Ms) {
        self.peek_until = Some(self.peek_until.map_or(until, |p| p.max(until)));
    }

    /// Saat diam (tanpa aktivitas), Island tetap tampil walau perilakunya otomatis
    /// atau ada jendela di bawahnya. Dipakai maskot yang menemani di samping layar.
    pub fn set_idle_pinned(&mut self, pinned: bool) {
        self.idle_pinned = pinned;
    }

    pub fn is_retracted(&self, now: Ms) -> bool {
        if !self.tucks() || self.hover || self.peek_until.is_some_and(|t| now < t) {
            return false;
        }
        match self.presentation(now) {
            Presentation::Idle => !self.idle_pinned,
            Presentation::Compact | Presentation::Minimal => true,
            _ => false,
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn presentation(&self, now: Ms) -> Presentation {
        if self.fullscreen || self.snoozed_until.is_some_and(|t| now < t) {
            return Presentation::Hidden;
        }
        if let Some(a) = self.shown_alert(now) {
            return Presentation::Alert(a.clone());
        }
        if self.expanded {
            return Presentation::Expanded;
        }
        match self.activity_count {
            0 => match self.behavior {
                Behavior::Autohide | Behavior::Sticky => Presentation::Idle,
                Behavior::Hide => Presentation::Hidden,
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
        if self.shown_alert(now).is_some() {
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

    /// Alert yang benar-benar tampil. HUD volume tidak menimpa kartu yang sedang terbuka:
    /// tanpa ini, scroll di atas kartu mengecilkannya menjadi HUD, kursor keluar dari pil,
    /// lalu kartu ikut tertutup.
    fn shown_alert(&self, now: Ms) -> Option<&AlertKind> {
        self.alert.as_ref().filter(|a| now < a.until && !(self.expanded && a.kind.is_hud())).map(|a| &a.kind)
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
    fn idle_mengikuti_perilaku() {
        let mut i = Island::new(Behavior::Sticky);
        assert_eq!(i.presentation(0), Presentation::Idle);
        i.set_behavior(Behavior::Autohide);
        assert_eq!(i.presentation(0), Presentation::Idle);
        i.set_behavior(Behavior::Hide);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        i.set_hover(true);
        assert_eq!(i.presentation(0), Presentation::Hidden, "hide tidak muncul hanya karena kursor");
    }

    #[test]
    fn jumlah_aktivitas_menentukan_bentuk() {
        let mut i = Island::new(Behavior::Hide);
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
        let mut i = Island::new(Behavior::Sticky);
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
        let mut i = Island::new(Behavior::Sticky);
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
        let mut i = Island::new(Behavior::Sticky);
        assert!(i.push_alert(AlertKind::LowBattery { percent: 10, critical: true }, 0));
        assert!(!i.push_alert(AlertKind::Volume { level: 30, muted: false }, 100));
        assert!(i.push_alert(charging(), 200));
        assert!(i.push_alert(AlertKind::Volume { level: 30, muted: false }, 10_000));
    }

    #[test]
    fn hud_volume_tidak_menutup_kartu() {
        let mut i = Island::new(Behavior::Sticky);
        i.set_activity_count(1);
        i.click(0);
        assert!(i.push_alert(AlertKind::Volume { level: 40, muted: false }, 10));
        assert_eq!(i.presentation(20), Presentation::Expanded, "kartu tetap terbuka saat volume diubah");
        i.click(30);
        assert!(!i.is_expanded(), "klik tetap menutup kartu, bukan menelan alert");
        assert_eq!(
            i.presentation(40),
            Presentation::Alert(AlertKind::Volume { level: 40, muted: false }),
            "HUD tampil setelah kartu ditutup"
        );
        assert_eq!(i.presentation(2_000), Presentation::Compact);
    }

    #[test]
    fn prioritas_alert_baru() {
        let mut i = Island::new(Behavior::Sticky);
        let done = AlertKind::DownloadDone { name: "a.zip".into(), bytes: 1 };
        assert!(i.push_alert(done.clone(), 0));
        assert!(!i.push_alert(AlertKind::Brightness { level: 50 }, 10), "HUD kecerahan tidak menimpa unduhan selesai");
        let bt = AlertKind::Bluetooth { name: "Buds".into(), kind: DeviceKind::Audio, connected: true, battery: Some(80) };
        assert!(i.push_alert(bt.clone(), 20), "prioritas sama: yang terbaru tampil");
        assert_eq!(i.presentation(30), Presentation::Alert(bt));
        assert!(AlertKind::Brightness { level: 1 }.is_hud());
    }

    #[test]
    fn klik_alert_membuka_kartu() {
        let mut i = Island::new(Behavior::Sticky);
        i.push_alert(AlertKind::TimerDone, 0);
        i.click(10);
        assert_eq!(i.presentation(10), Presentation::Expanded);
    }

    #[test]
    fn layar_penuh_menyembunyikan_dan_kembali() {
        let mut i = Island::new(Behavior::Sticky);
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
        let mut i = Island::new(Behavior::Sticky);
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
        let mut i = Island::new(Behavior::Sticky);
        i.set_activity_count(1);
        assert!(!i.is_retracted(0), "sticky tidak pernah masuk");
        i.set_behavior(Behavior::Autohide);
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
        let mut i = Island::new(Behavior::Autohide);
        i.peek(2_500);
        i.peek(1_000);
        assert!(!i.is_retracted(2_000), "peek tidak dipersingkat");
        assert_eq!(i.next_deadline(), Some(2_500));
        assert!(i.tick(2_500));
        assert!(i.is_retracted(2_500));
    }

    #[test]
    fn menyingkir_saat_jendela_menempati_tepi_atas() {
        let mut sticky = Island::new(Behavior::Sticky);
        sticky.set_activity_count(1);
        sticky.set_covered(true);
        assert!(!sticky.tucks() && !sticky.is_retracted(0), "menempel tetap tampil di atas browser");
        sticky.set_activity_count(0);
        assert!(!sticky.is_retracted(0));

        let mut i = Island::new(Behavior::Hide);
        i.set_activity_count(1);
        assert!(!i.is_retracted(0));
        i.set_covered(true);
        assert!(i.tucks() && i.is_covered());
        assert!(i.is_retracted(0), "browser maximize: pil masuk ke tepi agar tab tidak tertutup");
        assert_eq!(i.presentation(0), Presentation::Compact, "isi tetap, hanya posisinya");
        i.set_hover(true);
        assert!(!i.is_retracted(0), "kursor ditahan di notch: turun");
        i.set_hover(false);
        i.click(0);
        assert!(!i.is_retracted(0), "kartu terbuka tetap tampil");
        i.collapse();
        i.push_alert(charging(), 0);
        assert!(!i.is_retracted(100), "alert tetap menyembul");
        i.set_covered(false);
        assert!(!i.is_retracted(5_000), "kembali ke desktop: menempel lagi");

        let mut h = Island::new(Behavior::Hide);
        h.set_covered(true);
        assert_eq!(h.presentation(0), Presentation::Hidden);
        assert!(!h.is_retracted(0));
        h.set_activity_count(1);
        assert!(h.is_retracted(0), "aktivitas di atas browser cukup tampil sebagai bibir");
    }

    #[test]
    fn maskot_samping_tetap_tampil_saat_diam() {
        let mut i = Island::new(Behavior::Autohide);
        i.set_idle_pinned(true);
        assert!(!i.is_retracted(0), "maskot di samping tidak masuk ke tepi");
        i.set_covered(true);
        assert!(!i.is_retracted(0), "tetap menemani di atas browser");
        i.set_activity_count(1);
        assert!(i.is_retracted(0), "aktivitas tetap mengikuti aturan menyingkir");
        i.set_activity_count(0);
        i.set_idle_pinned(false);
        assert!(i.is_retracted(0));
        let mut h = Island::new(Behavior::Hide);
        h.set_idle_pinned(true);
        assert_eq!(h.presentation(0), Presentation::Hidden, "perilaku sembunyi tetap dihormati");
    }

    #[test]
    fn tertutup_membatalkan_peek() {
        let mut i = Island::new(Behavior::Autohide);
        i.set_activity_count(1);
        i.peek(2_500);
        assert!(!i.is_retracted(100));
        i.set_covered(true);
        assert!(i.is_retracted(100), "aktivitas baru tidak menyembul di atas tab");
        assert_eq!(i.next_deadline(), None);
    }

    #[test]
    fn hide_muncul_hanya_saat_aktivitas() {
        let mut i = Island::new(Behavior::Hide);
        assert_eq!(i.presentation(0), Presentation::Hidden);
        assert!(!i.is_retracted(0), "tidak ada yang perlu disembunyikan");
        i.set_activity_count(1);
        assert_eq!(i.presentation(0), Presentation::Compact);
        assert!(!i.is_retracted(0), "aktivitas tampil penuh, tidak masuk tepi");
        i.set_activity_count(0);
        i.push_alert(charging(), 0);
        assert_eq!(i.presentation(10), Presentation::Alert(charging()));
    }

    #[test]
    fn kunci_perilaku() {
        assert_eq!(Behavior::from_key("STICKY"), Some(Behavior::Sticky));
        assert_eq!(Behavior::from_key("hide"), Some(Behavior::Hide));
        assert_eq!(Behavior::from_key("kadang"), None);
        assert_eq!(Behavior::default(), Behavior::Autohide);
        assert_eq!(Behavior::Hide.index(), 2);
    }
}
