//! String antarmuka dalam bahasa Indonesia dan Inggris (PRD §5.4).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Id,
    En,
}

impl Lang {
    pub fn from_key(key: &str) -> Option<Lang> {
        match key.to_ascii_lowercase().as_str() {
            "id" | "id-id" => Some(Lang::Id),
            "en" | "en-us" | "en-gb" => Some(Lang::En),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text {
    MenuSettings,
    MenuHideHour,
    MenuShowAgain,
    MenuTimer,
    MenuStopwatch,
    MenuMode,
    ModeAlways,
    ModeActivity,
    ModeHover,
    MenuLaunchAtLogin,
    MenuAbout,
    MenuQuit,
    Minutes,
    Charging,
    Unplugged,
    BatteryLow,
    BatteryCritical,
    TimerDone,
    TimerLabel,
    TimerPaused,
    StopwatchLabel,
    Volume,
    Muted,
    NothingPlaying,
    AboutBody,
    TrayTooltip,
    AlreadyRunning,
    ConfigInvalid,
    HotkeyTaken,
}

pub fn t(lang: Lang, key: Text) -> &'static str {
    use Text::*;
    match (lang, key) {
        (Lang::Id, MenuSettings) => "Pengaturan…",
        (Lang::En, MenuSettings) => "Settings…",
        (Lang::Id, MenuHideHour) => "Sembunyikan 1 jam",
        (Lang::En, MenuHideHour) => "Hide for 1 hour",
        (Lang::Id, MenuShowAgain) => "Tampilkan lagi",
        (Lang::En, MenuShowAgain) => "Show again",
        (Lang::Id, MenuTimer) => "Timer",
        (Lang::En, MenuTimer) => "Timer",
        (Lang::Id, MenuStopwatch) => "Stopwatch",
        (Lang::En, MenuStopwatch) => "Stopwatch",
        (Lang::Id, MenuMode) => "Mode tampil",
        (Lang::En, MenuMode) => "Display mode",
        (Lang::Id, ModeAlways) => "Selalu tampil",
        (Lang::En, ModeAlways) => "Always visible",
        (Lang::Id, ModeActivity) => "Hanya saat ada aktivitas",
        (Lang::En, ModeActivity) => "Only during activity",
        (Lang::Id, ModeHover) => "Muncul saat hover",
        (Lang::En, ModeHover) => "Show on hover",
        (Lang::Id, MenuLaunchAtLogin) => "Mulai bersama Windows",
        (Lang::En, MenuLaunchAtLogin) => "Start with Windows",
        (Lang::Id, MenuAbout) => "Tentang Notchi",
        (Lang::En, MenuAbout) => "About Notchi",
        (Lang::Id, MenuQuit) => "Keluar",
        (Lang::En, MenuQuit) => "Quit",
        (Lang::Id, Minutes) => "menit",
        (Lang::En, Minutes) => "min",
        (Lang::Id, Charging) => "Mengisi daya",
        (Lang::En, Charging) => "Charging",
        (Lang::Id, Unplugged) => "Pengisi daya dicabut",
        (Lang::En, Unplugged) => "Charger disconnected",
        (Lang::Id, BatteryLow) => "Baterai rendah",
        (Lang::En, BatteryLow) => "Battery low",
        (Lang::Id, BatteryCritical) => "Baterai hampir habis",
        (Lang::En, BatteryCritical) => "Battery critically low",
        (Lang::Id, TimerDone) => "Timer selesai",
        (Lang::En, TimerDone) => "Timer finished",
        (Lang::Id, TimerLabel) => "Timer",
        (Lang::En, TimerLabel) => "Timer",
        (Lang::Id, TimerPaused) => "Timer dijeda",
        (Lang::En, TimerPaused) => "Timer paused",
        (Lang::Id, StopwatchLabel) => "Stopwatch",
        (Lang::En, StopwatchLabel) => "Stopwatch",
        (Lang::Id, Volume) => "Volume",
        (Lang::En, Volume) => "Volume",
        (Lang::Id, Muted) => "Senyap",
        (Lang::En, Muted) => "Muted",
        (Lang::Id, NothingPlaying) => "Tidak ada yang diputar",
        (Lang::En, NothingPlaying) => "Nothing playing",
        (Lang::Id, AboutBody) => {
            "Island interaktif bergaya macOS untuk Windows 11.\n\nGratis dan sumber terbuka (MIT). Maskot dan aset: CC BY 4.0.\nTidak berafiliasi dengan Apple, Microsoft, Google, atau OpenAI."
        }
        (Lang::En, AboutBody) => {
            "A macOS-style interactive island for Windows 11.\n\nFree and open source (MIT). Mascot and assets: CC BY 4.0.\nNot affiliated with Apple, Microsoft, Google, or OpenAI."
        }
        (Lang::Id, TrayTooltip) => "Notchi — klik untuk membuka Island",
        (Lang::En, TrayTooltip) => "Notchi — click to open the island",
        (Lang::Id, AlreadyRunning) => "Notchi sudah berjalan.",
        (Lang::En, AlreadyRunning) => "Notchi is already running.",
        (Lang::Id, ConfigInvalid) => "Sebagian pengaturan tidak valid dan diganti nilai default",
        (Lang::En, ConfigInvalid) => "Some settings were invalid and replaced with defaults",
        (Lang::Id, HotkeyTaken) => "Hotkey sudah dipakai aplikasi lain",
        (Lang::En, HotkeyTaken) => "Hotkey is already used by another app",
    }
}

const DAYS_ID: [&str; 7] = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];
const DAYS_EN: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS_ID: [&str; 12] =
    ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
const MONTHS_EN: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// Tanggal panjang, misalnya "Sabtu, 31 Oktober". `weekday` 0 = Minggu,
/// `month` 1–12.
pub fn long_date(lang: Lang, weekday: u8, day: u8, month: u8) -> String {
    let w = weekday as usize % 7;
    let m = (month.clamp(1, 12) - 1) as usize;
    match lang {
        Lang::Id => format!("{}, {} {}", DAYS_ID[w], day, MONTHS_ID[m]),
        Lang::En => format!("{}, {} {}", DAYS_EN[w], MONTHS_EN[m], day),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanggal() {
        assert_eq!(long_date(Lang::Id, 6, 31, 10), "Sabtu, 31 Oktober");
        assert_eq!(long_date(Lang::En, 6, 31, 10), "Saturday, October 31");
    }

    #[test]
    fn bahasa() {
        assert_eq!(Lang::from_key("EN"), Some(Lang::En));
        assert_eq!(Lang::from_key("fr"), None);
        assert_eq!(t(Lang::Id, Text::MenuQuit), "Keluar");
    }
}
