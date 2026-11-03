use std::ffi::c_void;

use notchi_core::privacy::{Privacy, in_use};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, REG_NOTIFY_CHANGE_LAST_SET, REG_NOTIFY_CHANGE_NAME, RRF_RT_REG_QWORD, RegCloseKey,
    RegEnumKeyExW, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW,
};
use windows::core::{PCWSTR, PWSTR, w};
use windows_core::HSTRING;

use crate::window::{Event, UiProxy};

const STORE: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore");
const MAX_APPS: u32 = 512;

struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

fn open(parent: HKEY, sub: &HSTRING) -> Option<Key> {
    let mut key = HKEY::default();
    let r = unsafe { RegOpenKeyExW(parent, sub, None, KEY_READ, &mut key) };
    (r == ERROR_SUCCESS).then_some(Key(key))
}

fn qword(key: HKEY, name: PCWSTR) -> u64 {
    let mut value = 0u64;
    let mut size = size_of::<u64>() as u32;
    let r = unsafe {
        RegGetValueW(key, PCWSTR::null(), name, RRF_RT_REG_QWORD, None, Some(&mut value as *mut u64 as *mut c_void), Some(&mut size))
    };
    if r == ERROR_SUCCESS { value } else { 0 }
}

fn subkeys(key: HKEY) -> Vec<HSTRING> {
    let mut out = Vec::new();
    let mut buf = [0u16; 512];
    for i in 0..MAX_APPS {
        let mut len = buf.len() as u32;
        let r = unsafe { RegEnumKeyExW(key, i, Some(PWSTR(buf.as_mut_ptr())), &mut len, None, None, None, None) };
        if r != ERROR_SUCCESS {
            break;
        }
        out.push(HSTRING::from_wide(&buf[..(len as usize).min(buf.len())]));
    }
    out
}

fn any_in_use(key: HKEY, nested: bool) -> bool {
    subkeys(key).iter().any(|name| {
        let Some(app) = open(key, name) else { return false };
        if nested && name.to_string().eq_ignore_ascii_case("NonPackaged") {
            return any_in_use(app.0, false);
        }
        in_use(qword(app.0, w!("LastUsedTimeStart")), qword(app.0, w!("LastUsedTimeStop")))
    })
}

fn capability(root: HKEY, name: &str) -> bool {
    open(root, &HSTRING::from(name)).is_some_and(|k| any_in_use(k.0, true))
}

fn read(root: HKEY) -> Privacy {
    Privacy { mic: capability(root, "microphone"), camera: capability(root, "webcam") }
}

pub fn watch(proxy: UiProxy) -> bool {
    let mut root = HKEY::default();
    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, STORE, None, KEY_READ, &mut root) } != ERROR_SUCCESS {
        return false;
    }
    let raw = root.0 as isize;
    let spawned = std::thread::Builder::new().name("notchi-privacy".into()).stack_size(256 * 1024).spawn(move || {
        let root = Key(HKEY(raw as *mut c_void));
        let mut last = Privacy::default();
        loop {
            let now = read(root.0);
            if now != last {
                proxy.post(Event::Privacy(now));
                last = now;
            }
            let r = unsafe { RegNotifyChangeKeyValue(root.0, true, REG_NOTIFY_CHANGE_LAST_SET | REG_NOTIFY_CHANGE_NAME, None, false) };
            if r != ERROR_SUCCESS {
                break;
            }
        }
    });
    if spawned.is_err() {
        unsafe {
            let _ = RegCloseKey(HKEY(raw as *mut c_void));
        }
        return false;
    }
    true
}
