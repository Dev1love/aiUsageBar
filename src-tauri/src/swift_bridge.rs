use crate::system_monitor::{TempSensor, FanInfo, BtDevice};

#[cfg(has_swift_dylib)]
mod ffi {
    use std::ffi::{c_char, CStr};
    use crate::system_monitor::{TempSensor, FanInfo, BtDevice};
    use serde::Deserialize;

    #[link(name = "system_monitor")]
    extern "C" {
        fn smc_read_all() -> *mut c_char;
        fn bt_get_devices() -> *mut c_char;
        fn free_string(ptr: *mut c_char);
        pub fn tray_init(callback: extern "C" fn()) -> bool;
        pub fn tray_set_title(cstr: *const c_char);
        pub fn tray_set_icon_rgba(bytes: *const u8, width: i32, height: i32);
        pub fn notification_show(title: *const c_char, body: *const c_char);
    }

    #[derive(Deserialize, Default)]
    struct SmcData {
        temps: Vec<TempSensor>,
        fans: Vec<FanInfo>,
    }

    pub fn get_smc_data() -> (Vec<TempSensor>, Vec<FanInfo>) {
        unsafe {
            let ptr = smc_read_all();
            if ptr.is_null() {
                return (vec![], vec![]);
            }
            let json_owned = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            free_string(ptr);
            let data: SmcData = serde_json::from_str(&json_owned).unwrap_or_default();
            (data.temps, data.fans)
        }
    }

    pub fn get_bluetooth_devices() -> Vec<BtDevice> {
        unsafe {
            let ptr = bt_get_devices();
            if ptr.is_null() {
                return vec![];
            }
            let json_owned = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            free_string(ptr);
            serde_json::from_str(&json_owned).unwrap_or_default()
        }
    }
}

pub fn get_smc_data() -> (Vec<TempSensor>, Vec<FanInfo>) {
    #[cfg(has_swift_dylib)]
    { ffi::get_smc_data() }
    #[cfg(not(has_swift_dylib))]
    { (vec![], vec![]) }
}

pub fn get_bluetooth_devices() -> Vec<BtDevice> {
    #[cfg(has_swift_dylib)]
    { ffi::get_bluetooth_devices() }
    #[cfg(not(has_swift_dylib))]
    { vec![] }
}

pub mod tray {
    pub fn init(callback: extern "C" fn()) -> bool {
        #[cfg(has_swift_dylib)]
        unsafe { super::ffi::tray_init(callback) }
        #[cfg(not(has_swift_dylib))]
        { let _ = callback; false }
    }

    pub fn set_title(s: &str) {
        #[cfg(has_swift_dylib)]
        {
            if let Ok(cstr) = std::ffi::CString::new(s) {
                unsafe { super::ffi::tray_set_title(cstr.as_ptr()); }
            }
        }
        #[cfg(not(has_swift_dylib))]
        { let _ = s; }
    }

    pub fn set_icon_rgba(rgba: &[u8], width: u32, height: u32) {
        #[cfg(has_swift_dylib)]
        unsafe {
            super::ffi::tray_set_icon_rgba(rgba.as_ptr(), width as i32, height as i32);
        }
        #[cfg(not(has_swift_dylib))]
        { let _ = (rgba, width, height); }
    }
}

pub mod notification {
    pub fn show(title: &str, body: &str) {
        #[cfg(has_swift_dylib)]
        {
            if let (Ok(t), Ok(b)) = (
                std::ffi::CString::new(title),
                std::ffi::CString::new(body),
            ) {
                unsafe { super::ffi::notification_show(t.as_ptr(), b.as_ptr()); }
            }
        }
        #[cfg(not(has_swift_dylib))]
        { let _ = (title, body); }
    }
}
