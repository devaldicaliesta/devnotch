use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, SetDefaultDllDirectories};
use windows::Win32::System::Memory::{GetProcessHeap, HEAP_FLAGS, HeapCompact};
use windows::Win32::System::SystemServices::{
    PROCESS_MITIGATION_EXTENSION_POINT_DISABLE_POLICY, PROCESS_MITIGATION_IMAGE_LOAD_POLICY,
    PROCESS_MITIGATION_STRICT_HANDLE_CHECK_POLICY,
};
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, ProcessExtensionPointDisablePolicy, ProcessImageLoadPolicy,
    ProcessStrictHandleCheckPolicy, SetProcessMitigationPolicy, SetProcessWorkingSetSize,
};
use windows::Win32::System::WinRT::{RO_INIT_SINGLETHREADED, RoInitialize};
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};
use windows_core::HSTRING;

#[derive(Debug, Default)]
pub struct Hardening {
    pub dll_directories: bool,
    pub image_load: bool,
    pub extension_points: bool,
    pub strict_handles: bool,
}

pub fn init_process() -> Hardening {
    let mut h = Hardening::default();
    unsafe {
        h.dll_directories = SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32).is_ok();

        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let mut image = PROCESS_MITIGATION_IMAGE_LOAD_POLICY::default();
        image.Anonymous.Flags = 0b11;
        h.image_load = set_policy(ProcessImageLoadPolicy, &image);

        let mut ext = PROCESS_MITIGATION_EXTENSION_POINT_DISABLE_POLICY::default();
        ext.Anonymous.Flags = 0b1;
        h.extension_points = set_policy(ProcessExtensionPointDisablePolicy, &ext);

        let mut strict = PROCESS_MITIGATION_STRICT_HANDLE_CHECK_POLICY::default();
        strict.Anonymous.Flags = 0b11;
        h.strict_handles = set_policy(ProcessStrictHandleCheckPolicy, &strict);

        let _ = RoInitialize(RO_INIT_SINGLETHREADED);
    }
    h
}

unsafe fn set_policy<T>(policy: windows::Win32::System::Threading::PROCESS_MITIGATION_POLICY, value: &T) -> bool {
    unsafe { SetProcessMitigationPolicy(policy, value as *const T as *const _, size_of::<T>()).is_ok() }
}

#[derive(Debug)]
pub struct InstanceLock(HANDLE);

impl InstanceLock {
    pub fn acquire(name: &str) -> Option<InstanceLock> {
        let name = HSTRING::from(format!("Local\\{name}"));
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
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn attach_parent_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

pub fn trim_memory() {
    unsafe {
        if let Ok(heap) = GetProcessHeap() {
            HeapCompact(heap, HEAP_FLAGS(0));
        }
        let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}
