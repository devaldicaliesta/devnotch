use notchi_core::Ms;
use notchi_core::color::{self, Color, Skin};
use notchi_core::fmt::{countdown, elapsed};
use notchi_core::geom::{Rect, Size};
use notchi_core::i18n::{Lang, Text, long_date, short_date, t};
use notchi_core::island::AlertKind;
use notchi_core::layout::{self, BATTERY_BADGE, Button};
use notchi_core::lyrics::Lyrics;
use notchi_core::mascot::Expression;
use notchi_core::media::{Artwork, MediaSnapshot};
use notchi_core::power::PowerReading;
use notchi_core::spring::SpringPreset;
use notchi_core::timer::{Countdown, Stopwatch};
use notchi_platform_win::Result;
use notchi_platform_win::comp::{CompositionObject, ContainerVisual, Interface, ShapeVisual, SpriteVisual, v2, v3};
use notchi_platform_win::gfx::{TextStyle, Weight};
use notchi_platform_win::shell::LocalTime;

use crate::ui::{Align, DynText, Mascot, Tone, Ui, mascot, waveform};

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
pub const G_LYRICS: &str = "\u{E720}";
pub const G_CLOCK: &str = "\u{E823}";
pub const G_CHEVRON: &str = "\u{E70D}";

const LOW_BATTERY: u8 = 20;
const KNOB: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Media,
    Timer,
    Stopwatch,
}

#[derive(Debug, Clone, Copy)]
pub enum LyricState<'a> {
    Loading,
    Missing,
    Offline,
    Ready(&'a Lyrics),
}

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
    pub lyrics: Option<LyricState<'a>>,
    pub battery: Option<PowerReading>,
}

#[derive(Debug)]
pub struct SeekBar {
    pub rect: Rect,
    knob: ShapeVisual,
    shown: bool,
}

impl SeekBar {
    pub fn show(&mut self, ui: &Ui, on: bool) -> Result<()> {
        if on == self.shown {
            return Ok(());
        }
        self.shown = on;
        ui.anim.linear(&self.knob, "Opacity", if on { 1.0 } else { 0.0 }, 120.0, 0.0)?;
        ui.anim.spring(&self.knob, "Scale.X", if on { 1.0 } else { 0.4 }, SpringPreset::SNAPPY)?;
        ui.anim.spring(&self.knob, "Scale.Y", if on { 1.0 } else { 0.4 }, SpringPreset::SNAPPY)
    }
}

#[derive(Debug, Default)]
pub struct Dyn {
    pub timer: Option<DynText>,
    pub stopwatch: Option<DynText>,
    pub elapsed: Option<DynText>,
    pub remaining: Option<DynText>,
    pub progress: Option<(SpriteVisual, f32)>,
    pub seek: Option<SeekBar>,
    pub clock: Option<DynText>,
    pub hud_bar: Option<(SpriteVisual, f32)>,
    pub hud_text: Option<DynText>,
    pub buttons: Vec<(Button, Rect)>,
    pub hosts: Vec<(Button, ContainerVisual)>,
    pub mascot: Option<Mascot>,
    pub lyrics: Option<LyricPanel>,
}

impl Dyn {
    fn add(&mut self, b: Button, r: Rect, host: ContainerVisual) {
        self.buttons.push((b, r));
        self.hosts.push((b, host));
    }

    fn mascot(&mut self, m: Mascot, interactive: bool) {
        if interactive {
            self.buttons.push((Button::Mascot, m.rect));
        }
        self.mascot = Some(m);
    }

    pub fn host(&self, b: Button) -> Option<&ContainerVisual> {
        self.hosts.iter().find(|(x, _)| *x == b).map(|(_, h)| h)
    }
}

/// Warna teks utama dengan opasitas tertentu, mengikuti tema (putih di gelap, tinta di terang).
fn white(ui: &Ui, alpha: f32) -> Color {
    ui.pal.text.with_opacity(alpha)
}

fn timer_color(ui: &Ui) -> Color {
    ui.pal.ink(color::accent_alert())
}

fn fill_color(ui: &Ui) -> Color {
    if ui.pal.is_flat() { ui.pal.text } else { ui.accent }
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

fn left_icon(ui: &Ui, parent: &ContainerVisual, rect: Rect, slot: Slot, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    match (slot, m.art) {
        (Slot::Media, Some(art)) => {
            ui.image(parent, art, rect, 6.0)?;
        }
        _ if m.mascot_on => {
            d.mascot(mascot(ui, parent, rect, m.skin, m.expression)?, false);
        }
        _ => {
            let glyph = if slot == Slot::Media { G_MUSIC } else { G_STOPWATCH };
            let style = TextStyle::new(rect.h * 0.62, Weight::Regular, ui.pal.ink(m.skin.color())).icon().centered();
            ui.text(parent, glyph, style, rect, Align::Center)?;
        }
    }
    Ok(())
}

pub fn build_compact(ui: &Ui, parent: &ContainerVisual, slot: Slot, size: Size, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let (left, right) = layout::compact_slots(size);
    left_icon(ui, parent, left, slot, m, d)?;
    let text_rect = Rect::new(left.x + left.w + 8.0, 0.0, size.w - left.w - 8.0 - 12.0 - 8.0, size.h);
    match slot {
        Slot::Media => {
            let playing = m.media.is_some_and(|s| s.is_playing());
            waveform(ui, parent, right, ui.pal.ink(m.skin.color()), playing)?;
        }
        Slot::Timer => {
            if let Some(timer) = m.timer {
                let color = if timer.is_running() { timer_color(ui) } else { white(ui, 0.6) };
                let style = TextStyle::new(15.0, Weight::Semibold, color).tabular();
                d.timer = Some(ui.text(parent, &timer_text(timer, m.now), style, text_rect, Align::Right)?);
            }
        }
        Slot::Stopwatch => {
            if let Some(sw) = m.stopwatch {
                let style = TextStyle::new(15.0, Weight::Semibold, ui.pal.text).tabular();
                d.stopwatch = Some(ui.text(parent, &stopwatch_text(sw, m.now), style, text_rect, Align::Right)?);
            }
        }
    }
    Ok(())
}

pub fn build_bubble(ui: &Ui, parent: &ContainerVisual, slot: Slot, m: &Model<'_>) -> Result<()> {
    let r = Rect::new(6.0, 6.0, 25.0, 25.0);
    match (slot, m.art) {
        (Slot::Media, Some(art)) => {
            ui.image(parent, art, r, 12.5)?;
        }
        _ => {
            let (glyph, c) = match slot {
                Slot::Media => (G_MUSIC, ui.pal.ink(m.skin.color())),
                Slot::Timer => (G_STOPWATCH, timer_color(ui)),
                Slot::Stopwatch => (G_STOPWATCH, ui.pal.text),
            };
            let style = TextStyle::new(15.0, Weight::Regular, c).icon().centered();
            ui.text(parent, glyph, style, r, Align::Center)?;
        }
    }
    Ok(())
}

pub fn build_media_card(ui: &Ui, parent: &ContainerVisual, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::media_card(m.lyrics.is_some());
    let snap = m.media;
    if !ui.pal.is_flat() {
        ui.surface(parent, lay.art, 12.0, m.art.is_some(), ui.pal.well)?;
    }
    match m.art {
        Some(art) => {
            ui.image(parent, art, lay.art, 12.0)?;
        }
        None => {
            if ui.pal.is_flat() {
                ui.rounded(parent, lay.art, 12.0, white(ui, 0.1))?;
            }
            if m.mascot_on {
                let inner = lay.art.inflate(-10.0);
                d.mascot(mascot(ui, parent, inner, m.skin, m.expression)?, true);
            } else {
                let style = TextStyle::new(24.0, Weight::Regular, white(ui, 0.6)).icon().centered();
                ui.text(parent, G_MUSIC, style, lay.art, Align::Center)?;
            }
        }
    }
    let title = snap.map(|s| s.title.as_str()).filter(|s| !s.is_empty()).unwrap_or(t(m.lang, Text::NothingPlaying));
    ui.text(parent, title, TextStyle::new(15.0, Weight::Semibold, ui.pal.text), lay.title, Align::Left)?;
    if let Some(s) = snap {
        let sub = if s.artist.is_empty() { s.album.as_str() } else { s.artist.as_str() };
        if !sub.is_empty() {
            ui.text(parent, sub, TextStyle::new(13.0, Weight::Regular, ui.pal.text_dim), lay.subtitle, Align::Left)?;
        }
        let mut source = s.source_name();
        if s.session_count > 1 {
            source = format!("{source} · {}/{}", s.session_index + 1, s.session_count);
        }
        ui.text(parent, &source, TextStyle::new(11.0, Weight::Medium, white(ui, 0.45)), lay.source, Align::Left)?;
    }

    if let (Some(rect), Some(state)) = (lay.lyrics, m.lyrics) {
        ui.surface(parent, rect, 14.0, false, ui.pal.well)?;
        let root = ui.container(parent)?;
        root.SetOffset(v3(rect.x, rect.y, 0.0))?;
        root.SetSize(v2(rect.w, rect.h))?;
        root.SetClip(&ui.c.CreateInsetClip()?)?;
        let local = Rect::new(0.0, 0.0, rect.w, rect.h);
        match state {
            LyricState::Ready(l) if !l.instrumental && !l.is_empty() => {
                d.lyrics = Some(LyricPanel { root, rect: local, index: None, shown: false, current: None, leaving: None });
            }
            other => {
                let key = match other {
                    LyricState::Loading => Text::LyricsLoading,
                    LyricState::Ready(l) if l.instrumental => Text::Instrumental,
                    LyricState::Offline => Text::LyricsOffline,
                    _ => Text::LyricsMissing,
                };
                let style = TextStyle::new(13.0, Weight::Medium, ui.pal.text_dim).centered();
                let msg = ui.text(&root, t(m.lang, key), style, local, Align::Center)?;
                if matches!(other, LyricState::Loading) {
                    ui.anim.repeat(&msg.visual, "Opacity", &[(0.0, 1.0), (0.5, 0.45), (1.0, 1.0)], 1_400.0, 0.0)?;
                }
            }
        }
    }

    if let Some(s) = snap.filter(|s| s.timeline.is_some()) {
        if ui.pal.is_flat() {
            ui.rounded(parent, lay.progress, 2.0, white(ui, 0.2))?;
        } else {
            let track = Rect::new(lay.progress.x - 1.0, lay.progress.y - 1.0, lay.progress.w + 2.0, lay.progress.h + 2.0);
            ui.surface(parent, track, 3.0, false, ui.pal.well)?;
        }
        let fill = ui.bar(parent, lay.progress, fill_color(ui))?;
        let geom = ui.c.CreateRoundedRectangleGeometry()?;
        geom.SetSize(v2(lay.progress.w, lay.progress.h))?;
        geom.SetCornerRadius(v2(2.0, 2.0))?;
        fill.SetClip(&ui.c.CreateGeometricClipWithGeometry(&geom)?)?;
        if s.controls.seek {
            let (knob, _) = ui.dot(KNOB, ui.pal.text)?;
            knob.SetOpacity(0.0)?;
            knob.SetScale(v3(0.4, 0.4, 1.0))?;
            parent.Children()?.InsertAtTop(&knob)?;
            let source: CompositionObject = fill.cast()?;
            let x0 = lay.progress.x - KNOB / 2.0;
            let y0 = lay.progress.y + lay.progress.h / 2.0 - KNOB / 2.0;
            ui.anim.bind(&knob, "Offset", &format!("Vector3(fill.Size.X + {x0}, {y0}, 0)"), &[("fill", &source)])?;
            d.seek = Some(SeekBar { rect: lay.progress, knob, shown: false });
        }
        d.progress = Some((fill, lay.progress.w));
        let small = TextStyle::new(11.0, Weight::Medium, white(ui, 0.6)).tabular();
        let (el, rem) = media_times(s, m.now);
        d.elapsed = Some(ui.text(parent, &el, small, lay.elapsed, Align::Left)?);
        d.remaining = Some(ui.text(parent, &rem, small, lay.remaining, Align::Right)?);
    }

    let c = snap.map(|s| s.controls).unwrap_or_default();
    let playing = snap.is_some_and(|s| s.is_playing());
    for (b, r) in &lay.buttons {
        let (glyph, enabled, tone) = match b {
            Button::Prev => (G_PREV, c.prev, Tone::Neutral),
            Button::Next => (G_NEXT, c.next, Tone::Neutral),
            Button::Lyrics => {
                let tone = if m.lyrics.is_some() { Tone::Tinted(ui.accent) } else { Tone::Neutral };
                (G_LYRICS, true, tone)
            }
            _ if playing => (G_PAUSE, c.can_play_pause(true), Tone::Tinted(ui.accent)),
            _ => (G_PLAY, c.can_play_pause(false), Tone::Tinted(ui.accent)),
        };
        let tone = if enabled { tone } else { Tone::Disabled };
        let tone = if ui.pal.is_flat() && matches!(tone, Tone::Tinted(_)) && *b != Button::Lyrics { Tone::Neutral } else { tone };
        let host = ui.button(parent, *r, glyph, true, tone)?;
        if enabled {
            d.add(*b, *r, host);
        }
    }
    Ok(())
}

pub fn times_at(pos: Ms, end: Ms) -> (String, String) {
    (elapsed(pos), format!("-{}", elapsed(end.saturating_sub(pos))))
}

pub fn media_times(s: &MediaSnapshot, now: Ms) -> (String, String) {
    match (s.position_at(now), s.timeline) {
        (Some(pos), Some(tl)) => times_at(pos, tl.end),
        _ => (String::new(), String::new()),
    }
}

#[derive(Debug)]
pub struct LyricPanel {
    root: ContainerVisual,
    rect: Rect,
    index: Option<usize>,
    shown: bool,
    current: Option<ContainerVisual>,
    leaving: Option<ContainerVisual>,
}

const LYRIC_SHIFT: f32 = 18.0;

impl LyricPanel {
    pub fn show(&mut self, ui: &Ui, lyrics: &Lyrics, index: Option<usize>) -> Result<()> {
        if self.shown && index == self.index {
            return Ok(());
        }
        let animate = self.shown;
        self.shown = true;
        self.index = index;
        if let Some(old) = self.leaving.take() {
            let _ = self.root.Children()?.Remove(&old);
        }
        if let Some(cur) = self.current.take() {
            ui.anim.spring(&cur, "Offset.Y", -LYRIC_SHIFT, SpringPreset::SMOOTH)?;
            ui.anim.linear(&cur, "Opacity", 0.0, 180.0, 0.0)?;
            self.leaving = Some(cur);
        }
        let c = ui.container(&self.root)?;
        c.SetSize(v2(self.rect.w, self.rect.h))?;
        let pad = 16.0;
        let w = self.rect.w - 2.0 * pad;
        let line = |i: usize| lyrics.line(i).map(str::trim);
        let (cur, accent) = match index.and_then(line) {
            Some(s) if !s.is_empty() => (s.to_string(), false),
            _ => ("♪".to_string(), true),
        };
        let next = (index.map_or(0, |i| i + 1)..lyrics.lines.len()).filter_map(line).find(|s| !s.is_empty());
        let big = TextStyle::new(15.0, Weight::Semibold, if accent { ui.pal.ink(ui.accent) } else { ui.pal.text });
        let fits = ui.gfx.measure(&cur, &big)?.w <= w;
        if fits {
            ui.text(&c, &cur, big, Rect::new(pad, 8.0, w, 22.0), Align::Left)?;
            if let Some(n) = next {
                ui.text(&c, n, TextStyle::new(13.0, Weight::Regular, ui.pal.text_dim), Rect::new(pad, 32.0, w, 18.0), Align::Left)?;
            }
        } else {
            let style = TextStyle::new(13.5, Weight::Semibold, ui.pal.text).wrap(2);
            ui.text(&c, &cur, style, Rect::new(pad, 4.0, w, self.rect.h - 8.0), Align::Left)?;
        }
        if animate {
            c.SetOffset(v3(0.0, LYRIC_SHIFT, 0.0))?;
            c.SetOpacity(0.0)?;
            ui.anim.spring(&c, "Offset.Y", 0.0, SpringPreset::SMOOTH)?;
            ui.anim.linear(&c, "Opacity", 1.0, 200.0, 40.0)?;
        }
        self.current = Some(c);
        Ok(())
    }
}

pub fn build_timer_card(ui: &Ui, parent: &ContainerVisual, stopwatch: bool, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::timer_card(stopwatch);
    if m.mascot_on {
        d.mascot(mascot(ui, parent, lay.mascot, m.skin, m.expression)?, true);
    } else {
        let style = TextStyle::new(30.0, Weight::Regular, ui.pal.ink(m.skin.color())).icon().centered();
        ui.text(parent, G_STOPWATCH, style, lay.mascot, Align::Center)?;
    }
    let (label, running) = if stopwatch {
        (t(m.lang, Text::StopwatchLabel), m.stopwatch.is_some_and(|s| s.is_running()))
    } else {
        let running = m.timer.is_some_and(|t| t.is_running());
        (t(m.lang, if running { Text::TimerLabel } else { Text::TimerPaused }), running)
    };
    ui.text(parent, label, TextStyle::new(13.0, Weight::Medium, ui.pal.text_dim), lay.label, Align::Left)?;
    let big = |c: Color| TextStyle::new(34.0, Weight::Semibold, c).tabular();
    if stopwatch {
        if let Some(sw) = m.stopwatch {
            d.stopwatch = Some(ui.text(parent, &stopwatch_text(sw, m.now), big(ui.pal.text), lay.time, Align::Left)?);
        }
    } else if let Some(timer) = m.timer {
        let c = if running { timer_color(ui) } else { white(ui, 0.6) };
        d.timer = Some(ui.text(parent, &timer_text(timer, m.now), big(c), lay.time, Align::Left)?);
    }
    for (b, r) in &lay.buttons {
        let (label, icon, tone) = match b {
            Button::TimerAddMinute => ("+1", false, Tone::Neutral),
            Button::TimerPauseResume | Button::StopwatchPauseResume => {
                (if running { G_PAUSE } else { G_PLAY }, true, Tone::Tinted(timer_color(ui)))
            }
            Button::TimerCancel => (G_CANCEL, true, Tone::Tinted(color::STATUS_CRITICAL)),
            Button::StopwatchStop => (G_STOP, true, Tone::Tinted(color::STATUS_CRITICAL)),
            _ => (G_RESET, true, Tone::Neutral),
        };
        let host = ui.button(parent, *r, label, icon, tone)?;
        d.add(*b, *r, host);
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
        d.mascot(mascot(ui, parent, lay.mascot, m.skin, m.expression)?, true);
    }
    let big = TextStyle::new(28.0, Weight::Semibold, ui.pal.text).tabular();
    d.clock = Some(ui.text(parent, &clock_text(m.clock), big, lay.clock, Align::Left)?);
    let date = long_date(m.lang, m.clock.weekday, m.clock.day, m.clock.month);
    ui.text(parent, &date, TextStyle::new(13.0, Weight::Regular, ui.pal.text_dim), lay.date, Align::Left)?;
    if let Some(r) = m.battery {
        battery_badge(ui, parent, lay.battery, r)?;
    }
    for (b, r) in &lay.buttons {
        let (label, icon) = match b {
            Button::Preset(i) => (format!("{} {}", presets[*i as usize], t(m.lang, Text::Minutes)), false),
            _ => (G_STOPWATCH.to_string(), true),
        };
        let host = ui.button(parent, *r, &label, icon, Tone::Neutral)?;
        d.add(*b, *r, host);
    }
    Ok(())
}

pub fn battery_level(r: PowerReading) -> Option<(u8, bool)> {
    if !r.has_battery {
        return None;
    }
    r.percent.map(|p| (p.min(100), r.ac_online))
}

fn battery_badge(ui: &Ui, parent: &ContainerVisual, rect: Rect, r: PowerReading) -> Result<bool> {
    let Some((pct, charging)) = battery_level(r) else { return Ok(false) };
    let body = Size::new(21.0, 11.0);
    let bx = rect.x + rect.w - body.w - 3.0;
    let by = (rect.y + (rect.h - body.h) / 2.0).round();
    let outline = white(ui, 0.55);
    ui.rounded(parent, Rect::new(bx, by, body.w, body.h), 3.0, outline)?;
    ui.rounded(parent, Rect::new(bx + 1.5, by + 1.5, body.w - 3.0, body.h - 3.0), 2.0, ui.pal.surface())?;
    ui.rounded(parent, Rect::new(bx + body.w + 1.0, by + 3.5, 2.0, body.h - 7.0), 1.0, outline)?;
    let level = if charging {
        color::accent_charging()
    } else if pct <= LOW_BATTERY {
        color::STATUS_CRITICAL
    } else {
        white(ui, 0.9)
    };
    let inner = Size::new(body.w - 5.0, body.h - 5.0);
    let w = (inner.w * pct as f32 / 100.0).max(1.5);
    ui.rounded(parent, Rect::new(bx + 2.5, by + 2.5, w, inner.h), 1.0, level)?;
    if charging {
        let size = Size::new(6.0, 9.0);
        ui.bolt(parent, Rect::new(bx + (body.w - size.w) / 2.0, by + (body.h - size.h) / 2.0, size.w, size.h), ui.pal.text)?;
    }
    let text = Rect::new(rect.x, rect.y, bx - rect.x - 5.0, rect.h);
    let tint = if pct <= LOW_BATTERY && !charging { ui.pal.ink(color::STATUS_CRITICAL) } else { white(ui, 0.85) };
    ui.text(parent, &format!("{pct}%"), TextStyle::new(12.0, Weight::Semibold, tint).tabular(), text, Align::Right)?;
    Ok(true)
}

pub fn build_buddy(ui: &Ui, parent: &ContainerVisual, size: Size, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let s = layout::BUDDY_MASCOT;
    let r = Rect::new((size.w - s) / 2.0, (size.h - s) / 2.0, s, s);
    d.mascot(mascot(ui, parent, r, m.skin, m.expression)?, true);
    Ok(())
}

pub fn build_peek_idle(ui: &Ui, parent: &ContainerVisual, size: Size, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::peek_pill(size);
    if m.mascot_on {
        d.mascot(mascot(ui, parent, lay.icon, m.skin, m.expression)?, true);
    } else {
        let style = TextStyle::new(lay.icon.h * 0.62, Weight::Regular, ui.pal.ink(m.skin.color())).icon().centered();
        ui.text(parent, G_CLOCK, style, lay.icon, Align::Center)?;
    }
    let badge = Rect::new(size.w - 14.0 - BATTERY_BADGE.w, (size.h - BATTERY_BADGE.h) / 2.0, BATTERY_BADGE.w, BATTERY_BADGE.h);
    let shown = match m.battery {
        Some(r) => battery_badge(ui, parent, badge, r)?,
        None => false,
    };
    let end = if shown {
        badge.x + 4.0
    } else {
        let style = TextStyle::new(12.0, Weight::Regular, white(ui, 0.45)).icon().centered();
        ui.text(parent, G_CHEVRON, style, lay.trailing, Align::Center)?;
        lay.trailing.x
    };
    let x = lay.title.x;
    let clock = TextStyle::new(15.0, Weight::Semibold, ui.pal.text).tabular();
    let cw = ui.gfx.measure("00:00", &clock)?.w.ceil() + 2.0;
    d.clock = Some(ui.text(parent, &clock_text(m.clock), clock, Rect::new(x, 0.0, cw, size.h), Align::Left)?);
    let dx = x + cw + 8.0;
    let date = short_date(m.lang, m.clock.weekday, m.clock.day, m.clock.month);
    let style = TextStyle::new(12.0, Weight::Medium, ui.pal.text_dim);
    ui.text(parent, &date, style, Rect::new(dx, 1.0, (end - 8.0 - dx).max(1.0), size.h), Align::Left)?;
    Ok(())
}

pub fn build_peek_media(ui: &Ui, parent: &ContainerVisual, size: Size, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::peek_pill(size);
    left_icon(ui, parent, lay.icon, Slot::Media, m, d)?;
    let Some(s) = m.media else { return Ok(()) };
    let title = if s.title.trim().is_empty() { t(m.lang, Text::NothingPlaying) } else { s.title.as_str() };
    let sub = if s.artist.is_empty() { s.album.as_str() } else { s.artist.as_str() };
    let style = TextStyle::new(13.0, Weight::Semibold, ui.pal.text);
    if sub.is_empty() {
        ui.text(parent, title, style, Rect::new(lay.title.x, 0.0, lay.title.w, size.h), Align::Left)?;
    } else {
        ui.text(parent, title, style, lay.title, Align::Left)?;
        ui.text(parent, sub, TextStyle::new(11.0, Weight::Regular, ui.pal.text_dim), lay.subtitle, Align::Left)?;
    }
    let playing = s.is_playing();
    let enabled = s.controls.can_play_pause(playing);
    let tone = if !enabled {
        Tone::Disabled
    } else if ui.pal.is_flat() {
        Tone::Neutral
    } else {
        Tone::Tinted(ui.accent)
    };
    let host = ui.button(parent, lay.trailing, if playing { G_PAUSE } else { G_PLAY }, true, tone)?;
    if enabled {
        d.add(Button::PlayPause, lay.trailing, host);
    }
    Ok(())
}

pub fn build_alert(ui: &Ui, parent: &ContainerVisual, alert: &AlertKind, m: &Model<'_>, d: &mut Dyn) -> Result<()> {
    let lay = layout::alert_card();
    if m.mascot_on {
        d.mascot(mascot(ui, parent, lay.mascot, m.skin, m.expression)?, false);
    }
    let pct = |p: Option<u8>| p.map(|p| format!("{p}%")).unwrap_or_default();
    let (title, subtitle, value, accent) = match alert {
        AlertKind::Charging { percent } => (t(m.lang, Text::Charging), String::new(), pct(*percent), ui.pal.ink(color::accent_charging())),
        AlertKind::Unplugged { percent } => (t(m.lang, Text::Unplugged), String::new(), pct(*percent), ui.pal.text),
        AlertKind::LowBattery { percent, critical } => {
            let key = if *critical { Text::BatteryCritical } else { Text::BatteryLow };
            (t(m.lang, key), String::new(), format!("{percent}%"), ui.pal.ink(color::STATUS_CRITICAL))
        }
        AlertKind::TimerDone => {
            let total = m.timer.map(|t| countdown(t.total())).unwrap_or_default();
            (t(m.lang, Text::TimerDone), total, String::new(), timer_color(ui))
        }
        AlertKind::Volume { .. } => (t(m.lang, Text::Volume), String::new(), String::new(), ui.pal.text),
    };
    let title_rect = if subtitle.is_empty() { Rect::new(lay.title.x, 0.0, lay.title.w, layout::EXPANDED_MIN_H) } else { lay.title };
    ui.text(parent, title, TextStyle::new(15.0, Weight::Semibold, ui.pal.text), title_rect, Align::Left)?;
    if !subtitle.is_empty() {
        ui.text(parent, &subtitle, TextStyle::new(13.0, Weight::Regular, ui.pal.text_dim).tabular(), lay.subtitle, Align::Left)?;
    }
    if !value.is_empty() {
        ui.text(parent, &value, TextStyle::new(28.0, Weight::Semibold, accent).tabular(), lay.value, Align::Right)?;
    }
    Ok(())
}

pub fn build_hud(ui: &Ui, parent: &ContainerVisual, level: u8, muted: bool, d: &mut Dyn) -> Result<()> {
    let size = layout::HUD;
    let glyph = if muted || level == 0 { G_MUTE } else { G_VOLUME };
    let icon = Rect::new(10.0, 0.0, 24.0, size.h);
    ui.text(parent, glyph, TextStyle::new(15.0, Weight::Regular, ui.pal.text).icon().centered(), icon, Align::Center)?;
    let track = Rect::new(42.0, (size.h - 4.0) / 2.0, size.w - 42.0 - 48.0, 4.0);
    if ui.pal.is_flat() {
        ui.rounded(parent, track, 2.0, white(ui, 0.2))?;
    } else {
        ui.surface(parent, Rect::new(track.x - 1.0, track.y - 1.0, track.w + 2.0, track.h + 2.0), 3.0, false, ui.pal.well)?;
    }
    let fill = ui.bar(parent, Rect::new(track.x, track.y, 0.0, track.h), fill_color(ui))?;
    let geom = ui.c.CreateRoundedRectangleGeometry()?;
    geom.SetSize(v2(track.w, track.h))?;
    geom.SetCornerRadius(v2(2.0, 2.0))?;
    fill.SetClip(&ui.c.CreateGeometricClipWithGeometry(&geom)?)?;
    fill.SetSize(v2(track.w * level as f32 / 100.0, track.h))?;
    d.hud_bar = Some((fill, track.w));
    let text = Rect::new(size.w - 44.0, 0.0, 32.0, size.h);
    d.hud_text = Some(ui.text(parent, &level.to_string(), TextStyle::new(13.0, Weight::Semibold, ui.pal.text).tabular(), text, Align::Right)?);
    Ok(())
}
