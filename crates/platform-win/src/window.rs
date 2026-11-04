use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use notchi_core::geom::{Point, scale_for_dpi};
use notchi_core::color::Color;
use notchi_core::hotkey::Hotkey;
use notchi_core::lyrics::Lyrics;
use notchi_core::media::{Artwork, MediaSnapshot};
use notchi_core::privacy::Privacy;
use windows::Win32::Foundation::{HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_CLOAKED, DWMWA_EXCLUDED_FROM_PEEK, DWMWA_EXTENDED_FRAME_BOUNDS,
    DWMWA_TRANSITIONS_FORCEDISABLED, DWMWA_USE_HOSTBACKDROPBRUSH, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
    DwmGetWindowAttribute, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, CreateRectRgn, SetWindowRgn, DIB_RGB_COLORS, DeleteObject,
    GetMonitorInfoW, HGDIOBJ, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{HPOWERNOTIFY, RegisterPowerSettingNotification, UnregisterPowerSettingNotification};
use windows::Win32::System::SystemServices::{GUID_ACDC_POWER_SOURCE, GUID_BATTERY_PERCENTAGE_REMAINING};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey, ReleaseCapture,
    SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, UnregisterHotKey,
};
use windows::Win32::UI::Shell::{
    ABM_NEW, ABM_REMOVE, ABN_FULLSCREENAPP, APPBARDATA, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD,
    NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, SHAppBarMessage, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu,
    DestroyWindow, DispatchMessageW, EVENT_SYSTEM_CAPTURESTART, EVENT_SYSTEM_FOREGROUND, FindWindowW,
    GWLP_USERDATA, GetCursorPos, GetForegroundWindow, GetMessageW, GetWindowLongPtrW, HICON, HMENU, HWND_TOPMOST,
    ICONINFO, IDC_ARROW, IsWindowVisible, KillTimer, LoadCursorW, MA_NOACTIVATE, MB_ICONINFORMATION, MB_OK,
    MENU_ITEM_FLAGS, MF_CHECKED, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, MessageBoxW, PBT_APMPOWERSTATUSCHANGE,
    PBT_POWERSETTINGCHANGE, PostMessageW, PostQuitMessage, RegisterClassExW, RegisterWindowMessageW,
    SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetForegroundWindow, SetTimer, SetWindowLongPtrW,
    SetWindowPos, ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
    ICON_BIG, ICON_SMALL, IsIconic, SW_RESTORE, SW_SHOW, SWP_NOZORDER, SendMessageW, WM_CLOSE, WM_KEYDOWN, WM_SETICON,
    WS_CAPTION, WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
    GetClientRect, MINMAXINFO, WM_GETMINMAXINFO, WM_SIZE, SIZE_MINIMIZED,
    WINDOW_EX_STYLE, WINDOW_STYLE,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP, WM_CONTEXTMENU, WM_DESTROY, WM_DISPLAYCHANGE,
    WM_DPICHANGED, WM_HOTKEY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONUP, WM_MOUSEACTIVATE,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NULL, WM_POWERBROADCAST, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETTINGCHANGE,
    WM_TIMER, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    DeregisterShellHookWindow, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MOVESIZEEND, GW_HWNDNEXT, GW_OWNER, GWL_EXSTYLE,
    GetClassNameW, GetTopWindow, GetWindow, GetWindowLongW, GetWindowRect, HSHELL_GETMINRECT, HSHELL_MONITORCHANGED,
    HSHELL_WINDOWACTIVATED, HSHELL_WINDOWCREATED, HSHELL_WINDOWDESTROYED, RegisterShellHookWindow, WS_EX_TRANSPARENT,
};
use windows::core::{BOOL, PCWSTR, w};
use windows_core::{HSTRING, Result};

pub const CLASS_NAME: &str = "Notchi.Island";
pub const PANEL_CLASS: &str = "Notchi.Panel";

const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_APP_WAKE: u32 = WM_APP + 1;
const WM_APP_TRAY: u32 = WM_APP + 2;
const WM_APP_APPBAR: u32 = WM_APP + 3;
const WM_APP_FOREGROUND: u32 = WM_APP + 4;
const WM_APP_CAPTURE: u32 = WM_APP + 5;
const WM_APP_SECOND_INSTANCE: u32 = WM_APP + 6;
const WM_APP_ARRANGE: u32 = WM_APP + 7;
const MAX_Z_WALK: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Click,
    Menu,
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseMove(Point),
    MouseLeave,
    MouseDown(Point, MouseButton),
    MouseUp(Point, MouseButton),
    Wheel(i32),
    /// Scroll sambil menahan Shift.
    ShiftWheel(i32),
    Timer(usize),
    Hotkey(i32),
    DpiChanged(u32),
    Resized,
    DisplayChanged,
    SettingsChanged,
    PowerChanged,
    FullscreenHint(bool),
    WindowsArranged,
    ForegroundChanged,
    ForeignMouseDown,
    Tray(TrayEvent),
    TaskbarCreated,
    SecondInstance,
    OpenSettings,
    Media(Option<MediaSnapshot>),
    MediaArt { track_id: u64, art: Option<Artwork> },
    FileChanged,
    DownloadsChanged,
    Bluetooth(notchi_core::bluetooth::Update),
    BluetoothRemoved(String),
    BluetoothReady,
    Brightness(Option<u8>),
    User(u64),
    Close,
    Key(u32),
    Panel(Box<Event>),
    Lyrics { track_id: u64, lyrics: Option<Lyrics>, failed: bool },
    Privacy(Privacy),
}

type Handler = Box<dyn FnMut(&Window, Event)>;

struct WindowState {
    handler: RefCell<Option<Handler>>,
    queue: RefCell<VecDeque<Event>>,
    remote: Arc<Mutex<VecDeque<Event>>>,
    dispatching: Cell<bool>,
    tracking_mouse: Cell<bool>,
    taskbar_created: u32,
    shell_hook: u32,
    panel: bool,
}

impl WindowState {
    fn dispatch(&self, window: &Window, ev: Event) {
        self.queue.borrow_mut().push_back(ev);
        if self.dispatching.replace(true) {
            return;
        }
        loop {
            let next = self.queue.borrow_mut().pop_front();
            let Some(ev) = next else { break };
            if let Some(h) = self.handler.borrow_mut().as_mut() {
                h(window, ev);
            }
        }
        self.dispatching.set(false);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    hwnd: HWND,
}

pub struct OwnedWindow {
    window: Window,
    state: *const WindowState,
    power: Vec<HPOWERNOTIFY>,
    appbar: bool,
    shell: bool,
    arrange: Vec<HWINEVENTHOOK>,
    icon: Option<HICON>,
}

impl std::fmt::Debug for OwnedWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedWindow").field("window", &self.window).finish()
    }
}

#[derive(Clone)]
pub struct UiProxy {
    hwnd: isize,
    remote: Arc<Mutex<VecDeque<Event>>>,
}

impl std::fmt::Debug for UiProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiProxy").field("hwnd", &self.hwnd).finish()
    }
}

impl UiProxy {
    pub fn post(&self, ev: Event) {
        if let Ok(mut q) = self.remote.lock() {
            if q.len() >= 256 {
                q.pop_front();
            }
            q.push_back(ev);
        }
        unsafe {
            let _ = PostMessageW(Some(HWND(self.hwnd as *mut c_void)), WM_APP_WAKE, WPARAM(0), LPARAM(0));
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn mouse_point(hwnd: HWND, lparam: LPARAM) -> Point {
    let x = (lparam.0 & 0xFFFF) as u16 as i16 as f32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as u16 as i16 as f32;
    let scale = scale_for_dpi(unsafe { GetDpiForWindow(hwnd) });
    Point::new(x / scale, y / scale)
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const WindowState;
    if state.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    let state = unsafe { &*state };
    let window = Window { hwnd };
    let send = |ev: Event| state.dispatch(&window, ev);
    match msg {
        WM_MOUSEACTIVATE if !state.panel => return LRESULT(MA_NOACTIVATE as isize),
        WM_CLOSE if state.panel => {
            send(Event::Close);
            return LRESULT(0);
        }
        WM_KEYDOWN => send(Event::Key(wparam.0 as u32)),
        WM_SIZE if state.panel => {
            if wparam.0 as u32 != SIZE_MINIMIZED {
                send(Event::Resized);
            }
        }
        WM_GETMINMAXINFO if state.panel => {
            let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
            let scale = scale_for_dpi(dpi);
            let mut r = RECT {
                left: 0,
                top: 0,
                right: (PANEL_MIN_DP.0 * scale).ceil() as i32,
                bottom: (PANEL_MIN_DP.1 * scale).ceil() as i32,
            };
            let _ = unsafe { AdjustWindowRectExForDpi(&mut r, PANEL_STYLE, false, PANEL_EX, dpi) };
            let info = unsafe { &mut *(lparam.0 as *mut MINMAXINFO) };
            info.ptMinTrackSize = POINT { x: r.right - r.left, y: r.bottom - r.top };
            return LRESULT(0);
        }
        WM_MOUSEMOVE => {
            if !state.tracking_mouse.replace(true) {
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                unsafe {
                    let _ = TrackMouseEvent(&mut tme);
                }
            }
            send(Event::MouseMove(mouse_point(hwnd, lparam)));
        }
        WM_MOUSELEAVE => {
            state.tracking_mouse.set(false);
            send(Event::MouseLeave);
        }
        WM_LBUTTONDOWN => {
            unsafe {
                SetCapture(hwnd);
            }
            send(Event::MouseDown(mouse_point(hwnd, lparam), MouseButton::Left));
        }
        WM_LBUTTONUP => {
            unsafe {
                let _ = ReleaseCapture();
            }
            send(Event::MouseUp(mouse_point(hwnd, lparam), MouseButton::Left));
        }
        WM_RBUTTONDOWN => send(Event::MouseDown(mouse_point(hwnd, lparam), MouseButton::Right)),
        WM_RBUTTONUP => send(Event::MouseUp(mouse_point(hwnd, lparam), MouseButton::Right)),
        WM_MBUTTONUP => send(Event::MouseUp(mouse_point(hwnd, lparam), MouseButton::Middle)),
        WM_MOUSEWHEEL => {
            const MK_SHIFT: usize = 0x0004;
            let delta = ((wparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32;
            send(if wparam.0 & MK_SHIFT != 0 { Event::ShiftWheel(delta) } else { Event::Wheel(delta) })
        }
        WM_TIMER => send(Event::Timer(wparam.0)),
        WM_HOTKEY => send(Event::Hotkey(wparam.0 as i32)),
        WM_DPICHANGED => {
            if state.panel {
                let r = unsafe { *(lparam.0 as *const RECT) };
                unsafe {
                    let _ = SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, SWP_NOZORDER | SWP_NOACTIVATE);
                }
            }
            send(Event::DpiChanged((wparam.0 & 0xFFFF) as u32));
            return LRESULT(0);
        }
        WM_DISPLAYCHANGE => send(Event::DisplayChanged),
        WM_SETTINGCHANGE => send(Event::SettingsChanged),
        WM_POWERBROADCAST => {
            if wparam.0 as u32 == PBT_POWERSETTINGCHANGE || wparam.0 as u32 == PBT_APMPOWERSTATUSCHANGE {
                send(Event::PowerChanged);
            }
            return LRESULT(1);
        }
        WM_APP_WAKE => {
            let pending: Vec<Event> = match state.remote.lock() {
                Ok(mut q) => q.drain(..).collect(),
                Err(_) => Vec::new(),
            };
            for ev in pending {
                send(ev);
            }
        }
        WM_APP_TRAY => match lparam.0 as u32 {
            WM_LBUTTONUP => send(Event::Tray(TrayEvent::Click)),
            WM_RBUTTONUP | WM_CONTEXTMENU => send(Event::Tray(TrayEvent::Menu)),
            _ => {}
        },
        WM_APP_APPBAR => {
            if wparam.0 as u32 == ABN_FULLSCREENAPP {
                send(Event::FullscreenHint(lparam.0 != 0));
            }
        }
        WM_APP_FOREGROUND => send(Event::ForegroundChanged),
        WM_APP_CAPTURE => send(Event::ForeignMouseDown),
        WM_APP_ARRANGE => send(Event::WindowsArranged),
        WM_APP_SECOND_INSTANCE => send(if wparam.0 == 1 { Event::OpenSettings } else { Event::SecondInstance }),
        WM_DESTROY => {}
        m if m == state.taskbar_created && m != 0 => send(Event::TaskbarCreated),
        m if m == state.shell_hook && m != 0 => {
            let code = wparam.0 as u32 & 0x7FFF;
            if matches!(
                code,
                HSHELL_WINDOWCREATED | HSHELL_WINDOWDESTROYED | HSHELL_WINDOWACTIVATED | HSHELL_GETMINRECT | HSHELL_MONITORCHANGED
            ) {
                send(Event::WindowsArranged);
            }
        }
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn create_overlay(title: &str) -> Result<OwnedWindow> {
    let class = wide(CLASS_NAME);
    let title = wide(title);
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(instance),
            None,
        )?;

        let corner = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &corner as *const _ as *const c_void, 4);
        let no_border: u32 = 0xFFFF_FFFE;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &no_border as *const _ as *const c_void, 4);
        let yes = BOOL(1);
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_TRANSITIONS_FORCEDISABLED, &yes as *const _ as *const c_void, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_EXCLUDED_FROM_PEEK, &yes as *const _ as *const c_void, 4);
        // Izinkan brush host backdrop (blur acrylic dari isi di belakang jendela) untuk gaya Kaca.
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_USE_HOSTBACKDROPBRUSH, &yes as *const _ as *const c_void, 4);

        let state = Box::new(WindowState {
            handler: RefCell::new(None),
            queue: RefCell::new(VecDeque::new()),
            remote: Arc::new(Mutex::new(VecDeque::new())),
            dispatching: Cell::new(false),
            tracking_mouse: Cell::new(false),
            taskbar_created: RegisterWindowMessageW(w!("TaskbarCreated")),
            shell_hook: RegisterWindowMessageW(w!("SHELLHOOK")),
            panel: false,
        });
        let state = Box::into_raw(state);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        Ok(OwnedWindow { window: Window { hwnd }, state, power: Vec::new(), appbar: false, shell: false, arrange: Vec::new(), icon: None })
    }
}

/// Warna bilah judul dan mode gelap/terang bingkai jendela panel.
fn set_caption(hwnd: HWND, caption: Color, dark: bool) {
    let flag = BOOL(dark as i32);
    let tone = colorref(caption);
    unsafe {
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &flag as *const _ as *const c_void, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_CAPTION_COLOR, &tone as *const _ as *const c_void, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &tone as *const _ as *const c_void, 4);
    }
}

fn colorref(c: Color) -> u32 {
    c.r as u32 | (c.g as u32) << 8 | (c.b as u32) << 16
}

/// Ukuran klien terkecil jendela panel (dp) saat diubah ukurannya.
pub const PANEL_MIN_DP: (f32, f32) = (360.0, 320.0);
const PANEL_STYLE: WINDOW_STYLE = WINDOW_STYLE(
    WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0 | WS_THICKFRAME.0,
);
const PANEL_EX: WINDOW_EX_STYLE = WINDOW_EX_STYLE(WS_EX_APPWINDOW.0 | WS_EX_NOREDIRECTIONBITMAP.0);

pub fn create_panel(title: &str, client_dp: (f32, f32), caption: Color, dark: bool, icon: &IconImage) -> Result<OwnedWindow> {
    let class = wide(PANEL_CLASS);
    let title = wide(title);
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);
        let hmon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
        let _ = GetMonitorInfoW(hmon, &mut info);
        let (mut dx, mut dy) = (96u32, 96u32);
        let _ = GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
        let dpi = dx.max(96);
        let scale = scale_for_dpi(dpi);
        let (style, ex) = (PANEL_STYLE, PANEL_EX);
        let mut r = RECT { left: 0, top: 0, right: (client_dp.0 * scale).ceil() as i32, bottom: (client_dp.1 * scale).ceil() as i32 };
        let _ = AdjustWindowRectExForDpi(&mut r, style, false, ex, dpi);
        let work = info.rcWork;
        // Jangan lebih besar dari area kerja; isi yang tidak muat bisa di-scroll.
        let w = (r.right - r.left).min(work.right - work.left);
        let h = (r.bottom - r.top).min(work.bottom - work.top);
        let x = work.left + ((work.right - work.left) - w).max(0) / 2;
        let y = work.top + ((work.bottom - work.top) - h).max(0) / 2;
        let hwnd = CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, x, y, w, h, None, None, Some(instance), None)?;

        set_caption(hwnd, caption, dark);

        let state = Box::new(WindowState {
            handler: RefCell::new(None),
            queue: RefCell::new(VecDeque::new()),
            remote: Arc::new(Mutex::new(VecDeque::new())),
            dispatching: Cell::new(false),
            tracking_mouse: Cell::new(false),
            taskbar_created: 0,
            shell_hook: 0,
            panel: true,
        });
        let state = Box::into_raw(state);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        let hicon = create_hicon(icon).ok();
        if let Some(i) = hicon {
            for kind in [ICON_BIG, ICON_SMALL] {
                SendMessageW(hwnd, WM_SETICON, Some(WPARAM(kind as usize)), Some(LPARAM(i.0 as isize)));
            }
        }
        Ok(OwnedWindow { window: Window { hwnd }, state, power: Vec::new(), appbar: false, shell: false, arrange: Vec::new(), icon: hicon })
    }
}

thread_local! {
    static HOOK_TARGETS: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
    static OUTSIDE_HOOKS: RefCell<Vec<HWINEVENTHOOK>> = const { RefCell::new(Vec::new()) };
    static ARRANGE_TARGETS: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn arrange_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    ARRANGE_TARGETS.with(|t| {
        for &h in t.borrow().iter() {
            unsafe {
                let _ = PostMessageW(Some(HWND(h as *mut c_void)), WM_APP_ARRANGE, WPARAM(0), LPARAM(0));
            }
        }
    });
}

fn frame_bounds(hwnd: HWND) -> Option<RECT> {
    let mut r = RECT::default();
    let dwm = unsafe {
        DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut RECT as *mut c_void, size_of::<RECT>() as u32)
    };
    (dwm.is_ok() || unsafe { GetWindowRect(hwnd, &mut r) }.is_ok()).then_some(r)
}

fn is_app_window(hwnd: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() || GetWindow(hwnd, GW_OWNER).is_ok() {
            return false;
        }
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if ex & (WS_EX_TOOLWINDOW.0 | WS_EX_TRANSPARENT.0) != 0 {
            return false;
        }
        let mut cloaked = 0u32;
        let read = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut u32 as *mut c_void, size_of::<u32>() as u32);
        !(read.is_ok() && cloaked != 0)
    }
}

fn is_desktop(hwnd: HWND) -> bool {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) }.max(0) as usize;
    let class = String::from_utf16_lossy(&buf[..n.min(buf.len())]);
    ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"].iter().any(|c| class.eq_ignore_ascii_case(c))
}

extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    let msg = if event == EVENT_SYSTEM_FOREGROUND { WM_APP_FOREGROUND } else { WM_APP_CAPTURE };
    HOOK_TARGETS.with(|t| {
        for &h in t.borrow().iter() {
            unsafe {
                let _ = PostMessageW(Some(HWND(h as *mut c_void)), msg, WPARAM(0), LPARAM(0));
            }
        }
    });
}

impl OwnedWindow {
    pub fn window(&self) -> Window {
        self.window
    }

    fn state(&self) -> &WindowState {
        unsafe { &*self.state }
    }

    pub fn set_handler(&self, handler: impl FnMut(&Window, Event) + 'static) {
        *self.state().handler.borrow_mut() = Some(Box::new(handler));
    }

    pub fn proxy(&self) -> UiProxy {
        UiProxy { hwnd: self.window.hwnd.0 as isize, remote: self.state().remote.clone() }
    }

    pub fn register_power_notifications(&mut self) {
        for guid in [GUID_ACDC_POWER_SOURCE, GUID_BATTERY_PERCENTAGE_REMAINING] {
            if let Ok(h) = unsafe {
                RegisterPowerSettingNotification(HANDLE(self.window.hwnd.0), &guid, windows::Win32::UI::WindowsAndMessaging::DEVICE_NOTIFY_WINDOW_HANDLE)
            } {
                self.power.push(h);
            }
        }
    }

    pub fn register_fullscreen_notifications(&mut self) {
        if self.appbar {
            self.unregister_appbar();
        }
        let mut abd = self.appbar_data();
        self.appbar = unsafe { SHAppBarMessage(ABM_NEW, &mut abd) } != 0;
    }

    fn appbar_data(&self) -> APPBARDATA {
        APPBARDATA {
            cbSize: size_of::<APPBARDATA>() as u32,
            hWnd: self.window.hwnd,
            uCallbackMessage: WM_APP_APPBAR,
            ..Default::default()
        }
    }

    fn unregister_appbar(&mut self) {
        let mut abd = self.appbar_data();
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut abd);
        }
        self.appbar = false;
    }

    pub fn register_window_notifications(&mut self) {
        if !self.shell {
            self.shell = unsafe { RegisterShellHookWindow(self.window.hwnd) }.as_bool();
        }
        let target = self.window.hwnd.0 as isize;
        ARRANGE_TARGETS.with(|t| {
            let mut t = t.borrow_mut();
            if !t.contains(&target) {
                t.push(target);
            }
        });
        if !self.arrange.is_empty() {
            return;
        }
        for event in [EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MINIMIZEEND] {
            let hook = unsafe {
                SetWinEventHook(event, event, None, Some(arrange_event_proc), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)
            };
            if !hook.is_invalid() {
                self.arrange.push(hook);
            }
        }
    }

    fn unregister_window_notifications(&mut self) {
        for h in self.arrange.drain(..) {
            unsafe {
                let _ = UnhookWinEvent(h);
            }
        }
        let target = self.window.hwnd.0 as isize;
        ARRANGE_TARGETS.with(|t| t.borrow_mut().retain(|&h| h != target));
        if self.shell {
            unsafe {
                let _ = DeregisterShellHookWindow(self.window.hwnd);
            }
            self.shell = false;
        }
    }
}

impl Drop for OwnedWindow {
    fn drop(&mut self) {
        self.window.watch_outside(false);
        self.unregister_window_notifications();
        unsafe {
            for p in self.power.drain(..) {
                let _ = UnregisterPowerSettingNotification(p);
            }
            if self.appbar {
                self.unregister_appbar();
            }
            SetWindowLongPtrW(self.window.hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.window.hwnd);
            drop(Box::from_raw(self.state as *mut WindowState));
            if let Some(i) = self.icon.take() {
                let _ = DestroyIcon(i);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MenuItem {
    Item { id: u32, label: String, checked: bool, enabled: bool },
    Separator,
    Submenu { label: String, items: Vec<MenuItem> },
}

impl MenuItem {
    pub fn item(id: u32, label: impl Into<String>) -> Self {
        MenuItem::Item { id, label: label.into(), checked: false, enabled: true }
    }

    pub fn check(id: u32, label: impl Into<String>, checked: bool) -> Self {
        MenuItem::Item { id, label: label.into(), checked, enabled: true }
    }
}

unsafe fn fill_menu(menu: HMENU, items: &[MenuItem]) -> Result<()> {
    for item in items {
        match item {
            MenuItem::Item { id, label, checked, enabled } => {
                let mut flags: MENU_ITEM_FLAGS = MF_STRING;
                if *checked {
                    flags |= MF_CHECKED;
                }
                if !*enabled {
                    flags |= MF_GRAYED;
                }
                let text = wide(label);
                unsafe { AppendMenuW(menu, flags, *id as usize, PCWSTR(text.as_ptr()))? };
            }
            MenuItem::Separator => unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())? },
            MenuItem::Submenu { label, items } => {
                unsafe {
                    let sub = CreatePopupMenu()?;
                    fill_menu(sub, items)?;
                    let text = wide(label);
                    AppendMenuW(menu, MF_POPUP, sub.0 as usize, PCWSTR(text.as_ptr()))?;
                }
            }
        }
    }
    Ok(())
}

impl Window {
    /// Samakan bilah judul dengan tema (gelap/terang) jendela panel.
    pub fn set_caption(&self, caption: Color, dark: bool) {
        set_caption(self.hwnd, caption, dark);
    }

    /// Ukuran area klien dalam piksel.
    pub fn client_px(&self) -> (i32, i32) {
        let mut r = RECT::default();
        let _ = unsafe { GetClientRect(self.hwnd, &mut r) };
        (r.right - r.left, r.bottom - r.top)
    }

    pub fn dpi(&self) -> u32 {
        unsafe { GetDpiForWindow(self.hwnd) }.max(96)
    }

    pub fn raw_hwnd(&self) -> isize {
        self.hwnd.0 as isize
    }

    pub fn set_bounds(&self, x: i32, y: i32, w: i32, h: i32, show: bool) {
        let mut flags = SWP_NOACTIVATE;
        if show {
            flags |= SWP_SHOWWINDOW;
        }
        unsafe {
            let _ = SetWindowPos(self.hwnd, Some(HWND_TOPMOST), x, y, w.max(1), h.max(1), flags);
        }
    }

    pub fn set_region(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            let rgn = CreateRectRgn(x, y, x + w.max(1), y + h.max(1));
            if SetWindowRgn(self.hwnd, Some(rgn), true) == 0 {
                let _ = DeleteObject(HGDIOBJ(rgn.0));
            }
        }
    }

    pub fn watch_outside(&self, on: bool) {
        let target = self.hwnd.0 as isize;
        let remaining = HOOK_TARGETS.with(|t| {
            let mut t = t.borrow_mut();
            if on {
                if !t.contains(&target) {
                    t.push(target);
                }
            } else {
                t.retain(|&h| h != target);
            }
            t.len()
        });
        OUTSIDE_HOOKS.with(|hooks| {
            let mut hooks = hooks.borrow_mut();
            if remaining == 0 {
                for h in hooks.drain(..) {
                    unsafe {
                        let _ = UnhookWinEvent(h);
                    }
                }
                return;
            }
            if !hooks.is_empty() {
                return;
            }
            for event in [EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_CAPTURESTART] {
                let hook = unsafe {
                    SetWinEventHook(event, event, None, Some(win_event_proc), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)
                };
                if !hook.is_invalid() {
                    hooks.push(hook);
                }
            }
        });
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn focus(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if IsIconic(self.hwnd).as_bool() { SW_RESTORE } else { SW_SHOW });
            let _ = SetForegroundWindow(self.hwnd);
        }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }

    pub fn app_beneath(&self, x: i32, y: i32) -> bool {
        let mut next = unsafe { GetTopWindow(None) }.ok();
        for _ in 0..MAX_Z_WALK {
            let Some(hwnd) = next else { break };
            if hwnd != self.hwnd
                && is_app_window(hwnd)
                && let Some(r) = frame_bounds(hwnd)
                && x >= r.left
                && x < r.right
                && y >= r.top
                && y < r.bottom
            {
                return !is_desktop(hwnd);
            }
            next = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok();
        }
        false
    }

    pub fn set_timer(&self, id: usize, ms: u32) {
        unsafe {
            SetTimer(Some(self.hwnd), id, ms.max(10), None);
        }
    }

    pub fn kill_timer(&self, id: usize) {
        unsafe {
            let _ = KillTimer(Some(self.hwnd), id);
        }
    }

    pub fn register_hotkey(&self, id: i32, hk: &Hotkey) -> bool {
        let mut mods: HOT_KEY_MODIFIERS = MOD_NOREPEAT;
        if hk.ctrl {
            mods |= MOD_CONTROL;
        }
        if hk.alt {
            mods |= MOD_ALT;
        }
        if hk.shift {
            mods |= MOD_SHIFT;
        }
        if hk.win {
            mods |= MOD_WIN;
        }
        unsafe { RegisterHotKey(Some(self.hwnd), id, mods, hk.vk as u32).is_ok() }
    }

    pub fn unregister_hotkey(&self, id: i32) {
        unsafe {
            let _ = UnregisterHotKey(Some(self.hwnd), id);
        }
    }

    pub fn popup_menu(&self, items: &[MenuItem]) -> Option<u32> {
        unsafe {
            let menu = CreatePopupMenu().ok()?;
            if fill_menu(menu, items).is_err() {
                let _ = DestroyMenu(menu);
                return None;
            }
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let previous = GetForegroundWindow();
            let _ = SetForegroundWindow(self.hwnd);
            let cmd = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON, pt.x, pt.y, None, self.hwnd, None);
            let _ = PostMessageW(Some(self.hwnd), WM_NULL, WPARAM(0), LPARAM(0));
            let _ = DestroyMenu(menu);
            if !previous.is_invalid() && previous != self.hwnd {
                let _ = SetForegroundWindow(previous);
            }
            (cmd.0 > 0).then_some(cmd.0 as u32)
        }
    }

    pub fn reregister_appbar(&self) {
        let mut abd = APPBARDATA {
            cbSize: size_of::<APPBARDATA>() as u32,
            hWnd: self.hwnd,
            uCallbackMessage: WM_APP_APPBAR,
            ..Default::default()
        };
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut abd);
            SHAppBarMessage(ABM_NEW, &mut abd);
        }
    }

    pub fn message_box(&self, caption: &str, text: &str) {
        let (c, t) = (HSTRING::from(caption), HSTRING::from(text));
        unsafe {
            MessageBoxW(Some(self.hwnd), &t, &c, MB_OK | MB_ICONINFORMATION);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub dpi: u32,
}

impl Monitor {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
}

pub fn cursor_pos() -> Option<(i32, i32)> {
    let mut pt = POINT::default();
    unsafe { GetCursorPos(&mut pt) }.ok().map(|_| (pt.x, pt.y))
}

pub fn primary_monitor() -> Monitor {
    unsafe {
        let hmon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
        let _ = GetMonitorInfoW(hmon, &mut info);
        let (mut dx, mut dy) = (96u32, 96u32);
        let _ = GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
        let r: RECT = info.rcMonitor;
        Monitor { left: r.left, top: r.top, right: r.right, bottom: r.bottom, dpi: dx.max(96) }
    }
}

pub fn run_message_loop() {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

pub fn quit() {
    unsafe { PostQuitMessage(0) }
}

pub fn notify_existing_instance(settings: bool) -> bool {
    let class = wide(CLASS_NAME);
    unsafe {
        match FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
            Ok(h) if !h.is_invalid() => PostMessageW(Some(h), WM_APP_SECOND_INSTANCE, WPARAM(settings as usize), LPARAM(0)).is_ok(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IconImage {
    pub size: u32,
    pub rgba: Vec<u8>,
}

fn create_hicon(img: &IconImage) -> Result<HICON> {
    let s = img.size as i32;
    unsafe {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: s,
                biHeight: -s,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let color = CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
        let n = (s * s) as usize;
        let dst = std::slice::from_raw_parts_mut(bits as *mut u8, n * 4);
        for i in 0..n {
            let [r, g, b, a] = [img.rgba[i * 4], img.rgba[i * 4 + 1], img.rgba[i * 4 + 2], img.rgba[i * 4 + 3]];
            dst[i * 4..i * 4 + 4].copy_from_slice(&[b, g, r, a]);
        }
        let mask_bits = vec![0u8; ((s + 15) / 16 * 2 * s) as usize];
        let mask = CreateBitmap(s, s, 1, 1, Some(mask_bits.as_ptr() as *const c_void));
        let info = ICONINFO { fIcon: BOOL(1), xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
        let icon = CreateIconIndirect(&info);
        let _ = DeleteObject(HGDIOBJ(color.0));
        let _ = DeleteObject(HGDIOBJ(mask.0));
        icon
    }
}

pub struct TrayIcon {
    hwnd: HWND,
    icon: HICON,
    tooltip: String,
}

impl std::fmt::Debug for TrayIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayIcon").field("tooltip", &self.tooltip).finish()
    }
}

impl TrayIcon {
    pub fn new(window: &Window, image: &IconImage, tooltip: &str) -> Result<TrayIcon> {
        let icon = create_hicon(image)?;
        let tray = TrayIcon { hwnd: window.hwnd, icon, tooltip: tooltip.to_string() };
        tray.notify(NIM_ADD);
        Ok(tray)
    }

    fn data(&self) -> NOTIFYICONDATAW {
        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_APP_TRAY,
            hIcon: self.icon,
            ..Default::default()
        };
        for (dst, src) in nid.szTip.iter_mut().zip(self.tooltip.encode_utf16().take(127)) {
            *dst = src;
        }
        nid
    }

    fn notify(&self, msg: windows::Win32::UI::Shell::NOTIFY_ICON_MESSAGE) -> bool {
        let nid = self.data();
        unsafe { Shell_NotifyIconW(msg, &nid).as_bool() }
    }

    pub fn readd(&self) {
        self.notify(NIM_ADD);
    }

    pub fn set_tooltip(&mut self, tooltip: &str) {
        self.tooltip = tooltip.to_string();
        self.notify(NIM_MODIFY);
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.notify(NIM_DELETE);
        unsafe {
            let _ = DestroyIcon(self.icon);
        }
    }
}

pub fn error_box(caption: &str, text: &str) {
    use windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
    let (c, t) = (HSTRING::from(caption), HSTRING::from(text));
    unsafe {
        MessageBoxW(None, &t, &c, MB_OK | MB_ICONERROR);
    }
}
