use crate::color::Skin;
use crate::hotkey::Hotkey;
use crate::i18n::Lang;
use crate::island::DisplayMode;
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
    pub mode: DisplayMode,
    pub hotkey: Hotkey,
    pub hide_in_fullscreen: bool,
    pub autohide: bool,
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
    pub paused_linger_seconds: u32,
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
    pub mascot: MascotConfig,
    pub media: MediaConfig,
    pub power: PowerConfig,
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
                mode: DisplayMode::Always,
                hotkey: Hotkey::parse(DEFAULT_HOTKEY).expect("hotkey default valid"),
                hide_in_fullscreen: true,
                autohide: true,
            },
            mascot: MascotConfig { enabled: true, skin: Skin::Langit },
            media: MediaConfig {
                enabled: true,
                allow: Vec::new(),
                block: Vec::new(),
                scroll_volume: true,
                paused_linger_seconds: 60,
            },
            power: PowerConfig { enabled: true, low_thresholds: vec![20, 10] },
            timer: TimerConfig { presets_minutes: vec![1, 5, 10, 25], sound: true },
            reduce_motion: ReduceMotion::System,
            liquid: true,
            launch_at_login: false,
        }
    }
}

pub const DEFAULT_TOML: &str = r#"# Pengaturan Notchi. Berkas ini dimuat ulang otomatis setelah disimpan.
# Nilai yang tidak valid diganti default; aplikasi tidak akan crash.

# Bahasa antarmuka: "id" atau "en".
language = "id"

[island]
# "always" (selalu tampil), "activity" (hanya saat ada aktivitas), atau "hover".
mode = "always"
# Buka/tutup Island tanpa mouse.
hotkey = "Ctrl+Alt+N"
hide_in_fullscreen = true
# Pil menempel di tepi atas layar. Dengan autohide, pil masuk ke tepi dan
# hanya menyisakan bibir tipis; ia turun saat kursor didekatkan ke notch.
# Alert dan aktivitas baru tetap menyembul sebentar.
autohide = true

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
# Berapa detik media yang dijeda tetap tampil (0–3600).
paused_linger_seconds = 60

[power]
enabled = true
# Alert baterai rendah, dalam persen.
low_thresholds = [20, 10]

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

const KNOWN: &[(&str, &[&str])] = &[
    ("island", &["mode", "hotkey", "hide_in_fullscreen", "autohide", "top_margin"]),
    ("mascot", &["enabled", "skin"]),
    ("media", &["enabled", "allow", "block", "scroll_volume", "paused_linger_seconds"]),
    ("power", &["enabled", "low_thresholds"]),
    ("timer", &["presets_minutes", "sound"]),
    ("motion", &["reduce", "liquid"]),
    ("startup", &["launch_at_login"]),
];

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
    let mode = r.parsed("island.mode", d.island.mode, |s| {
        DisplayMode::from_key(s).ok_or("harus \"always\", \"activity\", atau \"hover\"".into())
    });
    let hotkey = r.parsed("island.hotkey", d.island.hotkey, Hotkey::parse);
    let hide_in_fullscreen = r.bool("island.hide_in_fullscreen", d.island.hide_in_fullscreen);
    let autohide = r.bool("island.autohide", d.island.autohide);

    let mascot_enabled = r.bool("mascot.enabled", d.mascot.enabled);
    let skin = r.parsed("mascot.skin", d.mascot.skin, |s| Skin::from_key(s).ok_or(format!("varian tidak dikenal: {s:?}")));

    let media_enabled = r.bool("media.enabled", d.media.enabled);
    let allow = r.string_list("media.allow", 64);
    let block = r.string_list("media.block", 64);
    let scroll_volume = r.bool("media.scroll_volume", d.media.scroll_volume);
    let linger = r.int("media.paused_linger_seconds", 0, 3600, d.media.paused_linger_seconds as i64) as u32;

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
        island: IslandConfig { mode, hotkey, hide_in_fullscreen, autohide },
        mascot: MascotConfig { enabled: mascot_enabled, skin },
        media: MediaConfig { enabled: media_enabled, allow, block, scroll_volume, paused_linger_seconds: linger },
        power: PowerConfig { enabled: power_enabled, low_thresholds: thresholds.into_iter().map(|t| t as u8).collect() },
        timer: TimerConfig { presets_minutes: presets.into_iter().map(|t| t as u32).collect(), sound },
        reduce_motion,
        liquid,
        launch_at_login,
    };
    (config, warnings)
}

pub fn set_value(text: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let header = format!("[{section}]");
    let is_key_line = |l: &str| {
        let t = l.trim_start();
        t.strip_prefix(key).is_some_and(|rest| rest.trim_start().starts_with('='))
    };
    let new_line = format!("{key} = {value}");
    match lines.iter().position(|l| l.trim() == header) {
        Some(start) => {
            let end = lines[start + 1..]
                .iter()
                .position(|l| l.trim_start().starts_with('['))
                .map_or(lines.len(), |p| start + 1 + p);
            match (start + 1..end).find(|&i| is_key_line(&lines[i])) {
                Some(i) => lines[i] = new_line,
                None => {
                    let mut at = end;
                    while at > start + 1 && lines[at - 1].trim().is_empty() {
                        at -= 1;
                    }
                    lines.insert(at, new_line);
                }
            }
        }
        None => {
            if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push(header);
            lines.push(new_line);
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
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
mode = "hover"
autohide = 2
hotkey = "Shift+N"
[mascot]
skin = "ungu"
enabled = "ya"
[power]
low_thresholds = [15, 0, "x"]
"#;
        let (c, w) = parse(text);
        let d = Config::default();
        assert_eq!(c.island.mode, DisplayMode::Hover, "nilai valid tetap dipakai");
        assert_eq!(c.language, d.language);
        assert_eq!(c.island.autohide, d.island.autohide);
        assert_eq!(c.island.hotkey, d.island.hotkey);
        assert_eq!(c.mascot.skin, d.mascot.skin);
        assert_eq!(c.mascot.enabled, d.mascot.enabled);
        assert_eq!(c.power.low_thresholds, vec![15]);
        let keys: Vec<&str> = w.iter().map(|w| w.key.as_str()).collect();
        for k in ["language", "island.autohide", "island.hotkey", "mascot.skin", "mascot.enabled", "power.low_thresholds"] {
            assert!(keys.contains(&k), "peringatan untuk {k} hilang: {keys:?}");
        }
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
        let out = set_value(DEFAULT_TOML, "island", "mode", "\"hover\"");
        assert!(out.contains("mode = \"hover\""));
        assert!(out.contains("# \"always\" (selalu tampil)"));
        let (c, w) = parse(&out);
        assert!(w.is_empty());
        assert_eq!(c.island.mode, DisplayMode::Hover);
        let out = set_value(&out, "startup", "launch_at_login", "true");
        let (c, _) = parse(&out);
        assert!(c.launch_at_login);
        assert_eq!(c.island.mode, DisplayMode::Hover);
    }

    #[test]
    fn set_value_menambah_kunci_dan_bagian() {
        let out = set_value("[island]\nmode = \"always\"\n\n[media]\nenabled = true\n", "island", "autohide", "false");
        let (c, w) = parse(&out);
        assert!(w.is_empty(), "{w:?}");
        assert!(!c.island.autohide);
        assert!(out.find("autohide").unwrap() < out.find("[media]").unwrap());
        let out = set_value("", "startup", "launch_at_login", "true");
        assert!(parse(&out).0.launch_at_login);
    }
}
