use crate::color::Skin;
use crate::hotkey::Hotkey;
use crate::i18n::Lang;
use crate::island::Behavior;
use crate::layout::{HoverZone, Placement};
use crate::theme::{Style, Theme};
use toml::{Table, Value};

pub const MAX_CONFIG_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReduceMotion {
    #[default]
    System,
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IslandConfig {
    pub behavior: Behavior,
    pub hotkey: Hotkey,
    pub hide_in_fullscreen: bool,
    pub avoid_windows: bool,
    /// Titik tengah pil sebagai pecahan lebar layar: 0 = kiri, 0,5 = tengah, 1 = kanan.
    pub x: f32,
    pub hover_zone: HoverZone,
    pub side_mascot: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MascotConfig {
    pub enabled: bool,
    pub skin: Skin,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaConfig {
    pub enabled: bool,
    pub allow: Vec<String>,
    pub block: Vec<String>,
    pub scroll_volume: bool,
    pub scroll_brightness: bool,
    pub paused_linger_seconds: u32,
    pub lyrics: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DownloadsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShelfConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BluetoothConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PowerConfig {
    pub enabled: bool,
    pub low_thresholds: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimerConfig {
    pub presets_minutes: Vec<u32>,
    pub sound: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub language: Lang,
    pub island: IslandConfig,
    pub style: Style,
    pub theme: Theme,
    /// Blur acrylic sungguhan untuk gaya Kaca (bila efek transparansi Windows aktif).
    pub blur: bool,
    pub mascot: MascotConfig,
    pub media: MediaConfig,
    pub power: PowerConfig,
    pub downloads: DownloadsConfig,
    pub bluetooth: BluetoothConfig,
    pub shelf: ShelfConfig,
    pub timer: TimerConfig,
    pub reduce_motion: ReduceMotion,
    pub liquid: bool,
    pub launch_at_login: bool,
}

pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+N";

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Lang::Id,
            island: IslandConfig {
                behavior: Behavior::Autohide,
                hotkey: Hotkey::parse(DEFAULT_HOTKEY).expect("hotkey default valid"),
                hide_in_fullscreen: true,
                avoid_windows: true,
                x: 0.5,
                hover_zone: HoverZone::Small,
                side_mascot: true,
            },
            style: Style::Neumorphism,
            theme: Theme::System,
            blur: true,
            mascot: MascotConfig { enabled: true, skin: Skin::Langit },
            media: MediaConfig {
                enabled: true,
                allow: Vec::new(),
                block: Vec::new(),
                scroll_volume: true,
                scroll_brightness: true,
                paused_linger_seconds: 60,
                lyrics: false,
            },
            power: PowerConfig { enabled: true, low_thresholds: vec![20, 10] },
            downloads: DownloadsConfig { enabled: true },
            bluetooth: BluetoothConfig { enabled: true },
            shelf: ShelfConfig { enabled: true },
            timer: TimerConfig { presets_minutes: vec![1, 5, 10, 25], sound: true },
            reduce_motion: ReduceMotion::System,
            liquid: true,
            launch_at_login: false,
        }
    }
}

pub const DEFAULT_TOML: &str = r#"# Pengaturan Notchi. Berkas ini dimuat ulang otomatis setelah disimpan.
# Sebagian besar pilihan juga bisa diubah di jendela Pengaturan (klik kanan pil).
# Nilai yang tidak valid diganti default; aplikasi tidak akan crash.

# Bahasa antarmuka: "id" atau "en".
language = "id"

[island]
# Perilaku pil yang menempel di tepi atas layar:
#   "autohide"  masuk ke tepi dan menyisakan bibir tipis; turun saat kursor
#               ditahan sebentar di tepi paling atas, tepat di notch.
#               Alert dan aktivitas baru tetap menyembul.
#   "sticky"    selalu tampil menempel di tepi atas.
#   "hide"      tersembunyi; hanya muncul saat ada aktivitas atau alert.
behavior = "autohide"
# Buka/tutup Island tanpa mouse.
hotkey = "Ctrl+Alt+N"
hide_in_fullscreen = true
# Saat jendela aplikasi (misalnya browser yang di-maximize) berada tepat di
# bawah Island, Island masuk ke tepi atas agar tab dan bilah judul tidak tertutup.
# Tidak berlaku untuk behavior = "sticky", yang selalu tampil.
avoid_windows = true
# Letak pil di tepi atas, sebagai pecahan lebar layar: 0 = kiri, 0.5 = tengah
# (di notch), 1 = kanan. Seret pil ke kiri atau kanan untuk mengubahnya.
x = 0.5
# Zona pemicu saat pil masuk ke tepi:
#   "small"   selebar bibir, setipis 2 dp, kursor harus diam 0,4 detik.
#   "normal"  selebar pil, 4 dp, ditahan 0,2 detik.
hover_zone = "small"
# Saat pil berada di samping (x di bawah 0.3 atau di atas 0.7) dan tidak ada aktivitas, tampilkan maskot kecil
# yang menemani di tepi atas (tidak masuk ke tepi). Matanya mengikuti kursor.
side_mascot = true

[appearance]
# "neumorphism" (permukaan lembut timbul), "clay" (tombol 3D seperti tanah liat),
# "glass" (kaca modern tembus pandang), atau "classic" (pekat, datar).
style = "neumorphism"
# Tema warna: "system" mengikuti Windows (mode aplikasi terang/gelap),
# "dark" selalu gelap, "light" selalu terang.
theme = "system"
# Gaya "glass" memakai blur acrylic dari isi di belakang pil, seperti macOS. Butuh
# "Efek transparansi" Windows aktif; tanpanya, atau bila false, pil cukup tembus pandang.
blur = true

[mascot]
enabled = true
# langit, karang, mint, lemon, lavender, atau permen.
skin = "langit"

[media]
enabled = true
# Daftar izin/blokir sumber, dicocokkan dengan sebagian ID aplikasi,
# misalnya ["chrome"] atau ["spotify", "applemusic"]. Kosong = semua sumber.
allow = []
block = []
# Scroll di atas pil untuk mengatur volume.
scroll_volume = true
# Shift + scroll di atas pil untuk mengatur kecerahan layar (layar laptop, atau monitor
# eksternal yang mendukung DDC/CI).
scroll_brightness = true
# Berapa detik media yang dijeda tetap tampil (0–3600).
paused_linger_seconds = 60
# Lirik tersinkron dari lrclib.net (cadangan: lyrics.ovh). Judul, artis, album, dan durasi lagu
# dikirim ke layanan itu, jadi pilihan ini mati secara default.
lyrics = false

[power]
enabled = true
# Alert baterai rendah, dalam persen.
low_thresholds = [20, 10]

[downloads]
# Tampilkan unduhan browser yang sedang berjalan dan alert saat selesai. Dibaca dari berkas
# sementara (.crdownload, .part) di folder Downloads; ukuran totalnya tidak diketahui.
enabled = true

[bluetooth]
# Alert saat perangkat Bluetooth tersambung atau terputus, beserta baterainya bila dilaporkan.
enabled = true

[shelf]
# Rak berkas: seret berkas ke tepi atas (Island) untuk disimpan sementara, lalu seret keluar
# ke folder atau aplikasi lain. Hanya jalurnya yang disimpan; berkas tidak disalin.
enabled = true

[timer]
# Tombol timer cepat di kartu beranda, dalam menit (maks. 5 tombol).
presets_minutes = [1, 5, 10, 25]
sound = true

[motion]
# "system" mengikuti Windows, "on" selalu tanpa animasi pegas, "off" selalu beranimasi.
reduce = "system"
# Efek cairan: lengkung cekung di pertemuan pil dengan tepi atas layar dan
# pantulan saat pil turun.
liquid = true

[startup]
launch_at_login = false
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub key: String,
    pub message: String,
}

struct Reader<'a> {
    root: &'a Table,
    warnings: Vec<Warning>,
}

impl<'a> Reader<'a> {
    fn warn(&mut self, key: &str, message: impl Into<String>) {
        self.warnings.push(Warning { key: key.to_string(), message: message.into() });
    }

    fn lookup(&mut self, path: &str) -> Option<&'a Value> {
        let (section, key) = match path.split_once('.') {
            Some((s, k)) => (Some(s), k),
            None => (None, path),
        };
        let table = match section {
            None => self.root,
            Some(s) => match self.root.get(s)? {
                Value::Table(t) => t,
                _ => {
                    self.warn(s, "harus berupa tabel [bagian]");
                    return None;
                }
            },
        };
        table.get(key)
    }

    fn bool(&mut self, path: &str, default: bool) -> bool {
        match self.lookup(path) {
            None => default,
            Some(Value::Boolean(b)) => *b,
            Some(_) => {
                self.warn(path, "harus true atau false");
                default
            }
        }
    }

    fn int(&mut self, path: &str, min: i64, max: i64, default: i64) -> i64 {
        match self.lookup(path) {
            None => default,
            Some(Value::Integer(i)) if (min..=max).contains(i) => *i,
            Some(Value::Float(f)) if f.fract() == 0.0 && (min as f64..=max as f64).contains(f) => *f as i64,
            Some(_) => {
                self.warn(path, format!("harus bilangan bulat {min}–{max}"));
                default
            }
        }
    }

    fn float(&mut self, path: &str, min: f32, max: f32, default: f32) -> f32 {
        let v = match self.lookup(path) {
            None => return default,
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Integer(i)) => *i as f32,
            Some(_) => f32::NAN,
        };
        if v.is_finite() && (min..=max).contains(&v) {
            v
        } else {
            self.warn(path, format!("harus angka {min}–{max}"));
            default
        }
    }

    fn string(&mut self, path: &str) -> Option<&'a str> {
        match self.lookup(path) {
            None => None,
            Some(Value::String(s)) => Some(s.as_str()),
            Some(_) => {
                self.warn(path, "harus berupa teks");
                None
            }
        }
    }

    fn parsed<T>(&mut self, path: &str, default: T, parse: impl Fn(&str) -> Result<T, String>) -> T {
        match self.string(path) {
            None => default,
            Some(s) => match parse(s) {
                Ok(v) => v,
                Err(e) => {
                    self.warn(path, e);
                    default
                }
            },
        }
    }

    fn string_list(&mut self, path: &str, max_items: usize) -> Vec<String> {
        match self.lookup(path) {
            None => Vec::new(),
            Some(Value::Array(items)) => {
                let mut out = Vec::new();
                for v in items {
                    match v {
                        Value::String(s) if !s.trim().is_empty() && s.len() <= 256 => out.push(s.trim().to_string()),
                        _ => self.warn(path, "setiap item harus teks tidak kosong (≤ 256 karakter)"),
                    }
                }
                if out.len() > max_items {
                    self.warn(path, format!("maksimal {max_items} item"));
                    out.truncate(max_items);
                }
                out
            }
            Some(_) => {
                self.warn(path, "harus berupa daftar teks");
                Vec::new()
            }
        }
    }

    fn int_list(&mut self, path: &str, min: i64, max: i64, max_items: usize, default: &[i64]) -> Vec<i64> {
        match self.lookup(path) {
            None => default.to_vec(),
            Some(Value::Array(items)) => {
                let mut out = Vec::new();
                for v in items {
                    match v {
                        Value::Integer(i) if (min..=max).contains(i) => out.push(*i),
                        _ => self.warn(path, format!("setiap item harus bilangan bulat {min}–{max}")),
                    }
                }
                if out.len() > max_items {
                    self.warn(path, format!("maksimal {max_items} item"));
                    out.truncate(max_items);
                }
                out
            }
            Some(_) => {
                self.warn(path, "harus berupa daftar angka");
                default.to_vec()
            }
        }
    }
}

pub const LEGACY_ISLAND_KEYS: [&str; 3] = ["mode", "autohide", "top_margin"];

const KNOWN: &[(&str, &[&str])] = &[
    ("island", &["behavior", "hotkey", "hide_in_fullscreen", "avoid_windows", "x", "position", "hover_zone", "side_mascot", "mode", "autohide", "top_margin"]),
    ("appearance", &["style", "theme", "blur"]),
    ("mascot", &["enabled", "skin"]),
    ("media", &["enabled", "allow", "block", "scroll_volume", "scroll_brightness", "paused_linger_seconds", "lyrics"]),
    ("downloads", &["enabled"]),
    ("bluetooth", &["enabled"]),
    ("shelf", &["enabled"]),
    ("power", &["enabled", "low_thresholds"]),
    ("timer", &["presets_minutes", "sound"]),
    ("motion", &["reduce", "liquid"]),
    ("startup", &["launch_at_login"]),
];

fn legacy_behavior(r: &mut Reader<'_>, default: Behavior) -> Behavior {
    let mode = r.parsed("island.mode", None, |s| match s.to_ascii_lowercase().as_str() {
        "always" => Ok(None),
        "activity" => Ok(Some(Behavior::Hide)),
        "hover" => Ok(Some(Behavior::Autohide)),
        _ => Err("harus \"always\", \"activity\", atau \"hover\"".into()),
    });
    let autohide = r.bool("island.autohide", default == Behavior::Autohide);
    mode.unwrap_or(if autohide { Behavior::Autohide } else { Behavior::Sticky })
}

fn check_unknown(root: &Table, warnings: &mut Vec<Warning>) {
    for (k, v) in root {
        if k == "language" {
            continue;
        }
        match KNOWN.iter().find(|(s, _)| s == k) {
            None => warnings.push(Warning { key: k.clone(), message: "kunci tidak dikenal, diabaikan".into() }),
            Some((_, keys)) => {
                if let Value::Table(t) = v {
                    for sub in t.keys() {
                        if !keys.contains(&sub.as_str()) {
                            warnings.push(Warning {
                                key: format!("{k}.{sub}"),
                                message: "kunci tidak dikenal, diabaikan".into(),
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn parse(text: &str) -> (Config, Vec<Warning>) {
    let d = Config::default();
    if text.len() > MAX_CONFIG_BYTES {
        let w = Warning { key: String::new(), message: format!("berkas lebih dari {MAX_CONFIG_BYTES} byte") };
        return (d, vec![w]);
    }
    let root: Table = match text.parse::<Table>() {
        Ok(t) => t,
        Err(e) => {
            let msg = e.to_string().lines().next().unwrap_or("sintaks TOML tidak valid").to_string();
            return (d, vec![Warning { key: String::new(), message: msg }]);
        }
    };
    let mut r = Reader { root: &root, warnings: Vec::new() };

    let language = r.parsed("language", d.language, |s| Lang::from_key(s).ok_or("harus \"id\" atau \"en\"".into()));
    let behavior = if r.lookup("island.behavior").is_some() {
        r.parsed("island.behavior", d.island.behavior, |s| {
            Behavior::from_key(s).ok_or("harus \"autohide\", \"sticky\", atau \"hide\"".into())
        })
    } else {
        legacy_behavior(&mut r, d.island.behavior)
    };
    let hotkey = r.parsed("island.hotkey", d.island.hotkey, Hotkey::parse);
    let hide_in_fullscreen = r.bool("island.hide_in_fullscreen", d.island.hide_in_fullscreen);
    let avoid_windows = r.bool("island.avoid_windows", d.island.avoid_windows);
    let x = if r.lookup("island.x").is_some() {
        r.float("island.x", 0.0, 1.0, d.island.x)
    } else {
        r.parsed("island.position", Placement::Center, |s| {
            Placement::from_key(s).ok_or("harus \"center\", \"left\", atau \"right\"".into())
        })
        .x()
    };
    let hover_zone = r.parsed("island.hover_zone", d.island.hover_zone, |s| {
        HoverZone::from_key(s).ok_or("harus \"small\" atau \"normal\"".into())
    });
    let side_mascot = r.bool("island.side_mascot", d.island.side_mascot);
    let style = r.parsed("appearance.style", d.style, |s| {
        Style::from_key(s).ok_or("harus \"neumorphism\", \"clay\", \"glass\", atau \"classic\"".into())
    });
    let blur = r.bool("appearance.blur", d.blur);
    let theme = r.parsed("appearance.theme", d.theme, |s| {
        Theme::from_key(s).ok_or("harus \"system\", \"dark\", atau \"light\"".into())
    });

    let mascot_enabled = r.bool("mascot.enabled", d.mascot.enabled);
    let skin = r.parsed("mascot.skin", d.mascot.skin, |s| Skin::from_key(s).ok_or(format!("varian tidak dikenal: {s:?}")));

    let media_enabled = r.bool("media.enabled", d.media.enabled);
    let allow = r.string_list("media.allow", 64);
    let block = r.string_list("media.block", 64);
    let scroll_volume = r.bool("media.scroll_volume", d.media.scroll_volume);
    let scroll_brightness = r.bool("media.scroll_brightness", d.media.scroll_brightness);
    let downloads_enabled = r.bool("downloads.enabled", d.downloads.enabled);
    let bluetooth_enabled = r.bool("bluetooth.enabled", d.bluetooth.enabled);
    let shelf_enabled = r.bool("shelf.enabled", d.shelf.enabled);
    let linger = r.int("media.paused_linger_seconds", 0, 3600, d.media.paused_linger_seconds as i64) as u32;
    let lyrics = r.bool("media.lyrics", d.media.lyrics);

    let power_enabled = r.bool("power.enabled", d.power.enabled);
    let default_thresholds: Vec<i64> = d.power.low_thresholds.iter().map(|&t| t as i64).collect();
    let thresholds = r.int_list("power.low_thresholds", 1, 99, 4, &default_thresholds);

    let default_presets: Vec<i64> = d.timer.presets_minutes.iter().map(|&t| t as i64).collect();
    let presets = r.int_list("timer.presets_minutes", 1, 1440, 5, &default_presets);
    let sound = r.bool("timer.sound", d.timer.sound);

    let reduce_motion = r.parsed("motion.reduce", d.reduce_motion, |s| match s.to_ascii_lowercase().as_str() {
        "system" => Ok(ReduceMotion::System),
        "on" => Ok(ReduceMotion::On),
        "off" => Ok(ReduceMotion::Off),
        _ => Err("harus \"system\", \"on\", atau \"off\"".into()),
    });
    let liquid = r.bool("motion.liquid", d.liquid);
    let launch_at_login = r.bool("startup.launch_at_login", d.launch_at_login);

    let mut warnings = r.warnings;
    check_unknown(&root, &mut warnings);

    let config = Config {
        language,
        island: IslandConfig { behavior, hotkey, hide_in_fullscreen, avoid_windows, x, hover_zone, side_mascot },
        style,
        theme,
        blur,
        mascot: MascotConfig { enabled: mascot_enabled, skin },
        media: MediaConfig { enabled: media_enabled, allow, block, scroll_volume, scroll_brightness, paused_linger_seconds: linger, lyrics },
        downloads: DownloadsConfig { enabled: downloads_enabled },
        bluetooth: BluetoothConfig { enabled: bluetooth_enabled },
        shelf: ShelfConfig { enabled: shelf_enabled },
        power: PowerConfig { enabled: power_enabled, low_thresholds: thresholds.into_iter().map(|t| t as u8).collect() },
        timer: TimerConfig { presets_minutes: presets.into_iter().map(|t| t as u32).collect(), sound },
        reduce_motion,
        liquid,
        launch_at_login,
    };
    (config, warnings)
}

fn is_key_line(line: &str, key: &str) -> bool {
    let t = line.trim_start();
    t.strip_prefix(key).is_some_and(|rest| rest.trim_start().starts_with('='))
}

fn section_range(lines: &[String], section: &str) -> Option<(usize, usize)> {
    let next_header = |from: usize| {
        lines[from..].iter().position(|l| l.trim_start().starts_with('[')).map_or(lines.len(), |p| from + p)
    };
    if section.is_empty() {
        return Some((0, next_header(0)));
    }
    let header = format!("[{section}]");
    let start = lines.iter().position(|l| l.trim() == header)? + 1;
    Some((start, next_header(start)))
}

fn finish(lines: Vec<String>) -> String {
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

pub fn set_value(text: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let new_line = format!("{key} = {value}");
    match section_range(&lines, section) {
        Some((start, end)) => match (start..end).find(|&i| is_key_line(&lines[i], key)) {
            Some(i) => lines[i] = new_line,
            None => {
                let mut at = end;
                while at > start && lines[at - 1].trim().is_empty() {
                    at -= 1;
                }
                lines.insert(at, new_line);
            }
        },
        None => {
            if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push(format!("[{section}]"));
            lines.push(new_line);
        }
    }
    finish(lines)
}

pub fn remove_value(text: &str, section: &str, key: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some((start, end)) = section_range(&lines, section) else { return text.to_string() };
    let Some(i) = (start..end).find(|&i| is_key_line(&lines[i], key)) else { return text.to_string() };
    let mut from = i;
    while from > start && lines[from - 1].trim_start().starts_with('#') {
        from -= 1;
    }
    lines.drain(from..=i);
    finish(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_default_valid_dan_sama_dengan_default() {
        let (c, w) = parse(DEFAULT_TOML);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(c, Config::default());
    }

    #[test]
    fn berkas_kosong_memakai_default() {
        let (c, w) = parse("");
        assert!(w.is_empty());
        assert_eq!(c, Config::default());
    }

    #[test]
    fn nilai_salah_diganti_default_per_field() {
        let text = r#"
language = "fr"
[island]
behavior = "sticky"
hotkey = "Shift+N"
[appearance]
style = "kaca"
[mascot]
skin = "ungu"
enabled = "ya"
[media]
lyrics = "ya"
[power]
low_thresholds = [15, 0, "x"]
"#;
        let (c, w) = parse(text);
        let d = Config::default();
        assert_eq!(c.island.behavior, Behavior::Sticky, "nilai valid tetap dipakai");
        assert_eq!(c.language, d.language);
        assert_eq!(c.style, d.style);
        assert_eq!(c.island.hotkey, d.island.hotkey);
        assert_eq!(c.mascot.skin, d.mascot.skin);
        assert_eq!(c.mascot.enabled, d.mascot.enabled);
        assert!(!c.media.lyrics);
        assert_eq!(c.power.low_thresholds, vec![15]);
        let keys: Vec<&str> = w.iter().map(|w| w.key.as_str()).collect();
        for k in ["language", "appearance.style", "island.hotkey", "mascot.skin", "mascot.enabled", "media.lyrics", "power.low_thresholds"] {
            assert!(keys.contains(&k), "peringatan untuk {k} hilang: {keys:?}");
        }
    }

    #[test]
    fn kunci_lama_dimigrasikan() {
        let cases = [
            ("", Behavior::Autohide),
            ("[island]\nmode = \"always\"\nautohide = true\n", Behavior::Autohide),
            ("[island]\nmode = \"always\"\nautohide = false\ntop_margin = 8\n", Behavior::Sticky),
            ("[island]\nmode = \"activity\"\nautohide = true\n", Behavior::Hide),
            ("[island]\nmode = \"hover\"\nautohide = false\n", Behavior::Autohide),
            ("[island]\nbehavior = \"hide\"\nmode = \"always\"\nautohide = true\n", Behavior::Hide),
        ];
        for (text, want) in cases {
            let (c, w) = parse(text);
            assert_eq!(c.island.behavior, want, "{text:?}");
            assert!(w.is_empty(), "{text:?}: {w:?}");
        }
        let (c, w) = parse("[island]\nmode = \"kadang\"\n");
        assert_eq!(c.island.behavior, Behavior::Autohide);
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn lirik_mati_secara_default() {
        assert!(!Config::default().media.lyrics);
        assert!(parse("[media]\nlyrics = true\n").0.media.lyrics);
    }

    #[test]
    fn sintaks_rusak_tidak_crash() {
        let (c, w) = parse("[island\nmode = ");
        assert_eq!(c, Config::default());
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn berkas_terlalu_besar_ditolak() {
        let big = "#".repeat(MAX_CONFIG_BYTES + 1);
        let (c, w) = parse(&big);
        assert_eq!(c, Config::default());
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn kunci_tidak_dikenal_diperingatkan() {
        let (_, w) = parse("rahasia = 1\n[island]\nwarna = \"merah\"\n[media]\nenabled = false\n");
        let mut keys: Vec<&str> = w.iter().map(|w| w.key.as_str()).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["island.warna", "rahasia"]);
    }

    #[test]
    fn bagian_bukan_tabel() {
        let (c, w) = parse("island = 3\n");
        assert_eq!(c, Config::default());
        assert!(!w.is_empty());
    }

    #[test]
    fn daftar_dibatasi() {
        let (c, w) = parse("[timer]\npresets_minutes = [1, 2, 3, 4, 5, 6, 7]\n");
        assert_eq!(c.timer.presets_minutes, vec![1, 2, 3, 4, 5]);
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn set_value_mengganti_dan_menjaga_komentar() {
        let out = set_value(DEFAULT_TOML, "island", "behavior", "\"hide\"");
        assert!(out.contains("behavior = \"hide\""));
        assert!(out.contains("#   \"sticky\"    selalu tampil"));
        let (c, w) = parse(&out);
        assert!(w.is_empty());
        assert_eq!(c.island.behavior, Behavior::Hide);
        let out = set_value(&out, "startup", "launch_at_login", "true");
        let (c, _) = parse(&out);
        assert!(c.launch_at_login);
        assert_eq!(c.island.behavior, Behavior::Hide);
    }

    #[test]
    fn set_value_menambah_kunci_dan_bagian() {
        let out = set_value("[island]\nbehavior = \"sticky\"\n\n[media]\nenabled = true\n", "island", "hide_in_fullscreen", "false");
        let (c, w) = parse(&out);
        assert!(w.is_empty(), "{w:?}");
        assert!(!c.island.hide_in_fullscreen);
        assert!(out.find("hide_in_fullscreen").unwrap() < out.find("[media]").unwrap());
        let out = set_value("", "startup", "launch_at_login", "true");
        assert!(parse(&out).0.launch_at_login);
    }

    #[test]
    fn menyingkir_dari_jendela_bisa_dimatikan() {
        assert!(Config::default().island.avoid_windows);
        let out = set_value(DEFAULT_TOML, "island", "avoid_windows", "false");
        let (c, w) = parse(&out);
        assert!(w.is_empty(), "{w:?}");
        assert!(!c.island.avoid_windows);
    }

    #[test]
    fn posisi_dan_zona_hover() {
        let d = Config::default();
        assert_eq!(d.island.x, 0.5);
        assert_eq!(d.island.hover_zone, HoverZone::Small);
        let out = set_value(DEFAULT_TOML, "island", "x", "0.93");
        let out = set_value(&out, "island", "hover_zone", "\"normal\"");
        let (c, w) = parse(&out);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(c.island.x, 0.93);
        assert_eq!(c.island.hover_zone, HoverZone::Normal);
        assert_eq!(parse("[island]\nx = 1\n").0.island.x, 1.0, "bilangan bulat diterima");
        let (c, w) = parse("[island]\nx = 1.5\nhover_zone = 3\n");
        assert_eq!(c.island.x, 0.5);
        assert_eq!(c.island.hover_zone, HoverZone::Small);
        assert_eq!(w.len(), 2, "{w:?}");
        assert_eq!(parse("[island]\nposition = \"right\"\n").0.island.x, 1.0, "kunci lama position dimigrasikan");
        assert_eq!(parse("[island]\nposition = \"left\"\nx = 0.4\n").0.island.x, 0.4, "x menang atas position");
        let (c, w) = parse("[island]\nposition = \"atas\"\n");
        assert_eq!((c.island.x, w.len()), (0.5, 1));
        assert!(d.island.side_mascot);
        let (c, w) = parse(&set_value(DEFAULT_TOML, "island", "side_mascot", "false"));
        assert!(w.is_empty(), "{w:?}");
        assert!(!c.island.side_mascot);
    }

    #[test]
    fn unduhan_bluetooth_dan_kecerahan() {
        let d = Config::default();
        assert!(d.downloads.enabled && d.bluetooth.enabled && d.media.scroll_brightness);
        let out = set_value(DEFAULT_TOML, "downloads", "enabled", "false");
        let out = set_value(&out, "bluetooth", "enabled", "false");
        let out = set_value(&out, "media", "scroll_brightness", "false");
        let (c, w) = parse(&out);
        assert!(w.is_empty(), "{w:?}");
        assert!(!c.downloads.enabled && !c.bluetooth.enabled && !c.media.scroll_brightness);
    }

    #[test]
    fn tema_dan_gaya_kaca() {
        assert_eq!(Config::default().theme, Theme::System);
        let out = set_value(DEFAULT_TOML, "appearance", "theme", "\"light\"");
        let out = set_value(&out, "appearance", "style", "\"glass\"");
        let (c, w) = parse(&out);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!((c.theme, c.style), (Theme::Light, Style::Glass));
        let (c, w) = parse("[appearance]\ntheme = \"senja\"\n");
        assert_eq!((c.theme, w.len()), (Theme::System, 1));
    }

    #[test]
    fn set_value_kunci_tingkat_atas() {
        let out = set_value(DEFAULT_TOML, "", "language", "\"en\"");
        assert_eq!(parse(&out).0.language, Lang::En);
        assert_eq!(out.matches("language =").count(), 1);
        let out = set_value("# hanya komentar\n\n[island]\nbehavior = \"hide\"\n", "", "language", "\"en\"");
        assert!(out.starts_with("# hanya komentar\nlanguage = \"en\"\n"), "{out}");
        assert_eq!(parse(&out).0.island.behavior, Behavior::Hide);
    }

    #[test]
    fn remove_value_membuang_kunci_dan_komentarnya() {
        let old = "[island]\n# mode lama\nmode = \"always\"\n# Buka/tutup\nhotkey = \"Ctrl+Alt+N\"\n# a\n# b\nautohide = true\ntop_margin = 8\n\n[media]\n# biarkan\nenabled = true\n";
        let mut out = old.to_string();
        for k in LEGACY_ISLAND_KEYS {
            out = remove_value(&out, "island", k);
        }
        let out = set_value(&out, "island", "behavior", "\"autohide\"");
        assert_eq!(
            out,
            "[island]\n# Buka/tutup\nhotkey = \"Ctrl+Alt+N\"\nbehavior = \"autohide\"\n\n[media]\n# biarkan\nenabled = true\n"
        );
        assert_eq!(remove_value(&out, "power", "enabled"), out);
        assert_eq!(remove_value(&out, "media", "allow"), out);
    }
}
