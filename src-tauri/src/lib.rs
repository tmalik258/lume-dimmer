use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Listener, Manager, RunEvent, WindowEvent,
};

mod flyout;
mod flyout_win;
mod gamma;
mod gamma_mag;

use flyout::{FlyoutState, MAIN_WINDOW_LABEL};

const MENU_TOGGLE_ID: &str = "toggle";
const MENU_QUIT_ID: &str = "quit";

fn restore_gamma_on_exit(event: &RunEvent) {
    if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. }) {
        gamma::restore();
    }
}

fn on_main_window_event(window: &tauri::Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    match event {
        WindowEvent::CloseRequested { api, .. } => {
            flyout::hide_control_flyout(window.app_handle());
            api.prevent_close();
        }
        WindowEvent::Focused(false) => {
            let ignore_blur = window
                .app_handle()
                .state::<FlyoutState>()
                .ignore_next_blur
                .swap(false, Ordering::SeqCst);
            if ignore_blur {
                return;
            }
            flyout::hide_control_flyout(window.app_handle());
        }
        _ => {}
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
            flyout::bind_app(app.handle().clone());
            gamma::install_panic_hook();
            gamma::initialize();

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
                    MENU_TOGGLE_ID => flyout::toggle_control_flyout(app),
                    MENU_QUIT_ID => {
                        gamma::restore();
                        app.exit(0);
                    }
                    other => panic!("Unhandled tray menu id {other}"),
                })
                .on_tray_icon_event(|tray, event| {
                    flyout::on_tray_icon_event(tray.app_handle(), event);
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
        .on_window_event(|window, event| on_main_window_event(window, event))
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| restore_gamma_on_exit(&event));
}
