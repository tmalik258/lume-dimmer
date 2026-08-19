use std::ffi::c_void;
use std::mem::size_of;

const DISPLAY_DEVICE_ATTACHED_TO_DESKTOP: u32 = 0x0000_0001;
const DISPLAY_DEVICE_MIRRORING_DRIVER: u32 = 0x0000_0008;

#[repr(C)]
struct DisplayDeviceW {
    cb: u32,
    device_name: [u16; 32],
    device_string: [u16; 128],
    state_flags: u32,
    device_id: [u16; 128],
    device_key: [u16; 128],
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateDCW(
        driver: *const u16,
        device: *const u16,
        output: *const u16,
        init_data: *const c_void,
    ) -> *mut c_void;
    fn DeleteDC(hdc: *mut c_void) -> i32;
    fn GetDeviceGammaRamp(hdc: *mut c_void, ramp: *mut u16) -> i32;
    fn SetDeviceGammaRamp(hdc: *mut c_void, ramp: *const u16) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn EnumDisplayDevicesW(
        device: *const u16,
        dev_num: u32,
        display_device: *mut DisplayDeviceW,
        flags: u32,
    ) -> i32;
}

#[derive(Clone)]
pub struct DeviceRamp {
    pub device_name: String,
    pub ramp: [u16; 768],
}

struct DeviceContext {
    hdc: *mut c_void,
    device_name: String,
}

impl DeviceContext {
    fn try_open(device_name: &str) -> Result<Self, String> {
        let driver: Vec<u16> = "DISPLAY\0".encode_utf16().collect();
        let mut device: Vec<u16> = device_name.encode_utf16().collect();
        device.push(0);
        let hdc = unsafe {
            CreateDCW(driver.as_ptr(), device.as_ptr(), std::ptr::null(), std::ptr::null())
        };
        if hdc.is_null() {
            return Err(format!("CreateDCW failed for display {device_name}"));
        }
        Ok(Self {
            hdc,
            device_name: device_name.to_string(),
        })
    }

    fn open(device_name: &str) -> Self {
        Self::try_open(device_name).unwrap_or_else(|err| panic!("{err}"))
    }

    fn get_ramp(&self) -> [u16; 768] {
        let mut ramp = [0u16; 768];
        let ok = unsafe { GetDeviceGammaRamp(self.hdc, ramp.as_mut_ptr()) };
        if ok == 0 {
            panic!(
                "GetDeviceGammaRamp failed for display {} (HDR or this driver may reject gamma ramps)",
                self.device_name
            );
        }
        ramp
    }

    fn set_ramp(&self, ramp: &[u16; 768]) {
        let ok = unsafe { SetDeviceGammaRamp(self.hdc, ramp.as_ptr()) };
        if ok == 0 {
            panic!(
                "SetDeviceGammaRamp failed for display {} (HDR or this driver may reject gamma ramps)",
                self.device_name
            );
        }
    }

    fn try_set_ramp(&self, ramp: &[u16; 768]) -> Result<(), String> {
        let ok = unsafe { SetDeviceGammaRamp(self.hdc, ramp.as_ptr()) };
        if ok == 0 {
            return Err(format!(
                "SetDeviceGammaRamp failed for display {}",
                self.device_name
            ));
        }
        Ok(())
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        if !self.hdc.is_null() {
            unsafe { DeleteDC(self.hdc); }
        }
    }
}

fn wide_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16(&buf[..end]).expect("Display device name is not valid UTF-16")
}

fn attached_display_names() -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0_u32;
    loop {
        let mut device = DisplayDeviceW {
            cb: size_of::<DisplayDeviceW>() as u32,
            device_name: [0; 32],
            device_string: [0; 128],
            state_flags: 0,
            device_id: [0; 128],
            device_key: [0; 128],
        };
        let found = unsafe { EnumDisplayDevicesW(std::ptr::null(), index, &mut device, 0) };
        if found == 0 {
            break;
        }
        index += 1;
        if device.state_flags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0 {
            continue;
        }
        if device.state_flags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0 {
            continue;
        }
        names.push(wide_to_string(&device.device_name));
    }
    names
}

pub fn capture_baselines() -> Vec<DeviceRamp> {
    let names = attached_display_names();
    if names.is_empty() {
        panic!("Expected at least one display attached to the desktop");
    }

    names
        .into_iter()
        .map(|device_name| {
            let dc = DeviceContext::open(&device_name);
            DeviceRamp {
                ramp: dc.get_ramp(),
                device_name,
            }
        })
        .collect()
}

pub fn apply_scaled(baselines: &[DeviceRamp], scale: f32) {
    if !(scale > 0.0 && scale <= 1.0) {
        panic!("Expected gamma scale in (0, 1], got {scale}");
    }

    for device in baselines {
        let mut scaled = [0u16; 768];
        for (index, value) in device.ramp.iter().enumerate() {
            scaled[index] = (*value as f32 * scale).round() as u16;
        }
        DeviceContext::open(&device.device_name).set_ramp(&scaled);
    }
}

pub fn restore(baselines: &[DeviceRamp]) {
    for device in baselines {
        DeviceContext::open(&device.device_name).set_ramp(&device.ramp);
    }
}

pub fn restore_best_effort(baselines: &[DeviceRamp]) {
    for device in baselines {
        let result = DeviceContext::try_open(&device.device_name)
            .and_then(|dc| dc.try_set_ramp(&device.ramp));
        if let Err(err) = result {
            eprintln!("{err}");
        }
    }
}
