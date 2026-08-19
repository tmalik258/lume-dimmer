use std::sync::Mutex;

use serde::Deserialize;
use tauri::AppHandle;

use crate::gamma_win::{self, DeviceRamp};

pub const UPDATE_OPACITY_EVENT: &str = "update-opacity";
pub const SET_DIMMER_ENABLED_EVENT: &str = "set-dimmer-enabled";
pub const DEFAULT_OPACITY: f32 = 0.5;
pub const MAX_OPACITY: f32 = 0.9;

struct GammaState {
    enabled: bool,
    opacity: f32,
    baselines: Vec<DeviceRamp>,
}

static STATE: Mutex<GammaState> = Mutex::new(GammaState {
    enabled: false,
    opacity: DEFAULT_OPACITY,
    baselines: Vec::new(),
});

#[derive(Deserialize)]
struct OpacityPayload {
    opacity: f32,
}

#[derive(Deserialize)]
struct DimmerEnabledPayload {
    enabled: bool,
}

fn lock_state() -> std::sync::MutexGuard<'static, GammaState> {
    STATE
        .lock()
        .expect("Gamma state mutex was poisoned")
}

fn brightness_scale(opacity: f32) -> f32 {
    if opacity.is_nan() || !(0.0..=MAX_OPACITY).contains(&opacity) {
        panic!("Expected opacity in 0.0..={MAX_OPACITY}, got {opacity}");
    }
    1.0 - opacity
}

pub fn set_enabled(enabled: bool) {
    let mut state = lock_state();
    if enabled {
        if !state.enabled {
            state.baselines = gamma_win::capture_baselines();
            state.enabled = true;
        }
        let scale = brightness_scale(state.opacity);
        let baselines = state.baselines.clone();
        drop(state);
        gamma_win::apply_scaled(&baselines, scale);
        return;
    }

    let baselines = std::mem::take(&mut state.baselines);
    state.enabled = false;
    drop(state);
    gamma_win::restore(&baselines);
}

pub fn restore() {
    set_enabled(false);
}

pub fn set_opacity(opacity: f32) {
    let scale = brightness_scale(opacity);
    let mut state = lock_state();
    state.opacity = opacity;
    if !state.enabled {
        return;
    }
    let baselines = state.baselines.clone();
    drop(state);
    gamma_win::apply_scaled(&baselines, scale);
}

fn parse_opacity_payload(payload: &str) -> f32 {
    let parsed: OpacityPayload = serde_json::from_str(payload).unwrap_or_else(|err| {
        panic!("Expected update-opacity payload to be {{ opacity: number }}, got {payload}: {err}");
    });
    parsed.opacity
}

fn parse_enabled_payload(payload: &str) -> bool {
    let parsed: DimmerEnabledPayload = serde_json::from_str(payload).unwrap_or_else(|err| {
        panic!(
            "Expected set-dimmer-enabled payload to be {{ enabled: bool }}, got {payload}: {err}"
        );
    });
    parsed.enabled
}

pub fn on_opacity_event(app: &AppHandle, payload: &str) {
    let opacity = parse_opacity_payload(payload);
    let app = app.clone();
    app.run_on_main_thread(move || set_opacity(opacity))
        .expect("Failed to apply opacity on the main thread");
}

pub fn on_enabled_event(app: &AppHandle, payload: &str) {
    let enabled = parse_enabled_payload(payload);
    let app = app.clone();
    app.run_on_main_thread(move || set_enabled(enabled))
        .expect("Failed to apply dimmer enable on the main thread");
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_best_effort();
        previous(info);
    }));
}

fn restore_best_effort() {
    let mut state = match STATE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if state.baselines.is_empty() {
        return;
    }
    let baselines = std::mem::take(&mut state.baselines);
    state.enabled = false;
    drop(state);
    gamma_win::restore_best_effort(&baselines);
}
