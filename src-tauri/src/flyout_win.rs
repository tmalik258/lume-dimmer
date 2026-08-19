use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;

const WH_MOUSE_LL: i32 = 14;
const WM_LBUTTONDOWN: usize = 0x0201;
const WM_RBUTTONDOWN: usize = 0x0204;
const WM_MBUTTONDOWN: usize = 0x0207;

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct WinRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MsllHookStruct {
    pt: Point,
    mouse_data: u32,
    flags: u32,
    time: u32,
    extra_info: usize,
}

#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(id_hook: i32, proc: HookProc, module: isize, thread_id: u32) -> isize;
    fn UnhookWindowsHookEx(hook: isize) -> i32;
    fn CallNextHookEx(hook: isize, code: i32, wparam: usize, lparam: isize) -> isize;
    fn GetWindowRect(hwnd: isize, rect: *mut WinRect) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetLastError() -> u32;
}

type HookProc = unsafe extern "system" fn(i32, usize, isize) -> isize;

#[derive(Clone, Copy)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

static HOOK: AtomicIsize = AtomicIsize::new(0);
static FLYOUT_HWND: AtomicIsize = AtomicIsize::new(0);
static TRAY_RECT: Mutex<Option<ScreenRect>> = Mutex::new(None);

pub fn set_tray_rect(rect: ScreenRect) {
    *TRAY_RECT.lock().expect("Tray rect mutex was poisoned") = Some(rect);
}

pub fn install(hwnd: isize) {
    if hwnd == 0 {
        panic!("Expected a non-null control flyout HWND");
    }
    FLYOUT_HWND.store(hwnd, Ordering::SeqCst);
    if HOOK.load(Ordering::SeqCst) != 0 {
        return;
    }

    let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, mouse_hook, 0, 0) };
    if hook == 0 {
        panic!(
            "SetWindowsHookExW(WH_MOUSE_LL) failed: GetLastError={}",
            unsafe { GetLastError() }
        );
    }
    HOOK.store(hook, Ordering::SeqCst);
}

pub fn uninstall() {
    FLYOUT_HWND.store(0, Ordering::SeqCst);
    let hook = HOOK.swap(0, Ordering::SeqCst);
    if hook == 0 {
        return;
    }
    if unsafe { UnhookWindowsHookEx(hook) } == 0 {
        panic!(
            "UnhookWindowsHookEx failed: GetLastError={}",
            unsafe { GetLastError() }
        );
    }
}

fn tray_rect() -> Option<ScreenRect> {
    *TRAY_RECT.lock().expect("Tray rect mutex was poisoned")
}

fn flyout_rect(hwnd: isize) -> Option<ScreenRect> {
    let mut rect = WinRect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return None;
    }
    Some(ScreenRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    })
}

fn should_dismiss(x: i32, y: i32) -> bool {
    let hwnd = FLYOUT_HWND.load(Ordering::SeqCst);
    if hwnd == 0 {
        return false;
    }
    if let Some(rect) = flyout_rect(hwnd) {
        if rect.contains(x, y) {
            return false;
        }
    }
    if let Some(rect) = tray_rect() {
        if rect.contains(x, y) {
            return false;
        }
    }
    true
}

fn is_mouse_down(wparam: usize) -> bool {
    wparam == WM_LBUTTONDOWN || wparam == WM_RBUTTONDOWN || wparam == WM_MBUTTONDOWN
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    if code >= 0 && is_mouse_down(wparam) {
        let info = unsafe { &*(lparam as *const MsllHookStruct) };
        if should_dismiss(info.pt.x, info.pt.y) {
            crate::flyout::request_hide_from_outside_click();
        }
    }
    unsafe { CallNextHookEx(0, code, wparam, lparam) }
}
