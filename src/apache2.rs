#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr;

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

pub fn log_error(
    file: *const c_char,
    line: u32,
    level: u32,
    status_code: i32,
    server_info: *const server_rec,
    msg: CString,
) {
    unsafe {
        let module_index: c_int = if aplog_module_index == ptr::null_mut() {
            *aplog_module_index
        } else {
            APLOG_NO_MODULE as c_int
        };
        ap_log_error_(
            file,
            line as c_int,
            module_index,
            level as c_int,
            status_code,
            server_info,
            msg.as_ptr(),
        );
    }
}
