use std::path::Path;

use windows::Win32::Foundation::{WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE, FindCloseChangeNotification,
    FindFirstChangeNotificationW, FindNextChangeNotification,
};
use windows::Win32::System::Threading::{INFINITE, WaitForSingleObject};
use windows_core::HSTRING;

use crate::window::{Event, UiProxy};

/// Pantau perubahan isi folder; `event` dibuat ulang untuk setiap perubahan.
pub fn watch_dir(dir: &Path, proxy: UiProxy, name: &str, event: fn() -> Event) -> bool {
    let path = HSTRING::from(dir.as_os_str());
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
        .name(name.into())
        .spawn(move || {
            let h = windows::Win32::Foundation::HANDLE(handle as *mut _);
            loop {
                let r = unsafe { WaitForSingleObject(h, INFINITE) };
                if r != WAIT_OBJECT_0 {
                    break;
                }
                proxy.post(event());
                if unsafe { FindNextChangeNotification(h) }.is_err() {
                    break;
                }
            }
            unsafe {
                let _ = FindCloseChangeNotification(h);
            }
        })
        .is_ok()
}
