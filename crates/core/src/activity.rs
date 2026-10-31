//! Aktivitas live dan aturan prioritas (PRD §3.4).
//!
//! Urutan: (P0) mikrofon/kamera, (P1) alert sistem, (P2) timer,
//! (P3) media, (P4) unduhan dan lainnya. Dua aktivitas teratas tampil sebagai
//! Minimal; sisanya masuk carousel yang bisa digeser.

/// Prioritas; angka lebih kecil berarti lebih penting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
    P4 = 4,
}

/// Jenis aktivitas yang dapat tampil di Island. Satu instans per jenis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityKind {
    Privacy,
    Timer,
    Stopwatch,
    Media,
    Download,
}

impl ActivityKind {
    pub fn priority(self) -> Priority {
        match self {
            ActivityKind::Privacy => Priority::P0,
            ActivityKind::Timer | ActivityKind::Stopwatch => Priority::P2,
            ActivityKind::Media => Priority::P3,
            ActivityKind::Download => Priority::P4,
        }
    }
}

/// Himpunan aktivitas yang sedang berjalan, terurut menurut prioritas lalu
/// waktu mulai, dengan rotasi carousel dari geseran pengguna.
#[derive(Debug, Clone, Default)]
pub struct Activities {
    /// Nomor urut masuk dipakai sebagai pemecah seri.
    items: Vec<(ActivityKind, u64)>,
    seq: u64,
    rotation: usize,
}

impl Activities {
    pub fn new() -> Self {
        Self::default()
    }

    /// Menandai aktivitas berjalan atau berhenti. Mengembalikan `true` bila
    /// himpunan berubah.
    pub fn set(&mut self, kind: ActivityKind, running: bool) -> bool {
        let pos = self.items.iter().position(|(k, _)| *k == kind);
        match (pos, running) {
            (None, true) => {
                self.seq += 1;
                self.items.push((kind, self.seq));
                self.items.sort_by_key(|(k, s)| (k.priority(), *s));
                true
            }
            (Some(i), false) => {
                self.items.remove(i);
                if self.rotation >= self.items.len() {
                    self.rotation = 0;
                }
                true
            }
            _ => false,
        }
    }

    pub fn contains(&self, kind: ActivityKind) -> bool {
        self.items.iter().any(|(k, _)| *k == kind)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Urutan tampil: P0 selalu di depan; sisanya dirotasi oleh carousel.
    pub fn ordered(&self) -> Vec<ActivityKind> {
        let (pinned, rest): (Vec<_>, Vec<_>) =
            self.items.iter().map(|(k, _)| *k).partition(|k| k.priority() == Priority::P0);
        let mut out = pinned;
        if !rest.is_empty() {
            let r = self.rotation % rest.len();
            out.extend(rest[r..].iter().chain(rest[..r].iter()).copied());
        }
        out
    }

    pub fn primary(&self) -> Option<ActivityKind> {
        self.ordered().first().copied()
    }

    pub fn secondary(&self) -> Option<ActivityKind> {
        self.ordered().get(1).copied()
    }

    /// Geser carousel ke aktivitas berikutnya (`+1`) atau sebelumnya (`-1`).
    pub fn rotate(&mut self, step: i32) {
        let n = self.items.iter().filter(|(k, _)| k.priority() != Priority::P0).count();
        if n < 2 {
            return;
        }
        let r = self.rotation as i64 + step as i64;
        self.rotation = r.rem_euclid(n as i64) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ActivityKind::*;

    #[test]
    fn urut_menurut_prioritas() {
        let mut a = Activities::new();
        a.set(Media, true);
        a.set(Download, true);
        a.set(Timer, true);
        assert_eq!(a.ordered(), vec![Timer, Media, Download]);
        a.set(Privacy, true);
        assert_eq!(a.primary(), Some(Privacy));
        assert_eq!(a.secondary(), Some(Timer));
    }

    #[test]
    fn seri_diputus_waktu_mulai() {
        let mut a = Activities::new();
        a.set(Stopwatch, true);
        a.set(Timer, true);
        assert_eq!(a.ordered(), vec![Stopwatch, Timer]);
    }

    #[test]
    fn set_idempoten() {
        let mut a = Activities::new();
        assert!(a.set(Media, true));
        assert!(!a.set(Media, true));
        assert!(a.set(Media, false));
        assert!(!a.set(Media, false));
        assert!(a.is_empty());
    }

    #[test]
    fn carousel_berputar_tanpa_menggeser_p0() {
        let mut a = Activities::new();
        a.set(Privacy, true);
        a.set(Timer, true);
        a.set(Media, true);
        a.rotate(1);
        assert_eq!(a.ordered(), vec![Privacy, Media, Timer]);
        a.rotate(1);
        assert_eq!(a.ordered(), vec![Privacy, Timer, Media]);
        a.rotate(-1);
        assert_eq!(a.ordered(), vec![Privacy, Media, Timer]);
    }

    #[test]
    fn rotasi_aman_saat_item_dihapus() {
        let mut a = Activities::new();
        a.set(Timer, true);
        a.set(Media, true);
        a.rotate(1);
        a.set(Media, false);
        assert_eq!(a.ordered(), vec![Timer]);
    }
}
