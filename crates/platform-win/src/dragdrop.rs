//! Seret-lepas berkas (OLE) untuk rak berkas: menerima berkas yang dilepas di Island dan
//! menyeret berkas dari Island ke aplikasi lain.
//!
//! Notchi tidak pernah menyalin atau memindahkan berkas sendiri. Saat menerima, efek yang
//! dilaporkan ke sumber adalah salin/tautan (bukan pindah) sehingga sumber tidak menghapus
//! aslinya. Saat menyeret keluar, tujuanlah yang menyalin atau memindahkan, seperti Explorer.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{HWND, POINTL};
use windows::Win32::System::Com::{DVASPECT_CONTENT, FORMATETC, IDataObject, TYMED_HGLOBAL};
use windows::Win32::System::Ole::{
    CF_HDROP, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE, DROPEFFECT_NONE, IDropTarget,
    IDropTarget_Impl, OleInitialize, RegisterDragDrop, ReleaseStgMedium,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    BHID_DataObject, DragQueryFileW, HDROP, ILFree, SHCreateDataObject, SHCreateShellItemArrayFromIDLists, SHDoDragDrop,
    SHParseDisplayName,
};
use windows_core::{HSTRING, Ref, Result, implement};

use crate::window::{Event, UiProxy};

/// Sedang menyeret keluar dari Island: lepasan kembali ke Island sendiri diabaikan.
static DRAGGING_OUT: AtomicBool = AtomicBool::new(false);

fn hdrop_format() -> FORMATETC {
    FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    }
}

fn has_files(data: &IDataObject) -> bool {
    unsafe { data.QueryGetData(&hdrop_format()) }.is_ok()
}

fn files_of(data: &IDataObject) -> Vec<PathBuf> {
    let mut out = Vec::new();
    unsafe {
        let Ok(mut medium) = data.GetData(&hdrop_format()) else { return out };
        let hdrop = HDROP(medium.u.hGlobal.0);
        let n = DragQueryFileW(hdrop, u32::MAX, None);
        for i in 0..n.min(256) {
            let len = DragQueryFileW(hdrop, i, None) as usize;
            if len == 0 || len > 32_767 {
                continue;
            }
            let mut buf = vec![0u16; len + 1];
            let got = DragQueryFileW(hdrop, i, Some(&mut buf)) as usize;
            out.push(PathBuf::from(String::from_utf16_lossy(&buf[..got.min(len)])));
        }
        ReleaseStgMedium(&mut medium);
    }
    out
}

/// Salin bila sumber mengizinkan, selain itu tautan; tidak pernah pindah.
fn accept(allowed: DROPEFFECT) -> DROPEFFECT {
    if allowed.0 & DROPEFFECT_COPY.0 != 0 {
        DROPEFFECT_COPY
    } else if allowed.0 & DROPEFFECT_LINK.0 != 0 {
        DROPEFFECT_LINK
    } else {
        DROPEFFECT_NONE
    }
}

#[implement(IDropTarget)]
struct ShelfTarget {
    proxy: UiProxy,
    accepting: AtomicBool,
}

impl IDropTarget_Impl for ShelfTarget_Impl {
    fn DragEnter(&self, data: Ref<IDataObject>, _: MODIFIERKEYS_FLAGS, _: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        let ok = !DRAGGING_OUT.load(Ordering::SeqCst) && data.as_ref().is_some_and(has_files);
        self.accepting.store(ok, Ordering::SeqCst);
        unsafe {
            if !effect.is_null() {
                *effect = if ok { accept(*effect) } else { DROPEFFECT_NONE };
            }
        }
        if ok {
            self.proxy.post(Event::ShelfDrag(true));
        }
        Ok(())
    }

    fn DragOver(&self, _: MODIFIERKEYS_FLAGS, _: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        let ok = self.accepting.load(Ordering::SeqCst);
        unsafe {
            if !effect.is_null() {
                *effect = if ok { accept(*effect) } else { DROPEFFECT_NONE };
            }
        }
        Ok(())
    }

    fn DragLeave(&self) -> Result<()> {
        if self.accepting.swap(false, Ordering::SeqCst) {
            self.proxy.post(Event::ShelfDrag(false));
        }
        Ok(())
    }

    fn Drop(&self, data: Ref<IDataObject>, _: MODIFIERKEYS_FLAGS, _: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        let ok = self.accepting.swap(false, Ordering::SeqCst);
        let files = if ok { data.as_ref().map(files_of).unwrap_or_default() } else { Vec::new() };
        unsafe {
            if !effect.is_null() {
                *effect = if ok && !files.is_empty() { accept(*effect) } else { DROPEFFECT_NONE };
            }
        }
        if ok {
            if !files.is_empty() {
                self.proxy.post(Event::ShelfDrop(files));
            }
            self.proxy.post(Event::ShelfDrag(false));
        }
        Ok(())
    }
}

/// Daftarkan jendela sebagai tujuan seret-lepas berkas. Harus di thread UI (STA).
pub(crate) fn register(hwnd: HWND, proxy: UiProxy) -> bool {
    unsafe {
        let _ = OleInitialize(None);
        let target: IDropTarget = ShelfTarget { proxy, accepting: AtomicBool::new(false) }.into();
        RegisterDragDrop(hwnd, &target).is_ok()
    }
}

struct Pidls(Vec<*mut ITEMIDLIST>);

impl Drop for Pidls {
    fn drop(&mut self) {
        for p in self.0.drain(..) {
            unsafe { ILFree(Some(p)) };
        }
    }
}

/// Objek data seperti yang dibuat Explorer saat menyeret berkas: lewat `IShellItemArray`
/// dan `BHID_DataObject`, sehingga memuat `CF_HDROP` (dibutuhkan Chrome, WhatsApp, email,
/// dan aplikasi Electron/WebView2), daftar ID shell, nama berkas, dan dukungan gambar seret.
/// `SHCreateDataObject` saja hanya memuat daftar ID shell, yang diabaikan banyak aplikasi.
fn data_object(list: &[*const ITEMIDLIST]) -> Result<IDataObject> {
    unsafe {
        match SHCreateShellItemArrayFromIDLists(list).and_then(|a| a.BindToHandler::<_, IDataObject>(None, &BHID_DataObject)) {
            Ok(d) => Ok(d),
            Err(_) => SHCreateDataObject(None, Some(list), None),
        }
    }
}

/// Seret berkas keluar dari Island. Memblokir sampai tombol mouse dilepas (loop modal OLE).
/// Mengembalikan efek yang dipilih tujuan, atau `DROPEFFECT_NONE` bila dibatalkan.
pub(crate) fn drag_out(hwnd: HWND, paths: &[PathBuf]) -> Result<DROPEFFECT> {
    let mut pidls = Pidls(Vec::new());
    for p in paths {
        let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
        let name = HSTRING::from(p.as_os_str());
        if unsafe { SHParseDisplayName(&name, None, &mut pidl, 0, None) }.is_ok() && !pidl.is_null() {
            pidls.0.push(pidl);
        }
    }
    if pidls.0.is_empty() {
        return Ok(DROPEFFECT_NONE);
    }
    let list: Vec<*const ITEMIDLIST> = pidls.0.iter().map(|p| *p as *const _).collect();
    let data = data_object(&list)?;
    DRAGGING_OUT.store(true, Ordering::SeqCst);
    let allowed = DROPEFFECT(DROPEFFECT_COPY.0 | DROPEFFECT_MOVE.0 | DROPEFFECT_LINK.0);
    let r = unsafe { SHDoDragDrop(Some(hwnd), &data, None, allowed) };
    DRAGGING_OUT.store(false, Ordering::SeqCst);
    r
}
