//! Pemantau perubahan folder untuk *hot reload* `config.toml` (PRD §5.4).

use std::path::Path;

use windows::Win32::Foundation::{WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE, FindCloseChangeNotification,
    FindFirstChangeNotificationW, FindNextChangeNotification,
};
use windows::Win32::System::Threading::{INFINITE, WaitForSingleObject};
use windows_core::HSTRING;

use crate::window::{Event, UiProxy};

/// Mengirim `Event::FileChanged` setiap kali isi folder `dir` berubah.
/// Thread berjalan sampai proses berakhir; pemanggil melakukan debounce.
pub fn watch_dir(dir: &Path, proxy: UiProxy) -> bool {
    let path = HSTRING::from(dir.as_os_str());
    // SAFETY: path hidup selama pemanggilan; handle dipindahkan ke thread
    // pemantau sebagai angka dan hanya dipakai di sana.
    let handle = match unsafe {
        FindFirstChangeNotificationW(
            &path,
            false,
            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_SIZE,
        )
    } {
        Ok(h) if !h.is_invalid() => h.0 as isize,
        _ => return false,
    };
    std::thread::Builder::new()
        .name("notchi-config-watch".into())
        .spawn(move || {
            let h = windows::Win32::Foundation::HANDLE(handle as *mut _);
            loop {
                // SAFETY: handle notifikasi valid sampai ditutup di bawah.
                let r = unsafe { WaitForSingleObject(h, INFINITE) };
                if r != WAIT_OBJECT_0 {
                    break;
                }
                proxy.post(Event::FileChanged);
                // SAFETY: handle valid.
                if unsafe { FindNextChangeNotification(h) }.is_err() {
                    break;
                }
            }
            // SAFETY: handle dimiliki thread ini.
            unsafe {
                let _ = FindCloseChangeNotification(h);
            }
        })
        .is_ok()
}
