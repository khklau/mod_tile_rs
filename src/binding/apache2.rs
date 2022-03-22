#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

unsafe impl Send for module {}
unsafe impl Sync for module {}
unsafe impl Send for command_rec {}
unsafe impl Sync for command_rec {}

use std::ffi::CStr;

pub fn get_module_name() -> &'static str {
    unsafe {
        CStr::from_ptr(crate::TILE_MODULE.name).to_str().unwrap()
    }
}
