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
pub const EXPANDED_MAX_H: f32 = 240.0;
pub const HUD: Size = Size::new(200.0, 37.0);
pub const HOVER_SCALE: f32 = 1.05;
pub const MAX_RADIUS: f32 = 40.0;
pub const WINDOW_PAD: f32 = 12.0;
pub const PAD: f32 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    Home,
    Media,
    Lyrics,
    Timer,
    Stopwatch,
}

impl Card {
    pub fn height(self) -> f32 {
        let h: f32 = match self {
            Card::Home => 132.0,
            Card::Media => 168.0,
            Card::Lyrics => 240.0,
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

pub fn compact_size(content_w: f32) -> Size {
    Size::new(content_w.clamp(COMPACT_MIN_W, COMPACT_MAX_W), PILL_H)
}

pub fn corner_radius(size: Size) -> f32 {
    (size.h / 2.0).min(MAX_RADIUS)
}

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

    pub fn main_rect(&self, window_w: f32) -> Rect {
        let x = (window_w - self.total_width()) / 2.0;
        Rect::new(x, 0.0, self.main.w, self.main.h)
    }

    pub fn bubble_rect(&self, window_w: f32) -> Option<Rect> {
        self.bubble.then(|| {
            let m = self.main_rect(window_w);
            Rect::new(m.x + m.w + BUBBLE_GAP, 0.0, BUBBLE.w, BUBBLE.h)
        })
    }

    pub fn retracted(&self) -> Shape {
        Shape::single(Size::new((self.total_width() * LIP_RATIO).max(LIP_MIN_W).round(), LIP))
    }
}

pub const FILLET: f32 = 10.0;
pub const LIP: f32 = 4.0;
const LIP_RATIO: f32 = 0.5;
const LIP_MIN_W: f32 = 56.0;
pub const NEAR_X: f32 = 40.0;
pub const NEAR_Y: f32 = 10.0;

pub fn near_zone(shape: &Shape) -> Size {
    Size::new((shape.total_width() + 2.0 * NEAR_X).ceil(), NEAR_Y)
}

pub fn hit_box(shapes: &[Shape]) -> Size {
    let (w, h) = shapes
        .iter()
        .fold((0.0f32, 0.0f32), |(w, h), s| (w.max(s.total_width()), h.max(s.main.h)));
    Size::new(
        (w * HOVER_SCALE + 2.0 * (WINDOW_PAD + FILLET)).ceil(),
        (h * HOVER_SCALE + WINDOW_PAD).ceil(),
    )
}

pub fn union(a: Size, b: Size) -> Size {
    Size::new(a.w.max(b.w), a.h.max(b.h))
}

pub fn window_max() -> Size {
    let card = Shape::single(Size::new(EXPANDED_W, EXPANDED_MAX_H));
    let minimal = Shape { main: compact_size(COMPACT_MAX_W), bubble: true };
    union(hit_box(&[card, minimal]), near_zone(&card))
}

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
    Preset(u8),
    StartStopwatch,
    Lyrics,
    Mascot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaCard {
    pub art: Rect,
    pub title: Rect,
    pub subtitle: Rect,
    pub source: Rect,
    pub elapsed: Rect,
    pub progress: Rect,
    pub remaining: Rect,
    pub lyrics: Option<Rect>,
    pub buttons: Vec<(Button, Rect)>,
}

pub const LYRICS_H: f32 = 58.0;

pub fn media_card(lyrics: bool) -> MediaCard {
    let w = EXPANDED_W;
    let art = Rect::new(PAD, PAD, 60.0, 60.0);
    let tx = art.x + art.w + 14.0;
    let tw = w - tx - PAD - 34.0;
    let panel = lyrics.then(|| Rect::new(PAD, art.y + art.h + 14.0, w - 2.0 * PAD, LYRICS_H));
    let py = panel.map_or(art.y + art.h + 18.0, |p| p.y + p.h + 18.0);
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
        lyrics: panel,
        buttons: vec![
            (Button::Prev, btn(cx - 72.0)),
            (Button::PlayPause, btn(cx)),
            (Button::Next, btn(cx + 72.0)),
            (Button::Lyrics, Rect::new(w - PAD - 30.0, PAD + 2.0, 30.0, 30.0)),
        ],
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct HomeCard {
    pub mascot: Rect,
    pub clock: Rect,
    pub date: Rect,
    pub buttons: Vec<(Button, Rect)>,
}

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

pub fn compact_slots(size: Size) -> (Rect, Rect) {
    let s = 25.0;
    let y = (size.h - s) / 2.0;
    (Rect::new(8.0, y, s, s), Rect::new(size.w - 12.0 - s, y, s, s))
}

pub fn hit(buttons: &[(Button, Rect)], p: Point) -> Option<Button> {
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
        for c in [Card::Home, Card::Media, Card::Lyrics, Card::Timer, Card::Stopwatch] {
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
        let m = s.main_rect(win);
        let b = s.bubble_rect(win).unwrap();
        assert_eq!(b.x + b.w - m.x, s.total_width());
        assert!(((m.x) - (win - (b.x + b.w))).abs() < 0.01, "harus simetris");
        assert_eq!((m.y, b.y), (0.0, 0.0), "menempel di tepi atas");
    }

    #[test]
    fn region_memuat_bentuk_terbesar() {
        let a = Shape::single(IDLE);
        let b = Shape::single(Card::Media.size());
        let w = hit_box(&[a, b]);
        assert!(w.w >= EXPANDED_W * HOVER_SCALE + 2.0 * FILLET);
        assert!(w.h >= Card::Media.height() * HOVER_SCALE);
        let max = window_max();
        assert!(max.w >= w.w && max.h >= w.h, "jendela tetap memuat semua region");
        assert!(max.w >= near_zone(&b).w);
    }

    #[test]
    fn bibir_dan_zona_dekat() {
        let s = Shape::single(compact_size(200.0));
        let r = s.retracted();
        assert_eq!(r.main.h, LIP, "hanya bibir yang terlihat");
        assert!(r.main.w < s.main.w && !r.bubble);
        let m = Shape { main: compact_size(0.0), bubble: true };
        assert!(m.retracted().main.w <= m.total_width());
        let z = near_zone(&s);
        assert!(z.w > s.total_width() && z.h >= LIP);
        assert!(near_zone(&m).w > m.total_width());
        let open = union(hit_box(&[s]), near_zone(&s));
        assert!(open.w >= z.w && open.h >= z.h);
    }

    #[test]
    fn semua_elemen_kartu_di_dalam_kartu() {
        for (lyrics, card) in [(false, Card::Media), (true, Card::Lyrics)] {
            let mc = media_card(lyrics);
            let ms = card.size();
            for r in [mc.art, mc.title, mc.subtitle, mc.source, mc.progress, mc.elapsed, mc.remaining] {
                assert!(inside(ms, &r), "{r:?}");
            }
            for (_, r) in &mc.buttons {
                assert!(inside(ms, r), "{r:?}");
                assert!(r.y + r.h <= ms.h - 12.0, "{r:?}");
            }
            assert_eq!(mc.lyrics.is_some(), lyrics);
            if let Some(p) = mc.lyrics {
                assert!(inside(ms, &p) && p.y >= mc.art.y + mc.art.h && p.y + p.h < mc.progress.y);
            }
            let (_, lb) = mc.buttons.iter().find(|(b, _)| *b == Button::Lyrics).unwrap();
            assert!(mc.title.x + mc.title.w <= lb.x, "judul tidak menabrak tombol lirik");
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
        let mc = media_card(true);
        for (i, (_, a)) in mc.buttons.iter().enumerate() {
            for (_, b) in mc.buttons.iter().skip(i + 1) {
                assert!(a.x + a.w <= b.x || b.x + b.w <= a.x);
            }
        }
    }

    #[test]
    fn hit_test_tombol_media() {
        let mc = media_card(false);
        let play = mc.buttons[1].1.center();
        assert_eq!(hit(&mc.buttons, play), Some(Button::PlayPause));
        assert_eq!(hit(&mc.buttons, Point::new(1.0, 1.0)), None);
    }
}
