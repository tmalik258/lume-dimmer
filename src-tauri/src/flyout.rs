use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Position, Size, WebviewWindow,
};

use crate::flyout_win::{self, ScreenRect};

pub const MAIN_WINDOW_LABEL: &str = "main";
const FLYOUT_GAP_PX: i32 = 12;

static APP: OnceLock<AppHandle> = OnceLock::new();
static HIDE_FROM_HOOK_QUEUED: AtomicBool = AtomicBool::new(false);

pub struct FlyoutState {
    pub ignore_next_blur: AtomicBool,
}

pub fn bind_app(app: AppHandle) {
    APP.set(app)
        .unwrap_or_else(|_| panic!("Expected bind_app to run once during setup"));
}

fn app_handle() -> &'static AppHandle {
    APP.get()
        .expect("Expected flyout app handle to be bound during setup")
}

fn main_window(app: &AppHandle) -> WebviewWindow {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .expect("Expected webview window with label main")
}

fn flyout_state(app: &AppHandle) -> &FlyoutState {
    app.state::<FlyoutState>().inner()
}

fn position_control_flyout_bottom_right(window: &WebviewWindow) {
    let size = window
        .outer_size()
        .expect("Failed to read control flyout size");
    let width = i32::try_from(size.width).expect("Flyout width does not fit in i32");
    let height = i32::try_from(size.height).expect("Flyout height does not fit in i32");

    let monitor = window
        .current_monitor()
        .expect("Failed to read current monitor")
        .expect("Expected a current monitor");
    let work = monitor.work_area();
    let work_width = i32::try_from(work.size.width).expect("Work area width does not fit in i32");
    let work_height = i32::try_from(work.size.height).expect("Work area height does not fit in i32");

    let x = work.position.x + work_width - width - FLYOUT_GAP_PX;
    let y = work.position.y + work_height - height - FLYOUT_GAP_PX;

    window
        .set_position(PhysicalPosition::new(x, y))
        .expect("Failed to position control flyout");
}

fn flyout_hwnd(window: &WebviewWindow) -> isize {
    window
        .hwnd()
        .expect("Failed to read control flyout HWND")
        .0 as isize
}

pub fn show_control_flyout(app: &AppHandle) {
    let window = main_window(app);
    position_control_flyout_bottom_right(&window);
    flyout_state(app)
        .ignore_next_blur
        .store(true, Ordering::SeqCst);
    window.show().expect("Failed to show control flyout");
    window.set_focus().expect("Failed to focus control flyout");
    flyout_win::install(flyout_hwnd(&window));
}

pub fn hide_control_flyout(app: &AppHandle) {
    HIDE_FROM_HOOK_QUEUED.store(false, Ordering::SeqCst);
    flyout_win::uninstall();
    flyout_state(app)
        .ignore_next_blur
        .store(false, Ordering::SeqCst);
    main_window(app)
        .hide()
        .expect("Failed to hide control flyout");
}

pub fn toggle_control_flyout(app: &AppHandle) {
    let is_visible = main_window(app)
        .is_visible()
        .expect("Failed to read control flyout visibility");

    if is_visible {
        hide_control_flyout(app);
        return;
    }

    show_control_flyout(app);
}

pub fn request_hide_from_outside_click() {
    if HIDE_FROM_HOOK_QUEUED.swap(true, Ordering::SeqCst) {
        return;
    }

    let app = app_handle().clone();
    std::thread::Builder::new()
        .name("lume-flyout-hide".into())
        .spawn(move || {
            let app_for_hide = app.clone();
            app.run_on_main_thread(move || hide_control_flyout(&app_for_hide))
                .expect("Failed to hide control flyout after outside click");
        })
        .expect("Failed to queue control flyout hide after outside click");
}

pub fn remember_tray_rect(event: &TrayIconEvent) {
    let rect = match event {
        TrayIconEvent::Click { rect, .. }
        | TrayIconEvent::DoubleClick { rect, .. }
        | TrayIconEvent::Enter { rect, .. }
        | TrayIconEvent::Move { rect, .. }
        | TrayIconEvent::Leave { rect, .. } => rect,
        _ => return,
    };

    let Position::Physical(pos) = rect.position else {
        panic!("Expected tray icon rect position in physical pixels");
    };
    let Size::Physical(size) = rect.size else {
        panic!("Expected tray icon rect size in physical pixels");
    };
    let width = i32::try_from(size.width).expect("Tray icon width does not fit in i32");
    let height = i32::try_from(size.height).expect("Tray icon height does not fit in i32");

    flyout_win::set_tray_rect(ScreenRect {
        left: pos.x,
        top: pos.y,
        right: pos.x + width,
        bottom: pos.y + height,
    });
}

pub fn on_tray_icon_event(app: &AppHandle, event: TrayIconEvent) {
    remember_tray_rect(&event);
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        toggle_control_flyout(app);
    }
}
