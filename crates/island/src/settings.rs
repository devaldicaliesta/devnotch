use notchi_core::color::{Color, Skin};
use notchi_core::config::{Config, ReduceMotion};
use notchi_core::geom::{Point, Rect, scale_for_dpi};
use notchi_core::i18n::{Lang, Text, t};
use notchi_core::mascot::Expression;
use notchi_core::spring::SpringPreset;
use notchi_core::theme::{Mode, Style};
use notchi_platform_win::Result;
use notchi_platform_win::comp::{CompositionHost, ContainerVisual, ShapeVisual, SpriteVisual, color, v2, v3};
use notchi_platform_win::gfx::{TextStyle, Weight};
use notchi_platform_win::window::{Event, IconImage, OwnedWindow, UiProxy, create_panel};

use crate::fx::{self, Burst, Spark};
use crate::ui::{Align, DynText, Mascot, Tone, Ui, mascot};

pub const SIZE: (f32, f32) = (700.0, 940.0);
pub const TAG_PANEL_FX: u64 = 4 << 40;

const M: f32 = 24.0;
const GAP: f32 = 20.0;
const COL: f32 = (SIZE.0 - 2.0 * M - GAP) / 2.0;
const ROW: f32 = 46.0;
const SECTION: f32 = 26.0;
const CARD_R: f32 = 16.0;
const INNER: f32 = 16.0;
const TOP: f32 = 82.0;
const TRACK_W: f32 = 44.0;
const TRACK_H: f32 = 26.0;
const KNOB: f32 = 20.0;
const WHEEL_STEP: f32 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Behavior,
    Liquid,
    Fullscreen,
    AvoidWindows,
    Position,
    SideMascot,
    HoverZone,
    Style,
    Theme,
    Reduce,
    Mascot,
    Skin,
    Media,
    Lyrics,
    Scroll,
    ScrollBrightness,
    Downloads,
    Bluetooth,
    Shelf,
    Sound,
    Language,
    Login,
    Advanced,
    Boop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action {
    pub setting: Setting,
    pub index: usize,
}

pub fn value(cfg: &Config, s: Setting) -> usize {
    match s {
        Setting::Behavior => cfg.island.behavior.index(),
        Setting::Liquid => cfg.liquid as usize,
        Setting::Fullscreen => cfg.island.hide_in_fullscreen as usize,
        Setting::AvoidWindows => cfg.island.avoid_windows as usize,
        Setting::Position => notchi_core::layout::Placement::nearest(cfg.island.x).index(),
        Setting::SideMascot => cfg.island.side_mascot as usize,
        Setting::HoverZone => cfg.island.hover_zone.index(),
        Setting::Style => cfg.style.index(),
        Setting::Theme => cfg.theme.index(),
        Setting::Reduce => match cfg.reduce_motion {
            ReduceMotion::System => 0,
            ReduceMotion::On => 1,
            ReduceMotion::Off => 2,
        },
        Setting::Mascot => cfg.mascot.enabled as usize,
        Setting::Skin => Skin::ALL.iter().position(|s| *s == cfg.mascot.skin).unwrap_or(0),
        Setting::Media => cfg.media.enabled as usize,
        Setting::Lyrics => cfg.media.lyrics as usize,
        Setting::Scroll => cfg.media.scroll_volume as usize,
        Setting::ScrollBrightness => cfg.media.scroll_brightness as usize,
        Setting::Downloads => cfg.downloads.enabled as usize,
        Setting::Bluetooth => cfg.bluetooth.enabled as usize,
        Setting::Shelf => cfg.shelf.enabled as usize,
        Setting::Sound => cfg.timer.sound as usize,
        Setting::Language => (cfg.language == Lang::En) as usize,
        Setting::Login => cfg.launch_at_login as usize,
        Setting::Advanced | Setting::Boop => 0,
    }
}

#[derive(Debug)]
enum Kind {
    Toggle { knob: ContainerVisual, lit: ContainerVisual, off_x: f32, on_x: f32, on: bool },
    Choice { segs: Vec<Rect>, pill: ContainerVisual, hover: ShapeVisual, selected: usize },
    Swatch { rects: Vec<Rect>, discs: Vec<ContainerVisual>, ring: ShapeVisual, selected: usize },
    Button { host: ContainerVisual },
    Mascot,
}

#[derive(Debug)]
struct Control {
    setting: Setting,
    hit: Vec<Rect>,
    kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Look {
    lang: Lang,
    style: Style,
    mode: Mode,
    skin: Skin,
    reduce: bool,
}

pub struct Settings {
    controls: Vec<Control>,
    desc: Option<DynText>,
    mascot: Option<Mascot>,
    hover: Option<(usize, usize)>,
    pressed: Option<(usize, usize)>,
    fx: Vec<(u64, ContainerVisual)>,
    fx_seq: u64,
    look: Option<Look>,
    hotkey: String,
    proxy: UiProxy,
    ui: Ui,
    bg: SpriteVisual,
    stage: ContainerVisual,
    zoom: f32,
    offset_x: f32,
    scroll: f32,
    _host: CompositionHost,
    window: OwnedWindow,
}

impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings").field("controls", &self.controls.len()).finish()
    }
}

impl Settings {
    pub fn open(base: &Ui, proxy: &UiProxy, cfg: &Config, hotkey: String, icon: &IconImage) -> Result<Settings> {
        let window = create_panel(t(cfg.language, Text::SettingsTitle), SIZE, base.pal.backdrop, !base.pal.is_light(), icon)?;
        let w = window.window();
        let fwd = proxy.clone();
        window.set_handler(move |_, ev| fwd.post(Event::Panel(Box::new(ev))));
        let host = CompositionHost::attach(&base.c, &w)?;
        let ui = base.fork(scale_for_dpi(w.dpi()));
        // Latar memenuhi seluruh jendela, berapa pun ukurannya.
        let bg = ui.c.CreateSpriteVisual()?;
        bg.SetRelativeSizeAdjustment(v2(1.0, 1.0))?;
        bg.SetBrush(&ui.c.CreateColorBrushWithColor(color(ui.pal.backdrop))?)?;
        host.root.Children()?.InsertAtTop(&bg)?;
        let stage = ui.container(&host.root)?;
        let mut s = Settings {
            controls: Vec::new(),
            desc: None,
            mascot: None,
            hover: None,
            pressed: None,
            fx: Vec::new(),
            fx_seq: 0,
            look: None,
            hotkey,
            proxy: proxy.clone(),
            ui,
            bg,
            stage,
            zoom: 1.0,
            offset_x: 0.0,
            scroll: 0.0,
            _host: host,
            window,
        };
        s.build(cfg)?;
        w.focus();
        Ok(s)
    }

    pub fn focus(&self) {
        self.window.window().focus();
    }

    pub fn set_hotkey(&mut self, hotkey: String) {
        self.hotkey = hotkey;
        self.look = None;
    }

    pub fn rescale(&mut self, cfg: &Config) -> Result<()> {
        self.ui.scale = scale_for_dpi(self.window.window().dpi());
        self.build(cfg)
    }

    /// Jendela bisa diubah ukurannya: isi menyusut bila jendela lebih sempit dari tata letak,
    /// berada di tengah bila lebih lebar, dan bisa di-scroll bila lebih pendek.
    pub fn resize(&mut self) -> Result<()> {
        let (cw, ch) = self.window.window().client_px();
        let s = self.ui.scale;
        let (w, h) = (cw as f32 / s, ch as f32 / s);
        if w < 1.0 || h < 1.0 {
            return Ok(());
        }
        self.zoom = (w / SIZE.0).min(1.0);
        self.offset_x = ((w - SIZE.0 * self.zoom) / 2.0).max(0.0);
        let range = (SIZE.1 * self.zoom - h).max(0.0);
        self.scroll = self.scroll.clamp(0.0, range);
        let z = s * self.zoom;
        self.stage.SetScale(v3(z, z, 1.0))?;
        self.stage.SetOffset(v3(self.offset_x * s, -self.scroll * s, 0.0))
    }

    pub fn wheel(&mut self, delta: i32) -> Result<()> {
        self.scroll -= delta as f32 / 120.0 * WHEEL_STEP;
        self.resize()
    }

    fn to_content(&self, p: Point) -> Point {
        Point::new((p.x - self.offset_x) / self.zoom, (p.y + self.scroll) / self.zoom)
    }

    fn look_of(&self, cfg: &Config) -> Look {
        Look { lang: cfg.language, style: cfg.style, mode: self.ui.pal.mode, skin: cfg.mascot.skin, reduce: self.ui.anim.reduce_motion }
    }

    pub fn update(&mut self, base: &Ui, cfg: &Config) -> Result<()> {
        self.ui.pal = base.pal;
        self.ui.accent = base.accent;
        self.ui.anim.reduce_motion = base.anim.reduce_motion;
        if self.look != Some(self.look_of(cfg)) {
            let skin_moved = self.look.is_some_and(|l| l.skin != cfg.mascot.skin);
            self.build(cfg)?;
            if skin_moved {
                self.celebrate_skin(cfg)?;
            }
            return Ok(());
        }
        self.sync(cfg)
    }

    fn celebrate_skin(&mut self, cfg: &Config) -> Result<()> {
        let i = value(cfg, Setting::Skin);
        let at = self.controls.iter().find_map(|c| match &c.kind {
            Kind::Swatch { rects, .. } => rects.get(i).map(|r| r.center()),
            _ => None,
        });
        if let Some(p) = at {
            self.burst(p, Burst::Sparkle, &fx::sparkle_colors(cfg.mascot.skin.color()))?;
        }
        if let Some(m) = &mut self.mascot {
            m.boop(&self.ui)?;
        }
        Ok(())
    }

    fn burst(&mut self, at: Point, kind: Burst, colors: &[Color]) -> Result<()> {
        self.fx_seq += 1;
        let seed = self.fx_seq.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((at.x.to_bits() as u64) << 7);
        if let Some(v) = fx::burst(&self.ui, &self.stage, Spark { at, kind, colors, seed }, &self.proxy, TAG_PANEL_FX | self.fx_seq)? {
            self.fx.push((self.fx_seq, v));
        }
        Ok(())
    }

    pub fn retire_fx(&mut self, seq: u64) {
        let (gone, keep): (Vec<_>, Vec<_>) = self.fx.drain(..).partition(|(id, _)| *id <= seq);
        self.fx = keep;
        if let Ok(children) = self.stage.Children() {
            for (_, v) in gone {
                let _ = children.Remove(&v);
            }
        }
    }

    fn build(&mut self, cfg: &Config) -> Result<()> {
        self.stage.Children()?.RemoveAll()?;
        self.controls.clear();
        self.desc = None;
        self.mascot = None;
        self.hover = None;
        self.pressed = None;
        self.fx.clear();
        let lang = cfg.language;
        let ui = self.ui.fork(self.ui.scale);
        let pal = ui.pal;
        let stage = self.stage.clone();
        self.bg.SetBrush(&ui.c.CreateColorBrushWithColor(color(pal.backdrop))?)?;
        self.window.window().set_caption(pal.backdrop, !pal.is_light());
        ui.bar(&stage, Rect::new(0.0, 0.0, SIZE.0, SIZE.1), pal.backdrop)?;

        let mrect = Rect::new(M, 18.0, 42.0, 42.0);
        let mut m = mascot(&ui, &stage, mrect, cfg.mascot.skin, Expression::Idle)?;
        m.look(&ui, 0.0, 0.0)?;
        self.mascot = Some(m);
        self.controls.push(Control { setting: Setting::Boop, hit: vec![mrect.inflate(4.0)], kind: Kind::Mascot });
        ui.text(&stage, "Notchi", TextStyle::new(19.0, Weight::Bold, pal.text), Rect::new(M + 56.0, 16.0, 300.0, 26.0), Align::Left)?;
        let sub = format!(
            "{} {} · {} {}",
            t(lang, Text::Version),
            env!("CARGO_PKG_VERSION"),
            t(lang, Text::Shortcut),
            self.hotkey
        );
        ui.text(&stage, &sub, TextStyle::new(12.0, Weight::Regular, pal.text_dim), Rect::new(M + 56.0, 42.0, 420.0, 18.0), Align::Left)?;

        let left = M;
        let right = M + COL + GAP;

        let mut y = self.section(&ui, left, TOP, COL, t(lang, Text::SecIsland), 116.0 + 5.0 * ROW)?;
        let behavior_row = Rect::new(left, y, COL, 116.0);
        ui.text(&stage, t(lang, Text::Behavior), TextStyle::new(14.0, Weight::Medium, pal.text), Rect::new(left + INNER, y + 10.0, COL - 2.0 * INNER, 22.0), Align::Left)?;
        let seg = Rect::new(left + INNER, y + 38.0, COL - 2.0 * INNER, 32.0);
        let labels = [t(lang, Text::SegAutohide), t(lang, Text::SegSticky), t(lang, Text::SegHide)];
        self.choice(&ui, Setting::Behavior, seg, &labels, value(cfg, Setting::Behavior))?;
        let desc_style = TextStyle::new(12.0, Weight::Regular, pal.text_dim).wrap(2);
        let desc_rect = Rect::new(left + INNER, y + 76.0, COL - 2.0 * INNER, 34.0);
        self.desc = Some(ui.text(&stage, behavior_desc(lang, value(cfg, Setting::Behavior)), desc_style, desc_rect, Align::Left)?);
        y = behavior_row.y + behavior_row.h;
        self.divider(&ui, left, y, COL)?;
        self.toggle(&ui, Setting::Liquid, Rect::new(left, y, COL, ROW), t(lang, Text::Liquid), None, cfg)?;
        y += ROW;
        self.divider(&ui, left, y, COL)?;
        self.toggle(&ui, Setting::Fullscreen, Rect::new(left, y, COL, ROW), t(lang, Text::Fullscreen), None, cfg)?;
        y += ROW;
        self.divider(&ui, left, y, COL)?;
        self.toggle(&ui, Setting::AvoidWindows, Rect::new(left, y, COL, ROW), t(lang, Text::AvoidWindows), None, cfg)?;
        y += ROW;
        self.divider(&ui, left, y, COL)?;
        let row = Rect::new(left, y, COL, ROW);
        self.label(&ui, row, t(lang, Text::Position))?;
        let places = [t(lang, Text::SegCenter), t(lang, Text::SegLeft), t(lang, Text::SegRight)];
        self.choice(&ui, Setting::Position, Rect::new(row.x + COL - INNER - 180.0, row.y + 7.0, 180.0, 32.0), &places, value(cfg, Setting::Position))?;
        y += ROW;
        self.divider(&ui, left, y, COL)?;
        let row = Rect::new(left, y, COL, ROW);
        self.label(&ui, row, t(lang, Text::HoverZone))?;
        let zones = [t(lang, Text::SegSmall), t(lang, Text::SegNormal)];
        self.choice(&ui, Setting::HoverZone, Rect::new(row.x + COL - INNER - 150.0, row.y + 7.0, 150.0, 32.0), &zones, value(cfg, Setting::HoverZone))?;
        y += ROW;

        let mut y2 = self.section(&ui, left, y + 18.0, COL, t(lang, Text::SecLook), 3.0 * ROW)?;
        let row = Rect::new(left, y2, COL, ROW);
        self.label(&ui, row, t(lang, Text::Style))?;
        let styles = [t(lang, Text::SegNeu), t(lang, Text::SegClay), t(lang, Text::SegGlass), t(lang, Text::SegClassic)];
        self.choice(&ui, Setting::Style, Rect::new(row.x + COL - INNER - 236.0, row.y + 7.0, 236.0, 32.0), &styles, value(cfg, Setting::Style))?;
        y2 += ROW;
        self.divider(&ui, left, y2, COL)?;
        let row = Rect::new(left, y2, COL, ROW);
        self.label(&ui, row, t(lang, Text::Theme))?;
        let themes = [t(lang, Text::SegSystem), t(lang, Text::SegDark), t(lang, Text::SegLight)];
        self.choice(&ui, Setting::Theme, Rect::new(row.x + COL - INNER - 198.0, row.y + 7.0, 198.0, 32.0), &themes, value(cfg, Setting::Theme))?;
        y2 += ROW;
        self.divider(&ui, left, y2, COL)?;
        let row = Rect::new(left, y2, COL, ROW);
        self.label(&ui, row, t(lang, Text::ReduceMotion))?;
        let opts = [t(lang, Text::SegSystem), t(lang, Text::SegOn), t(lang, Text::SegOff)];
        self.choice(&ui, Setting::Reduce, Rect::new(row.x + COL - INNER - 168.0, row.y + 7.0, 168.0, 32.0), &opts, value(cfg, Setting::Reduce))?;
        let left_bottom = y2 + ROW;

        let mut y = self.section(&ui, right, TOP, COL, t(lang, Text::SecMascot), 2.0 * ROW + 74.0)?;
        self.toggle(&ui, Setting::Mascot, Rect::new(right, y, COL, ROW), t(lang, Text::ShowMascot), None, cfg)?;
        y += ROW;
        self.divider(&ui, right, y, COL)?;
        let row = Rect::new(right, y, COL, ROW);
        self.label(&ui, row, t(lang, Text::MascotColor))?;
        self.swatches(&ui, row.x + COL - INNER, row.y + ROW / 2.0, value(cfg, Setting::Skin))?;
        y += ROW;
        self.divider(&ui, right, y, COL)?;
        self.toggle(&ui, Setting::SideMascot, Rect::new(right, y, COL, 74.0), t(lang, Text::SideMascot), Some(t(lang, Text::SideMascotDesc)), cfg)?;
        y += 74.0;

        let lyric_h = 74.0;
        let mut y = self.section(&ui, right, y + 18.0, COL, t(lang, Text::SecMedia), 4.0 * ROW + lyric_h)?;
        self.toggle(&ui, Setting::Media, Rect::new(right, y, COL, ROW), t(lang, Text::MediaControls), None, cfg)?;
        y += ROW;
        self.divider(&ui, right, y, COL)?;
        self.toggle(&ui, Setting::Lyrics, Rect::new(right, y, COL, lyric_h), t(lang, Text::Lyrics), Some(t(lang, Text::LyricsDesc)), cfg)?;
        y += lyric_h;
        self.divider(&ui, right, y, COL)?;
        self.toggle(&ui, Setting::Scroll, Rect::new(right, y, COL, ROW), t(lang, Text::ScrollVolume), None, cfg)?;
        y += ROW;
        self.divider(&ui, right, y, COL)?;
        self.toggle(&ui, Setting::ScrollBrightness, Rect::new(right, y, COL, ROW), t(lang, Text::ScrollBrightness), None, cfg)?;
        y += ROW;
        self.divider(&ui, right, y, COL)?;
        self.toggle(&ui, Setting::Sound, Rect::new(right, y, COL, ROW), t(lang, Text::TimerSound), None, cfg)?;
        y += ROW;

        let full = SIZE.0 - 2.0 * M;
        let gy = left_bottom.max(y) + 18.0;
        let act_h = 74.0;
        let y = self.section(&ui, left, gy, full, t(lang, Text::SecGeneral), ROW + 2.0 * act_h + 60.0)?;
        let row = Rect::new(left, y, COL, ROW);
        self.label(&ui, row, t(lang, Text::Language))?;
        self.choice(&ui, Setting::Language, Rect::new(row.x + COL - INNER - 176.0, row.y + 7.0, 176.0, 32.0), &["Indonesia", "English"], value(cfg, Setting::Language))?;
        self.toggle(&ui, Setting::Login, Rect::new(right, y, COL, ROW), t(lang, Text::MenuLaunchAtLogin), None, cfg)?;
        let y = y + ROW;
        self.divider(&ui, left, y, full)?;
        self.toggle(&ui, Setting::Downloads, Rect::new(left, y, COL, act_h), t(lang, Text::DownloadsCard), Some(t(lang, Text::DownloadsDesc)), cfg)?;
        self.toggle(&ui, Setting::Bluetooth, Rect::new(right, y, COL, act_h), t(lang, Text::BtCard), Some(t(lang, Text::BtDesc)), cfg)?;
        let y = y + act_h;
        self.divider(&ui, left, y, full)?;
        self.toggle(&ui, Setting::Shelf, Rect::new(left, y, COL, act_h), t(lang, Text::ShelfTitle), Some(t(lang, Text::ShelfDesc)), cfg)?;
        let y = y + act_h;
        self.divider(&ui, left, y, full)?;
        let adv = Rect::new(left, y, full, 60.0);
        ui.text(&stage, t(lang, Text::Advanced), TextStyle::new(14.0, Weight::Medium, pal.text), Rect::new(adv.x + INNER, adv.y + 10.0, 400.0, 22.0), Align::Left)?;
        ui.text(&stage, t(lang, Text::AdvancedDesc), TextStyle::new(12.0, Weight::Regular, pal.text_dim), Rect::new(adv.x + INNER, adv.y + 32.0, 400.0, 18.0), Align::Left)?;
        let b = Rect::new(adv.x + full - INNER - 128.0, adv.y + 14.0, 128.0, 32.0);
        let host = ui.button(&stage, b, t(lang, Text::OpenFile), false, Tone::Neutral)?;
        self.controls.push(Control { setting: Setting::Advanced, hit: vec![b], kind: Kind::Button { host } });

        self.look = Some(self.look_of(cfg));
        self.resize()?;
        Ok(())
    }

    fn section(&self, ui: &Ui, x: f32, y: f32, w: f32, title: &str, body_h: f32) -> Result<f32> {
        let style = TextStyle::new(11.5, Weight::Semibold, ui.pal.text_dim);
        ui.text(&self.stage, title, style, Rect::new(x + 6.0, y, w, 18.0), Align::Left)?;
        let card = Rect::new(x, y + SECTION, w, body_h);
        ui.surface(&self.stage, card, CARD_R, true, ui.pal.card)?;
        Ok(card.y)
    }

    fn divider(&self, ui: &Ui, x: f32, y: f32, w: f32) -> Result<()> {
        ui.bar(&self.stage, Rect::new(x + INNER, y, w - 2.0 * INNER, 1.0), ui.pal.divider)?;
        Ok(())
    }

    fn label(&self, ui: &Ui, row: Rect, text: &str) -> Result<()> {
        let style = TextStyle::new(14.0, Weight::Medium, ui.pal.text);
        ui.text(&self.stage, text, style, Rect::new(row.x + INNER, row.y, row.w - 2.0 * INNER, row.h), Align::Left)?;
        Ok(())
    }

    fn toggle(&mut self, ui: &Ui, setting: Setting, row: Rect, label: &str, desc: Option<&str>, cfg: &Config) -> Result<()> {
        let pal = ui.pal;
        let text_w = row.w - 2.0 * INNER - TRACK_W - 12.0;
        match desc {
            None => self.label(ui, Rect::new(row.x, row.y, text_w + 2.0 * INNER, row.h), label)?,
            Some(d) => {
                ui.text(&self.stage, label, TextStyle::new(14.0, Weight::Medium, pal.text), Rect::new(row.x + INNER, row.y + 10.0, text_w, 22.0), Align::Left)?;
                let style = TextStyle::new(11.5, Weight::Regular, pal.text_dim).wrap(2);
                ui.text(&self.stage, d, style, Rect::new(row.x + INNER, row.y + 32.0, text_w, 32.0), Align::Left)?;
            }
        }
        let track = Rect::new(row.x + row.w - INNER - TRACK_W, row.y + (row.h - TRACK_H) / 2.0, TRACK_W, TRACK_H);
        ui.surface(&self.stage, track, TRACK_H / 2.0, false, pal.well)?;
        let lit = ui.container(&self.stage)?;
        lit.SetOffset(v3(track.x, track.y, 0.0))?;
        let local = Rect::new(0.0, 0.0, TRACK_W, TRACK_H);
        match pal.style {
            Style::Classic => {
                ui.rounded(&lit, local, TRACK_H / 2.0, ui.accent)?;
            }
            Style::Clay => {
                ui.relief(&lit, local, TRACK_H / 2.0, &pal.clay(ui.accent, 2.0))?;
            }
            Style::Neumorphism | Style::Glass => {
                ui.relief(&lit, local, TRACK_H / 2.0, &pal.inset(ui.accent, 2.0))?;
            }
        }
        let on = value(cfg, setting) == 1;
        lit.SetOpacity(if on { 1.0 } else { 0.0 })?;
        let inset = (TRACK_H - KNOB) / 2.0;
        let off_x = track.x + inset;
        let on_x = track.x + TRACK_W - inset - KNOB;
        let knob = ui.container(&self.stage)?;
        knob.SetOffset(v3(if on { on_x } else { off_x }, track.y + inset, 0.0))?;
        knob.SetSize(v2(KNOB, KNOB))?;
        knob.SetCenterPoint(v3(KNOB / 2.0, KNOB / 2.0, 0.0))?;
        let kr = Rect::new(0.0, 0.0, KNOB, KNOB);
        match pal.style {
            Style::Classic => {
                ui.rounded(&knob, kr, KNOB / 2.0, pal.knob)?;
            }
            Style::Clay => {
                ui.relief(&knob, kr, KNOB / 2.0, &pal.clay(pal.knob, 2.0))?;
            }
            Style::Neumorphism | Style::Glass => {
                ui.relief(&knob, kr, KNOB / 2.0, &pal.raised(pal.knob, 2.5))?;
            }
        }
        self.controls.push(Control { setting, hit: vec![row], kind: Kind::Toggle { knob, lit, off_x, on_x, on } });
        Ok(())
    }

    fn choice(&mut self, ui: &Ui, setting: Setting, area: Rect, labels: &[&str], selected: usize) -> Result<()> {
        let pal = ui.pal;
        ui.surface(&self.stage, area, 10.0, false, pal.well)?;
        let n = labels.len().max(1);
        let sw = area.w / n as f32;
        let segs: Vec<Rect> = (0..n).map(|i| Rect::new(area.x + i as f32 * sw, area.y, sw, area.h)).collect();
        let hover = ui.rounded(&self.stage, Rect::new(0.0, area.y + 3.0, sw - 6.0, area.h - 6.0), 7.0, pal.text.with_opacity(0.06))?;
        hover.SetOpacity(0.0)?;
        let pill = ui.container(&self.stage)?;
        let sel = selected.min(n - 1);
        pill.SetOffset(v3(segs[sel].x + 3.0, area.y + 3.0, 0.0))?;
        let pr = Rect::new(0.0, 0.0, sw - 6.0, area.h - 6.0);
        match pal.style {
            Style::Classic if pal.is_light() => {
                ui.relief(&pill, pr, 7.0, &pal.glass(pal.pill, 2.0))?;
            }
            Style::Classic => {
                ui.rounded(&pill, pr, 7.0, pal.pill)?;
            }
            Style::Clay => {
                ui.relief(&pill, pr, 7.0, &pal.clay(pal.pill, 2.5))?;
            }
            Style::Neumorphism | Style::Glass => {
                ui.relief(&pill, pr, 7.0, &pal.raised(pal.pill, 2.5))?;
            }
        }
        for (r, l) in segs.iter().zip(labels) {
            let style = TextStyle::new(12.5, Weight::Semibold, pal.text).centered();
            ui.text(&self.stage, l, style, *r, Align::Center)?;
        }
        self.controls.push(Control { setting, hit: segs.clone(), kind: Kind::Choice { segs, pill, hover, selected: sel } });
        Ok(())
    }

    fn swatches(&mut self, ui: &Ui, right: f32, cy: f32, selected: usize) -> Result<()> {
        let d = 22.0;
        let gap = 10.0;
        let n = Skin::ALL.len();
        let x0 = right - (n as f32 * d + (n - 1) as f32 * gap);
        let rects: Vec<Rect> = (0..n).map(|i| Rect::new(x0 + i as f32 * (d + gap), cy - d / 2.0, d, d)).collect();
        let mut discs = Vec::new();
        for (r, skin) in rects.iter().zip(Skin::ALL) {
            let host = ui.container(&self.stage)?;
            host.SetOffset(v3(r.x, r.y, 0.0))?;
            host.SetSize(v2(d, d))?;
            host.SetCenterPoint(v3(d / 2.0, d / 2.0, 0.0))?;
            let local = Rect::new(0.0, 0.0, d, d);
            if ui.pal.is_flat() {
                ui.rounded(&host, local, d / 2.0, skin.color())?;
            } else {
                ui.relief(&host, local, d / 2.0, &ui.pal.clay(skin.color(), 2.0))?;
            }
            discs.push(host);
        }
        let sel = selected.min(n - 1);
        let ring = ui.ring(&self.stage, rects[sel].inflate(5.0), 2.0, ui.pal.text)?;
        let hit = rects.iter().map(|r| r.inflate(4.0)).collect();
        self.controls.push(Control { setting: Setting::Skin, hit, kind: Kind::Swatch { rects, discs, ring, selected: sel } });
        Ok(())
    }

    fn hit(&self, p: Point) -> Option<(usize, usize)> {
        self.controls
            .iter()
            .enumerate()
            .find_map(|(ci, c)| c.hit.iter().position(|r| r.contains(p)).map(|si| (ci, si)))
    }

    fn set_hover(&mut self, (ci, si): (usize, usize), on: bool) -> Result<()> {
        let ui = &self.ui;
        match &self.controls[ci].kind {
            Kind::Toggle { knob, .. } => ui.pop(knob, if on { 1.1 } else { 1.0 })?,
            Kind::Choice { segs, hover, selected, .. } => {
                if on && si != *selected {
                    hover.SetOffset(v3(segs[si].x + 3.0, hover.Offset()?.Y, 0.0))?;
                    ui.anim.linear(hover, "Opacity", 1.0, 90.0, 0.0)?;
                } else {
                    ui.anim.linear(hover, "Opacity", 0.0, 120.0, 0.0)?;
                }
            }
            Kind::Swatch { discs, .. } => ui.pop(&discs[si], if on { 1.2 } else { 1.0 })?,
            Kind::Button { host } => ui.pop(host, if on { 1.04 } else { 1.0 })?,
            Kind::Mascot => {
                if let Some(m) = &mut self.mascot {
                    m.perk(&self.ui, on)?;
                }
            }
        }
        Ok(())
    }

    fn press_visual(&self, (ci, si): (usize, usize), down: bool) -> Result<()> {
        let ui = &self.ui;
        match &self.controls[ci].kind {
            Kind::Toggle { knob, .. } => ui.pop(knob, if down { 0.86 } else { 1.1 }),
            Kind::Swatch { discs, .. } => ui.pop(&discs[si], if down { 0.9 } else { 1.2 }),
            Kind::Button { host } => ui.pop(host, if down { 0.94 } else { 1.04 }),
            _ => Ok(()),
        }
    }

    pub fn pointer_move(&mut self, p: Option<Point>) -> Result<()> {
        let p = p.map(|p| self.to_content(p));
        if let Some(m) = &mut self.mascot {
            let c = m.center();
            match p {
                Some(p) => m.look(&self.ui, (p.x - c.x) / 260.0, (p.y - c.y) / 200.0)?,
                None => m.look(&self.ui, 0.0, 0.0)?,
            }
        }
        let hit = p.and_then(|p| self.hit(p));
        if hit == self.hover {
            return Ok(());
        }
        if let Some(h) = self.hover.take() {
            self.set_hover(h, false)?;
        }
        if let Some(h) = hit {
            self.set_hover(h, true)?;
        }
        self.hover = hit;
        Ok(())
    }

    pub fn pointer_down(&mut self, p: Point) -> Result<()> {
        let p = self.to_content(p);
        self.pressed = self.hit(p);
        if let Some(h) = self.pressed {
            self.press_visual(h, true)?;
        }
        Ok(())
    }

    pub fn pointer_up(&mut self, p: Point) -> Result<Option<Action>> {
        let p = self.to_content(p);
        let Some(pressed) = self.pressed.take() else { return Ok(None) };
        self.press_visual(pressed, false)?;
        if self.hit(p) != Some(pressed) {
            return Ok(None);
        }
        let (ci, si) = pressed;
        let c = &self.controls[ci];
        let index = match &c.kind {
            Kind::Toggle { on, .. } => (!on) as usize,
            Kind::Mascot => {
                let at = self.mascot.as_ref().map(|m| m.center());
                if let Some(m) = &mut self.mascot {
                    m.boop(&self.ui)?;
                }
                if let Some(at) = at {
                    self.burst(at, Burst::Sparkle, &fx::sparkle_colors(self.ui.accent))?;
                }
                return Ok(None);
            }
            _ => si,
        };
        Ok(Some(Action { setting: c.setting, index }))
    }

    fn sync(&mut self, cfg: &Config) -> Result<()> {
        let mut pops = Vec::new();
        for c in &mut self.controls {
            let v = value(cfg, c.setting);
            match &mut c.kind {
                Kind::Toggle { knob, lit, off_x, on_x, on } => {
                    let want = v == 1;
                    if want != *on {
                        *on = want;
                        self.ui.anim.spring(knob, "Offset.X", if want { *on_x } else { *off_x }, SpringPreset::BOUNCY)?;
                        self.ui.anim.linear(lit, "Opacity", if want { 1.0 } else { 0.0 }, 160.0, 0.0)?;
                        if want {
                            let o = knob.Offset()?;
                            pops.push(Point::new(*on_x + KNOB / 2.0, o.Y + KNOB / 2.0));
                        }
                    }
                }
                Kind::Choice { segs, pill, selected, hover } => {
                    let want = v.min(segs.len() - 1);
                    if want != *selected {
                        *selected = want;
                        self.ui.anim.spring(pill, "Offset.X", segs[want].x + 3.0, SpringPreset::BOUNCY)?;
                        self.ui.anim.linear(hover, "Opacity", 0.0, 90.0, 0.0)?;
                        if c.setting == Setting::Behavior
                            && let Some(d) = &mut self.desc
                        {
                            d.set(&self.ui, behavior_desc(cfg.language, want))?;
                        }
                    }
                }
                Kind::Swatch { rects, ring, selected, .. } => {
                    let want = v.min(rects.len() - 1);
                    if want != *selected {
                        *selected = want;
                        self.ui.anim.spring(ring, "Offset.X", rects[want].x - 5.0, SpringPreset::BOUNCY)?;
                    }
                }
                Kind::Button { .. } | Kind::Mascot => {}
            }
        }
        for p in pops {
            self.burst(p, Burst::Pop, &fx::sparkle_colors(self.ui.accent))?;
        }
        Ok(())
    }
}

fn behavior_desc(lang: Lang, i: usize) -> &'static str {
    t(lang, [Text::DescAutohide, Text::DescSticky, Text::DescHide][i.min(2)])
}
