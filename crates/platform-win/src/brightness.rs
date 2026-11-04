//! Kecerahan layar: lewat WMI untuk layar laptop, atau DDC/CI untuk monitor eksternal.
//!
//! Kedua jalur bisa lambat (DDC/CI butuh puluhan milidetik per panggilan), jadi semuanya
//! berjalan di thread sendiri. Langkah scroll yang menumpuk digabung menjadi satu perubahan.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitors, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, PHYSICAL_MONITOR, SetMonitorBrightness,
};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTOPRIMARY, MonitorFromPoint};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx};
use windows::Win32::System::Variant::{VARIANT, VT_BSTR, VT_I2, VT_I4, VT_UI1, VT_UI4, VariantClear};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
    WBEM_GENERIC_FLAG_TYPE, WBEM_INFINITE, WbemLocator,
};
use windows_core::{BSTR, Result, w};

use crate::window::{Event, UiProxy};

const DDC_CACHE: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct Brightness {
    tx: Sender<i32>,
}

impl Brightness {
    pub fn start(proxy: UiProxy) -> Option<Brightness> {
        let (tx, rx) = channel();
        std::thread::Builder::new()
            .name("notchi-brightness".into())
            .stack_size(256 * 1024)
            .spawn(move || worker(rx, proxy))
            .ok()?;
        Some(Brightness { tx })
    }

    /// Ubah kecerahan sebanyak `percent` poin (negatif = lebih redup).
    pub fn step(&self, percent: i32) {
        let _ = self.tx.send(percent);
    }
}

enum Backend {
    Wmi(IWbemServices),
    Ddc { level: Option<(u8, Instant)> },
    None,
}

fn worker(rx: Receiver<i32>, proxy: UiProxy) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let mut backend: Option<Backend> = None;
    while let Ok(first) = rx.recv() {
        let mut delta = first;
        while let Ok(more) = rx.try_recv() {
            delta += more;
        }
        let b = backend.get_or_insert_with(open);
        let level = match b {
            Backend::Wmi(svc) => wmi_step(svc, delta).ok(),
            Backend::Ddc { level } => ddc_step(level, delta),
            Backend::None => None,
        };
        proxy.post(Event::Brightness(level));
    }
}

fn open() -> Backend {
    if let Ok(svc) = wmi_open()
        && wmi_get(&svc).is_ok()
    {
        return Backend::Wmi(svc);
    }
    if with_monitors(|_| Some(())).is_some() {
        return Backend::Ddc { level: None };
    }
    Backend::None
}

fn apply(current: u8, delta: i32) -> u8 {
    (current as i32 + delta).clamp(0, 100) as u8
}

fn wmi_open() -> Result<IWbemServices> {
    unsafe {
        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
        locator.ConnectServer(&BSTR::from("ROOT\\WMI"), &BSTR::new(), &BSTR::new(), &BSTR::new(), 0, &BSTR::new(), None)
    }
}

fn wmi_first(svc: &IWbemServices, query: &str) -> Result<IWbemClassObject> {
    unsafe {
        let flags = WBEM_GENERIC_FLAG_TYPE(WBEM_FLAG_FORWARD_ONLY.0 | WBEM_FLAG_RETURN_IMMEDIATELY.0);
        let en = svc.ExecQuery(&BSTR::from("WQL"), &BSTR::from(query), flags, None)?;
        let mut objs = [None];
        let mut got = 0u32;
        en.Next(WBEM_INFINITE, &mut objs, &mut got).ok()?;
        objs[0].take().ok_or_else(fail)
    }
}

fn fail() -> windows_core::Error {
    windows_core::Error::from_hresult(windows::Win32::Foundation::E_FAIL)
}

/// Baca properti objek WMI sebagai bilangan bulat atau teks.
fn prop(obj: &IWbemClassObject, name: windows_core::PCWSTR) -> Result<(Option<i64>, Option<String>)> {
    let mut v = VARIANT::default();
    unsafe {
        obj.Get(name, 0, &mut v, None, None)?;
        let inner = &v.Anonymous.Anonymous;
        let num = match inner.vt {
            VT_UI1 => Some(inner.Anonymous.bVal as i64),
            VT_I2 => Some(inner.Anonymous.iVal as i64),
            VT_I4 => Some(inner.Anonymous.lVal as i64),
            VT_UI4 => Some(inner.Anonymous.ulVal as i64),
            _ => None,
        };
        let text = (inner.vt == VT_BSTR).then(|| inner.Anonymous.bstrVal.to_string());
        let _ = VariantClear(&mut v);
        Ok((num, text))
    }
}

fn int_variant(vt: windows::Win32::System::Variant::VARENUM, value: i32) -> VARIANT {
    let mut v = VARIANT::default();
    unsafe {
        let inner = &mut *v.Anonymous.Anonymous;
        inner.vt = vt;
        if vt == VT_UI1 {
            inner.Anonymous.bVal = value.clamp(0, 255) as u8;
        } else {
            inner.Anonymous.lVal = value;
        }
    }
    v
}

fn wmi_get(svc: &IWbemServices) -> Result<u8> {
    let obj = wmi_first(svc, "SELECT CurrentBrightness FROM WmiMonitorBrightness WHERE Active=TRUE")?;
    let (level, _) = prop(&obj, w!("CurrentBrightness"))?;
    Ok(level.ok_or_else(fail)?.clamp(0, 100) as u8)
}

fn wmi_step(svc: &IWbemServices, delta: i32) -> Result<u8> {
    let next = apply(wmi_get(svc)?, delta);
    let methods = wmi_first(svc, "SELECT * FROM WmiMonitorBrightnessMethods WHERE Active=TRUE")?;
    let path = BSTR::from(prop(&methods, w!("__PATH"))?.1.ok_or_else(fail)?);
    unsafe {
        let mut class = None;
        svc.GetObject(&BSTR::from("WmiMonitorBrightnessMethods"), WBEM_GENERIC_FLAG_TYPE(0), None, Some(&mut class), None)?;
        let class: IWbemClassObject = class.ok_or_else(fail)?;
        let mut sig = None;
        class.GetMethod(w!("WmiSetBrightness"), 0, &mut sig, std::ptr::null_mut())?;
        let sig: IWbemClassObject = sig.ok_or_else(fail)?;
        let input = sig.SpawnInstance(0)?;
        input.Put(w!("Timeout"), 0, &int_variant(VT_I4, 0), 0)?;
        input.Put(w!("Brightness"), 0, &int_variant(VT_UI1, next as i32), 0)?;
        svc.ExecMethod(&path, &BSTR::from("WmiSetBrightness"), WBEM_GENERIC_FLAG_TYPE(0), None, &input, None, None)?;
    }
    Ok(next)
}

/// Jalankan `f` dengan semua monitor fisik di monitor utama.
fn with_monitors<T>(f: impl FnOnce(&[PHYSICAL_MONITOR]) -> Option<T>) -> Option<T> {
    unsafe {
        let hmon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut n = 0u32;
        GetNumberOfPhysicalMonitorsFromHMONITOR(hmon, &mut n).ok()?;
        if n == 0 {
            return None;
        }
        let mut mons = vec![PHYSICAL_MONITOR::default(); n as usize];
        GetPhysicalMonitorsFromHMONITOR(hmon, &mut mons).ok()?;
        let out = f(&mons);
        let _ = DestroyPhysicalMonitors(&mons);
        out
    }
}

fn ddc_read(m: &PHYSICAL_MONITOR) -> Option<(u32, u32, u32)> {
    let (mut lo, mut cur, mut hi) = (0u32, 0u32, 0u32);
    let ok = unsafe { GetMonitorBrightness(m.hPhysicalMonitor, &mut lo, &mut cur, &mut hi) } != 0;
    (ok && hi > lo).then_some((lo, cur, hi))
}

fn ddc_step(cache: &mut Option<(u8, Instant)>, delta: i32) -> Option<u8> {
    with_monitors(|mons| {
        let mut result = None;
        for m in mons {
            let Some((lo, cur, hi)) = ddc_read(m) else { continue };
            let pct = match cache {
                Some((p, at)) if at.elapsed() < DDC_CACHE => *p,
                _ => (((cur - lo) * 100) as f32 / (hi - lo) as f32).round() as u8,
            };
            let next = apply(pct, delta);
            let raw = lo + ((hi - lo) as f32 * next as f32 / 100.0).round() as u32;
            if unsafe { SetMonitorBrightness(m.hPhysicalMonitor, raw) } != 0 {
                result = Some(next);
            }
        }
        if let Some(p) = result {
            *cache = Some((p, Instant::now()));
        }
        result
    })
}
