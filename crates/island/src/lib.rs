//! Fitur Island Notchi (PRD §3): pil interaktif di tengah atas layar dengan
//! media, timer, stopwatch, alert daya, HUD volume, dan maskot Notchi.
//!
//! Crate ini tidak memakai `unsafe`; semua akses Windows lewat
//! `notchi-platform-win`, dan semua keputusan (state, prioritas, layout)
//! diambil oleh `notchi-core`.

#![forbid(unsafe_code)]

mod ui;
mod views;

use std::path::{Path, PathBuf};

use notchi_core::Ms;
use notchi_core::activity::{Activities, ActivityKind};
use notchi_core::color::ISLAND_BG;
use notchi_core::config::{self, Config, ReduceMotion, Warning};
use notchi_core::geom::{Point, Size, scale_for_dpi};
use notchi_core::i18n::{Lang, Text, t};
use notchi_core::icon::mascot_rgba;
use notchi_core::island::{AlertKind, DisplayMode, Island, Presentation};
use notchi_core::layout::{self, BUBBLE, BUBBLE_GAP, Button, Card, HOVER_SCALE, Shape};
use notchi_core::mascot::{Expression, MascotContext, expression_for};
use notchi_core::media::{Artwork, Controls, MediaPresence, MediaSnapshot, SourceFilter};
use notchi_core::power::{BatteryMonitor, PowerEvent};
use notchi_core::spring::SpringPreset;
use notchi_core::timer::{Countdown, Stopwatch};
use notchi_platform_win::audio::Volume;
use notchi_platform_win::comp::{Animator, CompositionHost, ContainerVisual, v2, v3};
use notchi_platform_win::gfx::Gfx;
use notchi_platform_win::media::{MediaCommand, MediaService};
use notchi_platform_win::window::{
    Event, IconImage, MenuItem, MouseButton, OwnedWindow, TrayEvent, TrayIcon, UiProxy, Window, primary_monitor, quit,
};
use notchi_platform_win::{Result, fswatch, now_ms, power, shell};

use crate::ui::{Pill, Ui};
use crate::views::{Dyn, Model, Slot};

const T_LONG_PRESS: usize = 1;
const T_COLLAPSE: usize = 2;
const T_TICK: usize = 3;
const T_DEADLINE: usize = 4;
const T_CONFIG: usize = 5;
const T_CLOCK: usize = 6;

const HK_TOGGLE: i32 = 1;
const HK_ESC: i32 = 2;

const TAG_MORPH: u64 = 1 << 40;
const TAG_RETIRE: u64 = 2 << 40;
const TAG_MASK: u64 = (1 << 40) - 1;

const M_PRESET: u32 = 100;
const M_STOPWATCH: u32 = 150;
const M_MODE: u32 = 200;
const M_HIDE: u32 = 300;
const M_SHOW: u32 = 301;
const M_LOGIN: u32 = 400;
const M_SETTINGS: u32 = 500;
const M_ABOUT: u32 = 600;
const M_QUIT: u32 = 900;

const LONG_PRESS_MS: u32 = 350;
const SWIPE_DP: f32 = 36.0;
const HOUR_MS: Ms = 3_600_000;

/// Lokasi berkas pengguna.
#[derive(Debug, Clone)]
pub struct Paths {
    pub dir: PathBuf,
    pub config: PathBuf,
}

impl Paths {
    /// `%APPDATA%\Notchi\config.toml` (PRD §5.4).
    pub fn default_location() -> Paths {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("Notchi");
        Paths { config: dir.join("config.toml"), dir }
    }
}

/// Membaca konfigurasi; membuat berkas default bila belum ada.
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewKind {
    Hidden,
    Empty,
    Compact(Slot),
    Minimal(Slot, Slot),
    Card(Card),
    Alert,
    Hud,
}

/// Semua yang, bila berubah, membuat isi digambar ulang.
#[derive(Debug, Clone, PartialEq)]
struct ViewKey {
    kind: ViewKind,
    alert: Option<AlertKind>,
    expression: Expression,
    skin: notchi_core::color::Skin,
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
}

struct Content {
    root: ContainerVisual,
    d: Dyn,
    /// Sinkronisasi terakhir batang progres: (track, posisi dilaporkan, diputar).
    progress_sync: Option<(u64, Ms, bool)>,
}

#[derive(Debug, Default)]
struct Pointer {
    down: Option<Point>,
    consumed: bool,
}

pub struct IslandApp {
    window: Window,
    proxy: UiProxy,
    _host: CompositionHost,
    ui: Ui,
    stage: ContainerVisual,
    main: Pill,
    bubble: Pill,
    content: Option<Content>,
    bubble_content: Option<ContainerVisual>,
    retired: Vec<(u64, ContainerVisual, bool)>,
    key: Option<ViewKey>,
    shape: Shape,
    visible: bool,
    window_px: (i32, i32),
    morph_seq: u64,
    retire_seq: u64,

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
    presence: MediaPresence,
    timer: Option<Countdown>,
    done_timer: Option<Countdown>,
    stopwatch: Option<Stopwatch>,
    battery: BatteryMonitor,
    volume: Option<Volume>,
    tray: Option<TrayIcon>,
    fullscreen_hint: bool,
    pointer: Pointer,
    esc_registered: bool,
    hotkey_ok: bool,
}

impl std::fmt::Debug for IslandApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IslandApp").field("shape", &self.shape).field("visible", &self.visible).finish()
    }
}

fn slot_of(kind: ActivityKind) -> Slot {
    match kind {
        ActivityKind::Timer => Slot::Timer,
        ActivityKind::Stopwatch => Slot::Stopwatch,
        _ => Slot::Media,
    }
}

impl IslandApp {
    /// Membuat Island untuk jendela `owned` lalu memasang handler-nya.
    pub fn install(owned: &mut OwnedWindow, paths: Paths, exe: PathBuf) -> Result<()> {
        owned.register_power_notifications();
        owned.register_fullscreen_notifications();
        owned.watch_foreground();
        let mut app = IslandApp::new(owned.window(), owned.proxy(), paths, exe)?;
        app.update();
        owned.set_handler(move |_, ev| app.handle(ev));
        Ok(())
    }

    fn new(window: Window, proxy: UiProxy, paths: Paths, exe: PathBuf) -> Result<IslandApp> {
        let (cfg, cfg_text, warnings) = load_config(&paths);
        let host = CompositionHost::new(&window)?;
        let anim = Animator::new(&host.compositor);
        let gfx = Gfx::new(&host.compositor)?;
        let scale = scale_for_dpi(primary_monitor().dpi);
        let ui = Ui { c: host.compositor.clone(), anim, gfx, scale };

        let stage = ui.container(&host.root)?;
        stage.SetRelativeOffsetAdjustment(v3(0.5, 0.0, 0.0))?;
        stage.SetScale(v3(scale, scale, 1.0))?;
        let bubble = Pill::new(&ui, &stage, ISLAND_BG)?;
        let main = Pill::new(&ui, &stage, ISLAND_BG)?;
        let top = cfg.island.top_margin;
        for (pill, size) in [(&main, layout::IDLE), (&bubble, BUBBLE)] {
            pill.visual.SetSize(v2(size.w, size.h))?;
            pill.visual.SetOffset(v3(0.0, top, 0.0))?;
            pill.visual.SetOpacity(0.0)?;
            pill.visual.SetCenterPoint(v3(size.w / 2.0, size.h / 2.0, 0.0))?;
        }

        let mut app = IslandApp {
            window,
            proxy,
            _host: host,
            ui,
            stage,
            main,
            bubble,
            content: None,
            bubble_content: None,
            retired: Vec::new(),
            key: None,
            shape: Shape::single(layout::IDLE),
            visible: false,
            window_px: (0, 0),
            morph_seq: 0,
            retire_seq: 0,
            paths,
            exe,
            island: Island::new(cfg.island.mode),
            acts: Activities::new(),
            media: None,
            snap: None,
            art: None,
            presence: MediaPresence::new(cfg.media.paused_linger_seconds as Ms * 1000),
            timer: None,
            done_timer: None,
            stopwatch: None,
            battery: BatteryMonitor::new(&cfg.power.low_thresholds),
            volume: None,
            tray: None,
            fullscreen_hint: false,
            pointer: Pointer::default(),
            esc_registered: false,
            hotkey_ok: false,
            cfg: Config::default(),
            cfg_text,
            warnings: Vec::new(),
        };
        let icon = IconImage { size: 32, rgba: mascot_rgba(32, cfg.mascot.skin) };
        app.tray = TrayIcon::new(&app.window, &icon, t(cfg.language, Text::TrayTooltip)).ok();
        app.apply_config(cfg, warnings, true);
        fswatch::watch_dir(&app.paths.dir, app.proxy.clone());
        // Pembacaan awal: tidak memicu alert colok/cabut, hanya baterai rendah.
        if app.cfg.power.enabled {
            let _ = app.battery.update(power::read()).map(|e| app.power_alert(e));
        }
        app.check_fullscreen();
        app.place_window(Shape::single(layout::IDLE), false);
        Ok(app)
    }

    fn top(&self) -> f32 {
        self.cfg.island.top_margin
    }

    // ---- konfigurasi -------------------------------------------------

    fn apply_config(&mut self, new: Config, warnings: Vec<Warning>, first: bool) {
        let old = std::mem::replace(&mut self.cfg, new);
        self.warnings = warnings;
        let cfg = &self.cfg;
        self.island.set_mode(cfg.island.mode);

        if first || old.island.hotkey != cfg.island.hotkey {
            self.window.unregister_hotkey(HK_TOGGLE);
            self.hotkey_ok = self.window.register_hotkey(HK_TOGGLE, &cfg.island.hotkey);
        }
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
        if first || old.launch_at_login != cfg.launch_at_login {
            if shell::launch_at_login_enabled() != cfg.launch_at_login {
                shell::set_launch_at_login(cfg.launch_at_login, &self.exe);
            }
        }
        if !first && old.mascot.skin != cfg.mascot.skin {
            if let Some(tray) = self.tray.take() {
                drop(tray);
                let icon = IconImage { size: 32, rgba: mascot_rgba(32, self.cfg.mascot.skin) };
                self.tray = TrayIcon::new(&self.window, &icon, "Notchi").ok();
            }
        }
        if !first && old.island.top_margin != self.cfg.island.top_margin {
            let top = self.top();
            let _ = self.ui.anim.spring(&self.main.visual, "Offset.Y", top, SpringPreset::SMOOTH);
            let _ = self.ui.anim.spring(&self.bubble.visual, "Offset.Y", top, SpringPreset::SMOOTH);
            self.place_window(self.shape, self.visible);
        }
        self.refresh_motion();
        self.update_tooltip();
        self.key = None;
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

    /// Mengubah satu nilai di `config.toml` tanpa menghapus komentar pengguna.
    fn persist(&mut self, section: &str, key: &str, value: &str) {
        let text = config::set_value(&self.cfg_text, section, key, value);
        if std::fs::write(&self.paths.config, &text).is_ok() {
            self.cfg_text = text.clone();
        }
        let (cfg, warnings) = config::parse(&text);
        self.apply_config(cfg, warnings, false);
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
        }
    }

    // ---- event -------------------------------------------------------

    fn handle(&mut self, ev: Event) {
        let now = now_ms();
        match ev {
            Event::MouseMove(p) => self.on_mouse_move(p),
            Event::MouseLeave => self.on_mouse_leave(),
            Event::MouseDown(p, MouseButton::Left) => {
                self.pointer = Pointer { down: Some(p), consumed: false };
                self.window.set_timer(T_LONG_PRESS, LONG_PRESS_MS);
            }
            Event::MouseUp(p, MouseButton::Left) => self.on_click(p, now),
            Event::MouseUp(_, MouseButton::Right) => self.show_menu(),
            Event::MouseDown(..) | Event::MouseUp(..) => {}
            Event::Wheel(delta) => self.on_wheel(delta, now),
            Event::Timer(id) => self.on_timer(id),
            Event::Hotkey(HK_TOGGLE) => self.island.hotkey(),
            Event::Hotkey(HK_ESC) => self.island.collapse(),
            Event::Hotkey(_) => {}
            Event::DpiChanged(_) | Event::DisplayChanged => self.on_display(),
            Event::SettingsChanged => self.refresh_motion(),
            Event::PowerChanged => {
                if self.cfg.power.enabled {
                    if let Some(e) = self.battery.update(power::read()) {
                        self.power_alert(e);
                    }
                }
            }
            Event::FullscreenHint(on) => {
                self.fullscreen_hint = on;
                self.check_fullscreen();
            }
            Event::ForegroundChanged => {
                self.check_fullscreen();
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
            Event::FileChanged => self.window.set_timer(T_CONFIG, 300),
            Event::User(tag) => self.on_batch(tag),
        }
        self.update();
    }

    fn on_timer(&mut self, id: usize) {
        match id {
            T_LONG_PRESS => {
                self.window.kill_timer(T_LONG_PRESS);
                if self.pointer.down.is_some() && !self.pointer.consumed {
                    self.pointer.consumed = true;
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
            // Tick, tenggat, dan jam cukup memicu `update`.
            _ => {}
        }
    }

    fn on_media(&mut self, snap: Option<MediaSnapshot>, now: Ms) {
        if snap.as_ref().map(|s| s.track_id) != self.snap.as_ref().map(|s| s.track_id) {
            self.art = None;
        }
        self.presence.update(snap.as_ref(), now);
        self.snap = snap;
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

    fn on_display(&mut self) {
        let scale = scale_for_dpi(primary_monitor().dpi);
        if (scale - self.ui.scale).abs() > f32::EPSILON {
            self.ui.scale = scale;
            let _ = self.stage.SetScale(v3(scale, scale, 1.0));
            self.key = None;
        }
        self.place_window(self.shape, self.window.is_visible());
    }

    // ---- pointer -----------------------------------------------------

    fn window_width_dp(&self) -> f32 {
        self.window_px.0 as f32 / self.ui.scale
    }

    /// Titik relatif pil utama, atau `None` bila di luar pil.
    fn pill_local(&self, p: Point) -> Option<Point> {
        let r = self.shape.main_rect(self.window_width_dp(), self.top());
        r.contains(p).then(|| Point::new(p.x - r.x, p.y - r.y))
    }

    fn on_mouse_move(&mut self, p: Point) {
        if !self.island.is_hover() {
            self.island.set_hover(true);
            self.window.kill_timer(T_COLLAPSE);
            if !self.island.is_expanded() && self.visible {
                let _ = self.hover_scale(HOVER_SCALE);
            }
        }
        if self.island.is_expanded() && !self.esc_registered {
            let esc = notchi_core::hotkey::Hotkey { ctrl: false, alt: false, shift: false, win: false, vk: 0x1B };
            self.esc_registered = self.window.register_hotkey(HK_ESC, &esc);
        }
        if let Some(down) = self.pointer.down {
            let dx = p.x - down.x;
            if !self.pointer.consumed && dx.abs() > SWIPE_DP {
                self.pointer.consumed = true;
                self.swipe(if dx > 0.0 { -1 } else { 1 });
            }
        }
    }

    fn on_mouse_leave(&mut self) {
        self.island.set_hover(false);
        self.pointer = Pointer::default();
        let _ = self.hover_scale(1.0);
        if self.esc_registered {
            self.window.unregister_hotkey(HK_ESC);
            self.esc_registered = false;
        }
        if self.island.is_expanded() {
            self.window.set_timer(T_COLLAPSE, 2_500);
        }
    }

    fn hover_scale(&self, s: f32) -> Result<()> {
        let v = &self.main.visual;
        let size = self.shape.main;
        v.SetCenterPoint(v3(size.w / 2.0, size.h / 2.0, 0.0))?;
        self.ui.anim.spring(v, "Scale.X", s, SpringPreset::SNAPPY)?;
        self.ui.anim.spring(v, "Scale.Y", s, SpringPreset::SNAPPY)
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
        let consumed = self.pointer.consumed;
        self.pointer = Pointer::default();
        if consumed {
            return;
        }
        let Some(local) = self.pill_local(p) else {
            self.island.collapse();
            return;
        };
        if self.island.is_expanded() {
            let hit = self.content.as_ref().and_then(|c| layout::hit(&c.d.buttons, local));
            if let Some(b) = hit {
                self.press(b, now);
                return;
            }
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
        }
    }

    fn start_timer(&mut self, minutes: u32, now: Ms) {
        self.timer = Some(Countdown::start(minutes as Ms * 60_000, now));
        self.done_timer = None;
    }

    fn on_wheel(&mut self, delta: i32, now: Ms) {
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
            // Perangkat keluaran mungkin berganti; buka ulang sekali.
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

    // ---- menu --------------------------------------------------------

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
        let mode = self.cfg.island.mode;
        let modes = DisplayMode::ALL
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let label = match m {
                    DisplayMode::Always => t(lang, Text::ModeAlways),
                    DisplayMode::ActivityOnly => t(lang, Text::ModeActivity),
                    DisplayMode::Hover => t(lang, Text::ModeHover),
                };
                MenuItem::check(M_MODE + i as u32, label, *m == mode)
            })
            .collect();
        let snoozed = self.island.is_snoozed(now);
        let items = vec![
            MenuItem::Submenu { label: t(lang, Text::MenuTimer).into(), items: presets },
            MenuItem::item(M_STOPWATCH, t(lang, Text::MenuStopwatch)),
            MenuItem::Separator,
            MenuItem::Submenu { label: t(lang, Text::MenuMode).into(), items: modes },
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
            id if (M_MODE..M_MODE + 3).contains(&id) => {
                let m = DisplayMode::ALL[(id - M_MODE) as usize];
                self.persist("island", "mode", &format!("\"{}\"", m.key()));
            }
            M_HIDE => self.island.snooze(now + HOUR_MS),
            M_SHOW => self.island.unsnooze(),
            M_LOGIN => {
                let on = !self.cfg.launch_at_login;
                self.persist("startup", "launch_at_login", if on { "true" } else { "false" });
            }
            M_SETTINGS => shell::edit_text_file(&self.paths.config),
            M_ABOUT => {
                let body = format!("Notchi {}\n\n{}", env!("CARGO_PKG_VERSION"), t(lang, Text::AboutBody));
                self.window.message_box("Notchi", &body);
            }
            M_QUIT => self.shutdown(),
            _ => {}
        }
    }

    fn shutdown(&mut self) {
        self.tray = None;
        self.media = None;
        self.window.unregister_hotkey(HK_TOGGLE);
        self.window.unregister_hotkey(HK_ESC);
        self.window.hide();
        quit();
    }

    // ---- model → tampilan --------------------------------------------

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
        self.island.set_activity_count(self.acts.len());
    }

    fn update(&mut self) {
        let now = now_ms();
        self.island.tick(now);
        self.sync_activities(now);
        if let Err(e) = self.render(now) {
            // Kegagalan gambar tidak boleh menjatuhkan aplikasi; coba bangun
            // ulang isi pada event berikutnya.
            log(&format!("render gagal: {e}"));
            self.key = None;
        }
        self.schedule(now);
    }

    fn view_kind(&self, pres: &Presentation) -> ViewKind {
        let ordered = self.acts.ordered();
        match pres {
            Presentation::Hidden => ViewKind::Hidden,
            Presentation::Idle => ViewKind::Empty,
            Presentation::Compact => ViewKind::Compact(slot_of(ordered[0])),
            Presentation::Minimal => ViewKind::Minimal(slot_of(ordered[0]), slot_of(ordered[1])),
            Presentation::Expanded => ViewKind::Card(match ordered.first() {
                Some(ActivityKind::Media) => Card::Media,
                Some(ActivityKind::Timer) => Card::Timer,
                Some(ActivityKind::Stopwatch) => Card::Stopwatch,
                _ => Card::Home,
            }),
            Presentation::Alert(a) if a.is_hud() => ViewKind::Hud,
            Presentation::Alert(_) => ViewKind::Alert,
        }
    }

    fn shape_for(kind: ViewKind, alert: Option<&AlertKind>) -> Shape {
        match kind {
            ViewKind::Hidden | ViewKind::Empty => Shape::single(layout::IDLE),
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
        let kind = self.view_kind(&pres);
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

        let shape = Self::shape_for(kind, alert.as_ref());
        let visible = kind != ViewKind::Hidden;
        let key = ViewKey {
            kind,
            // Level volume diperbarui tanpa membangun ulang HUD.
            alert: alert.clone().map(|a| match a {
                AlertKind::Volume { muted, level } => AlertKind::Volume { muted, level: (level == 0) as u8 },
                other => other,
            }),
            expression,
            skin: self.cfg.mascot.skin,
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
        };

        let morphing = shape != self.shape;
        if morphing {
            self.morph(shape)?;
        }
        if visible != self.visible {
            self.set_visible(visible)?;
        } else if visible && !self.window.is_visible() {
            self.place_window(self.shape, true);
        }
        if self.key.as_ref() != Some(&key) {
            self.rebuild(&key, alert.as_ref(), expression, now, morphing)?;
            self.key = Some(key);
        }
        self.refresh_dynamic(alert.as_ref(), now)?;
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
        }
    }

    fn retire(&mut self, v: ContainerVisual, bubble: bool) -> Result<()> {
        self.retire_seq += 1;
        let batch = self.ui.anim.batch()?;
        self.ui.anim.linear(&v, "Opacity", 0.0, 110.0, 0.0)?;
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
        let root = self.ui.container(&self.main.content)?;
        root.SetOpacity(0.0)?;
        let mut d = Dyn::default();
        let m = self.model(now, expression);
        let size = Self::shape_for(key.kind, alert).main;
        match key.kind {
            ViewKind::Hidden | ViewKind::Empty => {}
            ViewKind::Compact(slot) => views::build_compact(&self.ui, &root, slot, size, &m, &mut d)?,
            ViewKind::Minimal(primary, secondary) => {
                views::build_compact(&self.ui, &root, primary, size, &m, &mut d)?;
                let b = self.ui.container(&self.bubble.content)?;
                b.SetOpacity(0.0)?;
                views::build_bubble(&self.ui, &b, secondary, &m)?;
                self.ui.anim.linear(&b, "Opacity", 1.0, 160.0, if morphing { 160.0 } else { 0.0 })?;
                self.bubble_content = Some(b);
            }
            ViewKind::Card(Card::Media) => views::build_media_card(&self.ui, &root, &m, &mut d)?,
            ViewKind::Card(Card::Timer) => views::build_timer_card(&self.ui, &root, false, &m, &mut d)?,
            ViewKind::Card(Card::Stopwatch) => views::build_timer_card(&self.ui, &root, true, &m, &mut d)?,
            ViewKind::Card(Card::Home) => views::build_home_card(&self.ui, &root, &m, &mut d)?,
            ViewKind::Alert => {
                if let Some(a) = alert {
                    views::build_alert(&self.ui, &root, a, &m)?;
                }
            }
            ViewKind::Hud => {
                if let Some(AlertKind::Volume { level, muted }) = alert {
                    views::build_hud(&self.ui, &root, *level, *muted, &mut d)?;
                }
            }
        }
        // Konten baru muncul setelah bentuk hampir selesai (PRD §3.1).
        self.ui.anim.linear(&root, "Opacity", 1.0, 180.0, if morphing { 170.0 } else { 0.0 })?;
        self.content = Some(Content { root, d, progress_sync: None });
        Ok(())
    }

    fn refresh_dynamic(&mut self, alert: Option<&AlertKind>, now: Ms) -> Result<()> {
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
        if let Some(snap) = self.snap.as_ref() {
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
                        // Animasi linear panjang di compositor, tanpa membangunkan UI.
                        self.ui.anim.linear_always(fill, "Size.X", *width, left.max(1.0))?;
                    }
                }
            }
        }
        if let (Some((bar, width)), Some(AlertKind::Volume { level, muted })) = (d.hud_bar.as_ref(), alert) {
            let w = if *muted { 0.0 } else { width * *level as f32 / 100.0 };
            self.ui.anim.spring(bar, "Size.X", w, SpringPreset::SNAPPY)?;
            if let Some(t) = d.hud_text.as_mut() {
                t.set(&self.ui, &level.to_string())?;
            }
        }
        Ok(())
    }

    // ---- bentuk dan jendela ------------------------------------------

    fn morph(&mut self, shape: Shape) -> Result<()> {
        self.morph_seq += 1;
        let grow = layout::window_size(&[self.shape, shape], self.top());
        self.place_window_size(grow, self.window.is_visible() || self.visible);
        let p = SpringPreset::BOUNCY;
        let a = &self.ui.anim;
        let batch = a.batch()?;
        let main = &self.main.visual;
        a.spring(main, "Size.X", shape.main.w, p)?;
        a.spring(main, "Size.Y", shape.main.h, p)?;
        main.SetCenterPoint(v3(shape.main.w / 2.0, shape.main.h / 2.0, 0.0))?;
        let main_x = if shape.bubble { -(BUBBLE_GAP + BUBBLE.w) / 2.0 } else { 0.0 };
        a.spring(main, "Offset.X", main_x, p)?;
        let bubble_x = shape.total_width() / 2.0 - BUBBLE.w / 2.0;
        let bubble = &self.bubble.visual;
        a.spring(bubble, "Offset.X", if shape.bubble { bubble_x } else { main_x }, p)?;
        a.linear(bubble, "Opacity", if shape.bubble && self.visible { 1.0 } else { 0.0 }, 160.0, 0.0)?;
        batch.end(&self.proxy, TAG_MORPH | self.morph_seq)?;
        self.shape = shape;
        Ok(())
    }

    fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.visible = visible;
        if visible {
            self.place_window(self.shape, true);
        }
        let a = &self.ui.anim;
        let batch = a.batch()?;
        let target = if visible { 1.0 } else { 0.0 };
        a.linear(&self.main.visual, "Opacity", target, if visible { 120.0 } else { 180.0 }, 0.0)?;
        let bubble_target = if visible && self.shape.bubble { 1.0 } else { 0.0 };
        a.linear(&self.bubble.visual, "Opacity", bubble_target, 160.0, 0.0)?;
        if !visible {
            let _ = self.hover_scale(1.0);
        }
        self.morph_seq += 1;
        batch.end(&self.proxy, TAG_MORPH | self.morph_seq)?;
        Ok(())
    }

    fn on_batch(&mut self, tag: u64) {
        let seq = tag & TAG_MASK;
        match tag & !TAG_MASK {
            TAG_MORPH if seq == self.morph_seq => {
                // Animasi selesai: kecilkan jendela agar tidak menghalangi klik.
                let hover_mode = self.cfg.island.mode == DisplayMode::Hover && !self.island.is_fullscreen() && !self.island.is_snoozed(now_ms());
                if self.visible {
                    self.place_window(self.shape, true);
                } else if hover_mode {
                    // Zona hover tipis di tepi atas layar.
                    let zone = Size::new(layout::IDLE.w, self.top() + 10.0);
                    self.place_window_size(zone, true);
                } else {
                    self.window.hide();
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
            _ => {}
        }
    }

    fn place_window(&mut self, shape: Shape, show: bool) {
        let size = layout::window_size(&[shape], self.top());
        self.place_window_size(size, show);
    }

    fn place_window_size(&mut self, size: Size, show: bool) {
        let mon = primary_monitor();
        let w = (size.w * self.ui.scale).ceil() as i32;
        let h = (size.h * self.ui.scale).ceil() as i32;
        // Lebar genap menjaga pusat pil tetap di piksel yang sama saat berubah ukuran.
        let w = w + (mon.width() - w).rem_euclid(2);
        let x = mon.left + (mon.width() - w) / 2;
        self.window_px = (w, h);
        self.window.set_bounds(x, mon.top, w, h, show);
    }

    /// Menyetel timer Win32 hanya untuk hal yang benar-benar perlu, agar
    /// tidak ada wakeup saat idle (PRD §8.1).
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
        if showing == Some(ViewKind::Card(Card::Media)) {
            if let Some(pos) = self.snap.as_ref().filter(|s| s.is_playing()).and_then(|s| s.position_at(now)) {
                want(Some(now + 1000 - pos % 1000));
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

        if showing == Some(ViewKind::Card(Card::Home)) {
            let c = shell::local_time();
            let ms = (60 - c.second as u32) * 1000 - c.millis as u32;
            self.window.set_timer(T_CLOCK, ms.max(50));
        } else {
            self.window.kill_timer(T_CLOCK);
        }
    }
}

/// Jalur `config.toml` untuk ditampilkan oleh CLI.
pub fn config_path() -> PathBuf {
    Paths::default_location().config
}

/// Menulis ulang `config.toml` ke isi default.
pub fn reset_config(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, config::DEFAULT_TOML)
}

/// Log diagnostik ke `%LOCALAPPDATA%\Notchi\notchi.log`. Hanya pesan
/// kesalahan teknis; tidak pernah judul lagu, jendela, atau data pribadi
/// (PRD §7.1).
pub fn log(msg: &str) {
    use std::io::Write;
    if cfg!(debug_assertions) {
        eprintln!("[notchi] {msg}");
    }
    let Some(base) = std::env::var_os("LOCALAPPDATA") else { return };
    let dir = PathBuf::from(base).join("Notchi");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("notchi.log");
    // Batasi ukuran log agar tidak tumbuh tanpa batas.
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 256 * 1024) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{msg}");
    }
}
