use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Listener, Manager, PhysicalPosition, RunEvent, WindowEvent,
};

mod gamma;
mod gamma_win;

const MAIN_WINDOW_LABEL: &str = "main";
const MENU_TOGGLE_ID: &str = "toggle";
const MENU_QUIT_ID: &str = "quit";
const FLYOUT_GAP_PX: i32 = 12;

struct FlyoutState {
    ignore_next_blur: AtomicBool,
}

fn main_window(app: &AppHandle) -> tauri::WebviewWindow {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .expect("Expected webview window with label main")
}

fn flyout_state(app: &AppHandle) -> &FlyoutState {
    app.state::<FlyoutState>().inner()
}

fn position_control_flyout_bottom_right(window: &tauri::WebviewWindow) {
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

fn show_control_flyout(app: &AppHandle) {
    let window = main_window(app);
    position_control_flyout_bottom_right(&window);
    flyout_state(app)
        .ignore_next_blur
        .store(true, Ordering::SeqCst);
    window.show().expect("Failed to show control flyout");
    window.set_focus().expect("Failed to focus control flyout");
}

fn hide_control_flyout(app: &AppHandle) {
    flyout_state(app)
        .ignore_next_blur
        .store(false, Ordering::SeqCst);
    main_window(app)
        .hide()
        .expect("Failed to hide control flyout");
}

fn toggle_control_flyout(app: &AppHandle) {
    let is_visible = main_window(app)
        .is_visible()
        .expect("Failed to read control flyout visibility");

    if is_visible {
        hide_control_flyout(app);
        return;
    }

    show_control_flyout(app);
}

fn restore_gamma_on_exit(event: &RunEvent) {
    if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. }) {
        gamma::restore();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(FlyoutState {
            ignore_next_blur: AtomicBool::new(false),
        })
        .setup(|app| {
            gamma::install_panic_hook();

            let toggle_item = MenuItem::with_id(
                app,
                MENU_TOGGLE_ID,
                "Toggle Control Window",
                true,
                None::<&str>,
            )?;
            let quit_item =
                MenuItem::with_id(app, MENU_QUIT_ID, "Quit Lume", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;

            let icon = app
                .default_window_icon()
                .expect("Expected default window icon for the tray")
                .clone();

            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Lume")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    MENU_TOGGLE_ID => toggle_control_flyout(app),
                    MENU_QUIT_ID => {
                        gamma::restore();
                        app.exit(0);
                    }
                    other => panic!("Unhandled tray menu id {other}"),
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_control_flyout(tray.app_handle());
                    }
                })
                .build(app)?;

            let handle = app.handle().clone();
            app.listen(gamma::UPDATE_OPACITY_EVENT, move |event| {
                gamma::on_opacity_event(&handle, event.payload());
            });
            let handle = app.handle().clone();
            app.listen(gamma::SET_DIMMER_ENABLED_EVENT, move |event| {
                gamma::on_enabled_event(&handle, event.payload());
            });

            gamma::set_enabled(true);

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != MAIN_WINDOW_LABEL {
                return;
            }

            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    hide_control_flyout(window.app_handle());
                    api.prevent_close();
                }
                WindowEvent::Focused(false) => {
                    let ignore_blur = flyout_state(window.app_handle())
                        .ignore_next_blur
                        .swap(false, Ordering::SeqCst);
                    if ignore_blur {
                        return;
                    }
                    hide_control_flyout(window.app_handle());
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| restore_gamma_on_exit(&event));
}
