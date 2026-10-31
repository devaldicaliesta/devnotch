//! Ukuran dan tata letak Island dalam dp (PRD §3.1).
//!
//! Angka-angka ini adalah token awal yang meniru proporsi pada iPhone dan
//! akan dikalibrasi ulang; semua posisi kartu relatif terhadap sudut kiri
//! atas pil.

use crate::geom::{Point, Rect, Size};
use crate::island::AlertKind;

pub const IDLE: Size = Size::new(126.0, 37.0);
pub const PILL_H: f32 = 37.0;
pub const COMPACT_MIN_W: f32 = 160.0;
pub const COMPACT_MAX_W: f32 = 240.0;
pub const BUBBLE: Size = Size::new(37.0, 37.0);
pub const BUBBLE_GAP: f32 = 8.0;
pub const EXPANDED_W: f32 = 371.0;
pub const EXPANDED_MIN_H: f32 = 84.0;
pub const EXPANDED_MAX_H: f32 = 200.0;
pub const HUD: Size = Size::new(200.0, 37.0);
pub const HOVER_SCALE: f32 = 1.05;
/// Radius maksimum kartu; pil memakai busur lingkaran (tinggi / 2).
pub const MAX_RADIUS: f32 = 40.0;
/// Ruang kosong di sekitar pil di dalam jendela (untuk skala hover).
pub const WINDOW_PAD: f32 = 12.0;
pub const PAD: f32 = 18.0;

/// Isi kartu Expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    /// Kartu beranda saat tidak ada aktivitas: maskot, jam, timer cepat.
    Home,
    Media,
    Timer,
    Stopwatch,
}

impl Card {
    pub fn height(self) -> f32 {
        let h: f32 = match self {
            Card::Home => 132.0,
            Card::Media => 168.0,
            Card::Timer | Card::Stopwatch => 112.0,
        };
        h.clamp(EXPANDED_MIN_H, EXPANDED_MAX_H)
    }

    pub fn size(self) -> Size {
        Size::new(EXPANDED_W, self.height())
    }
}

pub fn alert_size(alert: &AlertKind) -> Size {
    if alert.is_hud() { HUD } else { Size::new(EXPANDED_W, EXPANDED_MIN_H) }
}

/// Lebar compact mengikuti isi, dibatasi 160–240 dp.
pub fn compact_size(content_w: f32) -> Size {
    Size::new(content_w.clamp(COMPACT_MIN_W, COMPACT_MAX_W), PILL_H)
}

/// Radius sudut untuk ukuran pil/kartu tertentu.
pub fn corner_radius(size: Size) -> f32 {
    (size.h / 2.0).min(MAX_RADIUS)
}

/// Bentuk yang digambar: pil utama dan (pada Minimal) gelembung kedua.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    pub main: Size,
    pub bubble: bool,
}

impl Shape {
    pub fn single(main: Size) -> Self {
        Self { main, bubble: false }
    }

    pub fn total_width(&self) -> f32 {
        if self.bubble { self.main.w + BUBBLE_GAP + BUBBLE.w } else { self.main.w }
    }

    /// Posisi pil utama di dalam jendela selebar `window_w`.
    pub fn main_rect(&self, window_w: f32, top: f32) -> Rect {
        let x = (window_w - self.total_width()) / 2.0;
        Rect::new(x, top, self.main.w, self.main.h)
    }

    pub fn bubble_rect(&self, window_w: f32, top: f32) -> Option<Rect> {
        self.bubble.then(|| {
            let m = self.main_rect(window_w, top);
            Rect::new(m.x + m.w + BUBBLE_GAP, top, BUBBLE.w, BUBBLE.h)
        })
    }
}

/// Ukuran jendela yang cukup untuk semua bentuk yang diberikan (bentuk lama
/// dan baru selama transisi), termasuk ruang untuk skala hover.
pub fn window_size(shapes: &[Shape], top_margin: f32) -> Size {
    let (w, h) = shapes
        .iter()
        .fold((0.0f32, 0.0f32), |(w, h), s| (w.max(s.total_width()), h.max(s.main.h)));
    Size::new(
        (w * HOVER_SCALE + 2.0 * WINDOW_PAD).ceil(),
        (top_margin + h * HOVER_SCALE + WINDOW_PAD).ceil(),
    )
}

/// Tombol yang dapat diklik di dalam kartu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Prev,
    PlayPause,
    Next,
    TimerAddMinute,
    TimerPauseResume,
    TimerCancel,
    StopwatchPauseResume,
    StopwatchReset,
    StopwatchStop,
    /// Mulai timer dengan preset ke-`n` dari konfigurasi.
    Preset(u8),
    StartStopwatch,
}

/// Tata letak kartu media (371 × 168).
#[derive(Debug, Clone, PartialEq)]
pub struct MediaCard {
    pub art: Rect,
    pub title: Rect,
    pub subtitle: Rect,
    pub source: Rect,
    pub elapsed: Rect,
    pub progress: Rect,
    pub remaining: Rect,
    pub buttons: Vec<(Button, Rect)>,
}

pub fn media_card() -> MediaCard {
    let w = EXPANDED_W;
    let art = Rect::new(PAD, PAD, 60.0, 60.0);
    let tx = art.x + art.w + 14.0;
    let tw = w - tx - PAD;
    let py = art.y + art.h + 18.0;
    let by = py + 16.0;
    let cx = w / 2.0;
    let btn = |c: f32| Rect::new(c - 20.0, by, 40.0, 40.0);
    MediaCard {
        art,
        title: Rect::new(tx, PAD + 4.0, tw, 22.0),
        subtitle: Rect::new(tx, PAD + 28.0, tw, 18.0),
        source: Rect::new(tx, PAD + 46.0, tw, 16.0),
        elapsed: Rect::new(PAD, py - 6.0, 40.0, 16.0),
        progress: Rect::new(PAD + 46.0, py, w - 2.0 * (PAD + 46.0), 4.0),
        remaining: Rect::new(w - PAD - 40.0, py - 6.0, 40.0, 16.0),
        buttons: vec![
            (Button::Prev, btn(cx - 72.0)),
            (Button::PlayPause, btn(cx)),
            (Button::Next, btn(cx + 72.0)),
        ],
    }
}

/// Tata letak kartu timer atau stopwatch (371 × 112).
#[derive(Debug, Clone, PartialEq)]
pub struct TimerCard {
    pub mascot: Rect,
    pub label: Rect,
    pub time: Rect,
    pub buttons: Vec<(Button, Rect)>,
}

pub fn timer_card(stopwatch: bool) -> TimerCard {
    let w = EXPANDED_W;
    let h = Card::Timer.height();
    let size = 44.0;
    let y = (h - size) / 2.0;
    let x3 = w - PAD - size;
    let x2 = x3 - size - 10.0;
    let x1 = x2 - size - 10.0;
    let buttons = if stopwatch {
        vec![
            (Button::StopwatchReset, Rect::new(x1, y, size, size)),
            (Button::StopwatchPauseResume, Rect::new(x2, y, size, size)),
            (Button::StopwatchStop, Rect::new(x3, y, size, size)),
        ]
    } else {
        vec![
            (Button::TimerAddMinute, Rect::new(x1, y, size, size)),
            (Button::TimerPauseResume, Rect::new(x2, y, size, size)),
            (Button::TimerCancel, Rect::new(x3, y, size, size)),
        ]
    };
    let mascot = Rect::new(PAD + 2.0, (h - 48.0) / 2.0, 48.0, 48.0);
    let tx = mascot.x + mascot.w + 14.0;
    TimerCard {
        mascot,
        label: Rect::new(tx, 24.0, x1 - tx - 8.0, 16.0),
        time: Rect::new(tx, 40.0, x1 - tx - 8.0, 44.0),
        buttons,
    }
}

/// Tata letak kartu beranda (371 × 132).
#[derive(Debug, Clone, PartialEq)]
pub struct HomeCard {
    pub mascot: Rect,
    pub clock: Rect,
    pub date: Rect,
    pub buttons: Vec<(Button, Rect)>,
}

/// `presets` adalah jumlah preset timer yang ditampilkan (maks. 5).
pub fn home_card(presets: usize) -> HomeCard {
    let w = EXPANDED_W;
    let mascot = Rect::new(PAD + 2.0, PAD, 52.0, 52.0);
    let tx = mascot.x + mascot.w + 16.0;
    let n = presets.min(5) + 1;
    let gap = 8.0;
    let bw = (w - 2.0 * PAD - gap * (n as f32 - 1.0)) / n as f32;
    let by = 132.0 - PAD - 30.0;
    let mut buttons: Vec<(Button, Rect)> = (0..n - 1)
        .map(|i| (Button::Preset(i as u8), Rect::new(PAD + i as f32 * (bw + gap), by, bw, 30.0)))
        .collect();
    buttons.push((
        Button::StartStopwatch,
        Rect::new(PAD + (n - 1) as f32 * (bw + gap), by, bw, 30.0),
    ));
    HomeCard {
        mascot,
        clock: Rect::new(tx, PAD - 2.0, w - tx - PAD, 34.0),
        date: Rect::new(tx, PAD + 34.0, w - tx - PAD, 18.0),
        buttons,
    }
}

/// Tata letak kartu alert (371 × 84) dan HUD (200 × 37).
#[derive(Debug, Clone, PartialEq)]
pub struct AlertCard {
    pub mascot: Rect,
    pub title: Rect,
    pub subtitle: Rect,
    pub value: Rect,
}

pub fn alert_card() -> AlertCard {
    let w = EXPANDED_W;
    let mascot = Rect::new(PAD + 2.0, (EXPANDED_MIN_H - 44.0) / 2.0, 44.0, 44.0);
    let tx = mascot.x + mascot.w + 14.0;
    AlertCard {
        mascot,
        title: Rect::new(tx, 20.0, w - tx - 100.0, 22.0),
        subtitle: Rect::new(tx, 44.0, w - tx - 100.0, 18.0),
        value: Rect::new(w - PAD - 84.0, 22.0, 84.0, 40.0),
    }
}

/// Slot kiri dan kanan pada pil compact.
pub fn compact_slots(size: Size) -> (Rect, Rect) {
    let s = 25.0;
    let y = (size.h - s) / 2.0;
    (Rect::new(8.0, y, s, s), Rect::new(size.w - 12.0 - s, y, s, s))
}

/// Tombol yang terkena titik `p` (koordinat relatif kartu).
pub fn hit(buttons: &[(Button, Rect)], p: Point) -> Option<Button> {
    // Area sentuh diperluas 4 dp agar mudah diklik dan disentuh.
    buttons.iter().find(|(_, r)| r.inflate(4.0).contains(p)).map(|(b, _)| *b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(outer: Size, r: &Rect) -> bool {
        r.x >= 0.0 && r.y >= 0.0 && r.x + r.w <= outer.w + 0.01 && r.y + r.h <= outer.h + 0.01
    }

    #[test]
    fn ukuran_sesuai_prd() {
        assert_eq!(IDLE, Size::new(126.0, 37.0));
        assert_eq!(compact_size(10.0).w, 160.0);
        assert_eq!(compact_size(999.0).w, 240.0);
        for c in [Card::Home, Card::Media, Card::Timer, Card::Stopwatch] {
            let h = c.height();
            assert!((EXPANDED_MIN_H..=EXPANDED_MAX_H).contains(&h), "{c:?}");
        }
    }

    #[test]
    fn pil_memakai_busur_lingkaran() {
        assert_eq!(corner_radius(IDLE), 18.5);
        assert_eq!(corner_radius(Card::Media.size()), MAX_RADIUS);
    }

    #[test]
    fn minimal_terpusat_dengan_gelembung() {
        let s = Shape { main: Size::new(160.0, 37.0), bubble: true };
        let win = 400.0;
        let m = s.main_rect(win, 8.0);
        let b = s.bubble_rect(win, 8.0).unwrap();
        assert_eq!(b.x + b.w - m.x, s.total_width());
        assert!(((m.x) - (win - (b.x + b.w))).abs() < 0.01, "harus simetris");
    }

    #[test]
    fn jendela_memuat_bentuk_terbesar() {
        let a = Shape::single(IDLE);
        let b = Shape::single(Card::Media.size());
        let w = window_size(&[a, b], 8.0);
        assert!(w.w >= EXPANDED_W * HOVER_SCALE);
        assert!(w.h >= 8.0 + Card::Media.height() * HOVER_SCALE);
    }

    #[test]
    fn semua_elemen_kartu_di_dalam_kartu() {
        let mc = media_card();
        let ms = Card::Media.size();
        for r in [mc.art, mc.title, mc.subtitle, mc.source, mc.progress, mc.elapsed, mc.remaining] {
            assert!(inside(ms, &r), "{r:?}");
        }
        for (_, r) in &mc.buttons {
            assert!(inside(ms, r), "{r:?}");
        }
        for sw in [false, true] {
            let tc = timer_card(sw);
            for (_, r) in &tc.buttons {
                assert!(inside(Card::Timer.size(), r));
            }
            assert!(inside(Card::Timer.size(), &tc.time));
        }
        let hc = home_card(4);
        assert_eq!(hc.buttons.len(), 5);
        for (_, r) in &hc.buttons {
            assert!(inside(Card::Home.size(), r), "{r:?}");
        }
        let ac = alert_card();
        for r in [ac.mascot, ac.title, ac.subtitle, ac.value] {
            assert!(inside(Size::new(EXPANDED_W, EXPANDED_MIN_H), &r), "{r:?}");
        }
    }

    #[test]
    fn tombol_tidak_bertumpuk() {
        let mc = media_card();
        for (i, (_, a)) in mc.buttons.iter().enumerate() {
            for (_, b) in mc.buttons.iter().skip(i + 1) {
                assert!(a.x + a.w <= b.x || b.x + b.w <= a.x);
            }
        }
    }

    #[test]
    fn hit_test_tombol_media() {
        let mc = media_card();
        let play = mc.buttons[1].1.center();
        assert_eq!(hit(&mc.buttons, play), Some(Button::PlayPause));
        assert_eq!(hit(&mc.buttons, Point::new(1.0, 1.0)), None);
    }
}
