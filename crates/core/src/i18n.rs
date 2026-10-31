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
    MenuBehavior,
    BehaviorAutohide,
    BehaviorSticky,
    BehaviorHide,
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
    SettingsTitle,
    SecIsland,
    SecLook,
    SecMascot,
    SecMedia,
    SecGeneral,
    Behavior,
    SegAutohide,
    SegSticky,
    SegHide,
    DescAutohide,
    DescSticky,
    DescHide,
    Style,
    SegNeu,
    SegClay,
    SegClassic,
    Liquid,
    ReduceMotion,
    SegSystem,
    SegOn,
    SegOff,
    Fullscreen,
    ShowMascot,
    MascotColor,
    MediaControls,
    Lyrics,
    LyricsDesc,
    ScrollVolume,
    TimerSound,
    Language,
    Advanced,
    AdvancedDesc,
    OpenFile,
    Version,
    Shortcut,
    LyricsLoading,
    LyricsMissing,
    Instrumental,
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
        (Lang::Id, MenuBehavior) => "Perilaku",
        (Lang::En, MenuBehavior) => "Behavior",
        (Lang::Id, BehaviorAutohide) => "Sembunyi otomatis di tepi atas",
        (Lang::En, BehaviorAutohide) => "Autohide at the top edge",
        (Lang::Id, BehaviorSticky) => "Selalu menempel",
        (Lang::En, BehaviorSticky) => "Sticky",
        (Lang::Id, BehaviorHide) => "Hanya saat ada aktivitas",
        (Lang::En, BehaviorHide) => "Only during activity",
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
        (Lang::Id, SettingsTitle) => "Pengaturan Notchi",
        (Lang::En, SettingsTitle) => "Notchi Settings",
        (Lang::Id, SecIsland) => "ISLAND",
        (Lang::En, SecIsland) => "ISLAND",
        (Lang::Id, SecLook) => "TAMPILAN",
        (Lang::En, SecLook) => "APPEARANCE",
        (Lang::Id, SecMascot) => "MASKOT",
        (Lang::En, SecMascot) => "MASCOT",
        (Lang::Id, SecMedia) => "MEDIA & TIMER",
        (Lang::En, SecMedia) => "MEDIA & TIMER",
        (Lang::Id, SecGeneral) => "UMUM",
        (Lang::En, SecGeneral) => "GENERAL",
        (Lang::Id, Behavior) => "Perilaku",
        (Lang::En, Behavior) => "Behavior",
        (Lang::Id, SegAutohide) => "Otomatis",
        (Lang::En, SegAutohide) => "Autohide",
        (Lang::Id, SegSticky) => "Menempel",
        (Lang::En, SegSticky) => "Sticky",
        (Lang::Id, SegHide) => "Sembunyi",
        (Lang::En, SegHide) => "Hide",
        (Lang::Id, DescAutohide) => "Masuk ke tepi atas; turun saat kursor mendekati notch.",
        (Lang::En, DescAutohide) => "Tucks into the top edge; drops down when the cursor nears the notch.",
        (Lang::Id, DescSticky) => "Selalu tampil menempel di tepi atas layar.",
        (Lang::En, DescSticky) => "Always visible, attached to the top edge of the screen.",
        (Lang::Id, DescHide) => "Tersembunyi; hanya muncul saat ada aktivitas atau alert.",
        (Lang::En, DescHide) => "Hidden; appears only for activities and alerts.",
        (Lang::Id, Style) => "Gaya",
        (Lang::En, Style) => "Style",
        (Lang::Id, SegNeu) => "Neumorph",
        (Lang::En, SegNeu) => "Neumorph",
        (Lang::Id, SegClay) => "Clay 3D",
        (Lang::En, SegClay) => "Clay 3D",
        (Lang::Id, SegClassic) => "Klasik",
        (Lang::En, SegClassic) => "Classic",
        (Lang::Id, Liquid) => "Efek cairan",
        (Lang::En, Liquid) => "Liquid effect",
        (Lang::Id, ReduceMotion) => "Kurangi gerakan",
        (Lang::En, ReduceMotion) => "Reduce motion",
        (Lang::Id, SegSystem) => "Sistem",
        (Lang::En, SegSystem) => "System",
        (Lang::Id, SegOn) => "Ya",
        (Lang::En, SegOn) => "On",
        (Lang::Id, SegOff) => "Tidak",
        (Lang::En, SegOff) => "Off",
        (Lang::Id, Fullscreen) => "Sembunyi saat layar penuh",
        (Lang::En, Fullscreen) => "Hide in full screen",
        (Lang::Id, ShowMascot) => "Tampilkan maskot",
        (Lang::En, ShowMascot) => "Show mascot",
        (Lang::Id, MascotColor) => "Warna",
        (Lang::En, MascotColor) => "Color",
        (Lang::Id, MediaControls) => "Kontrol media",
        (Lang::En, MediaControls) => "Media controls",
        (Lang::Id, Lyrics) => "Lirik tersinkron",
        (Lang::En, Lyrics) => "Synced lyrics",
        (Lang::Id, LyricsDesc) => "Dari lrclib.net; judul dan artis lagu dikirim ke sana.",
        (Lang::En, LyricsDesc) => "From lrclib.net; the song title and artist are sent there.",
        (Lang::Id, ScrollVolume) => "Scroll untuk volume",
        (Lang::En, ScrollVolume) => "Scroll for volume",
        (Lang::Id, TimerSound) => "Bunyi saat timer selesai",
        (Lang::En, TimerSound) => "Chime when a timer ends",
        (Lang::Id, Language) => "Bahasa",
        (Lang::En, Language) => "Language",
        (Lang::Id, Advanced) => "Pengaturan lanjutan",
        (Lang::En, Advanced) => "Advanced settings",
        (Lang::Id, AdvancedDesc) => "Pintasan, preset timer, sumber media",
        (Lang::En, AdvancedDesc) => "Hotkey, timer presets, media sources",
        (Lang::Id, OpenFile) => "Buka berkas",
        (Lang::En, OpenFile) => "Open file",
        (Lang::Id, Version) => "Versi",
        (Lang::En, Version) => "Version",
        (Lang::Id, Shortcut) => "Pintasan",
        (Lang::En, Shortcut) => "Shortcut",
        (Lang::Id, LyricsLoading) => "Mencari lirik…",
        (Lang::En, LyricsLoading) => "Finding lyrics…",
        (Lang::Id, LyricsMissing) => "Lirik tidak tersedia",
        (Lang::En, LyricsMissing) => "No lyrics found",
        (Lang::Id, Instrumental) => "Instrumental",
        (Lang::En, Instrumental) => "Instrumental",
    }
}

const DAYS_ID: [&str; 7] = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];
const DAYS_EN: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS_ID: [&str; 12] =
    ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
const MONTHS_EN: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

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
