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
