//! Melacak unduhan browser dari berkas sementara di folder Downloads.
//!
//! Browser menulis unduhan ke berkas sementara (`.crdownload` di Chrome/Edge, `.part` di
//! Firefox, dan sejenisnya) lalu mengganti namanya setelah selesai. Ukuran totalnya tidak
//! diketahui, jadi yang ditampilkan adalah byte yang sudah masuk dan kecepatannya.
//! Ini heuristik, bukan data resmi dari browser.

use crate::Ms;

/// Akhiran berkas sementara milik browser populer.
pub const PARTIAL_EXTS: [&str; 5] = ["crdownload", "part", "partial", "download", "opdownload"];

/// Berkas sementara yang tidak bertambah selama ini dianggap terhenti (dijeda, gagal,
/// atau sisa unduhan lama) dan tidak ditampilkan.
pub const STALL_MS: Ms = 60_000;

const RATE_SAMPLE_MS: Ms = 500;

/// Satu berkas sementara hasil pindaian folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partial {
    pub name: String,
    pub size: u64,
    /// Sudah berapa lama berkas tidak diubah.
    pub idle_ms: Ms,
}

/// Nama akhir unduhan bila `name` adalah berkas sementara browser.
pub fn final_name(name: &str) -> Option<&str> {
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() || !PARTIAL_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
        return None;
    }
    Some(stem)
}

pub fn is_partial(name: &str) -> bool {
    final_name(name).is_some()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Download {
    pub partial: String,
    pub name: String,
    pub bytes: u64,
    /// Kecepatan dalam byte per detik (dihaluskan).
    pub rate: f64,
    pub started: Ms,
    sample: (Ms, u64),
    changed: Ms,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    pub name: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Downloads {
    items: Vec<Download>,
}

impl Downloads {
    pub fn new() -> Self {
        Self::default()
    }

    /// Terapkan hasil pindaian. `final_size` mengembalikan ukuran berkas akhir bila sudah ada.
    /// Mengembalikan unduhan yang baru selesai.
    pub fn update(&mut self, partials: &[Partial], now: Ms, final_size: impl Fn(&str) -> Option<u64>) -> Vec<Finished> {
        let mut done = Vec::new();
        self.items.retain(|d| {
            if partials.iter().any(|p| p.name == d.partial) {
                return true;
            }
            // Berkas sementara hilang: selesai bila berkas akhirnya ada, selain itu dibatalkan.
            if let Some(bytes) = final_size(&d.name) {
                done.push(Finished { name: d.name.clone(), bytes: bytes.max(d.bytes) });
            }
            false
        });
        for p in partials {
            let Some(name) = final_name(&p.name) else { continue };
            match self.items.iter_mut().find(|d| d.partial == p.name) {
                Some(d) => {
                    if p.size != d.bytes {
                        d.changed = now;
                    }
                    d.bytes = p.size;
                    let (at, bytes) = d.sample;
                    let dt = now.saturating_sub(at);
                    if dt >= RATE_SAMPLE_MS {
                        let inst = p.size.saturating_sub(bytes) as f64 * 1000.0 / dt as f64;
                        d.rate = if d.rate == 0.0 { inst } else { d.rate * 0.6 + inst * 0.4 };
                        d.sample = (now, p.size);
                    }
                }
                None if p.idle_ms < STALL_MS => self.items.push(Download {
                    partial: p.name.clone(),
                    name: name.to_string(),
                    bytes: p.size,
                    rate: 0.0,
                    started: now,
                    sample: (now, p.size),
                    changed: now.saturating_sub(p.idle_ms),
                }),
                None => {}
            }
        }
        done
    }

    /// Unduhan yang masih berjalan, yang terbaru lebih dulu.
    pub fn active(&self, now: Ms) -> Vec<&Download> {
        let mut v: Vec<&Download> = self.items.iter().filter(|d| now.saturating_sub(d.changed) < STALL_MS).collect();
        v.sort_by_key(|d| std::cmp::Reverse(d.started));
        v
    }

    pub fn is_busy(&self, now: Ms) -> bool {
        !self.active(now).is_empty()
    }

    /// Ada berkas sementara yang dilacak (berjalan atau terhenti); pindai ulang berkala.
    pub fn is_tracking(&self) -> bool {
        !self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(name: &str, size: u64) -> Partial {
        Partial { name: name.into(), size, idle_ms: 0 }
    }

    #[test]
    fn nama_akhir_berkas_sementara() {
        assert_eq!(final_name("film.mp4.crdownload"), Some("film.mp4"));
        assert_eq!(final_name("arsip.zip.PART"), Some("arsip.zip"));
        assert_eq!(final_name("catatan.txt"), None);
        assert_eq!(final_name(".part"), None);
        assert!(!is_partial("partial"));
    }

    #[test]
    fn chrome_selesai_saat_berganti_nama() {
        let mut d = Downloads::new();
        assert!(d.update(&[p("a.zip.crdownload", 1_000)], 0, |_| None).is_empty());
        d.update(&[p("a.zip.crdownload", 2_000_000)], 1_000, |_| None);
        let a = &d.active(1_000)[0];
        assert_eq!((a.name.as_str(), a.bytes), ("a.zip", 2_000_000));
        assert!(a.rate > 1_000_000.0, "kecepatan terukur: {}", a.rate);
        let done = d.update(&[], 2_000, |n| (n == "a.zip").then_some(3_000_000));
        assert_eq!(done, vec![Finished { name: "a.zip".into(), bytes: 3_000_000 }]);
        assert!(!d.is_busy(2_000));
    }

    #[test]
    fn firefox_placeholder_tidak_dihitung_selesai_lebih_awal() {
        let mut d = Downloads::new();
        d.update(&[p("b.iso.part", 10)], 0, |n| (n == "b.iso").then_some(0));
        assert!(d.is_busy(0), "berkas akhir kosong sudah ada, tetapi .part masih berjalan");
        let done = d.update(&[], 500, |n| (n == "b.iso").then_some(900));
        assert_eq!(done.len(), 1);
    }

    #[test]
    fn dibatalkan_tanpa_alert() {
        let mut d = Downloads::new();
        d.update(&[p("c.exe.crdownload", 10)], 0, |_| None);
        assert!(d.update(&[], 100, |_| None).is_empty());
        assert!(!d.is_tracking());
    }

    #[test]
    fn sisa_lama_dan_unduhan_terhenti_disembunyikan() {
        let mut d = Downloads::new();
        let old = Partial { name: "lama.zip.crdownload".into(), size: 5, idle_ms: STALL_MS + 1 };
        d.update(&[old], 0, |_| None);
        assert!(!d.is_busy(0), "sisa unduhan yang sudah lama tidak ikut tampil");
        d.update(&[p("baru.zip.crdownload", 5)], 0, |_| None);
        assert!(d.is_busy(0));
        d.update(&[p("baru.zip.crdownload", 5)], STALL_MS, |_| None);
        assert!(!d.is_busy(STALL_MS), "tidak bertambah selama semenit: dianggap terhenti");
        assert!(d.is_tracking(), "tetap dilacak agar selesai/lanjut terdeteksi");
        d.update(&[p("baru.zip.crdownload", 9)], STALL_MS + 10, |_| None);
        assert!(d.is_busy(STALL_MS + 10), "lanjut lagi");
    }

    #[test]
    fn terbaru_lebih_dulu() {
        let mut d = Downloads::new();
        d.update(&[p("1.bin.part", 1)], 0, |_| None);
        d.update(&[p("1.bin.part", 1), p("2.bin.part", 1)], 10, |_| None);
        let names: Vec<_> = d.active(10).iter().map(|x| x.name.clone()).collect();
        assert_eq!(names, vec!["2.bin", "1.bin"]);
    }
}
