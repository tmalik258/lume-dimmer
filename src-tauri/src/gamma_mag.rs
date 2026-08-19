#[repr(C)]
struct MagColorEffect {
    transform: [f32; 25],
}

#[link(name = "magnification")]
extern "system" {
    fn MagInitialize() -> i32;
    fn MagUninitialize() -> i32;
    fn MagSetFullscreenTransform(mag_level: f32, x_offset: i32, y_offset: i32) -> i32;
    fn MagSetFullscreenColorEffect(effect: *const MagColorEffect) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetLastError() -> u32;
}

fn last_error() -> u32 {
    unsafe { GetLastError() }
}

fn brightness_effect(scale: f32) -> MagColorEffect {
    MagColorEffect {
        transform: [
            scale, 0.0, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    }
}

pub fn initialize() {
    if unsafe { MagInitialize() } == 0 {
        panic!("MagInitialize failed: GetLastError={}", last_error());
    }
    if unsafe { MagSetFullscreenTransform(1.0, 0, 0) } == 0 {
        panic!(
            "MagSetFullscreenTransform failed: GetLastError={}",
            last_error()
        );
    }
}

pub fn set_brightness(scale: f32) {
    if !(scale > 0.0 && scale <= 1.0) {
        panic!("Expected brightness scale in (0, 1], got {scale}");
    }
    let effect = brightness_effect(scale);
    if unsafe { MagSetFullscreenColorEffect(&effect) } == 0 {
        panic!(
            "MagSetFullscreenColorEffect failed: GetLastError={}",
            last_error()
        );
    }
}

pub fn restore_best_effort() {
    let effect = brightness_effect(1.0);
    unsafe {
        MagSetFullscreenColorEffect(&effect);
        MagUninitialize();
    }
}

pub fn shutdown() {
    set_brightness(1.0);
    if unsafe { MagUninitialize() } == 0 {
        panic!("MagUninitialize failed: GetLastError={}", last_error());
    }
}
