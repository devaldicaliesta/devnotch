//! Inisialisasi proses, pengerasan (PRD §7.2), dan instans tunggal.

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, SetDefaultDllDirectories};
use windows::Win32::System::SystemServices::{
    PROCESS_MITIGATION_EXTENSION_POINT_DISABLE_POLICY, PROCESS_MITIGATION_IMAGE_LOAD_POLICY,
    PROCESS_MITIGATION_STRICT_HANDLE_CHECK_POLICY,
};
use windows::Win32::System::Threading::{
    CreateMutexW, ProcessExtensionPointDisablePolicy, ProcessImageLoadPolicy,
    ProcessStrictHandleCheckPolicy, SetProcessMitigationPolicy,
};
use windows::Win32::System::WinRT::{RO_INIT_SINGLETHREADED, RoInitialize};
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};
use windows_core::HSTRING;

/// Hasil penerapan pengerasan; kegagalan tidak fatal, hanya dicatat.
#[derive(Debug, Default)]
pub struct Hardening {
    pub dll_directories: bool,
    pub image_load: bool,
    pub extension_points: bool,
    pub strict_handles: bool,
}

/// Dipanggil sekali di awal `main`, sebelum jendela apa pun dibuat.
pub fn init_process() -> Hardening {
    let mut h = Hardening::default();
    // SAFETY: fungsi-fungsi ini hanya mengubah kebijakan proses saat ini dan
    // menerima struktur bernilai yang hidup selama pemanggilan.
    unsafe {
        // Cegah DLL hijacking: hanya muat DLL dari System32.
        h.dll_directories = SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32).is_ok();

        // Manifest sudah meminta PerMonitorV2; panggilan ini cadangan bila
        // biner dijalankan tanpa manifest.
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        // Larang gambar dari jaringan dan berlabel integritas rendah.
        let mut image = PROCESS_MITIGATION_IMAGE_LOAD_POLICY::default();
        image.Anonymous.Flags = 0b11; // NoRemoteImages | NoLowMandatoryLabelImages
        h.image_load = set_policy(ProcessImageLoadPolicy, &image);

        // Nonaktifkan extension point lama (AppInit DLL dan sejenisnya).
        let mut ext = PROCESS_MITIGATION_EXTENSION_POINT_DISABLE_POLICY::default();
        ext.Anonymous.Flags = 0b1; // DisableExtensionPoints
        h.extension_points = set_policy(ProcessExtensionPointDisablePolicy, &ext);

        // Pemeriksaan handle ketat.
        let mut strict = PROCESS_MITIGATION_STRICT_HANDLE_CHECK_POLICY::default();
        strict.Anonymous.Flags = 0b11; // RaiseExceptionOnInvalidHandleReference | HandleExceptionsPermanentlyEnabled
        h.strict_handles = set_policy(ProcessStrictHandleCheckPolicy, &strict);

        // Thread UI adalah STA; DispatcherQueue dibuat di atasnya.
        let _ = RoInitialize(RO_INIT_SINGLETHREADED);
    }
    h
}

/// # Safety
/// `policy` harus cocok dengan tipe `T` sesuai dokumentasi
/// `SetProcessMitigationPolicy`.
unsafe fn set_policy<T>(policy: windows::Win32::System::Threading::PROCESS_MITIGATION_POLICY, value: &T) -> bool {
    // SAFETY: `value` adalah referensi valid berukuran `size_of::<T>()`.
    unsafe { SetProcessMitigationPolicy(policy, value as *const T as *const _, size_of::<T>()).is_ok() }
}

/// Kunci instans tunggal; dilepas saat di-drop.
#[derive(Debug)]
pub struct InstanceLock(HANDLE);

impl InstanceLock {
    /// `None` bila instans lain sudah memegang kunci bernama sama.
    pub fn acquire(name: &str) -> Option<InstanceLock> {
        let name = HSTRING::from(format!("Local\\{name}"));
        // SAFETY: nama adalah string UTF-16 yang hidup selama pemanggilan.
        unsafe {
            let handle = CreateMutexW(None, true, &name).ok()?;
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let _ = CloseHandle(handle);
                return None;
            }
            Some(InstanceLock(handle))
        }
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // SAFETY: handle dimiliki struct ini dan hanya ditutup sekali.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Menyambung ke konsol induk agar opsi CLI dapat mencetak keluaran,
/// karena biner rilis memakai subsistem `windows`.
pub fn attach_parent_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    // SAFETY: tanpa argumen pointer; gagal tanpa efek bila tidak ada konsol.
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
