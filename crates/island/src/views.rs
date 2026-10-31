//! Isi Island untuk setiap state (PRD §3.1–3.2).
//!
//! Setiap fungsi `build_*` menggambar ke container milik pil dan
//! mengembalikan elemen dinamis (teks hitung mundur, progres) yang
//! diperbarui tanpa membangun ulang seluruh isi.

use notchi_core::Ms;
use notchi_core::color::{self, Color, Skin, TEXT_PRIMARY};
use notchi_core::fmt::{countdown, elapsed};
use notchi_core::geom::{Rect, Size};
use notchi_core::i18n::{Lang, Text, long_date, t};
use notchi_core::island::AlertKind;
use notchi_core::layout::{self, Button, Card};
use notchi_core::mascot::Expression;
use notchi_core::media::{Artwork, MediaSnapshot};
use notchi_core::timer::{Countdown, Stopwatch};
use notchi_platform_win::Result;
use notchi_platform_win::comp::{ContainerVisual, SpriteVisual, v2};
use notchi_platform_win::gfx::{TextStyle, Weight};
use notchi_platform_win::shell::LocalTime;

use crate::ui::{Align, DynText, Ui, mascot, waveform};

// Glyph Segoe Fluent Icons (font sistem Windows 11, tidak dibundel).
pub const G_PREV: &str = "\u{E892}";
pub const G_NEXT: &str = "\u{E893}";
pub const G_PLAY: &str = "\u{E768}";
pub const G_PAUSE: &str = "\u{E769}";
pub const G_CANCEL: &str = "\u{E711}";
pub const G_RESET: &str = "\u{E72C}";
pub const G_STOP: &str = "\u{E71A}";
pub const G_STOPWATCH: &str = "\u{E916}";
pub const G_MUSIC: &str = "\u{E8D6}";
pub const G_VOLUME: &str = "\u{E767}";
pub const G_MUTE: &str = "\u{E74F}";

/// Aktivitas yang dapat mengisi slot pil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Media,
    Timer,
    Stopwatch,
}

/// Data yang dibaca oleh view.
#[derive(Debug, Clone, Copy)]
pub struct Model<'a> {
    pub lang: Lang,
    pub now: Ms,
    pub skin: Skin,
    pub mascot_on: bool,
    pub expression: Expression,
    pub media: Option<&'a MediaSnapshot>,
    pub art: Option<&'a Artwork>,
    pub timer: Option<&'a Countdown>,
    pub stopwatch: Option<&'a Stopwatch>,
    pub presets: &'a [u32],
    pub clock: LocalTime,
}

/// Elemen dinamis dari isi yang sedang tampil.
#[derive(Debug, Default)]
pub struct Dyn {
    pub timer: Option<DynText>,
    pub stopwatch: Option<DynText>,
    pub elapsed: Option<DynText>,
    pub remaining: Option<DynText>,
    pub progress: Option<(SpriteVisual, f32)>,
    pub clock: Option<DynText>,
    pub hud_bar: Option<(SpriteVisual, f32)>,
    pub hud_text: Option<DynText>,
    pub buttons: Vec<(Button, Rect)>,
}

fn white(alpha: f32) -> Color {
    TEXT_PRIMARY.with_opacity(alpha)
}

fn timer_color() -> Color {
    color::accent_alert()
}

pub fn timer_text(t: &Countdown, now: Ms) -> String {
    countdown(t.remaining(now))
}

pub fn stopwatch_text(s: &Stopwatch, now: Ms) -> String {
    elapsed(s.elapsed(now))
}

fn compact_width(slot: Slot) -> f32 {
    match slot {
        Slot::Media => 200.0,
        Slot::Timer | Slot::Stopwatch => 180.0,
    }
}

pub fn compact_size(slot: Slot, minimal: bool) -> Size {
    if minimal { layout::compact_size(0.0) } else { layout::compact_size(compact_width(slot)) }
}

/// Ikon kecil di slot kiri: maskot, sampul, atau glyph.
fn left_icon(ui: &Ui, parent: &ContainerVisual, rect: Rect, slot: Slot, m: &Model<'_>) -> Result<()> {
    match (slot, m.art) {
        (Slot::Media, Some(art)) => {
            ui.image(parent, art, rect, 6.0)?;
        }
        _ if m.mascot_on => {
            mascot(ui, parent, rect, m.skin, m.expression)?;
        }
        _ => {
            let glyph = if slot == Slot::Media { G_MUSIC } else { G_STOPWATCH };
            let style = TextStyle::new(rect.h * 0.62, Weight::Regular, m.skin.color()).icon().centered();
            ui.text(parent, glyph, style, rect, Align::Center)?;
        }
    }
    Ok(())
}

pub fn build_compact(ui: &Ui, parent: &ContainerVisual, slot: Slot, size: Size, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let (left, right) = layout::compact_slots(size);
    left_icon(ui, parent, left, slot, m)?;
    let text_rect = Rect::new(left.x + left.w + 8.0, 0.0, size.w - left.w - 8.0 - 12.0 - 8.0, size.h);
    match slot {
        Slot::Media => {
            let playing = m.media.is_some_and(|s| s.is_playing());
            waveform(ui, parent, right, m.skin.color(), playing)?;
        }
        Slot::Timer => {
            if let Some(timer) = m.timer {
                let color = if timer.is_running() { timer_color() } else { white(0.6) };
                let style = TextStyle::new(15.0, Weight::Semibold, color).tabular();
                d.timer = Some(ui.text(parent, &timer_text(timer, m.now), style, text_rect, Align::Right)?);
            }
        }
        Slot::Stopwatch => {
            if let Some(sw) = m.stopwatch {
                let style = TextStyle::new(15.0, Weight::Semibold, TEXT_PRIMARY).tabular();
                d.stopwatch = Some(ui.text(parent, &stopwatch_text(sw, m.now), style, text_rect, Align::Right)?);
            }
        }
    }
    Ok(())
}

/// Gelembung kecil aktivitas kedua pada state Minimal.
pub fn build_bubble(ui: &Ui, parent: &ContainerVisual, slot: Slot, m: &Model<'_>) -> Result<()> {
    let r = Rect::new(6.0, 6.0, 25.0, 25.0);
    match (slot, m.art) {
        (Slot::Media, Some(art)) => {
            ui.image(parent, art, r, 12.5)?;
        }
        _ => {
            let (glyph, c) = match slot {
                Slot::Media => (G_MUSIC, m.skin.color()),
                Slot::Timer => (G_STOPWATCH, timer_color()),
                Slot::Stopwatch => (G_STOPWATCH, TEXT_PRIMARY),
            };
            let style = TextStyle::new(15.0, Weight::Regular, c).icon().centered();
            ui.text(parent, glyph, style, r, Align::Center)?;
        }
    }
    Ok(())
}

fn button_colors(enabled: bool) -> (Color, Color) {
    if enabled { (white(0.14), TEXT_PRIMARY) } else { (white(0.06), white(0.3)) }
}

pub fn build_media_card(ui: &Ui, parent: &ContainerVisual, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::media_card();
    let snap = m.media;
    match m.art {
        Some(art) => {
            ui.image(parent, art, lay.art, 12.0)?;
        }
        None => {
            ui.rounded(parent, lay.art, 12.0, white(0.1))?;
            if m.mascot_on {
                let inner = lay.art.inflate(-10.0);
                mascot(ui, parent, inner, m.skin, m.expression)?;
            } else {
                let style = TextStyle::new(24.0, Weight::Regular, white(0.6)).icon().centered();
                ui.text(parent, G_MUSIC, style, lay.art, Align::Center)?;
            }
        }
    }
    let title = snap.map(|s| s.title.as_str()).filter(|s| !s.is_empty()).unwrap_or(t(m.lang, Text::NothingPlaying));
    ui.text(parent, title, TextStyle::new(15.0, Weight::Semibold, TEXT_PRIMARY), lay.title, Align::Left)?;
    if let Some(s) = snap {
        let sub = if s.artist.is_empty() { s.album.as_str() } else { s.artist.as_str() };
        if !sub.is_empty() {
            ui.text(parent, sub, TextStyle::new(13.0, Weight::Regular, color::text_secondary()), lay.subtitle, Align::Left)?;
        }
        let mut source = s.source_name();
        if s.session_count > 1 {
            source = format!("{source} · {}/{}", s.session_index + 1, s.session_count);
        }
        ui.text(parent, &source, TextStyle::new(11.0, Weight::Medium, white(0.45)), lay.source, Align::Left)?;
    }

    // Progres: batang isi dianimasikan linear oleh compositor sampai lagu
    // selesai, jadi thread UI hanya bangun 1 Hz untuk teks waktu.
    if let Some(s) = snap.filter(|s| s.timeline.is_some()) {
        ui.rounded(parent, lay.progress, 2.0, white(0.2))?;
        let fill = ui.bar(parent, lay.progress, TEXT_PRIMARY)?;
        let geom = ui.c.CreateRoundedRectangleGeometry()?;
        geom.SetSize(v2(lay.progress.w, lay.progress.h))?;
        geom.SetCornerRadius(v2(2.0, 2.0))?;
        fill.SetClip(&ui.c.CreateGeometricClipWithGeometry(&geom)?)?;
        d.progress = Some((fill, lay.progress.w));
        let small = TextStyle::new(11.0, Weight::Medium, white(0.6)).tabular();
        let (el, rem) = media_times(s, m.now);
        d.elapsed = Some(ui.text(parent, &el, small, lay.elapsed, Align::Left)?);
        d.remaining = Some(ui.text(parent, &rem, small, lay.remaining, Align::Right)?);
    }

    let c = snap.map(|s| s.controls).unwrap_or_default();
    let playing = snap.is_some_and(|s| s.is_playing());
    for (b, r) in &lay.buttons {
        let (glyph, enabled) = match b {
            Button::Prev => (G_PREV, c.prev),
            Button::Next => (G_NEXT, c.next),
            _ if playing => (G_PAUSE, c.pause),
            _ => (G_PLAY, c.play),
        };
        let (bg, fg) = button_colors(enabled);
        ui.button(parent, *r, glyph, true, bg, fg)?;
        // Tombol yang tidak didukung sesi tidak bisa diklik (PRD §3.6).
        if enabled {
            d.buttons.push((*b, *r));
        }
    }
    Ok(())
}

pub fn media_times(s: &MediaSnapshot, now: Ms) -> (String, String) {
    match (s.position_at(now), s.timeline) {
        (Some(pos), Some(tl)) => (elapsed(pos), format!("-{}", elapsed(tl.end.saturating_sub(pos)))),
        _ => (String::new(), String::new()),
    }
}

pub fn build_timer_card(ui: &Ui, parent: &ContainerVisual, stopwatch: bool, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::timer_card(stopwatch);
    if m.mascot_on {
        mascot(ui, parent, lay.mascot, m.skin, m.expression)?;
    } else {
        let style = TextStyle::new(30.0, Weight::Regular, m.skin.color()).icon().centered();
        ui.text(parent, G_STOPWATCH, style, lay.mascot, Align::Center)?;
    }
    let (label, running) = if stopwatch {
        (t(m.lang, Text::StopwatchLabel), m.stopwatch.is_some_and(|s| s.is_running()))
    } else {
        let running = m.timer.is_some_and(|t| t.is_running());
        (t(m.lang, if running { Text::TimerLabel } else { Text::TimerPaused }), running)
    };
    ui.text(parent, label, TextStyle::new(13.0, Weight::Medium, color::text_secondary()), lay.label, Align::Left)?;
    let big = |c: Color| TextStyle::new(34.0, Weight::Semibold, c).tabular();
    if stopwatch {
        if let Some(sw) = m.stopwatch {
            d.stopwatch = Some(ui.text(parent, &stopwatch_text(sw, m.now), big(TEXT_PRIMARY), lay.time, Align::Left)?);
        }
    } else if let Some(timer) = m.timer {
        let c = if running { timer_color() } else { white(0.6) };
        d.timer = Some(ui.text(parent, &timer_text(timer, m.now), big(c), lay.time, Align::Left)?);
    }
    for (b, r) in &lay.buttons {
        let (label, icon, bg, fg) = match b {
            Button::TimerAddMinute => ("+1", false, white(0.14), TEXT_PRIMARY),
            Button::TimerPauseResume | Button::StopwatchPauseResume => {
                let g = if running { G_PAUSE } else { G_PLAY };
                (g, true, timer_color().with_opacity(0.25), timer_color())
            }
            Button::TimerCancel | Button::StopwatchStop => {
                let g = if *b == Button::TimerCancel { G_CANCEL } else { G_STOP };
                (g, true, color::STATUS_CRITICAL.with_opacity(0.25), color::STATUS_CRITICAL)
            }
            _ => (G_RESET, true, white(0.14), TEXT_PRIMARY),
        };
        ui.button(parent, *r, label, icon, bg, fg)?;
        d.buttons.push((*b, *r));
    }
    Ok(())
}

pub fn clock_text(c: LocalTime) -> String {
    format!("{:02}:{:02}", c.hour, c.minute)
}

pub fn build_home_card(ui: &Ui, parent: &ContainerVisual, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let presets = &m.presets[..m.presets.len().min(5)];
    let lay = layout::home_card(presets.len());
    if m.mascot_on {
        mascot(ui, parent, lay.mascot, m.skin, m.expression)?;
    }
    let big = TextStyle::new(28.0, Weight::Semibold, TEXT_PRIMARY).tabular();
    d.clock = Some(ui.text(parent, &clock_text(m.clock), big, lay.clock, Align::Left)?);
    let date = long_date(m.lang, m.clock.weekday, m.clock.day, m.clock.month);
    ui.text(parent, &date, TextStyle::new(13.0, Weight::Regular, color::text_secondary()), lay.date, Align::Left)?;
    for (b, r) in &lay.buttons {
        let (label, icon) = match b {
            Button::Preset(i) => (format!("{} {}", presets[*i as usize], t(m.lang, Text::Minutes)), false),
            _ => (G_STOPWATCH.to_string(), true),
        };
        ui.button(parent, *r, &label, icon, white(0.12), TEXT_PRIMARY)?;
        d.buttons.push((*b, *r));
    }
    Ok(())
}

pub fn build_alert(ui: &Ui, parent: &ContainerVisual, alert: &AlertKind, m: &Model<'_>) -> Result<()> {
    let lay = layout::alert_card();
    if m.mascot_on {
        mascot(ui, parent, lay.mascot, m.skin, m.expression)?;
    }
    let pct = |p: Option<u8>| p.map(|p| format!("{p}%")).unwrap_or_default();
    let (title, subtitle, value, accent) = match alert {
        AlertKind::Charging { percent } => (t(m.lang, Text::Charging), String::new(), pct(*percent), color::accent_charging()),
        AlertKind::Unplugged { percent } => (t(m.lang, Text::Unplugged), String::new(), pct(*percent), TEXT_PRIMARY),
        AlertKind::LowBattery { percent, critical } => {
            let key = if *critical { Text::BatteryCritical } else { Text::BatteryLow };
            (t(m.lang, key), String::new(), format!("{percent}%"), color::STATUS_CRITICAL)
        }
        AlertKind::TimerDone => {
            let total = m.timer.map(|t| countdown(t.total())).unwrap_or_default();
            (t(m.lang, Text::TimerDone), total, String::new(), timer_color())
        }
        AlertKind::Volume { .. } => (t(m.lang, Text::Volume), String::new(), String::new(), TEXT_PRIMARY),
    };
    let title_rect = if subtitle.is_empty() { Rect::new(lay.title.x, 0.0, lay.title.w, layout::EXPANDED_MIN_H) } else { lay.title };
    ui.text(parent, title, TextStyle::new(15.0, Weight::Semibold, TEXT_PRIMARY), title_rect, Align::Left)?;
    if !subtitle.is_empty() {
        ui.text(parent, &subtitle, TextStyle::new(13.0, Weight::Regular, color::text_secondary()).tabular(), lay.subtitle, Align::Left)?;
    }
    if !value.is_empty() {
        ui.text(parent, &value, TextStyle::new(28.0, Weight::Semibold, accent).tabular(), lay.value, Align::Right)?;
    }
    Ok(())
}

/// HUD volume kecil (PRD §3.2: HUD volume dan kecerahan).
pub fn build_hud(ui: &Ui, parent: &ContainerVisual, level: u8, muted: bool, d: &mut Dyn) -> Result<()> {
    let size = layout::HUD;
    let glyph = if muted || level == 0 { G_MUTE } else { G_VOLUME };
    let icon = Rect::new(10.0, 0.0, 24.0, size.h);
    ui.text(parent, glyph, TextStyle::new(15.0, Weight::Regular, TEXT_PRIMARY).icon().centered(), icon, Align::Center)?;
    let track = Rect::new(42.0, (size.h - 4.0) / 2.0, size.w - 42.0 - 48.0, 4.0);
    ui.rounded(parent, track, 2.0, white(0.2))?;
    let fill = ui.bar(parent, Rect::new(track.x, track.y, 0.0, track.h), TEXT_PRIMARY)?;
    let geom = ui.c.CreateRoundedRectangleGeometry()?;
    geom.SetSize(v2(track.w, track.h))?;
    geom.SetCornerRadius(v2(2.0, 2.0))?;
    fill.SetClip(&ui.c.CreateGeometricClipWithGeometry(&geom)?)?;
    fill.SetSize(v2(track.w * level as f32 / 100.0, track.h))?;
    d.hud_bar = Some((fill, track.w));
    let text = Rect::new(size.w - 44.0, 0.0, 32.0, size.h);
    d.hud_text = Some(ui.text(parent, &level.to_string(), TextStyle::new(13.0, Weight::Semibold, TEXT_PRIMARY).tabular(), text, Align::Right)?);
    Ok(())
}
