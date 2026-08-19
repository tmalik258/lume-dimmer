use std::sync::Mutex;

use serde::Deserialize;
use tauri::AppHandle;

use crate::gamma_mag;

pub const UPDATE_OPACITY_EVENT: &str = "update-opacity";
pub const SET_DIMMER_ENABLED_EVENT: &str = "set-dimmer-enabled";
pub const DEFAULT_OPACITY: f32 = 0.5;
pub const MAX_OPACITY: f32 = 0.9;

struct DimmerState {
    enabled: bool,
    opacity: f32,
    initialized: bool,
}

static STATE: Mutex<DimmerState> = Mutex::new(DimmerState {
    enabled: false,
    opacity: DEFAULT_OPACITY,
    initialized: false,
});

#[derive(Deserialize)]
struct OpacityPayload {
    opacity: f32,
}

#[derive(Deserialize)]
struct DimmerEnabledPayload {
    enabled: bool,
}

fn lock_state() -> std::sync::MutexGuard<'static, DimmerState> {
    STATE.lock().expect("Dimmer state mutex was poisoned")
}

fn brightness_scale(opacity: f32) -> f32 {
    if opacity.is_nan() || !(0.0..=MAX_OPACITY).contains(&opacity) {
        panic!("Expected opacity in 0.0..={MAX_OPACITY}, got {opacity}");
    }
    1.0 - opacity
}

fn apply(state: &DimmerState) {
    if !state.initialized {
        panic!("Expected the magnifier dimmer to be initialized before applying brightness");
    }
    if state.enabled {
        gamma_mag::set_brightness(brightness_scale(state.opacity));
        return;
    }
    gamma_mag::set_brightness(1.0);
}

pub fn initialize() {
    let mut state = lock_state();
    if state.initialized {
        return;
    }
    gamma_mag::initialize();
    state.initialized = true;
}

pub fn set_enabled(enabled: bool) {
    let mut state = lock_state();
    state.enabled = enabled;
    apply(&state);
}

pub fn restore() {
    let mut state = lock_state();
    if !state.initialized {
        return;
    }
    state.enabled = false;
    drop(state);
    gamma_mag::shutdown();
    lock_state().initialized = false;
}

pub fn set_opacity(opacity: f32) {
    let mut state = lock_state();
    state.opacity = opacity;
    if !state.enabled {
        return;
    }
    apply(&state);
}

pub fn on_opacity_event(app: &AppHandle, payload: &str) {
    let parsed: OpacityPayload = serde_json::from_str(payload).unwrap_or_else(|err| {
        panic!("Expected update-opacity payload to be {{ opacity: number }}, got {payload}: {err}");
    });
    app.run_on_main_thread(move || set_opacity(parsed.opacity))
        .expect("Failed to apply dimmer opacity on the main thread");
}

pub fn on_enabled_event(app: &AppHandle, payload: &str) {
    let parsed: DimmerEnabledPayload = serde_json::from_str(payload).unwrap_or_else(|err| {
        panic!(
            "Expected set-dimmer-enabled payload to be {{ enabled: bool }}, got {payload}: {err}"
        );
    });
    app.run_on_main_thread(move || set_enabled(parsed.enabled))
        .expect("Failed to apply dimmer enable on the main thread");
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        gamma_mag::restore_best_effort();
        previous(info);
    }));
}
