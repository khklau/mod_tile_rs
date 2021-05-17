#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

#[macro_export]
macro_rules! try_log_else {
    (($level: expr, $server_expr: expr, $msg_expr: expr) {$log_failure_expr: expr}) => {
        unsafe {
            crate::apache2::bindings::ap_log_error_(
                cstr!(file!()),
                line!() as std::os::raw::c_int,
                crate::apache2::bindings::APLOG_NO_MODULE as c_int,
                $level as c_int,
                -1,
                $server_expr,
                match std::ffi::CString::new($msg_expr) {
                    Err(_) => $log_failure_expr,
                    Ok(err_msg) => err_msg.as_ptr(),
                },
            );
        }
    }
}

unsafe impl Send for module {}
unsafe impl Sync for module {}
