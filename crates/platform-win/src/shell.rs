use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
    RegQueryValueExW, RegSetValueExW,
};
use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::Shell::{
    QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MB_ICONASTERISK, SPI_GETCLIENTAREAANIMATION, SPI_GETHIGHCONTRAST,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SW_SHOWNORMAL, SystemParametersInfoW,
};
use windows::core::{BOOL, w};
use windows_core::HSTRING;

pub fn fullscreen_busy() -> bool {
    match unsafe { SHQueryUserNotificationState() } {
        Ok(s) => s == QUNS_BUSY || s == QUNS_RUNNING_D3D_FULL_SCREEN || s == QUNS_PRESENTATION_MODE,
        Err(_) => false,
    }
}

pub fn animations_enabled() -> bool {
    let mut on = BOOL(1);
    let ok = unsafe {
        SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, Some(&mut on as *mut BOOL as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))
    }
    .is_ok();
    !ok || on.as_bool()
}

/// Windows memakai tema terang untuk aplikasi (Pengaturan → Personalisasi → Warna).
pub fn apps_use_light_theme() -> bool {
    use windows::Win32::System::Registry::{RRF_RT_REG_DWORD, RegGetValueW};
    let mut value = 0u32;
    let mut size = size_of::<u32>() as u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    r == ERROR_SUCCESS && value != 0
}

pub fn high_contrast() -> bool {
    let mut hc = HIGHCONTRASTW { cbSize: size_of::<HIGHCONTRASTW>() as u32, ..Default::default() };
    let ok = unsafe {
        SystemParametersInfoW(SPI_GETHIGHCONTRAST, hc.cbSize, Some(&mut hc as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))
    }
    .is_ok();
    ok && hc.dwFlags.contains(HCF_HIGHCONTRASTON)
}

pub fn chime() {
    unsafe {
        let _ = MessageBeep(MB_ICONASTERISK);
    }
}

/// Folder Downloads milik pengguna (bisa sudah dipindah dari lokasi default).
pub fn downloads_dir() -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Downloads, KF_FLAG_DEFAULT, SHGetKnownFolderPath};
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None).ok()?;
        let path = p.to_string().ok().map(std::path::PathBuf::from);
        CoTaskMemFree(Some(p.0 as *const _));
        path
    }
}

/// Buka Explorer dengan berkas itu terpilih; bila berkasnya tidak ada, buka foldernya.
pub fn reveal(path: &std::path::Path) {
    if path.is_file() {
        let args = HSTRING::from(format!("/select,\"{}\"", path.display()));
        unsafe {
            ShellExecuteW(None, w!("open"), w!("explorer.exe"), &args, None, SW_SHOWNORMAL);
        }
    } else if let Some(dir) = path.parent().filter(|_| !path.is_dir()).or(Some(path)) {
        open_folder(dir);
    }
}

pub fn open_folder(dir: &std::path::Path) {
    let dir = HSTRING::from(dir.as_os_str());
    unsafe {
        ShellExecuteW(None, w!("open"), &dir, None, None, SW_SHOWNORMAL);
    }
}

pub fn edit_text_file(path: &std::path::Path) {
    let file = HSTRING::from(path.as_os_str());
    unsafe {
        ShellExecuteW(None, w!("open"), w!("notepad.exe"), &file, None, SW_SHOWNORMAL);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millis: u16,
    pub weekday: u8,
    pub day: u8,
    pub month: u8,
}

pub fn local_time() -> LocalTime {
    let t = unsafe { GetLocalTime() };
    LocalTime {
        hour: t.wHour as u8,
        minute: t.wMinute as u8,
        second: t.wSecond as u8,
        millis: t.wMilliseconds,
        weekday: t.wDayOfWeek as u8,
        day: t.wDay as u8,
        month: t.wMonth as u8,
    }
}

const RUN_KEY: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const RUN_VALUE: windows::core::PCWSTR = w!("Notchi");

fn open_run_key(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Option<HKEY> {
    let mut key = HKEY::default();
    let r = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, access, &mut key) };
    (r == ERROR_SUCCESS).then_some(key)
}

pub fn set_launch_at_login(enabled: bool, exe: &std::path::Path) -> bool {
    let Some(key) = open_run_key(KEY_SET_VALUE) else { return false };
    
    unsafe {
        let r = if enabled {
            let cmd: Vec<u16> = format!("\"{}\"", exe.display()).encode_utf16().chain(Some(0)).collect();
            let bytes = std::slice::from_raw_parts(cmd.as_ptr() as *const u8, cmd.len() * 2);
            RegSetValueExW(key, RUN_VALUE, None, REG_SZ, Some(bytes))
        } else {
            let r = RegDeleteValueW(key, RUN_VALUE);
            if r == windows::Win32::Foundation::ERROR_FILE_NOT_FOUND { ERROR_SUCCESS } else { r }
        };
        let _ = RegCloseKey(key);
        r == ERROR_SUCCESS
    }
}

pub fn launch_at_login_enabled() -> bool {
    let Some(key) = open_run_key(KEY_QUERY_VALUE) else { return false };
    unsafe {
        let r = RegQueryValueExW(key, RUN_VALUE, None, None, None, None);
        let _ = RegCloseKey(key);
        r == ERROR_SUCCESS
    }
}
