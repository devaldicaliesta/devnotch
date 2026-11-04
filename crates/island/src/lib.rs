#![forbid(unsafe_code)]

mod fx;
mod settings;
mod ui;
mod views;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use notchi_core::Ms;
use notchi_core::activity::{Activities, ActivityKind};
use notchi_core::bluetooth::Bluetooth;
use notchi_core::downloads::{Downloads, Partial};
use notchi_core::color::{self, Color, Skin};
use notchi_core::config::{self, Config, ReduceMotion, Warning};
use notchi_core::geom::{Point, Size, scale_for_dpi};
use notchi_core::i18n::{Lang, Text, t};
use notchi_core::icon::mascot_rgba;
use notchi_core::island::{AlertKind, Behavior, Island, Presentation};
use notchi_core::geom::Rect;
use notchi_core::layout::{self, BUBBLE, BUBBLE_GAP, Button, Card, HOVER_SCALE, HoverZone, Placement, Shape};
use notchi_core::lyrics::{Lyrics, Query};
use notchi_core::mascot::{Expression, MascotContext, expression_for};
use notchi_core::media::{Artwork, Controls, MediaPresence, MediaSnapshot, SourceFilter, seek_target};
use notchi_core::power::{BatteryMonitor, PowerEvent, PowerReading};
use notchi_core::privacy::Privacy;
use notchi_core::spring::SpringPreset;
use notchi_core::theme::{self, Mode, Style, Theme};
use notchi_core::timer::{Countdown, Stopwatch};
use notchi_platform_win::audio::Volume;
use notchi_platform_win::bluetooth::BluetoothWatcher;
use notchi_platform_win::brightness::Brightness;
use notchi_platform_win::comp::{Animator, CompositionHost, CompositionSpriteShape, ContainerVisual, ShapeVisual, v2, v3};
use notchi_platform_win::gfx::Gfx;
use notchi_platform_win::media::{MediaCommand, MediaService};
use notchi_platform_win::window::{
    Event, IconImage, MenuItem, Monitor, MouseButton, OwnedWindow, TrayEvent, TrayIcon, UiProxy, Window, primary_monitor,
    quit, cursor_pos,
};
use notchi_platform_win::{Result, comp, fswatch, net, now_ms, power, privacy, shell, sys};

use crate::fx::{Burst, Spark};
use crate::settings::{Action, Setting, Settings, TAG_PANEL_FX};
use crate::ui::{Pill, Ui};
use crate::views::{DownloadInfo, Dyn, LyricState, Model, Slot};

const T_LONG_PRESS: usize = 1;
const T_COLLAPSE: usize = 2;
const T_TICK: usize = 3;
const T_DEADLINE: usize = 4;
const T_CONFIG: usize = 5;
const T_CLOCK: usize = 6;
const T_RETRACT: usize = 7;
const T_TRIM: usize = 8;
const T_LYRICS: usize = 9;
const T_REVEAL: usize = 10;
const T_COVER: usize = 11;
const T_GAZE: usize = 12;
const T_DOWNLOADS: usize = 13;

const HK_TOGGLE: i32 = 1;
const HK_ESC: i32 = 2;

const TAG_MORPH: u64 = 1 << 40;
const TAG_RETIRE: u64 = 2 << 40;
const TAG_FX: u64 = 3 << 40;
const TAG_MASK: u64 = (1 << 40) - 1;

const M_PRESET: u32 = 100;
const M_STOPWATCH: u32 = 150;
const M_BEHAVIOR: u32 = 200;
const M_HIDE: u32 = 300;
const M_SHOW: u32 = 301;
const M_LOGIN: u32 = 400;
const M_SETTINGS: u32 = 500;
const M_ABOUT: u32 = 600;
const M_QUIT: u32 = 900;

const VK_ESCAPE: u32 = 0x1B;
const LONG_PRESS_MS: u32 = 350;
const DRAG_DP: f32 = 6.0;
const SWIPE_DP: f32 = 36.0;
const HOUR_MS: Ms = 3_600_000;
const PEEK_MS: Ms = 2_500;
const RETRACT_DELAY_MS: u32 = 300;
const COVER_SETTLE_MS: u32 = 150;
const DOWNLOAD_DEBOUNCE_MS: u32 = 250;
const DOWNLOAD_POLL_MS: u32 = 1_000;
/// Langkah kecerahan per takik scroll, dalam persen.
const BRIGHTNESS_STEP: f32 = 5.0;
const GAZE_MS: u32 = 66;
const GAZE_REACH: f32 = 60.0;
const COLLAPSE_DELAY_MS: u32 = 1_500;
const TRIM_DELAY_MS: u32 = 4_000;
const LYRICS_DEBOUNCE_MS: u32 = 350;
const LYRIC_LEAD_MS: Ms = 180;
const LYRIC_CACHE: usize = 12;
const LYRIC_RETRY_MS: Ms = 45_000;
const CONTENT_DELAY_MS: f32 = 70.0;
const CONTENT_FADE_MS: f32 = 130.0;
const RETIRE_FADE_MS: f32 = 80.0;
const FX_PAD: f32 = 34.0;
const HOVER_POP: f32 = 1.06;
const PRESS_POP: f32 = 0.92;
const REVEAL: SpringPreset = SpringPreset::new(0.36, 0.30);
const WHEEL_NOTCH: i32 = 120;
const MINUTE_MS: Ms = 60_000;
const DOT: f32 = 6.0;
const DOT_INSET: f32 = 5.0;
const LIP_TINT: f32 = 0.85;

#[derive(Debug, Clone)]
pub struct Paths {
    pub dir: PathBuf,
    pub config: PathBuf,
}

impl Paths {
    pub fn default_location() -> Paths {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("Notchi");
        Paths { config: dir.join("config.toml"), dir }
    }
}

pub fn load_config(paths: &Paths) -> (Config, String, Vec<Warning>) {
    let _ = std::fs::create_dir_all(&paths.dir);
    if !paths.config.exists() {
        let _ = std::fs::write(&paths.config, config::DEFAULT_TOML);
    }
    let too_big = std::fs::metadata(&paths.config).is_ok_and(|m| m.len() as usize > config::MAX_CONFIG_BYTES);
    let text = if too_big { String::new() } else { std::fs::read_to_string(&paths.config).unwrap_or_default() };
    let (cfg, mut warnings) = config::parse(&text);
    if too_big {
        warnings.push(Warning { key: String::new(), message: "berkas terlalu besar, memakai default".into() });
    }
    (cfg, text, warnings)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeekKind {
    Idle,
    Media,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewKind {
    Hidden,
    Empty,
    Buddy,
    Peek(PeekKind),
    Compact(Slot),
    Minimal(Slot, Slot),
    Card(Card),
    Alert,
    Hud,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewKey {
    kind: ViewKind,
    alert: Option<AlertKind>,
    expression: Expression,
    skin: Skin,
    style: Style,
    mascot_on: bool,
    lang: Lang,
    scale_bits: u32,
    track: u64,
    has_art: bool,
    playing: bool,
    controls: Controls,
    session: (usize, usize),
    timer_running: Option<bool>,
    stopwatch_running: Option<bool>,
    reduce_motion: bool,
    presets: Vec<u32>,
    lyric: u8,
    battery: Option<(u8, bool)>,
    download: Option<(String, usize)>,
}

struct Content {
    root: ContainerVisual,
    d: Dyn,
    progress_sync: Option<(u64, Ms, bool)>,
}

#[derive(Debug, Default)]
struct Pointer {
    down: Option<Point>,
    consumed: bool,
    seek: Option<f32>,
}

#[derive(Debug)]
struct PrivacyDot {
    visual: ShapeVisual,
    shape: CompositionSpriteShape,
    shown: bool,
    color: Option<Color>,
}

#[derive(Debug)]
enum LyricStatus {
    Loading,
    Missing,
    Offline(Ms),
    Ready(Lyrics),
}

#[derive(Debug)]
struct LyricSlot {
    track: u64,
    status: LyricStatus,
}

pub struct IslandApp {
    window: Window,
    proxy: UiProxy,
    _host: CompositionHost,
    ui: Ui,
    stage: ContainerVisual,
    main: Pill,
    bubble: Pill,
    retracted: bool,
    scale_rest: f32,
    last_ordered: Vec<ActivityKind>,
    full_shape: Shape,
    region: Rect,
    settled_region: Rect,
    watching: bool,
    mem_dirty: bool,
    content: Option<Content>,
    bubble_content: Option<ContainerVisual>,
    retired: Vec<(u64, ContainerVisual, bool)>,
    key: Option<ViewKey>,
    shape: Shape,
    visible: bool,
    window_px: (i32, i32),
    window_origin: (i32, i32),
    anchor_px: Option<i32>,
    drag: Option<(i32, i32)>,
    dragging: bool,
    gazing: bool,
    gaze_last: Option<(i32, i32)>,
    morph_seq: u64,
    morph_pending: bool,
    retire_seq: u64,
    fx: Vec<(u64, ContainerVisual)>,
    fx_seq: u64,
    hover_button: Option<Button>,
    pressed_button: Option<Button>,
    settings: Option<Settings>,

    paths: Paths,
    exe: PathBuf,
    cfg: Config,
    cfg_text: String,
    warnings: Vec<Warning>,
    island: Island,
    acts: Activities,
    media: Option<MediaService>,
    snap: Option<MediaSnapshot>,
    art: Option<(u64, Artwork)>,
    lyric: Option<LyricSlot>,
    lyric_cache: VecDeque<(String, Option<Lyrics>)>,
    lyric_inflight: Option<(u64, String)>,
    presence: MediaPresence,
    timer: Option<Countdown>,
    done_timer: Option<Countdown>,
    stopwatch: Option<Stopwatch>,
    battery: BatteryMonitor,
    power: Option<PowerReading>,
    privacy: Privacy,
    dot: PrivacyDot,
    lip_tint: Option<Color>,
    volume: Option<Volume>,
    tray: Option<TrayIcon>,
    fullscreen_hint: bool,
    pointer: Pointer,
    wheel_acc: i32,
    inside: bool,
    reveal_pending: bool,
    reveal_anchor: Option<Point>,
    esc_registered: bool,
    hotkey_ok: bool,
    downloads: Downloads,
    downloads_dir: Option<PathBuf>,
    downloads_watching: bool,
    downloads_scan_pending: bool,
    download_info: Option<DownloadInfo>,
    last_download: Option<PathBuf>,
    bluetooth: Bluetooth,
    bt_watcher: Option<BluetoothWatcher>,
    brightness: Option<Brightness>,
    brightness_acc: f32,
    brightness_warned: bool,
}

impl std::fmt::Debug for IslandApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IslandApp").field("shape", &self.shape).field("visible", &self.visible).finish()
    }
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    if a.w <= 0.0 {
        return b;
    }
    if b.w <= 0.0 {
        return a;
    }
    let x0 = a.x.min(b.x);
    let x1 = (a.x + a.w).max(b.x + b.w);
    Rect::new(x0, 0.0, x1 - x0, a.h.max(b.h))
}

fn slot_of(kind: ActivityKind) -> Slot {
    match kind {
        ActivityKind::Timer => Slot::Timer,
        ActivityKind::Stopwatch => Slot::Stopwatch,
        ActivityKind::Download => Slot::Download,
        _ => Slot::Media,
    }
}

fn flag(on: bool) -> &'static str {
    if on { "true" } else { "false" }
}

fn quoted(s: &str) -> String {
    format!("\"{s}\"")
}

fn lyric_key(q: &Query) -> String {
    format!("{}\u{1}{}\u{1}{}", q.title.to_lowercase(), q.artist.to_lowercase(), q.album.to_lowercase())
}

fn query_of(snap: &MediaSnapshot) -> Query {
    Query::new(&snap.title, &snap.artist, &snap.album, snap.timeline.map_or(0, |t| t.end))
}

fn resolve_mode(theme: Theme) -> Mode {
    theme.resolve(theme == Theme::System && shell::apps_use_light_theme())
}

fn lyric_timing(snap: &MediaSnapshot, now: Ms) -> (Ms, Ms) {
    let pos = snap.position_at(now).unwrap_or(0) + LYRIC_LEAD_MS;
    (pos, snap.timeline.map_or(0, |t| t.end))
}

impl IslandApp {
    pub fn install(owned: &mut OwnedWindow, paths: Paths, exe: PathBuf, open_settings: bool) -> Result<()> {
        owned.register_power_notifications();
        owned.register_fullscreen_notifications();
        owned.register_window_notifications();
        let mut app = IslandApp::new(owned.window(), owned.proxy(), paths, exe)?;
        app.update();
        if open_settings {
            app.open_settings();
        }
        owned.set_handler(move |_, ev| app.handle(ev));
        Ok(())
    }

    fn new(window: Window, proxy: UiProxy, paths: Paths, exe: PathBuf) -> Result<IslandApp> {
        let (cfg, cfg_text, warnings) = load_config(&paths);
        let host = CompositionHost::new(&window)?;
        let anim = Animator::new(&host.compositor);
        let gfx = Rc::new(Gfx::new(&host.compositor)?);
        let scale = scale_for_dpi(primary_monitor().dpi);
        let pal = theme::palette(cfg.style, resolve_mode(cfg.theme));
        let ui = Ui { c: host.compositor.clone(), anim, gfx, scale, pal, accent: cfg.mascot.skin.color() };

        let stage = ui.container(&host.root)?;
        stage.SetRelativeOffsetAdjustment(v3(0.5, 0.0, 0.0))?;
        stage.SetScale(v3(scale, scale, 1.0))?;
        let bubble = Pill::new(&ui, &stage, pal.base)?;
        let main = Pill::new(&ui, &stage, pal.base)?;
        for (pill, size) in [(&main, layout::IDLE), (&bubble, BUBBLE)] {
            pill.visual.SetSize(v2(size.w, size.h))?;
            pill.visual.SetOpacity(0.0)?;
        }
        let (dot_visual, dot_shape) = ui.dot(DOT, notchi_core::privacy::MIC_COLOR)?;
        dot_visual.SetOpacity(0.0)?;
        dot_visual.SetRelativeOffsetAdjustment(v3(1.0, 0.0, 0.0))?;
        dot_visual.SetOffset(v3(-DOT - DOT_INSET, DOT_INSET, 0.0))?;
        main.visual.Children()?.InsertAtTop(&dot_visual)?;
        let dot = PrivacyDot { visual: dot_visual, shape: dot_shape, shown: false, color: None };

        let mut app = IslandApp {
            window,
            proxy,
            _host: host,
            ui,
            stage,
            main,
            bubble,
            retracted: false,
            scale_rest: 1.0,
            last_ordered: Vec::new(),
            full_shape: Shape::single(layout::IDLE),
            region: Rect::new(0.0, 0.0, 0.0, 0.0),
            settled_region: Rect::new(0.0, 0.0, 0.0, 0.0),
            watching: false,
            mem_dirty: true,
            content: None,
            bubble_content: None,
            retired: Vec::new(),
            key: None,
            shape: Shape::single(layout::IDLE),
            visible: false,
            window_px: (0, 0),
            window_origin: (0, 0),
            anchor_px: None,
            drag: None,
            dragging: false,
            gazing: false,
            gaze_last: None,
            morph_seq: 0,
            morph_pending: false,
            retire_seq: 0,
            fx: Vec::new(),
            fx_seq: 0,
            hover_button: None,
            pressed_button: None,
            settings: None,
            paths,
            exe,
            island: Island::new(cfg.island.behavior),
            acts: Activities::new(),
            media: None,
            snap: None,
            art: None,
            lyric: None,
            lyric_cache: VecDeque::new(),
            lyric_inflight: None,
            presence: MediaPresence::new(cfg.media.paused_linger_seconds as Ms * 1000),
            timer: None,
            done_timer: None,
            stopwatch: None,
            battery: BatteryMonitor::new(&cfg.power.low_thresholds),
            power: None,
            privacy: Privacy::default(),
            dot,
            lip_tint: None,
            volume: None,
            tray: None,
            fullscreen_hint: false,
            pointer: Pointer::default(),
            wheel_acc: 0,
            inside: false,
            reveal_pending: false,
            reveal_anchor: None,
            esc_registered: false,
            hotkey_ok: false,
            downloads: Downloads::new(),
            downloads_dir: None,
            downloads_watching: false,
            downloads_scan_pending: false,
            download_info: None,
            last_download: None,
            bluetooth: Bluetooth::new(),
            bt_watcher: None,
            brightness: None,
            brightness_acc: 0.0,
            brightness_warned: false,
            cfg: Config::default(),
            cfg_text,
            warnings: Vec::new(),
        };
        let icon = IconImage { size: 32, rgba: mascot_rgba(32, cfg.mascot.skin) };
        app.tray = TrayIcon::new(&app.window, &icon, t(cfg.language, Text::TrayTooltip)).ok();
        app.apply_config(cfg, warnings, true);
        fswatch::watch_dir(&app.paths.dir, app.proxy.clone(), "notchi-config-watch", || Event::FileChanged);
        if !privacy::watch(app.proxy.clone()) {
            log("indikator privasi tidak tersedia");
        }
        let reading = power::read();
        app.power = Some(reading);
        if app.cfg.power.enabled {
            let _ = app.battery.update(reading).map(|e| app.power_alert(e));
        }
        app.check_fullscreen();
        app.layout_window();
        Ok(app)
    }

    fn apply_config(&mut self, new: Config, warnings: Vec<Warning>, first: bool) {
        let old = std::mem::replace(&mut self.cfg, new);
        self.warnings = warnings;
        self.island.set_behavior(self.cfg.island.behavior);
        self.island.set_idle_pinned(self.side_buddy());
        self.ui.accent = self.cfg.mascot.skin.color();
        let cfg = &self.cfg;

        let hotkey_changed = first || old.island.hotkey != cfg.island.hotkey;
        if hotkey_changed {
            self.window.unregister_hotkey(HK_TOGGLE);
            self.hotkey_ok = self.window.register_hotkey(HK_TOGGLE, &cfg.island.hotkey);
        }
        self.sync_sources();
        let cfg = &self.cfg;
        let filter = SourceFilter { allow: cfg.media.allow.clone(), block: cfg.media.block.clone() };
        match (&self.media, cfg.media.enabled) {
            (None, true) => self.media = Some(MediaService::start(self.proxy.clone(), filter)),
            (Some(_), false) => {
                self.media = None;
                self.snap = None;
                self.art = None;
            }
            (Some(m), true) if old.media.allow != cfg.media.allow || old.media.block != cfg.media.block => {
                m.send(MediaCommand::SetFilter(filter))
            }
            _ => {}
        }
        if first || old.media.paused_linger_seconds != cfg.media.paused_linger_seconds {
            self.presence = MediaPresence::new(cfg.media.paused_linger_seconds as Ms * 1000);
            self.presence.update(self.snap.as_ref(), now_ms());
        }
        if first || old.power.low_thresholds != cfg.power.low_thresholds {
            self.battery = BatteryMonitor::new(&cfg.power.low_thresholds);
        }
        if (first || old.launch_at_login != cfg.launch_at_login) && shell::launch_at_login_enabled() != cfg.launch_at_login {
            shell::set_launch_at_login(cfg.launch_at_login, &self.exe);
        }
        if !self.cfg.media.lyrics {
            self.lyric = None;
            self.window.kill_timer(T_LYRICS);
        }
        if !first
            && old.mascot.skin != self.cfg.mascot.skin
            && let Some(tray) = self.tray.take()
        {
            drop(tray);
            let icon = IconImage { size: 32, rgba: mascot_rgba(32, self.cfg.mascot.skin) };
            self.tray = TrayIcon::new(&self.window, &icon, "Notchi").ok();
        }
        if !self.refresh_theme(first) && old.liquid != self.cfg.liquid {
            self.apply_liquid();
        }
        if hotkey_changed && let Some(s) = self.settings.as_mut() {
            s.set_hotkey(self.cfg.island.hotkey.display());
        }
        self.refresh_motion();
        self.update_tooltip();
        self.sync_settings();
        self.key = None;
        if !first && old.island.x != self.cfg.island.x && !self.dragging {
            self.anchor_px = None;
            self.layout_window();
        }
        if !first && old.island.hover_zone != self.cfg.island.hover_zone {
            self.settled_region = self.region_of(self.shape);
            self.set_region(self.settled_region);
        }
        if first
            || old.island.avoid_windows != self.cfg.island.avoid_windows
            || old.island.x != self.cfg.island.x
        {
            self.check_covered();
        }
    }

    fn update_tooltip(&mut self) {
        let lang = self.cfg.language;
        let mut tip = t(lang, Text::TrayTooltip).to_string();
        if !self.hotkey_ok {
            tip = format!("Notchi — {} ({})", t(lang, Text::HotkeyTaken), self.cfg.island.hotkey.display());
        } else if let Some(w) = self.warnings.first() {
            tip = format!("Notchi — {}: {} {}", t(lang, Text::ConfigInvalid), w.key, w.message);
        }
        if let Some(tray) = &mut self.tray {
            tray.set_tooltip(&tip);
        }
    }

    fn reload_config(&mut self) {
        let (cfg, text, warnings) = load_config(&self.paths);
        if cfg == self.cfg && text == self.cfg_text {
            return;
        }
        self.cfg_text = text;
        self.apply_config(cfg, warnings, false);
    }

    fn persist_with(&mut self, edit: impl FnOnce(&str) -> String) {
        let text = edit(&self.cfg_text);
        if std::fs::write(&self.paths.config, &text).is_ok() {
            self.cfg_text = text.clone();
        }
        let (cfg, warnings) = config::parse(&text);
        self.apply_config(cfg, warnings, false);
    }

    fn persist(&mut self, section: &str, key: &str, value: &str) {
        self.persist_with(|text| config::set_value(text, section, key, value));
    }

    fn persist_x(&mut self, x: f32) {
        let value = format!("{:.4}", x.clamp(0.0, 1.0));
        self.persist_with(|text| config::set_value(&config::remove_value(text, "island", "position"), "island", "x", &value));
    }

    fn persist_behavior(&mut self, b: Behavior) {
        self.persist_with(|text| {
            let text = config::LEGACY_ISLAND_KEYS.iter().fold(text.to_string(), |acc, k| config::remove_value(&acc, "island", k));
            config::set_value(&text, "island", "behavior", &quoted(b.key()))
        });
    }

    /// Terapkan palet dari gaya dan tema (gelap/terang). Tema "system" dibaca ulang
    /// dari Windows setiap kali pengaturan sistem berubah.
    fn refresh_theme(&mut self, force: bool) -> bool {
        let pal = theme::palette(self.cfg.style, resolve_mode(self.cfg.theme));
        if !force && pal == self.ui.pal {
            return false;
        }
        self.ui.pal = pal;
        let _ = self.main.set_fill(&self.ui, pal.base);
        let _ = self.bubble.set_fill(&self.ui, pal.base);
        self.lip_tint = None;
        self.apply_liquid();
        self.key = None;
        true
    }

    fn refresh_motion(&mut self) {
        let reduce = match self.cfg.reduce_motion {
            ReduceMotion::System => !shell::animations_enabled(),
            ReduceMotion::On => true,
            ReduceMotion::Off => false,
        };
        if self.ui.anim.reduce_motion != reduce {
            self.ui.anim.reduce_motion = reduce;
            self.key = None;
            self.sync_settings();
        }
    }

    fn handle(&mut self, ev: Event) {
        let now = now_ms();
        if !matches!(ev, Event::Timer(T_TRIM)) {
            self.mem_dirty = true;
        }
        match ev {
            Event::MouseMove(p) => self.on_mouse_move(p),
            Event::MouseLeave => self.on_mouse_leave(),
            Event::MouseDown(p, MouseButton::Left) => self.on_mouse_down(p),
            Event::MouseUp(p, MouseButton::Left) => self.on_click(p, now),
            Event::MouseUp(_, MouseButton::Right) => self.show_menu(),
            Event::MouseUp(_, MouseButton::Middle) => self.on_middle(now),
            Event::MouseDown(..) => {}
            Event::Wheel(delta) => self.on_wheel(delta, now),
            Event::ShiftWheel(delta) => self.on_shift_wheel(delta, now),
            Event::Timer(id) => self.on_timer(id),
            Event::Hotkey(HK_TOGGLE) => self.island.hotkey(),
            Event::Hotkey(HK_ESC) => self.island.collapse(),
            Event::Hotkey(_) => {}
            Event::DpiChanged(_) | Event::DisplayChanged => self.on_display(),
            Event::Resized => {}
            Event::SettingsChanged => {
                self.refresh_motion();
                if self.cfg.theme == Theme::System && self.refresh_theme(false) {
                    self.sync_settings();
                }
            }
            Event::PowerChanged => {
                let reading = power::read();
                self.power = Some(reading);
                if self.cfg.power.enabled
                    && let Some(e) = self.battery.update(reading)
                {
                    self.power_alert(e);
                }
            }
            Event::Privacy(p) => self.privacy = p,
            Event::FullscreenHint(on) => {
                self.fullscreen_hint = on;
                self.check_fullscreen();
            }
            Event::WindowsArranged => self.window.set_timer(T_COVER, COVER_SETTLE_MS),
            Event::ForegroundChanged => {
                self.check_fullscreen();
                self.check_covered();
                if self.island.is_expanded() && !self.island.is_hover() {
                    self.island.collapse();
                }
            }
            Event::ForeignMouseDown => {
                if self.island.is_expanded() && !self.island.is_hover() {
                    self.island.collapse();
                }
            }
            Event::Tray(TrayEvent::Click) | Event::SecondInstance => self.island.hotkey(),
            Event::OpenSettings => self.open_settings(),
            Event::Tray(TrayEvent::Menu) => self.show_menu(),
            Event::TaskbarCreated => {
                if let Some(tray) = &self.tray {
                    tray.readd();
                }
                self.window.reregister_appbar();
            }
            Event::Media(snap) => self.on_media(snap, now),
            Event::MediaArt { track_id, art } => {
                if self.snap.as_ref().is_some_and(|s| s.track_id == track_id) {
                    self.art = art.map(|a| (track_id, a));
                }
            }
            Event::Lyrics { track_id, lyrics, failed } => self.on_lyrics(track_id, lyrics, failed),
            Event::FileChanged => self.window.set_timer(T_CONFIG, 300),
            Event::DownloadsChanged => self.queue_download_scan(DOWNLOAD_DEBOUNCE_MS),
            Event::Bluetooth(u) => {
                if let Some(c) = self.bluetooth.apply(u)
                    && self.cfg.bluetooth.enabled
                {
                    let alert = AlertKind::Bluetooth { name: c.name, kind: c.kind, connected: c.connected, battery: c.battery };
                    self.island.push_alert(alert, now);
                }
            }
            Event::BluetoothRemoved(id) => self.bluetooth.remove(&id),
            Event::BluetoothReady => self.bluetooth.ready(),
            Event::Brightness(level) => match level {
                Some(level) => {
                    self.island.push_alert(AlertKind::Brightness { level }, now);
                }
                None if !self.brightness_warned => {
                    self.brightness_warned = true;
                    log("kecerahan tidak bisa diatur: layar tidak mendukung WMI maupun DDC/CI");
                }
                None => {}
            },
            Event::User(tag) => self.on_batch(tag),
            Event::Panel(inner) => self.on_panel(*inner),
            Event::Close | Event::Key(_) => {}
        }
        self.update();
    }

    fn on_timer(&mut self, id: usize) {
        match id {
            T_LONG_PRESS => {
                self.window.kill_timer(T_LONG_PRESS);
                if self.pointer.down.is_some() && !self.pointer.consumed {
                    self.pointer.consumed = true;
                    self.release_button();
                    self.island.long_press();
                }
            }
            T_COLLAPSE => {
                self.window.kill_timer(T_COLLAPSE);
                if !self.island.is_hover() {
                    self.island.collapse();
                }
            }
            T_CONFIG => {
                self.window.kill_timer(T_CONFIG);
                self.reload_config();
            }
            T_RETRACT => {
                self.window.kill_timer(T_RETRACT);
                self.island.set_hover(false);
            }
            T_TRIM => {
                self.window.kill_timer(T_TRIM);
                if self.mem_dirty {
                    self.mem_dirty = false;
                    self.ui.gfx.trim();
                    sys::trim_memory();
                }
            }
            T_LYRICS => {
                self.window.kill_timer(T_LYRICS);
                self.fetch_lyrics();
            }
            T_REVEAL => {
                self.window.kill_timer(T_REVEAL);
                self.reveal_pending = false;
                self.reveal_anchor = None;
                if self.inside {
                    self.reveal();
                }
            }
            T_COVER => {
                self.window.kill_timer(T_COVER);
                self.check_covered();
            }
            T_GAZE => self.gaze(),
            T_DOWNLOADS => {
                self.window.kill_timer(T_DOWNLOADS);
                self.downloads_scan_pending = false;
                self.scan_downloads();
            }
            _ => {}
        }
    }

    fn reveal(&mut self) {
        self.island.set_hover(true);
        self.window.kill_timer(T_COLLAPSE);
    }

    fn on_media(&mut self, snap: Option<MediaSnapshot>, now: Ms) {
        let track = snap.as_ref().map(|s| s.track_id);
        if track != self.snap.as_ref().map(|s| s.track_id) {
            self.art = None;
            if self.lyric.as_ref().map(|l| l.track) != track {
                self.lyric = None;
            }
        }
        self.presence.update(snap.as_ref(), now);
        self.snap = snap;
    }

    fn ensure_lyrics(&mut self) {
        let Some(snap) = self.snap.as_ref() else { return };
        let track = snap.track_id;
        let now = now_ms();
        let fresh = |l: &LyricSlot| match l.status {
            LyricStatus::Offline(at) => now < at + LYRIC_RETRY_MS,
            _ => true,
        };
        if self.lyric.as_ref().is_some_and(|l| l.track == track && fresh(l)) {
            return;
        }
        let q = query_of(snap);
        let status = if !q.is_usable() {
            LyricStatus::Missing
        } else {
            let key = lyric_key(&q);
            match self.lyric_cache.iter().find(|(k, _)| *k == key) {
                Some((_, Some(l))) => LyricStatus::Ready(l.clone()),
                Some((_, None)) => LyricStatus::Missing,
                None => {
                    self.window.set_timer(T_LYRICS, LYRICS_DEBOUNCE_MS);
                    LyricStatus::Loading
                }
            }
        };
        self.lyric = Some(LyricSlot { track, status });
    }

    fn fetch_lyrics(&mut self) {
        let Some(snap) = self.snap.as_ref() else { return };
        let track = snap.track_id;
        let loading = self.lyric.as_ref().is_some_and(|l| l.track == track && matches!(l.status, LyricStatus::Loading));
        if !loading || self.lyric_inflight.as_ref().is_some_and(|(t, _)| *t == track) {
            return;
        }
        let q = query_of(snap);
        let key = lyric_key(&q);
        let proxy = self.proxy.clone();
        let spawned = std::thread::Builder::new().name("notchi-lyrics".into()).stack_size(256 * 1024).spawn(move || {
            let r = net::fetch_lyrics(&q);
            proxy.post(Event::Lyrics { track_id: track, lyrics: r.lyrics, failed: r.retry });
        });
        match spawned {
            Ok(_) => self.lyric_inflight = Some((track, key)),
            Err(_) => self.lyric = Some(LyricSlot { track, status: LyricStatus::Missing }),
        }
    }

    fn on_lyrics(&mut self, track: u64, lyrics: Option<Lyrics>, failed: bool) {
        let Some((t, key)) = self.lyric_inflight.take_if(|(t, _)| *t == track) else { return };
        if !failed {
            self.lyric_cache.retain(|(k, _)| *k != key);
            if self.lyric_cache.len() >= LYRIC_CACHE {
                self.lyric_cache.pop_front();
            }
            self.lyric_cache.push_back((key, lyrics.clone()));
        }
        if self.lyric.as_ref().is_some_and(|l| l.track == t) {
            let status = match lyrics {
                Some(l) => LyricStatus::Ready(l),
                None if failed => LyricStatus::Offline(now_ms()),
                None => LyricStatus::Missing,
            };
            self.lyric = Some(LyricSlot { track: t, status });
        }
    }

    fn lyric_state(&self) -> Option<LyricState<'_>> {
        if !self.cfg.media.lyrics {
            return None;
        }
        let track = self.snap.as_ref().map(|s| s.track_id);
        Some(match self.lyric.as_ref().filter(|l| Some(l.track) == track).map(|l| &l.status) {
            Some(LyricStatus::Ready(l)) => LyricState::Ready(l),
            Some(LyricStatus::Missing) => LyricState::Missing,
            Some(LyricStatus::Offline(_)) => LyricState::Offline,
            _ => LyricState::Loading,
        })
    }

    fn power_alert(&mut self, e: PowerEvent) {
        let alert = match e {
            PowerEvent::PluggedIn { percent } => AlertKind::Charging { percent },
            PowerEvent::Unplugged { percent } => AlertKind::Unplugged { percent },
            PowerEvent::Low { percent, critical } => AlertKind::LowBattery { percent, critical },
        };
        self.island.push_alert(alert, now_ms());
    }

    fn check_fullscreen(&mut self) {
        let fs = self.cfg.island.hide_in_fullscreen && (self.fullscreen_hint || shell::fullscreen_busy());
        if fs != self.island.is_fullscreen() {
            self.island.set_fullscreen(fs);
        }
    }

    fn check_covered(&mut self) {
        let covered = self.cfg.island.avoid_windows && {
            let mon = primary_monitor();
            let x = self.anchor(&mon);
            let y = mon.top + (layout::PILL_H * self.ui.scale).round() as i32 - 1;
            self.window.app_beneath(x, y)
        };
        if covered != self.island.is_covered() {
            self.island.set_covered(covered);
        }
    }

    fn on_display(&mut self) {
        let scale = scale_for_dpi(primary_monitor().dpi);
        if (scale - self.ui.scale).abs() > f32::EPSILON {
            self.ui.scale = scale;
            let _ = self.stage.SetScale(v3(scale, scale, 1.0));
            self.apply_liquid();
            self.key = None;
        }
        self.anchor_px = None;
        self.layout_window();
        self.check_covered();
    }

    fn window_width_dp(&self) -> f32 {
        self.window_px.0 as f32 / self.ui.scale
    }

    /// Geser pil dari tengah jendela (dp) agar mengikuti posisi pilihan pengguna.
    fn shift_of(&self, shape: &Shape) -> f32 {
        let w = self.window_width_dp();
        let anchor = self.anchor_px.map_or(w / 2.0, |a| (a - self.window_origin.0) as f32 / self.ui.scale);
        layout::pill_center(anchor, w, shape) - w / 2.0
    }

    fn main_rect(&self) -> Rect {
        let mut r = self.shape.main_rect(self.window_width_dp());
        r.x += self.shift_of(&self.shape);
        r
    }

    fn pill_local(&self, p: Point) -> Option<Point> {
        let r = self.main_rect();
        r.contains(p).then(|| Point::new(p.x - r.x, p.y - r.y))
    }

    fn stage_point(&self, local: Point) -> Point {
        let main_x = if self.shape.bubble { -(BUBBLE_GAP + BUBBLE.w) / 2.0 } else { 0.0 };
        Point::new(main_x - self.shape.main.w / 2.0 + local.x, local.y)
    }

    fn on_mouse_move(&mut self, p: Point) {
        self.inside = true;
        self.sync_gaze();
        self.window.kill_timer(T_RETRACT);
        if !self.island.is_hover() {
            if !self.retracted {
                self.reveal();
            } else {
                let zone = self.cfg.island.hover_zone;
                let moved = match (self.reveal_anchor, zone.rest_tolerance()) {
                    (Some(a), Some(tol)) => (p.x - a.x).abs() > tol || (p.y - a.y).abs() > tol,
                    _ => false,
                };
                if !self.reveal_pending || moved {
                    // Zona kecil: hitungan diulang selama kursor masih bergerak, jadi kursor
                    // yang melintas menuju tab browser tidak memunculkan Island.
                    self.reveal_pending = true;
                    self.reveal_anchor = Some(p);
                    self.window.set_timer(T_REVEAL, zone.dwell_ms());
                }
            }
        }
        if self.pointer.seek.is_some() {
            let pill = self.main_rect();
            if let Some(bar) = self.content.as_ref().and_then(|c| c.d.seek.as_ref()).map(|s| s.rect) {
                let frac = layout::seek_fraction(bar, p.x - pill.x);
                self.pointer.seek = Some(frac);
                self.preview_seek(frac);
            }
        }
        if let Some(down) = self.pointer.down {
            let dx = p.x - down.x;
            if self.dragging {
                self.drag_to();
            } else if !self.pointer.consumed && !self.island.is_expanded() && self.drag.is_some() && dx.abs() > DRAG_DP {
                // Seret pil yang tertutup ke kiri/kanan untuk memindahkannya di sepanjang tepi atas.
                self.pointer.consumed = true;
                self.window.kill_timer(T_LONG_PRESS);
                self.release_button();
                self.dragging = true;
                self.drag_to();
            } else if !self.pointer.consumed && dx.abs() > SWIPE_DP {
                self.pointer.consumed = true;
                self.release_button();
                self.swipe(if dx > 0.0 { -1 } else { 1 });
            }
        }
        if let Err(e) = self.track_pointer(Some(p)) {
            log(&format!("pointer gagal: {e}"));
        }
    }

    fn drag_to(&mut self) {
        let (Some((c0, a0)), Some(c)) = (self.drag, cursor_pos()) else { return };
        let mon = primary_monitor();
        let a = (a0 + c.0 - c0).clamp(mon.left, mon.right);
        if self.anchor_px != Some(a) {
            self.anchor_px = Some(a);
            self.layout_window();
        }
    }

    fn finish_drag(&mut self) {
        self.dragging = false;
        self.drag = None;
        let mon = primary_monitor();
        let x = (self.anchor(&mon) - mon.left) as f32 / mon.width().max(1) as f32;
        let x = (x.clamp(0.0, 1.0) * 10_000.0).round() / 10_000.0;
        self.cfg.island.x = x;
        self.island.set_idle_pinned(self.side_buddy());
        self.persist_x(x);
        self.key = None;
        self.check_covered();
        self.update();
        let s = self.ui.scale;
        let over = cursor_pos().is_some_and(|(cx, cy)| {
            let p = Point::new((cx - self.window_origin.0) as f32 / s, (cy - self.window_origin.1) as f32 / s);
            self.region.contains(p)
        });
        if !over {
            self.on_mouse_leave();
        }
    }

    fn on_mouse_leave(&mut self) {
        if self.dragging {
            // Selama diseret tetikus ditangkap; drag selesai saat tombol dilepas.
            return;
        }
        self.inside = false;
        self.reveal_pending = false;
        self.reveal_anchor = None;
        self.window.kill_timer(T_REVEAL);
        if self.pointer.seek.is_some()
            && let Some(c) = self.content.as_mut()
        {
            c.progress_sync = None;
        }
        self.pointer = Pointer::default();
        // Tombol ditekan lalu kursor keluar tanpa dilepas: jangan biarkan drag basi
        // menghalangi perubahan posisi dari Pengaturan atau config.toml.
        self.drag = None;
        self.window.kill_timer(T_LONG_PRESS);
        self.wheel_acc = 0;
        self.release_button();
        let _ = self.track_pointer(None);
        self.sync_gaze();
        if self.island.tucks() && !self.island.is_expanded() {
            self.window.set_timer(T_RETRACT, RETRACT_DELAY_MS);
        } else {
            self.island.set_hover(false);
        }
        if self.island.is_expanded() {
            self.window.set_timer(T_COLLAPSE, COLLAPSE_DELAY_MS);
        }
    }

    fn on_mouse_down(&mut self, p: Point) {
        if self.retracted {
            return;
        }
        if let Some(frac) = self.seek_hit(p) {
            self.window.kill_timer(T_LONG_PRESS);
            self.release_button();
            self.pointer = Pointer { down: Some(p), consumed: true, seek: Some(frac) };
            self.preview_seek(frac);
            return;
        }
        self.pointer = Pointer { down: Some(p), consumed: false, seek: None };
        let mon = primary_monitor();
        self.drag = cursor_pos().map(|c| (c.0, self.anchor(&mon)));
        self.window.set_timer(T_LONG_PRESS, LONG_PRESS_MS);
        let hit = self.button_at(p);
        if let Some(b) = hit.filter(|b| *b != Button::Mascot) {
            self.pop_button(b, PRESS_POP);
        }
        self.pressed_button = hit;
    }

    fn release_button(&mut self) {
        if let Some(b) = self.pressed_button.take()
            && b != Button::Mascot
        {
            self.pop_button(b, if self.hover_button == Some(b) { HOVER_POP } else { 1.0 });
        }
    }

    fn peeking(&self) -> bool {
        matches!(self.key.as_ref().map(|k| k.kind), Some(ViewKind::Peek(_)))
    }

    fn button_at(&self, p: Point) -> Option<Button> {
        if !self.island.is_expanded() && !self.peeking() {
            return None;
        }
        let local = self.pill_local(p)?;
        self.content.as_ref().and_then(|c| layout::hit(&c.d.buttons, local))
    }

    fn seek_hit(&self, p: Point) -> Option<f32> {
        if !self.island.is_expanded() {
            return None;
        }
        let local = self.pill_local(p)?;
        let bar = self.content.as_ref()?.d.seek.as_ref()?.rect;
        layout::seek_zone(bar).contains(local).then(|| layout::seek_fraction(bar, local.x))
    }

    fn preview_seek(&mut self, frac: f32) {
        let Some(end) = self.snap.as_ref().and_then(|s| s.timeline).map(|t| t.end) else { return };
        let Some(c) = self.content.as_mut() else { return };
        let ui = &self.ui;
        c.progress_sync = None;
        if let Some((fill, width)) = c.d.progress.as_ref() {
            let _ = ui.anim.stop(fill, "Size.X");
            let _ = fill.SetSize(v2(width * frac, 4.0));
        }
        let (el, rem) = views::times_at((end as f64 * frac as f64).round() as Ms, end);
        if let Some(t) = c.d.elapsed.as_mut() {
            let _ = t.set(ui, &el);
        }
        if let Some(t) = c.d.remaining.as_mut() {
            let _ = t.set(ui, &rem);
        }
        if let Some(s) = c.d.seek.as_mut() {
            let _ = s.show(ui, true);
        }
    }

    fn commit_seek(&mut self, frac: f32, now: Ms) {
        let target = self.snap.as_ref().and_then(|s| seek_target(s, frac));
        if let (Some(target), Some(m)) = (target, &self.media) {
            m.send(MediaCommand::Seek(target));
            if let Some(tl) = self.snap.as_mut().and_then(|s| s.timeline.as_mut()) {
                tl.position = target;
                tl.reported_at = now;
            }
        }
        if let Some(c) = self.content.as_mut() {
            c.progress_sync = None;
        }
    }

    fn on_middle(&mut self, now: Ms) {
        if self.retracted {
            return;
        }
        match self.acts.primary() {
            Some(ActivityKind::Media) => {
                let ok = self.snap.as_ref().is_some_and(|s| s.controls.can_play_pause(s.is_playing()));
                if ok && let Some(m) = &self.media {
                    m.send(MediaCommand::TogglePlayPause);
                }
            }
            Some(ActivityKind::Timer) => {
                if let Some(t) = &mut self.timer {
                    t.toggle(now);
                }
            }
            Some(ActivityKind::Stopwatch) => {
                if let Some(s) = &mut self.stopwatch {
                    s.toggle(now);
                }
            }
            _ => {}
        }
    }

    fn pop_button(&self, b: Button, scale: f32) {
        if let Some(h) = self.content.as_ref().and_then(|c| c.d.host(b)) {
            let _ = self.ui.pop(h, scale);
        }
    }

    fn track_pointer(&mut self, p: Option<Point>) -> Result<()> {
        let local = p.and_then(|p| self.pill_local(p));
        let hit = p.and_then(|p| self.button_at(p));
        if let Some(m) = self.content.as_mut().and_then(|c| c.d.mascot.as_mut()) {
            match local {
                Some(l) => {
                    let c = m.center();
                    m.look(&self.ui, (l.x - c.x) / 90.0, (l.y - c.y) / 50.0)?;
                }
                None => m.look(&self.ui, 0.0, 0.0)?,
            }
        }
        let over_seek = self.pointer.seek.is_some() || p.is_some_and(|p| self.seek_hit(p).is_some());
        if let Some(s) = self.content.as_mut().and_then(|c| c.d.seek.as_mut()) {
            s.show(&self.ui, over_seek)?;
        }
        if hit == self.hover_button {
            return Ok(());
        }
        let pressed = self.pressed_button;
        for (b, on) in [(self.hover_button, false), (hit, true)] {
            let Some(b) = b else { continue };
            if b == Button::Mascot {
                if let Some(m) = self.content.as_mut().and_then(|c| c.d.mascot.as_mut()) {
                    m.perk(&self.ui, on)?;
                }
            } else if pressed != Some(b) {
                self.pop_button(b, if on { HOVER_POP } else { 1.0 });
            }
        }
        self.hover_button = hit;
        Ok(())
    }

    fn swipe(&mut self, step: i32) {
        let primary = self.acts.primary();
        let multi_session = self.snap.as_ref().is_some_and(|s| s.session_count > 1);
        if primary == Some(ActivityKind::Media) && multi_session && self.acts.len() < 2 {
            if let Some(m) = &self.media {
                m.send(MediaCommand::CycleSession(step));
            }
        } else {
            self.acts.rotate(step);
        }
    }

    fn on_click(&mut self, p: Point, now: Ms) {
        self.window.kill_timer(T_LONG_PRESS);
        if self.retracted && self.pointer.down.is_none() {
            if self.inside {
                self.reveal_pending = false;
                self.reveal_anchor = None;
                self.window.kill_timer(T_REVEAL);
                self.reveal();
            }
            return;
        }
        if self.dragging {
            self.pointer = Pointer::default();
            self.finish_drag();
            return;
        }
        self.drag = None;
        if let Some(frac) = self.pointer.seek {
            self.pointer = Pointer::default();
            self.commit_seek(frac, now);
            return;
        }
        let consumed = self.pointer.consumed;
        self.pointer = Pointer::default();
        let pressed = self.pressed_button;
        self.release_button();
        if consumed {
            return;
        }
        let Some(_) = self.pill_local(p) else {
            self.island.collapse();
            return;
        };
        if matches!(self.key.as_ref().and_then(|k| k.alert.as_ref()), Some(AlertKind::DownloadDone { .. }))
            && let Some(path) = self.last_download.clone()
        {
            // Klik alert unduhan selesai: tampilkan berkasnya di Explorer.
            self.island.dismiss_alert();
            shell::reveal(&path);
            return;
        }
        if let Some(b) = self.button_at(p) {
            if pressed == Some(b) {
                self.press(b, now);
            }
            return;
        }
        self.island.click(now);
    }

    fn press(&mut self, b: Button, now: Ms) {
        let media = |cmd| {
            if let Some(m) = &self.media {
                m.send(cmd);
            }
        };
        match b {
            Button::Prev => media(MediaCommand::Prev),
            Button::Next => media(MediaCommand::Next),
            Button::PlayPause => media(MediaCommand::TogglePlayPause),
            Button::TimerAddMinute => {
                if let Some(t) = &mut self.timer {
                    t.add(60_000, now);
                }
            }
            Button::TimerPauseResume => {
                if let Some(t) = &mut self.timer {
                    t.toggle(now);
                }
            }
            Button::TimerCancel => {
                self.timer = None;
                self.island.collapse();
            }
            Button::StopwatchPauseResume => {
                if let Some(s) = &mut self.stopwatch {
                    s.toggle(now);
                }
            }
            Button::StopwatchReset => {
                if let Some(s) = &mut self.stopwatch {
                    s.reset(now);
                }
            }
            Button::StopwatchStop => {
                self.stopwatch = None;
                self.island.collapse();
            }
            Button::Preset(i) => {
                if let Some(&min) = self.cfg.timer.presets_minutes.get(i as usize) {
                    self.start_timer(min, now);
                }
            }
            Button::StartStopwatch => self.stopwatch = Some(Stopwatch::start(now)),
            Button::Lyrics => {
                let on = !self.cfg.media.lyrics;
                self.persist("media", "lyrics", flag(on));
            }
            Button::Mascot => self.boop(),
            Button::OpenDownloads => {
                if let Some(dir) = &self.downloads_dir {
                    shell::open_folder(dir);
                }
                self.island.collapse();
            }
        }
    }

    fn boop(&mut self) {
        let Some(m) = self.content.as_mut().and_then(|c| c.d.mascot.as_mut()) else { return };
        if let Err(e) = m.boop(&self.ui) {
            log(&format!("boop gagal: {e}"));
        }
        let center = m.center();
        let at = self.stage_point(center);
        self.burst(at, Burst::Sparkle, &fx::sparkle_colors(self.ui.accent));
    }

    fn burst(&mut self, at: Point, kind: Burst, colors: &[Color]) {
        self.fx_seq += 1;
        let seed = now_ms().wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (self.fx_seq << 21);
        match fx::burst(&self.ui, &self.stage, Spark { at, kind, colors, seed }, &self.proxy, TAG_FX | self.fx_seq) {
            Ok(Some(v)) => {
                let was_empty = self.fx.is_empty();
                self.fx.push((self.fx_seq, v));
                if was_empty {
                    self.apply_region();
                }
            }
            Ok(None) => {}
            Err(e) => log(&format!("partikel gagal: {e}")),
        }
    }

    fn start_timer(&mut self, minutes: u32, now: Ms) {
        self.timer = Some(Countdown::start(minutes as Ms * 60_000, now));
        self.done_timer = None;
    }

    /// Shift + scroll: kecerahan layar. Bila dimatikan, sama dengan scroll biasa.
    fn on_shift_wheel(&mut self, delta: i32, now: Ms) {
        if !self.cfg.media.scroll_brightness {
            return self.on_wheel(delta, now);
        }
        if self.retracted {
            return;
        }
        if self.brightness.is_none() {
            self.brightness = Brightness::start(self.proxy.clone());
        }
        self.brightness_acc += delta as f32 / WHEEL_NOTCH as f32 * BRIGHTNESS_STEP;
        let step = self.brightness_acc.trunc() as i32;
        if step != 0 {
            self.brightness_acc -= step as f32;
            if let Some(b) = &self.brightness {
                b.step(step);
            }
        }
    }

    /// Mulai atau hentikan sumber aktivitas yang bisa dimatikan di pengaturan.
    fn sync_sources(&mut self) {
        if self.cfg.downloads.enabled && !self.downloads_watching {
            self.downloads_dir = self.downloads_dir.take().or_else(shell::downloads_dir);
            if let Some(dir) = &self.downloads_dir {
                self.downloads_watching =
                    fswatch::watch_dir(dir, self.proxy.clone(), "notchi-downloads-watch", || Event::DownloadsChanged);
                self.queue_download_scan(DOWNLOAD_DEBOUNCE_MS);
            }
        }
        if !self.cfg.downloads.enabled {
            self.downloads = Downloads::new();
            self.window.kill_timer(T_DOWNLOADS);
            self.downloads_scan_pending = false;
        }
        match (self.cfg.bluetooth.enabled, self.bt_watcher.is_some()) {
            (true, false) => {
                self.bluetooth = Bluetooth::new();
                match BluetoothWatcher::start(self.proxy.clone()) {
                    Ok(w) => self.bt_watcher = Some(w),
                    Err(e) => log(&format!("pemantau Bluetooth gagal: {e}")),
                }
            }
            (false, true) => self.bt_watcher = None,
            _ => {}
        }
    }

    /// Jadwalkan pindaian folder Downloads. Tidak ditunda ulang selama sudah terjadwal, karena
    /// browser menulis terus-menerus dan penundaan ulang membuat pindaian tidak pernah jalan.
    fn queue_download_scan(&mut self, delay: u32) {
        if self.cfg.downloads.enabled && !self.downloads_scan_pending {
            self.downloads_scan_pending = true;
            self.window.set_timer(T_DOWNLOADS, delay);
        }
    }

    fn scan_downloads(&mut self) {
        let Some(dir) = self.downloads_dir.clone() else { return };
        if !self.cfg.downloads.enabled {
            return;
        }
        let wall = std::time::SystemTime::now();
        let mut partials = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if !notchi_core::downloads::is_partial(&name) {
                    continue;
                }
                let Ok(meta) = e.metadata() else { continue };
                if !meta.is_file() {
                    continue;
                }
                let idle = meta.modified().ok().and_then(|m| wall.duration_since(m).ok()).map_or(0, |d| d.as_millis() as Ms);
                partials.push(Partial { name, size: meta.len(), idle_ms: idle });
            }
        }
        let now = now_ms();
        let done = self.downloads.update(&partials, now, |name| {
            std::fs::metadata(dir.join(name)).ok().filter(|m| m.is_file()).map(|m| m.len())
        });
        if let Some(f) = done.into_iter().last() {
            self.last_download = Some(dir.join(&f.name));
            self.island.push_alert(AlertKind::DownloadDone { name: f.name, bytes: f.bytes }, now);
        }
        if self.downloads.is_tracking() {
            self.queue_download_scan(DOWNLOAD_POLL_MS);
        }
    }

    fn on_wheel(&mut self, delta: i32, now: Ms) {
        if self.retracted {
            return;
        }
        let timer_view = matches!(
            self.key.as_ref().map(|k| k.kind),
            Some(ViewKind::Compact(Slot::Timer) | ViewKind::Minimal(Slot::Timer, _) | ViewKind::Card(Card::Timer))
        );
        if timer_view && let Some(t) = self.timer.as_mut() {
            self.wheel_acc += delta;
            let notches = self.wheel_acc / WHEEL_NOTCH;
            self.wheel_acc -= notches * WHEEL_NOTCH;
            for _ in 0..notches.unsigned_abs() {
                if notches > 0 {
                    t.add(MINUTE_MS, now);
                } else {
                    t.subtract(MINUTE_MS, now);
                }
            }
            return;
        }
        self.wheel_acc = 0;
        if !self.cfg.media.scroll_volume {
            return;
        }
        if self.volume.is_none() {
            self.volume = Volume::open().ok();
        }
        let step = delta as f32 / 120.0 * 0.02;
        let result = self.volume.as_ref().map(|v| v.step(step));
        let result = match result {
            Some(Ok(r)) => Some(r),
            _ => {
                self.volume = Volume::open().ok();
                self.volume.as_ref().and_then(|v| v.step(step).ok())
            }
        };
        if let Some((level, muted)) = result {
            let level = (level * 100.0).round() as u8;
            self.island.push_alert(AlertKind::Volume { level, muted }, now);
        }
    }

    fn show_menu(&mut self) {
        let lang = self.cfg.language;
        let now = now_ms();
        let presets: Vec<MenuItem> = self
            .cfg
            .timer
            .presets_minutes
            .iter()
            .enumerate()
            .map(|(i, m)| MenuItem::item(M_PRESET + i as u32, format!("{m} {}", t(lang, Text::Minutes))))
            .collect();
        let current = self.cfg.island.behavior;
        let behaviors = Behavior::ALL
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let label = match b {
                    Behavior::Autohide => t(lang, Text::BehaviorAutohide),
                    Behavior::Sticky => t(lang, Text::BehaviorSticky),
                    Behavior::Hide => t(lang, Text::BehaviorHide),
                };
                MenuItem::check(M_BEHAVIOR + i as u32, label, *b == current)
            })
            .collect();
        let snoozed = self.island.is_snoozed(now);
        let items = vec![
            MenuItem::Submenu { label: t(lang, Text::MenuTimer).into(), items: presets },
            MenuItem::item(M_STOPWATCH, t(lang, Text::MenuStopwatch)),
            MenuItem::Separator,
            MenuItem::Submenu { label: t(lang, Text::MenuBehavior).into(), items: behaviors },
            if snoozed { MenuItem::item(M_SHOW, t(lang, Text::MenuShowAgain)) } else { MenuItem::item(M_HIDE, t(lang, Text::MenuHideHour)) },
            MenuItem::check(M_LOGIN, t(lang, Text::MenuLaunchAtLogin), self.cfg.launch_at_login),
            MenuItem::Separator,
            MenuItem::item(M_SETTINGS, t(lang, Text::MenuSettings)),
            MenuItem::item(M_ABOUT, t(lang, Text::MenuAbout)),
            MenuItem::Separator,
            MenuItem::item(M_QUIT, t(lang, Text::MenuQuit)),
        ];
        let Some(id) = self.window.popup_menu(&items) else { return };
        let now = now_ms();
        match id {
            M_STOPWATCH => {
                self.stopwatch = Some(Stopwatch::start(now));
                self.island.unsnooze();
            }
            id if (M_PRESET..M_PRESET + 10).contains(&id) => {
                if let Some(&m) = self.cfg.timer.presets_minutes.get((id - M_PRESET) as usize) {
                    self.start_timer(m, now);
                    self.island.unsnooze();
                }
            }
            id if (M_BEHAVIOR..M_BEHAVIOR + 3).contains(&id) => {
                self.persist_behavior(Behavior::ALL[(id - M_BEHAVIOR) as usize]);
            }
            M_HIDE => self.island.snooze(now + HOUR_MS),
            M_SHOW => self.island.unsnooze(),
            M_LOGIN => {
                let on = !self.cfg.launch_at_login;
                self.persist("startup", "launch_at_login", flag(on));
            }
            M_SETTINGS => self.open_settings(),
            M_ABOUT => {
                let body = format!("Notchi {}\n\n{}", env!("CARGO_PKG_VERSION"), t(lang, Text::AboutBody));
                self.window.message_box("Notchi", &body);
            }
            M_QUIT => self.shutdown(),
            _ => {}
        }
    }

    fn open_settings(&mut self) {
        if let Some(s) = &self.settings {
            s.focus();
            return;
        }
        let icon = IconImage { size: 32, rgba: mascot_rgba(32, self.cfg.mascot.skin) };
        match Settings::open(&self.ui, &self.proxy, &self.cfg, self.cfg.island.hotkey.display(), &icon) {
            Ok(s) => self.settings = Some(s),
            Err(e) => {
                log(&format!("jendela pengaturan gagal: {e}"));
                shell::edit_text_file(&self.paths.config);
            }
        }
    }

    fn close_settings(&mut self) {
        if self.settings.take().is_some() {
            self.window.set_timer(T_TRIM, TRIM_DELAY_MS);
        }
    }

    fn sync_settings(&mut self) {
        if let Some(s) = self.settings.as_mut()
            && let Err(e) = s.update(&self.ui, &self.cfg)
        {
            log(&format!("pengaturan gagal diperbarui: {e}"));
        }
    }

    fn on_panel(&mut self, ev: Event) {
        let Some(s) = self.settings.as_mut() else { return };
        let r = match ev {
            Event::MouseMove(p) => s.pointer_move(Some(p)).map(|_| None),
            Event::MouseLeave => s.pointer_move(None).map(|_| None),
            Event::MouseDown(p, MouseButton::Left) => s.pointer_down(p).map(|_| None),
            Event::MouseUp(p, MouseButton::Left) => s.pointer_up(p),
            Event::DpiChanged(_) => s.rescale(&self.cfg).map(|_| None),
            Event::Resized => s.resize().map(|_| None),
            Event::Wheel(d) | Event::ShiftWheel(d) => s.wheel(d).map(|_| None),
            Event::Close | Event::Key(VK_ESCAPE) => {
                self.close_settings();
                return;
            }
            _ => Ok(None),
        };
        match r {
            Ok(Some(a)) => self.apply_setting(a),
            Ok(None) => {}
            Err(e) => log(&format!("pengaturan gagal: {e}")),
        }
    }

    fn apply_setting(&mut self, a: Action) {
        let i = a.index;
        let on = i == 1;
        match a.setting {
            Setting::Behavior => self.persist_behavior(Behavior::ALL[i.min(2)]),
            Setting::Liquid => self.persist("motion", "liquid", flag(on)),
            Setting::Fullscreen => self.persist("island", "hide_in_fullscreen", flag(on)),
            Setting::AvoidWindows => self.persist("island", "avoid_windows", flag(on)),
            Setting::SideMascot => self.persist("island", "side_mascot", flag(on)),
            Setting::Position => self.persist_x(Placement::ALL[i.min(2)].x()),
            Setting::HoverZone => self.persist("island", "hover_zone", &quoted(HoverZone::ALL[i.min(1)].key())),
            Setting::Style => self.persist("appearance", "style", &quoted(Style::ALL[i.min(Style::ALL.len() - 1)].key())),
            Setting::Theme => self.persist("appearance", "theme", &quoted(Theme::ALL[i.min(Theme::ALL.len() - 1)].key())),
            Setting::Reduce => self.persist("motion", "reduce", &quoted(["system", "on", "off"][i.min(2)])),
            Setting::Mascot => self.persist("mascot", "enabled", flag(on)),
            Setting::Skin => {
                let skin = Skin::ALL[i.min(Skin::ALL.len() - 1)];
                self.persist("mascot", "skin", &quoted(skin.key()));
            }
            Setting::Media => self.persist("media", "enabled", flag(on)),
            Setting::Lyrics => self.persist("media", "lyrics", flag(on)),
            Setting::Scroll => self.persist("media", "scroll_volume", flag(on)),
            Setting::ScrollBrightness => self.persist("media", "scroll_brightness", flag(on)),
            Setting::Downloads => self.persist("downloads", "enabled", flag(on)),
            Setting::Bluetooth => self.persist("bluetooth", "enabled", flag(on)),
            Setting::Sound => self.persist("timer", "sound", flag(on)),
            Setting::Language => self.persist_with(|text| config::set_value(text, "", "language", &quoted(if on { "en" } else { "id" }))),
            Setting::Login => self.persist("startup", "launch_at_login", flag(on)),
            Setting::Advanced => shell::edit_text_file(&self.paths.config),
            Setting::Boop => {}
        }
    }

    fn shutdown(&mut self) {
        self.settings = None;
        self.tray = None;
        self.media = None;
        self.window.unregister_hotkey(HK_TOGGLE);
        self.window.unregister_hotkey(HK_ESC);
        self.window.hide();
        quit();
    }

    fn sync_activities(&mut self, now: Ms) {
        if let Some(t) = self.timer.filter(|t| t.is_finished(now)) {
            self.timer = None;
            self.done_timer = Some(t);
            self.island.push_alert(AlertKind::TimerDone, now);
            if self.cfg.timer.sound {
                shell::chime();
            }
        }
        let media = self.cfg.media.enabled && self.presence.is_present(self.snap.as_ref(), now);
        self.acts.set(ActivityKind::Media, media);
        self.acts.set(ActivityKind::Timer, self.timer.is_some());
        self.acts.set(ActivityKind::Stopwatch, self.stopwatch.is_some());
        self.download_info = self.cfg.downloads.enabled.then(|| self.downloads.active(now)).and_then(|a| {
            a.first().map(|d| DownloadInfo { name: d.name.clone(), bytes: d.bytes, rate: d.rate, count: a.len() })
        });
        self.acts.set(ActivityKind::Download, self.download_info.is_some());
        self.island.set_activity_count(self.acts.len());
        let ordered = self.acts.ordered();
        if ordered.iter().any(|k| !self.last_ordered.contains(k)) {
            self.check_covered();
            if !self.island.is_covered() {
                self.island.peek(now + PEEK_MS);
            }
        }
        self.last_ordered = ordered;
    }

    fn update(&mut self) {
        let now = now_ms();
        self.island.tick(now);
        self.sync_activities(now);
        if let Err(e) = self.render(now) {
            log(&format!("render gagal: {e}"));
            self.key = None;
        }
        self.schedule(now);
        self.sync_gaze();
    }

    fn side_buddy(&self) -> bool {
        let c = &self.cfg;
        layout::is_side(c.island.x) && c.island.side_mascot && c.mascot.enabled
    }

    /// Mata maskot mengikuti kursor di mana pun di layar, tidak hanya saat kursor di atas pil.
    /// Hanya berjalan selama maskot tampil dan kursor di luar pil; posisi yang sama dilewati.
    fn sync_gaze(&mut self) {
        let has_mascot = self.content.as_ref().is_some_and(|c| c.d.mascot.is_some());
        let want = has_mascot && self.visible && !self.retracted && !self.inside;
        if want != self.gazing {
            self.gazing = want;
            self.gaze_last = None;
            if want {
                self.window.set_timer(T_GAZE, GAZE_MS);
            } else {
                self.window.kill_timer(T_GAZE);
            }
        }
    }

    fn gaze(&mut self) {
        if !self.gazing || self.inside {
            return;
        }
        let Some(pos) = cursor_pos() else { return };
        if self.gaze_last == Some(pos) {
            return;
        }
        self.gaze_last = Some(pos);
        let s = self.ui.scale;
        let p = Point::new((pos.0 - self.window_origin.0) as f32 / s, (pos.1 - self.window_origin.1) as f32 / s);
        let r = self.main_rect();
        let Some(m) = self.content.as_mut().and_then(|c| c.d.mascot.as_mut()) else { return };
        let c = m.center();
        let (vx, vy) = (p.x - r.x - c.x, p.y - r.y - c.y);
        let len = vx.hypot(vy);
        let (dx, dy) = if len < 1.0 {
            (0.0, 0.0)
        } else {
            let k = (len / GAZE_REACH).min(1.0) / len;
            (vx * k, vy * k)
        };
        if let Err(e) = m.look(&self.ui, dx, dy) {
            log(&format!("pandangan maskot gagal: {e}"));
        }
    }

    fn view_kind(&self, pres: &Presentation) -> ViewKind {
        let ordered = self.acts.ordered();
        let glance = self.island.is_hover() && !self.island.is_expanded();
        match pres {
            Presentation::Hidden => ViewKind::Hidden,
            Presentation::Idle if glance => ViewKind::Peek(PeekKind::Idle),
            Presentation::Idle if self.side_buddy() => ViewKind::Buddy,
            Presentation::Idle => ViewKind::Empty,
            Presentation::Compact if glance && ordered[0] == ActivityKind::Media => ViewKind::Peek(PeekKind::Media),
            Presentation::Compact => ViewKind::Compact(slot_of(ordered[0])),
            Presentation::Minimal => ViewKind::Minimal(slot_of(ordered[0]), slot_of(ordered[1])),
            Presentation::Expanded => ViewKind::Card(match ordered.first() {
                Some(ActivityKind::Media) if self.cfg.media.lyrics => Card::Lyrics,
                Some(ActivityKind::Media) => Card::Media,
                Some(ActivityKind::Timer) => Card::Timer,
                Some(ActivityKind::Stopwatch) => Card::Stopwatch,
                Some(ActivityKind::Download) => Card::Download,
                _ => Card::Home,
            }),
            Presentation::Alert(a) if a.is_hud() => ViewKind::Hud,
            Presentation::Alert(_) => ViewKind::Alert,
        }
    }

    fn shape_for(kind: ViewKind, alert: Option<&AlertKind>) -> Shape {
        match kind {
            ViewKind::Hidden | ViewKind::Empty => Shape::single(layout::IDLE),
            ViewKind::Buddy => Shape::single(layout::BUDDY),
            ViewKind::Peek(PeekKind::Idle) => Shape::single(layout::PEEK_IDLE),
            ViewKind::Peek(PeekKind::Media) => Shape::single(layout::PEEK_MEDIA),
            ViewKind::Compact(s) => Shape::single(views::compact_size(s, false)),
            ViewKind::Minimal(s, _) => Shape { main: views::compact_size(s, true), bubble: true },
            ViewKind::Card(c) => Shape::single(c.size()),
            ViewKind::Alert | ViewKind::Hud => {
                Shape::single(alert.map(layout::alert_size).unwrap_or(Size::new(layout::EXPANDED_W, layout::EXPANDED_MIN_H)))
            }
        }
    }

    fn render(&mut self, now: Ms) -> Result<()> {
        let pres = self.island.presentation(now);
        let retracted = self.island.is_retracted(now);
        let full_kind = self.view_kind(&pres);
        let kind = if retracted { ViewKind::Empty } else { full_kind };
        if kind == ViewKind::Card(Card::Lyrics) {
            self.ensure_lyrics();
        }
        let alert = match &pres {
            Presentation::Alert(a) => Some(a.clone()),
            _ => None,
        };
        let primary = self.acts.primary();
        let playing = self.snap.as_ref().is_some_and(|s| s.is_playing());
        let expression = expression_for(MascotContext {
            alert: alert.as_ref(),
            primary,
            media_playing: playing,
            low_battery: self.battery.is_low(),
            reduce_motion: self.ui.anim.reduce_motion,
        });

        let full = Self::shape_for(full_kind, alert.as_ref());
        let shape = if retracted { full.retracted() } else { full };
        let visible = kind != ViewKind::Hidden;
        let lyric = match self.lyric_state() {
            None => 0,
            Some(LyricState::Loading) => 1,
            Some(LyricState::Missing) => 2,
            Some(LyricState::Ready(_)) => 3,
            Some(LyricState::Offline) => 4,
        };
        let key = ViewKey {
            kind,
            alert: alert.clone().map(|a| match a {
                AlertKind::Volume { muted, level } => AlertKind::Volume { muted, level: (level == 0) as u8 },
                AlertKind::Brightness { .. } => AlertKind::Brightness { level: 0 },
                other => other,
            }),
            expression,
            skin: self.cfg.mascot.skin,
            style: self.cfg.style,
            mascot_on: self.cfg.mascot.enabled,
            lang: self.cfg.language,
            scale_bits: self.ui.scale.to_bits(),
            track: self.snap.as_ref().map_or(0, |s| s.track_id),
            has_art: self.art.is_some(),
            playing,
            controls: self.snap.as_ref().map(|s| s.controls).unwrap_or_default(),
            session: self.snap.as_ref().map_or((0, 0), |s| (s.session_index, s.session_count)),
            timer_running: self.timer.map(|t| t.is_running()),
            stopwatch_running: self.stopwatch.map(|s| s.is_running()),
            reduce_motion: self.ui.anim.reduce_motion,
            presets: self.cfg.timer.presets_minutes.clone(),
            lyric: if kind == ViewKind::Card(Card::Lyrics) { lyric } else { 0 },
            battery: if matches!(kind, ViewKind::Card(Card::Home) | ViewKind::Peek(PeekKind::Idle)) {
                self.power.and_then(views::battery_level)
            } else {
                None
            },
            download: self.download_info.as_ref().map(|d| (d.name.clone(), d.count)),
        };

        let revealing = self.retracted && !retracted;
        let retracting = !self.retracted && retracted;
        self.retracted = retracted;
        self.full_shape = full;
        let morphing = shape != self.shape;
        if morphing {
            let preset = if retracting {
                SpringPreset::SMOOTH
            } else if revealing && self.cfg.liquid {
                REVEAL
            } else {
                SpringPreset::BOUNCY
            };
            self.morph(shape, preset)?;
        } else {
            // Bentuk sama tetapi aturan menyingkir berubah (perilaku, jendela di bawahnya,
            // zona hover): perbarui region agar zona pemicu tidak tertinggal.
            let settled = self.region_of(shape);
            if settled != self.settled_region {
                self.settled_region = settled;
                if !self.morph_pending {
                    self.set_region(settled);
                }
            }
        }
        if visible != self.visible {
            self.set_visible(visible)?;
        } else if visible && !self.window.is_visible() {
            self.window.show();
        }
        self.sync_privacy(retracted)?;
        let peek = matches!(kind, ViewKind::Peek(_));
        let hover = self.island.is_hover() && !self.island.tucks() && !self.island.is_expanded() && visible && !peek;
        let rest = if hover { HOVER_SCALE } else { 1.0 };
        if rest != self.scale_rest {
            self.scale_rest = rest;
            self.ui.anim.spring(&self.main.visual, "Scale.X", rest, SpringPreset::SNAPPY)?;
            self.ui.anim.spring(&self.main.visual, "Scale.Y", rest, SpringPreset::SNAPPY)?;
        }
        let watch = self.island.is_expanded() && visible;
        if watch != self.watching {
            self.watching = watch;
            self.window.watch_outside(watch);
        }
        // Esc menutup kartu yang terbuka, dari mana pun kartu dibuka (klik, tahan, hotkey, tray).
        // Hanya didaftarkan selama kartu terbuka agar tombol Esc aplikasi lain tidak tertelan.
        if watch != self.esc_registered {
            if watch {
                let esc = notchi_core::hotkey::Hotkey { ctrl: false, alt: false, shift: false, win: false, vk: VK_ESCAPE as u16 };
                self.esc_registered = self.window.register_hotkey(HK_ESC, &esc);
            } else {
                self.window.unregister_hotkey(HK_ESC);
                self.esc_registered = false;
            }
        }
        if self.key.as_ref() != Some(&key) {
            let fresh_alert = alert.is_some() && self.key.as_ref().map(|k| &k.alert) != Some(&key.alert);
            self.rebuild(&key, alert.as_ref(), expression, now, morphing)?;
            self.key = Some(key);
            if fresh_alert && let Some(a) = alert.as_ref() {
                self.celebrate(a, shape);
            }
        }
        self.refresh_dynamic(alert.as_ref(), now)?;
        Ok(())
    }

    fn celebrate(&mut self, alert: &AlertKind, shape: Shape) {
        let (kind, colors) = match alert {
            AlertKind::TimerDone => {
                (Burst::Confetti, vec![color::accent_alert(), self.ui.accent, color::accent_charging(), self.ui.pal.text])
            }
            AlertKind::Charging { .. } => (Burst::Sparkle, vec![color::accent_charging(), self.ui.pal.text]),
            _ => return,
        };
        let center = self.content.as_ref().and_then(|c| c.d.mascot.as_ref()).map(|m| m.center());
        let local = center.unwrap_or(Point::new(shape.main.w / 2.0, shape.main.h / 2.0));
        let at = self.stage_point(local);
        self.burst(at, kind, &colors);
    }

    fn sync_privacy(&mut self, retracted: bool) -> Result<()> {
        let tint = self.privacy.tint();
        let on = tint.is_some() && !retracted && self.visible;
        let ui = &self.ui;
        let dot = &mut self.dot;
        if let Some(c) = tint
            && dot.color != Some(c)
        {
            dot.color = Some(c);
            dot.shape.SetFillBrush(&ui.c.CreateColorBrushWithColor(comp::color(c))?)?;
        }
        if on != dot.shown {
            dot.shown = on;
            ui.anim.stop(&dot.visual, "Opacity")?;
            if on {
                dot.visual.SetOpacity(1.0)?;
                ui.anim.repeat(&dot.visual, "Opacity", &[(0.0, 1.0), (0.5, 0.45), (1.0, 1.0)], 2_000.0, 0.0)?;
            } else {
                ui.anim.linear(&dot.visual, "Opacity", 0.0, 120.0, 0.0)?;
            }
        }
        let lip = if retracted { tint } else { None };
        if lip != self.lip_tint {
            self.lip_tint = lip;
            let base = self.ui.pal.base;
            let fill = lip.map_or(base, |c| c.with_opacity(LIP_TINT).over(base));
            self.main.set_fill(&self.ui, fill)?;
            if self.cfg.liquid {
                self.main.set_liquid(&self.ui, true)?;
            }
        }
        Ok(())
    }

    fn model(&self, now: Ms, expression: Expression) -> Model<'_> {
        Model {
            lang: self.cfg.language,
            now,
            skin: self.cfg.mascot.skin,
            mascot_on: self.cfg.mascot.enabled,
            expression,
            media: self.snap.as_ref(),
            art: self.art.as_ref().map(|(_, a)| a),
            timer: self.timer.as_ref().or(self.done_timer.as_ref()),
            stopwatch: self.stopwatch.as_ref(),
            presets: &self.cfg.timer.presets_minutes,
            clock: shell::local_time(),
            lyrics: self.lyric_state(),
            battery: self.power,
            download: self.download_info.as_ref(),
        }
    }

    fn retire(&mut self, v: ContainerVisual, bubble: bool) -> Result<()> {
        self.retire_seq += 1;
        let batch = self.ui.anim.batch()?;
        self.ui.anim.linear(&v, "Opacity", 0.0, RETIRE_FADE_MS, 0.0)?;
        batch.end(&self.proxy, TAG_RETIRE | self.retire_seq)?;
        self.retired.push((self.retire_seq, v, bubble));
        Ok(())
    }

    fn rebuild(&mut self, key: &ViewKey, alert: Option<&AlertKind>, expression: Expression, now: Ms, morphing: bool) -> Result<()> {
        if let Some(old) = self.content.take() {
            self.retire(old.root, false)?;
        }
        if let Some(old) = self.bubble_content.take() {
            self.retire(old, true)?;
        }
        self.hover_button = None;
        self.pressed_button = None;
        let root = self.ui.container(&self.main.content)?;
        root.SetOpacity(0.0)?;
        let mut d = Dyn::default();
        let m = self.model(now, expression);
        let size = Self::shape_for(key.kind, alert).main;
        match key.kind {
            ViewKind::Hidden | ViewKind::Empty => {}
            ViewKind::Buddy => views::build_buddy(&self.ui, &root, size, &m, &mut d)?,
            ViewKind::Peek(PeekKind::Idle) => views::build_peek_idle(&self.ui, &root, size, &m, &mut d)?,
            ViewKind::Peek(PeekKind::Media) => views::build_peek_media(&self.ui, &root, size, &m, &mut d)?,
            ViewKind::Compact(slot) => views::build_compact(&self.ui, &root, slot, size, &m, &mut d)?,
            ViewKind::Minimal(primary, secondary) => {
                views::build_compact(&self.ui, &root, primary, size, &m, &mut d)?;
                let b = self.ui.container(&self.bubble.content)?;
                b.SetOpacity(0.0)?;
                views::build_bubble(&self.ui, &b, secondary, &m)?;
                self.ui.anim.linear(&b, "Opacity", 1.0, CONTENT_FADE_MS, if morphing { CONTENT_DELAY_MS } else { 0.0 })?;
                self.bubble_content = Some(b);
            }
            ViewKind::Card(Card::Media | Card::Lyrics) => views::build_media_card(&self.ui, &root, &m, &mut d)?,
            ViewKind::Card(Card::Timer) => views::build_timer_card(&self.ui, &root, false, &m, &mut d)?,
            ViewKind::Card(Card::Stopwatch) => views::build_timer_card(&self.ui, &root, true, &m, &mut d)?,
            ViewKind::Card(Card::Home) => views::build_home_card(&self.ui, &root, &m, &mut d)?,
            ViewKind::Card(Card::Download) => views::build_download_card(&self.ui, &root, &m, &mut d)?,
            ViewKind::Alert => {
                if let Some(a) = alert {
                    views::build_alert(&self.ui, &root, a, &m, &mut d)?;
                }
            }
            ViewKind::Hud => {
                if let Some(a) = alert {
                    views::build_hud(&self.ui, &root, a, &mut d)?;
                }
            }
        }
        self.ui.anim.linear(&root, "Opacity", 1.0, CONTENT_FADE_MS, if morphing { CONTENT_DELAY_MS } else { 0.0 })?;
        self.content = Some(Content { root, d, progress_sync: None });
        Ok(())
    }

    fn refresh_dynamic(&mut self, alert: Option<&AlertKind>, now: Ms) -> Result<()> {
        let scrubbing = self.pointer.seek.is_some();
        let Some(content) = self.content.as_mut() else { return Ok(()) };
        let d = &mut content.d;
        if let (Some(text), Some(timer)) = (d.timer.as_mut(), self.timer.as_ref()) {
            text.set(&self.ui, &views::timer_text(timer, now))?;
        }
        if let (Some(text), Some(sw)) = (d.stopwatch.as_mut(), self.stopwatch.as_ref()) {
            text.set(&self.ui, &views::stopwatch_text(sw, now))?;
        }
        if let Some(clock) = d.clock.as_mut() {
            clock.set(&self.ui, &views::clock_text(shell::local_time()))?;
        }
        if let Some(snap) = self.snap.as_ref().filter(|_| !scrubbing) {
            let (el, rem) = views::media_times(snap, now);
            if let Some(t) = d.elapsed.as_mut() {
                t.set(&self.ui, &el)?;
            }
            if let Some(t) = d.remaining.as_mut() {
                t.set(&self.ui, &rem)?;
            }
            if let (Some((fill, width)), Some(tl)) = (d.progress.as_ref(), snap.timeline) {
                let sync = (snap.track_id, tl.reported_at.wrapping_add(tl.position), snap.is_playing());
                if content.progress_sync != Some(sync) {
                    content.progress_sync = Some(sync);
                    let frac = snap.progress_at(now).unwrap_or(0.0).clamp(0.0, 1.0);
                    self.ui.anim.stop(fill, "Size.X")?;
                    fill.SetSize(v2(width * frac, 4.0))?;
                    if snap.is_playing() {
                        let left = tl.end.saturating_sub(snap.position_at(now).unwrap_or(0)) as f32;
                        self.ui.anim.linear_always(fill, "Size.X", *width, left.max(1.0))?;
                    }
                }
            }
            if let (Some(panel), Some(LyricSlot { track, status: LyricStatus::Ready(l) })) = (d.lyrics.as_mut(), self.lyric.as_ref())
                && *track == snap.track_id
            {
                let (pos, dur) = lyric_timing(snap, now);
                panel.show(&self.ui, l, l.index_at(pos, dur))?;
            }
        }
        if let (Some(text), Some(dl)) = (d.download.as_mut(), self.download_info.as_ref()) {
            let s = if matches!(self.key.as_ref().map(|k| k.kind), Some(ViewKind::Card(Card::Download))) {
                views::download_detail(self.cfg.language, dl)
            } else {
                views::download_short(self.cfg.language, dl)
            };
            text.set(&self.ui, &s)?;
        }
        let hud = match alert {
            Some(AlertKind::Volume { level, muted }) => Some((*level, *muted)),
            Some(AlertKind::Brightness { level }) => Some((*level, false)),
            _ => None,
        };
        if let (Some((bar, width)), Some((level, muted))) = (d.hud_bar.as_ref(), hud) {
            let level = &level;
            let w = if muted { 0.0 } else { width * *level as f32 / 100.0 };
            self.ui.anim.spring(bar, "Size.X", w, SpringPreset::SNAPPY)?;
            if let Some(t) = d.hud_text.as_mut() {
                t.set(&self.ui, &level.to_string())?;
            }
        }
        Ok(())
    }

    fn morph(&mut self, shape: Shape, p: SpringPreset) -> Result<()> {
        self.morph_seq += 1;
        self.morph_pending = true;
        self.settled_region = self.region_of(shape);
        let moving = rect_union(self.centered(layout::hit_box(&[shape]), &shape), self.settled_region);
        self.set_region(rect_union(self.region, moving));
        let shift = self.shift_of(&shape);
        let a = &self.ui.anim;
        let batch = a.batch()?;
        a.spring(&self.stage, "Offset.X", shift * self.ui.scale, p)?;
        let main = &self.main.visual;
        a.spring(main, "Size.X", shape.main.w, p)?;
        a.spring(main, "Size.Y", shape.main.h, p)?;
        let main_x = if shape.bubble { -(BUBBLE_GAP + BUBBLE.w) / 2.0 } else { 0.0 };
        a.spring(main, "Offset.X", main_x, p)?;
        let bubble_x = shape.total_width() / 2.0 - BUBBLE.w / 2.0;
        let bubble = &self.bubble.visual;
        a.spring(bubble, "Offset.X", if shape.bubble { bubble_x } else { main_x }, p)?;
        a.linear(bubble, "Opacity", if shape.bubble && self.visible { 1.0 } else { 0.0 }, CONTENT_FADE_MS, 0.0)?;
        batch.end(&self.proxy, TAG_MORPH | self.morph_seq)?;
        self.shape = shape;
        Ok(())
    }

    fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.visible = visible;
        if visible {
            self.settled_region = self.region_of(self.shape);
            self.set_region(self.settled_region);
            self.window.show();
        }
        let a = &self.ui.anim;
        let batch = a.batch()?;
        let target = if visible { 1.0 } else { 0.0 };
        a.linear(&self.main.visual, "Opacity", target, if visible { 90.0 } else { 140.0 }, 0.0)?;
        let bubble_target = if visible && self.shape.bubble { 1.0 } else { 0.0 };
        a.linear(&self.bubble.visual, "Opacity", bubble_target, CONTENT_FADE_MS, 0.0)?;
        self.morph_seq += 1;
        self.morph_pending = true;
        batch.end(&self.proxy, TAG_MORPH | self.morph_seq)?;
        Ok(())
    }

    fn on_batch(&mut self, tag: u64) {
        let seq = tag & TAG_MASK;
        match tag & !TAG_MASK {
            TAG_MORPH if seq == self.morph_seq => {
                self.morph_pending = false;
                if self.visible {
                    self.set_region(self.settled_region);
                } else {
                    self.window.hide();
                }
                if !self.island.is_expanded() {
                    self.window.set_timer(T_TRIM, TRIM_DELAY_MS);
                }
            }
            TAG_RETIRE => {
                let (gone, keep): (Vec<_>, Vec<_>) = self.retired.drain(..).partition(|(id, _, _)| *id <= seq);
                self.retired = keep;
                for (_, v, bubble) in gone {
                    let parent = if bubble { &self.bubble.content } else { &self.main.content };
                    if let Ok(children) = parent.Children() {
                        let _ = children.Remove(&v);
                    }
                }
            }
            TAG_FX => {
                let (gone, keep): (Vec<_>, Vec<_>) = self.fx.drain(..).partition(|(id, _)| *id <= seq);
                self.fx = keep;
                if let Ok(children) = self.stage.Children() {
                    for (_, v) in gone {
                        let _ = children.Remove(&v);
                    }
                }
                if self.fx.is_empty() {
                    self.apply_region();
                    self.window.set_timer(T_TRIM, TRIM_DELAY_MS);
                }
            }
            TAG_PANEL_FX => {
                if let Some(s) = self.settings.as_mut() {
                    s.retire_fx(seq);
                }
            }
            _ => {}
        }
    }

    fn apply_liquid(&mut self) {
        let on = self.cfg.liquid;
        let _ = self.main.set_liquid(&self.ui, on);
        let _ = self.bubble.set_liquid(&self.ui, on);
    }

    fn centered(&self, size: Size, shape: &Shape) -> Rect {
        let cx = self.window_width_dp() / 2.0 + self.shift_of(shape);
        Rect::new(cx - size.w / 2.0, 0.0, size.w, size.h)
    }

    fn region_of(&self, shape: Shape) -> Rect {
        let near = self.centered(layout::near_zone(&self.full_shape, self.cfg.island.hover_zone), &shape);
        if self.retracted {
            near
        } else if self.island.tucks() {
            rect_union(self.centered(layout::rest_box(&shape), &shape), near)
        } else {
            self.centered(layout::rest_box(&shape), &shape)
        }
    }

    fn set_region(&mut self, r: Rect) {
        if r != self.region {
            self.region = r;
            self.apply_region();
        }
    }

    fn apply_region(&self) {
        let s = self.ui.scale;
        let pad = if self.fx.is_empty() { 0.0 } else { FX_PAD };
        let (ww, wh) = self.window_px;
        let x0 = (((self.region.x - pad) * s).floor() as i32).clamp(0, ww);
        let x1 = (((self.region.x + self.region.w + pad) * s).ceil() as i32).clamp(x0, ww);
        let h = (((self.region.h + pad) * s).ceil() as i32).min(wh);
        self.window.set_region(x0, 0, x1 - x0, h);
    }

    /// Titik tengah pil yang diinginkan, dalam piksel layar.
    fn anchor(&self, mon: &Monitor) -> i32 {
        let a = self.anchor_px.unwrap_or_else(|| layout::anchor_px(self.cfg.island.x, mon.left, mon.width()));
        a.clamp(mon.left, mon.right)
    }

    fn window_bounds(&self, mon: &Monitor) -> (i32, i32, i32) {
        let size = layout::window_max();
        let w = (size.w * self.ui.scale).ceil() as i32;
        let h = (size.h * self.ui.scale).ceil() as i32;
        let w = w + (mon.width() - w).rem_euclid(2);
        (layout::window_left(self.anchor(mon), mon.left, mon.width(), w), w, h)
    }

    fn layout_window(&mut self) {
        let mon = primary_monitor();
        let (x, w, h) = self.window_bounds(&mon);
        self.window_px = (w, h);
        self.window_origin = (x, mon.top);
        self.anchor_px = Some(self.anchor(&mon));
        self.window.set_bounds(x, mon.top, w, h, false);
        let shift = self.shift_of(&self.shape);
        let _ = self.ui.anim.stop(&self.stage, "Offset.X");
        let _ = self.stage.SetOffset(v3(shift * self.ui.scale, 0.0, 0.0));
        self.settled_region = self.region_of(self.shape);
        self.region = self.settled_region;
        self.apply_region();
    }

    fn schedule(&mut self, now: Ms) {
        let showing = self.key.as_ref().map(|k| k.kind);
        let mut tick: Option<Ms> = None;
        let mut want = |t: Option<Ms>| {
            if let Some(t) = t {
                tick = Some(tick.map_or(t, |c: Ms| c.min(t)));
            }
        };
        let shows_timer = self.content.as_ref().is_some_and(|c| c.d.timer.is_some());
        let shows_sw = self.content.as_ref().is_some_and(|c| c.d.stopwatch.is_some());
        if shows_timer {
            want(self.timer.and_then(|t| t.next_tick(now)));
        }
        if shows_sw {
            want(self.stopwatch.and_then(|s| s.next_tick(now)));
        }
        let media_card = matches!(showing, Some(ViewKind::Card(Card::Media | Card::Lyrics)));
        if media_card && let Some(snap) = self.snap.as_ref().filter(|s| s.is_playing()) {
            if let Some(pos) = snap.position_at(now) {
                want(Some(now + 1000 - pos % 1000));
            }
            let shows_lyrics = self.content.as_ref().is_some_and(|c| c.d.lyrics.is_some());
            if shows_lyrics && let Some(LyricSlot { track, status: LyricStatus::Ready(l) }) = self.lyric.as_ref()
                && *track == snap.track_id
            {
                let (pos, dur) = lyric_timing(snap, now);
                want(l.next_change(pos, dur).map(|at| now + at.saturating_sub(pos)));
            }
        }
        match tick {
            Some(t) => self.window.set_timer(T_TICK, t.saturating_sub(now).max(16) as u32),
            None => self.window.kill_timer(T_TICK),
        }

        let mut deadline = self.island.next_deadline();
        let mut also = |t: Option<Ms>| {
            if let Some(t) = t {
                deadline = Some(deadline.map_or(t, |d| d.min(t)));
            }
        };
        also(self.presence.deadline().filter(|_| self.acts.contains(ActivityKind::Media)));
        also(self.timer.filter(|t| t.is_running()).map(|t| now + t.remaining(now)));
        match deadline {
            Some(d) => self.window.set_timer(T_DEADLINE, d.saturating_sub(now).clamp(16, 3_600_000) as u32),
            None => self.window.kill_timer(T_DEADLINE),
        }

        if matches!(showing, Some(ViewKind::Card(Card::Home) | ViewKind::Peek(PeekKind::Idle))) {
            let c = shell::local_time();
            let ms = (60 - c.second as u32) * 1000 - c.millis as u32;
            self.window.set_timer(T_CLOCK, ms.max(50));
        } else {
            self.window.kill_timer(T_CLOCK);
        }
    }
}

pub fn config_path() -> PathBuf {
    Paths::default_location().config
}

pub fn reset_config(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, config::DEFAULT_TOML)
}

pub fn log(msg: &str) {
    use std::io::Write;
    if cfg!(debug_assertions) {
        eprintln!("[notchi] {msg}");
    }
    let Some(base) = std::env::var_os("LOCALAPPDATA") else { return };
    let dir = PathBuf::from(base).join("Notchi");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("notchi.log");
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 256 * 1024) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{msg}");
    }
}
