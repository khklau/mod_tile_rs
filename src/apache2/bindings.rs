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
macro_rules! log {
    ($level: expr, $server_expr: expr, $msg_expr: expr) => {
        gensym::gensym!{
            _log!{ $level, $server_expr, $msg_expr } }
    };
}

macro_rules! _log { (
    $var_sym: ident,
    $level: expr,
    $server_expr: expr,
    $msg_expr: expr) => {
        let $var_sym = std::ffi::CString::new($msg_expr).unwrap();
        unsafe {
            crate::apache2::bindings::ap_log_error_(
                cstr!(file!()),
                line!() as std::os::raw::c_int,
                crate::apache2::bindings::APLOG_NO_MODULE as std::os::raw::c_int,
                $level as std::os::raw::c_int,
                -1,
                $server_expr,
                $var_sym.as_ptr(),
            );
        };
    }
}

#[macro_export]
macro_rules! plog {
    ($level: expr, $pool_expr: expr, $msg_expr: expr) => {
        gensym::gensym!{
            _plog!{ $level, $pool_expr, $msg_expr } }
    };
}

macro_rules! _plog { (
    $var_sym: ident,
    $level: expr,
    $pool_expr: expr,
    $msg_expr: expr) => {
        let $var_sym = std::ffi::CString::new($msg_expr).unwrap();
        unsafe {
            crate::apache2::bindings::ap_log_perror_(
                cstr!(file!()),
                line!() as std::os::raw::c_int,
                crate::apache2::bindings::APLOG_NO_MODULE as std::os::raw::c_int,
                $level as std::os::raw::c_int,
                -1,
                $pool_expr,
                $var_sym.as_ptr(),
            );
        };
    }
}

unsafe impl Send for module {}
unsafe impl Sync for module {}
